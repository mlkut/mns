use std::sync::Arc;

use axum::{
    extract::Path,
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    response::Response,
    routing::get,
    Json, Router,
};
use mns::Name;
use serde::Deserialize;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::content_type;
use crate::html;
use crate::resolver;
use crate::store::ZoneStore;

#[derive(Deserialize)]
pub struct FormatQuery {
    format: Option<String>,
}

pub fn build_router<S: ZoneStore + 'static>(store: Arc<S>) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/static/{file}", get(static_handler))
        .route("/avatar/{name}", get(avatar_handler::<S>))
        .route("/stats", get(stats_handler::<S>))
        .route("/owners", get(owners_handler::<S>))
        .route("/owner/{address}", get(owner_handler::<S>))
        .route("/{*name}", get(get_handler::<S>).put(put_handler::<S>))
        .layer(CorsLayer::permissive())
        .with_state(store)
}

#[derive(Serialize)]
struct StatsResponse {
    total_owners: u64,
}

async fn stats_handler<S: ZoneStore>(
    store: axum::extract::State<Arc<S>>,
) -> Result<Json<StatsResponse>, (StatusCode, String)> {
    let total_owners = store.total_owners().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    Ok(Json(StatsResponse { total_owners }))
}

const BATCH_SIZE: u64 = 256;

async fn owner_handler<S: ZoneStore>(
    store: axum::extract::State<Arc<S>>,
    Path(address_str): Path<String>,
) -> Response {
    let address_str = address_str.trim();
    if !address_str.starts_with("0x") || address_str.len() != 42 {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/html")],
            html::render_error("invalid address format (expected 0x-prefixed 20-byte hex)"),
        )
            .into_response();
    }

    let owner_bytes = match hex::decode(&address_str[2..]) {
        Ok(b) if b.len() == 20 => {
            let mut arr = [0u8; 20];
            arr.copy_from_slice(&b);
            arr
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                [("content-type", "text/html")],
                html::render_error("invalid address hex"),
            )
                .into_response()
        }
    };

    let batches = match store.get_owner_batches(&owner_bytes).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error(&format!("store error: {e}")),
            )
                .into_response()
        }
    };

    // Expand batches into individual names, filtering out entries owned by others
    let mut names: Vec<String> = Vec::new();
    for batch_start in &batches {
        for offset in 0..BATCH_SIZE {
            let ordinal = batch_start + offset;
            let name = Name::from_ordinal(ordinal);
            // Check if this ordinal has an entry owned by someone else
            if let Some(entry_owner) = store.get_name_owner(&name).await.unwrap_or(None) {
                if entry_owner != owner_bytes {
                    continue; // entry owned by someone else → not this owner's name
                }
            }
            names.push(name.to_string());
        }
    }

    // Also add entries owned by this owner (they might not be in any batch,
    // though they typically are; dedupe via the loop above already)
    // Actually they're already covered by the batch expansion loop.
    // But for safety, also fetch entry ordinals and include any not already present:
    if let Ok(entries) = store.get_owner_entries(&owner_bytes).await {
        for ordinal in entries {
            let name_str = Name::from_ordinal(ordinal).to_string();
            if !names.contains(&name_str) {
                names.push(name_str);
            }
        }
    }

    let html_body = html::render_owner_page(address_str, &names);
    (StatusCode::OK, [("content-type", "text/html")], html_body).into_response()
}

async fn owners_handler<S: ZoneStore>(store: axum::extract::State<Arc<S>>) -> Response {
    let owners = match store.all_owners().await {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error(&format!("store error: {e}")),
            )
                .into_response()
        }
    };

    let items: Vec<html::OwnerItemSimple> = owners
        .iter()
        .map(|addr| html::OwnerItemSimple {
            name_or_addr: format!("0x{}", hex::encode(addr)),
        })
        .collect();

    let html_body = html::render_owners_page(&items);
    (StatusCode::OK, [("content-type", "text/html")], html_body).into_response()
}

async fn root_handler() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        [("content-type", "text/html")],
        html::render_home_page(),
    )
}

async fn static_handler(Path(file): Path<String>) -> Response {
    match file.as_str() {
        "mlkut.png" => (
            StatusCode::OK,
            [("content-type", "image/png")],
            include_bytes!("../static/mlkut.png").as_slice(),
        )
            .into_response(),
        "favicon.png" => (
            StatusCode::OK,
            [("content-type", "image/png")],
            include_bytes!("../static/favicon.png").as_slice(),
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            [("content-type", "text/plain")],
            "not found",
        )
            .into_response(),
    }
}

async fn avatar_handler<S: ZoneStore>(
    _store: axum::extract::State<Arc<S>>,
    Path(name_str): Path<String>,
) -> Response {
    let name = match name_str.parse::<Name>() {
        Ok(n) => n,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                [("content-type", "text/plain")],
                "invalid mns name",
            )
                .into_response()
        }
    };
    (
        StatusCode::OK,
        [("content-type", "image/svg+xml")],
        name.render_avatar_svg(),
    )
        .into_response()
}

async fn get_handler<S: ZoneStore>(
    store: axum::extract::State<Arc<S>>,
    Path(name_str): Path<String>,
    Query(query): Query<FormatQuery>,
    headers: HeaderMap,
) -> Response {
    let name = match name_str.parse::<Name>() {
        Ok(n) => n,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                [("content-type", "text/html")],
                html::render_error("invalid mns name"),
            )
                .into_response()
        }
    };

    let accept = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*/*");

    tracing::debug!(%name, "GET handler");

    let owner = store.get_name_owner(&name).await.unwrap_or(None);

    match resolver::resolve_name(&**store, &name).await {
        Ok((packet_bytes, zsk, packet)) => {
            let format = query.format.as_deref();

            if resolver::wants_payload(accept, format) {
                (
                    StatusCode::OK,
                    [("content-type", content_type::MNS_PAYLOAD)],
                    packet_bytes,
                )
                    .into_response()
            } else {
                let records = packet.all_resource_records();
                let ts = packet.timestamp().as_u64();
                let ns = store
                    .get_ns(&name)
                    .await
                    .unwrap_or(None)
                    .unwrap_or_default();
                let html_body = html::render_html(
                    &name,
                    owner.as_ref(),
                    &zsk,
                    &ns,
                    &records,
                    &hex::encode(&packet_bytes),
                    ts,
                );
                (StatusCode::OK, [("content-type", "text/html")], html_body).into_response()
            }
        }
        Err(
            err @ (resolver::ResolveError::NameNotFound | resolver::ResolveError::PacketNotFound),
        ) => {
            tracing::debug!(%name, error = %err, "GET returning 404");

            let accept = headers
                .get("accept")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("*/*");

            if accept == content_type::MNS_PAYLOAD
                || accept == "application/octet-stream"
                || query.format.as_deref() == Some("payload")
            {
                (
                    StatusCode::NOT_FOUND,
                    [("content-type", content_type::MNS_PAYLOAD)],
                    Vec::<u8>::new(),
                )
                    .into_response()
            } else {
                let zsk = store.get_zsk(&name).await.unwrap_or(None);
                let ns = store.get_ns(&name).await.unwrap_or(None);
                (
                    StatusCode::NOT_FOUND,
                    [("content-type", "text/html")],
                    html::render_not_found_page(&name, owner.as_ref(), zsk.as_ref(), ns.as_deref()),
                )
                    .into_response()
            }
        }
        Err(
            err @ (resolver::ResolveError::ZskMismatch | resolver::ResolveError::InvalidSignature),
        ) => {
            tracing::debug!(%name, error = %err, "GET evicting packet and returning 404");
            let _ = store.set_signed_packet(&name, &[]).await;
            let zsk = store.get_zsk(&name).await.unwrap_or(None);
            let ns = store.get_ns(&name).await.unwrap_or(None);
            (
                StatusCode::NOT_FOUND,
                [("content-type", "text/html")],
                html::render_not_found_page(&name, owner.as_ref(), zsk.as_ref(), ns.as_deref()),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("resolve error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error("internal server error"),
            )
                .into_response()
        }
    }
}

async fn put_handler<S: ZoneStore>(
    store: axum::extract::State<Arc<S>>,
    Path(name_str): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let name = match name_str.parse::<Name>() {
        Ok(n) => n,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                [("content-type", "text/html")],
                html::render_error("invalid mns name"),
            )
                .into_response()
        }
    };

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type != content_type::MNS_PAYLOAD && content_type != "application/octet-stream" {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            [("content-type", "text/html")],
            html::render_error(&format!(
                "unsupported content type, expected {} or application/octet-stream",
                content_type::MNS_PAYLOAD
            )),
        )
            .into_response();
    }

    match resolver::put_packet(&**store, &name, &body).await {
        Ok(()) => (StatusCode::NO_CONTENT,).into_response(),
        Err(resolver::ResolveError::InvalidSignature | resolver::ResolveError::Parse(_)) => (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/html")],
            html::render_error("invalid signature or malformed packet"),
        )
            .into_response(),
        Err(resolver::ResolveError::NameMismatch) => (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/html")],
            html::render_error("name in packet does not match URL"),
        )
            .into_response(),
        Err(resolver::ResolveError::NameNotFound) => (
            StatusCode::NOT_FOUND,
            [("content-type", "text/html")],
            html::render_error("name not registered on-chain"),
        )
            .into_response(),
        Err(resolver::ResolveError::ZskMismatch) => (
            StatusCode::FORBIDDEN,
            [("content-type", "text/html")],
            html::render_error("key not authorized for this name"),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("put error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error("internal server error"),
            )
                .into_response()
        }
    }
}
