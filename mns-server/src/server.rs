use std::sync::Arc;

use alloy::network::Ethereum;
use alloy::primitives::{keccak256, Address, FixedBytes};
use alloy::providers::{Provider, RootProvider};
use alloy::sol_types::SolCall;
use axum::{
    extract::Path,
    extract::Query,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use bindings::mns_registry::MNSRegistry::registerCall;
use mns::Name;
use serde::Deserialize;
use serde::Serialize;
use tower_http::cors::CorsLayer;
use url::Url;

use crate::content_type;
use crate::html;
use crate::resolver;
use crate::store::ZoneStore;

#[derive(Deserialize)]
pub struct FormatQuery {
    format: Option<String>,
}

fn rlp_encode_bytes(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() == 1 && bytes[0] <= 0x7f {
        return bytes.to_vec();
    }
    if bytes.len() <= 55 {
        let mut out = vec![0x80 + bytes.len() as u8];
        out.extend_from_slice(bytes);
        return out;
    }
    let len_be = bytes.len().to_be_bytes();
    let zero_count = len_be.iter().position(|&b| b != 0).unwrap_or(8);
    let len_slice = &len_be[zero_count..];
    let mut out = vec![0xb7 + len_slice.len() as u8];
    out.extend_from_slice(len_slice);
    out.extend_from_slice(bytes);
    out
}

fn rlp_encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = Vec::new();
    for item in items {
        payload.extend_from_slice(item);
    }
    if payload.len() <= 55 {
        let mut out = vec![0xc0 + payload.len() as u8];
        out.extend_from_slice(&payload);
        return out;
    }
    let len_be = payload.len().to_be_bytes();
    let zero_count = len_be.iter().position(|&b| b != 0).unwrap_or(8);
    let len_slice = &len_be[zero_count..];
    let mut out = vec![0xf7 + len_slice.len() as u8];
    out.extend_from_slice(len_slice);
    out.extend_from_slice(&payload);
    out
}

fn encode_u128(val: u128) -> Vec<u8> {
    if val == 0 {
        return rlp_encode_bytes(&[]);
    }
    let be = val.to_be_bytes();
    let start = be.iter().position(|&b| b != 0).unwrap_or(16);
    rlp_encode_bytes(&be[start..])
}

fn build_unsigned_tx(
    nonce: u64,
    gas_price: u128,
    gas_limit: u64,
    to: &[u8],
    value: &[u8],
    data: &[u8],
    chain_id: u64,
) -> Vec<u8> {
    let items = vec![
        encode_u128(nonce as u128),
        encode_u128(gas_price),
        encode_u128(gas_limit as u128),
        rlp_encode_bytes(to),
        rlp_encode_bytes(value),
        rlp_encode_bytes(data),
        encode_u128(chain_id as u128),
        rlp_encode_bytes(&[]),
        rlp_encode_bytes(&[]),
    ];
    rlp_encode_list(&items)
}

fn build_signed_tx(
    nonce: u64,
    gas_price: u128,
    gas_limit: u64,
    to: &[u8],
    value: &[u8],
    data: &[u8],
    v: u64,
    r: &[u8; 32],
    s: &[u8; 32],
) -> Vec<u8> {
    let items = vec![
        encode_u128(nonce as u128),
        encode_u128(gas_price),
        encode_u128(gas_limit as u128),
        rlp_encode_bytes(to),
        rlp_encode_bytes(value),
        rlp_encode_bytes(data),
        encode_u128(v as u128),
        rlp_encode_bytes(r),
        rlp_encode_bytes(s),
    ];
    rlp_encode_list(&items)
}

fn hex_to_u64(s: &str) -> Result<u64, (StatusCode, String)> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).map_err(|_| (StatusCode::BAD_REQUEST, "invalid u64 hex".into()))
}

fn hex_to_u128(s: &str) -> Result<u128, (StatusCode, String)> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(s, 16).map_err(|_| (StatusCode::BAD_REQUEST, "invalid u128 hex".into()))
}

fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, (StatusCode, String)> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).map_err(|_| (StatusCode::BAD_REQUEST, "invalid hex bytes".into()))
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
        .route("/static/{file}", get(static_handler))
        .route("/avatar/{name}", get(avatar_handler::<S>))
        .route("/stats", get(stats_handler::<S>))
        .route("/owners", get(owners_handler::<S>))
        .route("/owner/{address}", get(owner_handler::<S>))
        .route("/tx/prepare-register", post(prepare_register_handler::<S>))
        .route("/tx/send-register", post(send_register_handler::<S>))
        .route("/{*name}", get(get_handler::<S>).put(put_handler::<S>))
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

#[derive(Deserialize)]
struct PrepareRegisterReq {
    ns: String,
    zsk_commitment: String,
    sender: String,
}

#[derive(Serialize)]
struct PrepareRegisterRes {
    nonce: String,
    gas_price: String,
    gas_limit: String,
    to: String,
    data: String,
    chain_id: u64,
    tx_hash: String,
}

#[derive(Deserialize)]
struct SendRegisterReq {
    nonce: String,
    gas_price: String,
    gas_limit: String,
    to: String,
    data: String,
    v: String,
    r: String,
    s: String,
}

#[derive(Serialize)]
struct SendRegisterRes {
    tx_hash: String,
}

async fn prepare_register_handler<S: ZoneStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(req): Json<PrepareRegisterReq>,
) -> Result<Json<PrepareRegisterRes>, (StatusCode, String)> {
    let zsk_hex = req.zsk_commitment.strip_prefix("0x").unwrap_or(&req.zsk_commitment);
    let zsk_bytes: [u8; 32] = hex::decode(zsk_hex)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid zsk commitment hex".into()))?
        .try_into()
        .map_err(|_| (StatusCode::BAD_REQUEST, "zsk commitment must be 32 bytes".into()))?;
    let zsk = FixedBytes::from(zsk_bytes);

    let sender: Address = req
        .sender
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid sender address".into()))?;

    let to_addr: Address = state
        .contract_address
        .parse()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "invalid contract address".into()))?;

    let call = registerCall { zsk, ns: req.ns.clone() };
    let calldata = call.abi_encode();

    let url = Url::parse(&state.rpc_url)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "invalid rpc url".into()))?;
    let provider: RootProvider<Ethereum> = RootProvider::new_http(url);

    let nonce: u64 = provider
        .get_transaction_count(sender)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("rpc error (nonce): {e}")))?;

    let gas_price: u128 = provider
        .get_gas_price()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("rpc error (gas price): {e}")))?;

    let gas_limit: u64 = 300_000;

    let unsigned_rlp = build_unsigned_tx(
        nonce,
        gas_price,
        gas_limit,
        to_addr.as_ref(),
        &[],
        &calldata,
        state.chain_id,
    );
    let tx_hash = keccak256(&unsigned_rlp);

    Ok(Json(PrepareRegisterRes {
        nonce: format!("{nonce:#x}"),
        gas_price: format!("{gas_price:#x}"),
        gas_limit: format!("{gas_limit:#x}"),
        to: format!("{to_addr:#x}"),
        data: format!("0x{}", hex::encode(&calldata)),
        chain_id: state.chain_id,
        tx_hash: format!("{tx_hash:#x}"),
    }))
}

async fn send_register_handler<S: ZoneStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(req): Json<SendRegisterReq>,
) -> Result<Json<SendRegisterRes>, (StatusCode, String)> {
    let nonce = hex_to_u64(&req.nonce)?;
    let gas_price = hex_to_u128(&req.gas_price)?;
    let gas_limit = hex_to_u64(&req.gas_limit)?;
    let to_bytes = parse_hex_bytes(&req.to)?;
    let data = parse_hex_bytes(&req.data)?;
    let v_bytes = parse_hex_bytes(&req.v)?;
    let r_vec = parse_hex_bytes(&req.r)?;
    let s_vec = parse_hex_bytes(&req.s)?;

    let v = if v_bytes.is_empty() {
        0u64
    } else {
        let mut buf = [0u8; 8];
        let offset = 8 - v_bytes.len();
        buf[offset..].copy_from_slice(&v_bytes);
        u64::from_be_bytes(buf)
    };

    if r_vec.len() != 32 || s_vec.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, "r and s must be 32 bytes".into()));
    }
    let mut r_arr = [0u8; 32];
    r_arr.copy_from_slice(&r_vec);
    let mut s_arr = [0u8; 32];
    s_arr.copy_from_slice(&s_vec);

    let signed_rlp = build_signed_tx(
        nonce, gas_price, gas_limit,
        &to_bytes, &[], &data,
        v, &r_arr, &s_arr,
    );

    let url = Url::parse(&state.rpc_url)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "invalid rpc url".into()))?;
    let provider: RootProvider<Ethereum> = RootProvider::new_http(url);

    let pending = provider
        .send_raw_transaction(&signed_rlp)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("rpc error: {e}")))?;
    let tx_hash = *pending.tx_hash();

    Ok(Json(SendRegisterRes {
        tx_hash: format!("{tx_hash:#x}"),
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
    _state: axum::extract::State<Arc<AppState<S>>>,
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
            let _ = state.store.set_signed_packet(&name, &[]).await;
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
