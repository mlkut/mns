use mns::Name;
use mns::SignedPacket;
use mns::SignedPacketError;
use mns::ZSK_LEN;

use crate::content_type;
use crate::store::ZoneStore;

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("name not found")]
    NameNotFound,

    #[error("no signed packet cached")]
    PacketNotFound,

    #[error("signature verification failed")]
    InvalidSignature,

    #[error("zsk mismatch: key not authorized for this name")]
    ZskMismatch,

    #[error("name mismatch between url and packet")]
    NameMismatch,

    #[error("store error: {0}")]
    Store(String),

    #[error("parse error: {0}")]
    Parse(String),
}

/// Resolve a signed packet for a name: verify sig + ZSK match against on-chain registry.
pub fn resolve(
    packet_bytes: &[u8],
    expected_zsk: &[u8; ZSK_LEN],
) -> Result<(SignedPacket, [u8; ZSK_LEN]), ResolveError> {
    let packet = SignedPacket::verify(packet_bytes).map_err(|e| match e {
        SignedPacketError::InvalidSignature => ResolveError::InvalidSignature,
        _ => ResolveError::Parse(e.to_string()),
    })?;

    let zsk = packet.zsk();
    if &zsk != expected_zsk {
        return Err(ResolveError::ZskMismatch);
    }

    Ok((packet, zsk))
}

/// Full resolution pipeline: store lookup → verify → records.
pub async fn resolve_name<S: ZoneStore>(
    store: &S,
    name: &Name,
) -> Result<(Vec<u8>, [u8; ZSK_LEN], SignedPacket), ResolveError> {
    tracing::debug!(%name, "resolving name");

    let zsk = store
        .get_zsk(name)
        .await
        .map_err(|e| ResolveError::Store(e.to_string()))?
        .ok_or_else(|| {
            tracing::debug!(%name, "ZSK not found in store");
            ResolveError::NameNotFound
        })?;
    tracing::debug!(%name, zsk = %hex::encode(zsk), "found ZSK");

    let packet_bytes = store
        .get_signed_packet(name)
        .await
        .map_err(|e| ResolveError::Store(e.to_string()))?
        .ok_or_else(|| {
            tracing::debug!(%name, "signed packet not found in store");
            ResolveError::PacketNotFound
        })?;
    tracing::debug!(%name, packet_len = packet_bytes.len(), "found signed packet");

    match resolve(&packet_bytes, &zsk) {
        Ok((packet, _)) => {
            tracing::debug!(%name, "packet verified successfully");
            Ok((packet_bytes, zsk, packet))
        }
        Err(e) => {
            tracing::debug!(%name, error = %e, "packet verification failed");
            Err(e)
        }
    }
}

/// Verify and store an incoming signed packet.
pub async fn put_packet<S: ZoneStore>(
    store: &S,
    url_name: &Name,
    packet_bytes: &[u8],
) -> Result<(), ResolveError> {
    let packet = SignedPacket::verify(packet_bytes).map_err(|e| match e {
        SignedPacketError::InvalidSignature => ResolveError::InvalidSignature,
        _ => ResolveError::Parse(e.to_string()),
    })?;

    if packet.name() != *url_name {
        return Err(ResolveError::NameMismatch);
    }

    let expected_zsk = store
        .get_zsk(url_name)
        .await
        .map_err(|e| ResolveError::Store(e.to_string()))?
        .ok_or(ResolveError::NameNotFound)?;

    let zsk = packet.zsk();
    if zsk != expected_zsk {
        return Err(ResolveError::ZskMismatch);
    }

    store
        .set_signed_packet(url_name, packet_bytes)
        .await
        .map_err(|e| ResolveError::Store(e.to_string()))?;

    Ok(())
}

pub fn wants_payload(accept: &str, format: Option<&str>) -> bool {
    if let Some(f) = format {
        return f == "payload";
    }
    accept.contains(content_type::MNS_PAYLOAD) || accept == "application/octet-stream"
}
