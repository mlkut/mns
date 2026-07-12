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
use tower_http::services::ServeDir;

use crate::content_type;
use crate::html;
use crate::resolver;
use crate::store::ZoneStore;

#[derive(Deserialize)]
pub struct FormatQuery {
    format: Option<String>,
}

pub struct AppState<S: ZoneStore> {
    pub store: Arc<S>,
    pub network: String,
    pub explorer_url: String,
    pub contract_address: String,
    pub chain_id: u64,
    pub rpc_url: String,
}

pub fn build_router<S: ZoneStore + 'static>(
    store: Arc<S>,
    network: &str,
    explorer_url: &str,
    contract_address: &str,
    chain_id: u64,
    rpc_url: &str,
) -> Router {
    let state = Arc::new(AppState {
        store,
        network: network.to_string(),
        explorer_url: explorer_url.to_string(),
        contract_address: contract_address.to_string(),
        chain_id,
        rpc_url: rpc_url.to_string(),
    });

    Router::new()
        .route("/", get(root_handler))
        .route("/wallet", get(wallet_handler))
        .route("/stats", get(stats_handler::<S>))
        .route("/owners", get(owners_handler::<S>))
        .route("/owner/{address}", get(owner_handler::<S>))
        .route("/api/batches/{address}", get(batches_handler::<S>))
        .route("/{*name}", get(get_handler::<S>).put(put_handler::<S>))
        .nest_service(
            "/static",
            ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn nav_info<S: ZoneStore>(state: &AppState<S>) -> html::Navbar {
    let (sync_block, sync_time) = tokio::join!(
        state.store.get_last_sync_block_number(),
        state.store.get_last_sync_block_time(),
    );
    html::Navbar {
        sync_block: sync_block.unwrap_or(None).unwrap_or(0),
        sync_time: sync_time.unwrap_or(None).unwrap_or(0),
        network: state.network.clone(),
        explorer_url: state.explorer_url.clone(),
        contract_address: state.contract_address.clone(),
        chain_id: state.chain_id,
        rpc_url: state.rpc_url.clone(),
    }
}

#[derive(Serialize)]
struct StatsResponse {
    total_owners: u64,
    total_names: u64,
    total_packets: u64,
    total_ns: u64,
    total_zsks: u64,
    last_block: u64,
    last_block_time: u64,
}

async fn stats_handler<S: ZoneStore>(
    state: axum::extract::State<Arc<AppState<S>>>,
) -> Result<Json<StatsResponse>, (StatusCode, String)> {
    let (
        total_owners,
        total_batches,
        total_entries,
        total_packets,
        total_ns,
        total_zsks,
        last_block,
        last_block_time,
    ) = tokio::join!(
        state.store.total_owners(),
        state.store.total_batches(),
        state.store.total_entries(),
        state.store.total_packets(),
        state.store.total_ns(),
        state.store.total_zsks(),
        state.store.get_last_sync_block_number(),
        state.store.get_last_sync_block_time(),
    );
    let total_owners = total_owners.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    let total_batches = total_batches.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    let total_entries = total_entries.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    let total_packets = total_packets.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    let total_ns = total_ns.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    let total_zsks = total_zsks.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    let last_block = last_block.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    let last_block_time = last_block_time.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("store error: {e}"),
        )
    })?;
    Ok(Json(StatsResponse {
        total_owners,
        total_names: total_batches * 256 + total_entries,
        total_packets,
        total_ns,
        total_zsks,
        last_block: last_block.unwrap_or(0),
        last_block_time: last_block_time.unwrap_or(0),
    }))
}

const BATCH_SIZE: u64 = 256;

async fn owner_handler<S: ZoneStore>(
    state: axum::extract::State<Arc<AppState<S>>>,
    Path(address_str): Path<String>,
) -> Response {
    let address_str = address_str.trim();
    if !address_str.starts_with("0x") || address_str.len() != 42 {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/html")],
            html::render_error(
                "invalid address format (expected 0x-prefixed 20-byte hex)",
                &nav_info(&state).await,
            ),
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
                html::render_error("invalid address hex", &nav_info(&state).await),
            )
                .into_response()
        }
    };

    let nav = nav_info(&state).await;
    let batches = match state.store.get_owner_batches(&owner_bytes).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error(&format!("store error: {e}"), &nav),
            )
                .into_response()
        }
    };

    let mut names: Vec<Name> = Vec::new();
    for batch_start in &batches {
        for offset in 0..BATCH_SIZE {
            let ordinal = batch_start + offset;
            let name = Name::from_ordinal(ordinal);
            if let Some(entry_owner) = state.store.get_name_owner(&name).await.unwrap_or(None) {
                if entry_owner != owner_bytes {
                    continue;
                }
            }
            names.push(name);
        }
    }

    if let Ok(entries) = state.store.get_owner_entries(&owner_bytes).await {
        for ordinal in entries {
            let name = Name::from_ordinal(ordinal);
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }

    let html_body = html::render_owner_page(address_str, &names, &nav);
    (StatusCode::OK, [("content-type", "text/html")], html_body).into_response()
}

async fn owners_handler<S: ZoneStore>(state: axum::extract::State<Arc<AppState<S>>>) -> Response {
    let owners = match state.store.all_owners().await {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error(&format!("store error: {e}"), &nav_info(&state).await),
            )
                .into_response()
        }
    };

    let nav = nav_info(&state).await;
    let items: Vec<html::OwnerItemSimple> = owners
        .iter()
        .map(|addr| html::OwnerItemSimple {
            name_or_addr: format!("0x{}", hex::encode(addr)),
        })
        .collect();

    let html_body = html::render_owners_page(&items, &nav);
    (StatusCode::OK, [("content-type", "text/html")], html_body).into_response()
}

#[derive(Serialize)]
struct BatchInfo {
    ordinal: u64,
    zsk: String,
    ns: String,
}

async fn batches_handler<S: ZoneStore>(
    state: axum::extract::State<Arc<AppState<S>>>,
    Path(address_str): Path<String>,
) -> Response {
    let address_str = address_str.trim();
    if !address_str.starts_with("0x") || address_str.len() != 42 {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            r#"{"error":"invalid address format"}"#,
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
                [("content-type", "application/json")],
                r#"{"error":"invalid address hex"}"#,
            )
                .into_response()
        }
    };

    let batches = match state.store.get_owner_batches(&owner_bytes).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "application/json")],
                format!(r#"{{"error":"{e}"}}"#),
            )
                .into_response()
        }
    };

    let mut infos = Vec::new();
    for ordinal in batches {
        if let Ok(Some(config)) = state.store.get_batch(ordinal).await {
            infos.push(BatchInfo {
                ordinal,
                zsk: format!("0x{}", hex::encode(config.zsk)),
                ns: config.ns,
            });
        }
    }

    (
        StatusCode::OK,
        [("content-type", "application/json")],
        Json(infos),
    )
        .into_response()
}

async fn root_handler(
    state: axum::extract::State<Arc<AppState<impl ZoneStore>>>,
) -> impl axum::response::IntoResponse {
    let nav = nav_info(&state).await;
    (
        axum::http::StatusCode::OK,
        [("content-type", "text/html")],
        html::render_home_page(&nav),
    )
}

async fn wallet_handler(
    state: axum::extract::State<Arc<AppState<impl ZoneStore>>>,
) -> impl axum::response::IntoResponse {
    let nav = nav_info(&state).await;
    (
        axum::http::StatusCode::OK,
        [("content-type", "text/html")],
        html::render_wallet_page(&nav),
    )
}

async fn get_handler<S: ZoneStore>(
    state: axum::extract::State<Arc<AppState<S>>>,
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
                html::render_error("invalid mns name", &nav_info(&state).await),
            )
                .into_response()
        }
    };

    let accept = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*/*");

    tracing::debug!(%name, "GET handler");

    let owner = state.store.get_name_owner(&name).await.unwrap_or(None);

    match resolver::resolve_name(&*state.store, &name).await {
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
                let ns = state
                    .store
                    .get_ns(&name)
                    .await
                    .unwrap_or(None)
                    .unwrap_or_default();
                let nav = nav_info(&state).await;
                let html_body =
                    html::render_html(&name, owner.as_ref(), &zsk, &ns, &records, ts, &nav);
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
                let nav = nav_info(&state).await;
                let zsk = state.store.get_zsk(&name).await.unwrap_or(None);
                let ns = state.store.get_ns(&name).await.unwrap_or(None);
                (
                    StatusCode::NOT_FOUND,
                    [("content-type", "text/html")],
                    html::render_not_found_page(
                        &name,
                        owner.as_ref(),
                        zsk.as_ref(),
                        ns.as_deref(),
                        &nav,
                    ),
                )
                    .into_response()
            }
        }
        Err(
            err @ (resolver::ResolveError::ZskMismatch | resolver::ResolveError::InvalidSignature),
        ) => {
            tracing::debug!(%name, error = %err, "GET evicting packet and returning 404");
            let _ = state.store.remove_signed_packet(&name).await;
            let nav = nav_info(&state).await;
            let zsk = state.store.get_zsk(&name).await.unwrap_or(None);
            let ns = state.store.get_ns(&name).await.unwrap_or(None);
            (
                StatusCode::NOT_FOUND,
                [("content-type", "text/html")],
                html::render_not_found_page(
                    &name,
                    owner.as_ref(),
                    zsk.as_ref(),
                    ns.as_deref(),
                    &nav,
                ),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("resolve error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error("internal server error", &nav_info(&state).await),
            )
                .into_response()
        }
    }
}

async fn put_handler<S: ZoneStore>(
    state: axum::extract::State<Arc<AppState<S>>>,
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
                html::render_error("invalid mns name", &nav_info(&state).await),
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
            html::render_error(
                &format!(
                    "unsupported content type, expected {} or application/octet-stream",
                    content_type::MNS_PAYLOAD
                ),
                &nav_info(&state).await,
            ),
        )
            .into_response();
    }

    match resolver::put_packet(&*state.store, &name, &body).await {
        Ok(()) => (StatusCode::NO_CONTENT,).into_response(),
        Err(resolver::ResolveError::InvalidSignature | resolver::ResolveError::Parse(_)) => (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/html")],
            html::render_error(
                "invalid signature or malformed packet",
                &nav_info(&state).await,
            ),
        )
            .into_response(),
        Err(resolver::ResolveError::NameMismatch) => (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/html")],
            html::render_error("name in packet does not match URL", &nav_info(&state).await),
        )
            .into_response(),
        Err(resolver::ResolveError::NameNotFound) => (
            StatusCode::NOT_FOUND,
            [("content-type", "text/html")],
            html::render_error("name not registered on-chain", &nav_info(&state).await),
        )
            .into_response(),
        Err(resolver::ResolveError::ZskMismatch) => (
            StatusCode::FORBIDDEN,
            [("content-type", "text/html")],
            html::render_error("key not authorized for this name", &nav_info(&state).await),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("put error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/html")],
                html::render_error("internal server error", &nav_info(&state).await),
            )
                .into_response()
        }
    }
}
