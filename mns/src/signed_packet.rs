//! Signed packets: a set of DNS-style resource records, signed by a
//! type-prefixed key (see [crate::keys]) and linked to a [crate::Name].
//!
//! This is adapted from Pkarr's `SignedPacket`
//! (<https://crates.io/crates/pkarr-client/4.1.0/code/src/signed_packet.rs>),
//! with two changes:
//!
//! 1. **Type-prefixed key.** Pkarr always embeds a raw 32-byte Ed25519 key.
//!    Here the embedded key is `<1 byte key type><key bytes>`, so a verifier
//!    doesn't need to assume the key type ahead of time.
//!
//! 2. **Name-based origin.** Instead of normalising record names against a
//!    z-base-32 encoded public key, the origin here is the human-readable
//!    [crate::Name] (e.g. `mokomedu-tasosuna`) whose 40-bit internal value
//!    is embedded in the wire format. This lets [`resource_records`] filter
//!    and normalise names the same way Pkarr does, but against a name the
//!    user actually sees in their browser.
//!
//! Wire format (all integers big-endian):
//!
//! ```text
//! <1 byte key type><key bytes><signature bytes><5 byte name><8 byte timestamp><encoded dns packet>
//! ```
//!
//! `key bytes` and `signature bytes` both have lengths implied by the key
//! type ([KeyType::public_key_len], [KeyType::signature_len]), so the offset
//! of every field is computable once the type byte has been read.
//!
//! The signature covers everything after the signature itself in the wire
//! format — i.e. `name (5) || timestamp (8) || encoded dns packet`.

use crate::Name;
use crate::keys::{KeyError, KeyType, Keypair, PublicKey, ZSK_LEN};
use ntimestamp::Timestamp;
use simple_dns::{Name as DnsName, Packet, ResourceRecord, SimpleDnsError};
use std::{char, fmt::Debug};
use thiserror::Error;

/// Maximum size, in bytes, of the *encoded DNS packet* portion.
pub const MAX_ENCODED_PACKET_BYTES: usize = 1024;

const DOT: char = '.';

#[derive(Debug, Error)]
pub enum SignedPacketError {
    #[error(transparent)]
    Key(#[from] KeyError),

    #[error(transparent)]
    Dns(#[from] SimpleDnsError),

    #[error("buffer too short: expected at least {expected} bytes, got {got}")]
    TooShort { expected: usize, got: usize },

    #[error("encoded dns packet too large: {0} bytes (max {MAX_ENCODED_PACKET_BYTES})")]
    PacketTooLarge(usize),

    #[error("signature verification failed")]
    InvalidSignature,

    #[error("zsk mismatch: expected {expected:02x?}, got {got:02x?}")]
    ZskMismatch {
        expected: [u8; ZSK_LEN],
        got: [u8; ZSK_LEN],
    },

    #[error("a name is required to build a signed packet")]
    NameRequired,
}

/// A set of signed, DNS-style resource records linked to a [crate::Name].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedPacket {
    bytes: Vec<u8>,
    key_type: KeyType,
    pubkey_end: usize,
    sig_end: usize,
    timestamp_end: usize,
    name: Name,
}

impl SignedPacket {
    pub fn builder() -> SignedPacketBuilder {
        SignedPacketBuilder::default()
    }

    pub fn new(
        keypair: &Keypair,
        name: Name,
        records: &[ResourceRecord<'_>],
        timestamp: Timestamp,
    ) -> Result<Self, SignedPacketError> {
        let public_key = keypair.public_key();

        let normalized_names: Vec<String> = records
            .iter()
            .map(|record| normalize_record_name(name, &record.name.to_string()))
            .collect();

        let mut packet = Packet::new_reply(0);
        for (record, normalized_name) in records.iter().zip(normalized_names.iter()) {
            packet.answers.push(ResourceRecord::new(
                DnsName::new_unchecked(normalized_name).to_owned(),
                record.class,
                record.ttl,
                record.rdata.clone(),
            ));
        }

        let encoded_packet = packet.build_bytes_vec_compressed()?;
        if encoded_packet.len() > MAX_ENCODED_PACKET_BYTES {
            return Err(SignedPacketError::PacketTooLarge(encoded_packet.len()));
        }

        let name_wire = name.to_wire_bytes();
        let timestamp_bytes = timestamp.to_bytes();
        let signable = [&name_wire[..], &timestamp_bytes[..], &encoded_packet[..]].concat();
        let signature = keypair.sign(&signable);

        Self::assemble(&public_key, name, &signature, timestamp, &encoded_packet)
    }

    pub fn assemble(
        public_key: &PublicKey,
        name: Name,
        signature: &[u8],
        timestamp: Timestamp,
        encoded_packet: &[u8],
    ) -> Result<Self, SignedPacketError> {
        let key_type = public_key.key_type();

        if signature.len() != key_type.signature_len() {
            return Err(SignedPacketError::TooShort {
                expected: key_type.signature_len(),
                got: signature.len(),
            });
        }
        if encoded_packet.len() > MAX_ENCODED_PACKET_BYTES {
            return Err(SignedPacketError::PacketTooLarge(encoded_packet.len()));
        }

        let prefixed_key = public_key.to_prefixed_bytes();
        let name_wire = name.to_wire_bytes();

        let mut bytes = Vec::with_capacity(
            prefixed_key.len() + signature.len() + name_wire.len() + 8 + encoded_packet.len(),
        );
        bytes.extend_from_slice(&prefixed_key);
        bytes.extend_from_slice(signature);
        bytes.extend_from_slice(&name_wire);
        bytes.extend_from_slice(&timestamp.to_bytes());
        bytes.extend_from_slice(encoded_packet);

        let pubkey_end = prefixed_key.len();
        let sig_end = pubkey_end + signature.len();
        let timestamp_end = sig_end + name_wire.len() + 8;

        Packet::parse(&bytes[timestamp_end..])?;

        Ok(SignedPacket {
            bytes,
            key_type,
            pubkey_end,
            sig_end,
            timestamp_end,
            name,
        })
    }

    /// Parse and fully verify a [SignedPacket] from wire bytes.
    /// The ZSK is accessible via [`SignedPacket::zsk`] and the linked name
    /// via [`SignedPacket::name`].
    pub fn verify(bytes: &[u8]) -> Result<Self, SignedPacketError> {
        let public_key = PublicKey::from_prefixed_bytes(bytes)?;
        let key_type = public_key.key_type();
        let pubkey_end = 1 + key_type.public_key_len();

        let sig_end = pubkey_end + key_type.signature_len();
        let name_end = sig_end + 5;
        let timestamp_end = name_end + 8;
        if bytes.len() < timestamp_end {
            return Err(SignedPacketError::TooShort {
                expected: timestamp_end,
                got: bytes.len(),
            });
        }

        let signature = &bytes[pubkey_end..sig_end];

        public_key
            .verify(&bytes[sig_end..], signature)
            .map_err(|_| SignedPacketError::InvalidSignature)?;

        let name_wire: [u8; 5] = bytes[sig_end..name_end]
            .try_into()
            .expect("length checked above");
        let name = Name::from_wire_bytes(&name_wire);
        let encoded_packet = &bytes[timestamp_end..];

        if encoded_packet.len() > MAX_ENCODED_PACKET_BYTES {
            return Err(SignedPacketError::PacketTooLarge(encoded_packet.len()));
        }

        Packet::parse(encoded_packet)?;

        Ok(SignedPacket {
            bytes: bytes.to_vec(),
            key_type,
            pubkey_end,
            sig_end,
            timestamp_end,
            name,
        })
    }

    /// Parse and verify a [SignedPacket], then check that its ZSK matches
    /// `expected_zsk`.
    pub fn verify_for_zsk(
        bytes: &[u8],
        expected_zsk: &[u8; ZSK_LEN],
    ) -> Result<Self, SignedPacketError> {
        let packet = Self::verify(bytes)?;
        let zsk = packet.zsk();
        if &zsk != expected_zsk {
            return Err(SignedPacketError::ZskMismatch {
                expected: *expected_zsk,
                got: packet.zsk(),
            });
        }
        Ok(packet)
    }

    // === Getters ===

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn key_type(&self) -> KeyType {
        self.key_type
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_prefixed_bytes(&self.bytes[..self.pubkey_end])
            .expect("already validated at construction/verify time")
    }

    /// ZSK (Zone Signing Key hash) extracted from the embedded public key.
    ///
    /// See [`crate::keys::compute_zsk`] for the definition.
    pub fn zsk(&self) -> [u8; ZSK_LEN] {
        self.public_key().zsk()
    }

    /// The [crate::Name] this signed packet is linked to.
    pub fn name(&self) -> Name {
        self.name
    }

    pub fn signature(&self) -> &[u8] {
        &self.bytes[self.pubkey_end..self.sig_end]
    }

    pub fn timestamp(&self) -> Timestamp {
        Timestamp::from(u64::from_be_bytes(
            self.bytes[self.sig_end + 5..self.timestamp_end]
                .try_into()
                .expect("length fixed at construction"),
        ))
    }

    pub fn encoded_packet(&self) -> &[u8] {
        &self.bytes[self.timestamp_end..]
    }

    pub fn packet(&self) -> Packet<'_> {
        Packet::parse(self.encoded_packet()).expect("validated at construction/verify time")
    }

    /// Return the [ResourceRecord]s whose name matches the given name after
    /// normalising against this packet's [Name](crate::Name) origin.
    ///
    /// - `@` matches records at the zone apex (the name itself).
    /// - A relative name like `foo` matches `foo.<name>`.
    /// - A fully-qualified name like `foo.<name>` matches exactly.
    /// - Wildcards are supported: `*.foo` matches `bar.foo.<name>`.
    pub fn resource_records(&self, name: &str) -> Vec<ResourceRecord<'_>> {
        let normalized = normalize_record_name(self.name, name);
        let is_wildcard = normalized.starts_with('*');

        self.packet()
            .answers
            .into_iter()
            .filter(|rr| {
                if is_wildcard {
                    rr.name
                        .to_string()
                        .strip_suffix(&normalized[1..])
                        .map(|m| !m.contains(DOT))
                        .unwrap_or_default()
                } else {
                    rr.name.to_string() == normalized
                }
            })
            .collect()
    }

    pub fn all_resource_records(&self) -> Vec<ResourceRecord<'_>> {
        self.packet().answers
    }
}

#[derive(Debug, Clone, Default)]
pub struct SignedPacketBuilder {
    records: Vec<ResourceRecord<'static>>,
    timestamp: Option<Timestamp>,
    name: Option<Name>,
}

impl SignedPacketBuilder {
    /// Insert a [ResourceRecord]
    pub fn record(mut self, record: ResourceRecord<'_>) -> Self {
        self.records.push(record.into_owned());
        self
    }

    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn name(mut self, name: Name) -> Self {
        self.name = Some(name);
        self
    }

    pub fn sign(self, keypair: &Keypair) -> Result<SignedPacket, SignedPacketError> {
        let timestamp = self.timestamp.unwrap_or_else(Timestamp::now);
        let name = self.name.ok_or(SignedPacketError::NameRequired)?;
        SignedPacket::new(keypair, name, &self.records, timestamp)
    }
}

/// Normalizes any incoming query string against this zone context.
/// Safely handles arbitrary subdomains, multiple `.mns` tokens, relative syntax,
/// and varying `.mns.*` TLD extensions by anchoring to the rightmost valid Name match.
fn normalize_record_name(signed_packet_name: Name, record_name: &str) -> String {
    let canonical_apex = signed_packet_name.canonical_domain();
    let name_str = signed_packet_name.to_string();

    if record_name == "@" || record_name.is_empty() {
        return canonical_apex;
    }

    let bytes = record_name.as_bytes();
    let zone_bytes = name_str.as_bytes();
    let mut match_start = None;

    // Scan right-to-left to find the rightmost occurrence of our valid Name token
    for start in (0..bytes.len().saturating_sub(16)).rev() {
        if &bytes[start..start + 17] == zone_bytes {
            // 1. Strict Left Boundary Check (Start of string or preceded by a dot)
            let left_boundary = start == 0 || bytes[start - 1] == b'.';

            // 2. Strict Right Boundary Check (End of string or followed by a dot)
            let right_idx = start + 17;
            let right_boundary = right_idx == bytes.len() || bytes[right_idx] == b'.';

            if left_boundary && right_boundary {
                match_start = Some(start);
                break;
            }
        }
    }

    match match_start {
        Some(start) => {
            // If there's a prefix before the name token (e.g., "_foo.name..."), preserve it
            if start > 0 {
                let prefix = &record_name[..start]; // includes the trailing dot
                format!("{}{}", prefix, canonical_apex)
            } else {
                canonical_apex
            }
        }
        None => {
            // No matching zone token found. Treat as a pure relative query (e.g., "_foo")
            if record_name.ends_with(DOT) {
                format!("{}{}", record_name, canonical_apex)
            } else {
                format!("{}.{}", record_name, canonical_apex)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::Keypair;
    use simple_dns::{CLASS, rdata::RData};

    fn test_keypair() -> Keypair {
        Keypair::from_ed25519_bytes([9u8; 32])
    }

    fn test_name() -> Name {
        Name::from_raw(0x4A3B5C1D2E)
    }

    #[test]
    fn build_and_verify_roundtrip() {
        let keypair = test_keypair();
        let name = test_name();

        let signed = SignedPacket::builder()
            .name(name)
            .record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("hello".try_into().unwrap()),
            ))
            .timestamp(Timestamp::from(1))
            .sign(&keypair)
            .unwrap();

        assert_eq!(signed.key_type(), KeyType::Ed25519);
        assert_eq!(signed.zsk(), keypair.zsk());
        assert_eq!(signed.name(), name);

        let origin = name.canonical_domain();
        assert_eq!(
            signed.packet().answers[0].name.to_string(),
            format!("_foo.{origin}")
        );

        let verified = SignedPacket::verify(signed.as_bytes()).unwrap();
        assert_eq!(verified, signed);
        assert_eq!(verified.zsk(), keypair.zsk());
        assert_eq!(verified.name(), name);

        let records = verified.resource_records("_foo");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn tampered_bytes_fail_verification() {
        let keypair = test_keypair();
        let signed = SignedPacket::builder()
            .name(test_name())
            .record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("hello".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        let mut tampered = signed.as_bytes().to_vec();
        let last = tampered.len() - 1;
        tampered[last] ^= 0xFF;

        let err = SignedPacket::verify(&tampered).unwrap_err();
        assert!(matches!(
            err,
            SignedPacketError::InvalidSignature | SignedPacketError::Dns(_)
        ));
    }

    #[test]
    fn rejects_unknown_key_type_byte() {
        let mut bytes = vec![42u8];
        bytes.extend_from_slice(&[0u8; 32 + 64 + 8 + 5]);
        let err = SignedPacket::verify(&bytes).unwrap_err();
        assert!(matches!(
            err,
            SignedPacketError::Key(KeyError::UnsupportedKeyType(42))
        ));
    }

    #[test]
    fn verify_for_zsk_ok() {
        let keypair = test_keypair();
        let signed = SignedPacket::builder()
            .name(test_name())
            .record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("hello".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        let expected = keypair.zsk();
        let verified = SignedPacket::verify_for_zsk(signed.as_bytes(), &expected).unwrap();
        assert_eq!(verified, signed);
    }

    #[test]
    fn verify_for_zsk_mismatch() {
        let keypair = test_keypair();
        let signed = SignedPacket::builder()
            .name(test_name())
            .record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("hello".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        let wrong = [0u8; ZSK_LEN];
        let err = SignedPacket::verify_for_zsk(signed.as_bytes(), &wrong).unwrap_err();
        assert!(matches!(err, SignedPacketError::ZskMismatch { .. }));
    }

    #[test]
    fn resource_records_with_at_sign() {
        let keypair = test_keypair();
        let name = test_name();
        let signed = SignedPacket::builder()
            .name(name)
            .record(ResourceRecord::new(
                DnsName::new_unchecked("@").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("apex".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        let records = signed.resource_records("@");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn resource_records_with_relative_name() {
        let keypair = test_keypair();
        let name = test_name();
        let signed = SignedPacket::builder()
            .name(name)
            .record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("bar".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        let records = signed.resource_records("_foo");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn resource_records_with_fully_qualified_name() {
        let keypair = test_keypair();
        let name = test_name();
        let origin = name.encode();
        let signed = SignedPacket::builder()
            .name(name)
            .record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("bar".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        let records = signed.resource_records(&format!("_foo.{origin}"));
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn resource_records_with_wildcard() {
        let keypair = test_keypair();
        let name = test_name();
        let signed = SignedPacket::builder()
            .name(name)
            .record(ResourceRecord::new(
                DnsName::new_unchecked("*.foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("wild".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        assert_eq!(signed.resource_records("*.foo").len(), 1);
    }

    #[test]
    fn name_required_in_builder() {
        let keypair = test_keypair();
        let err = SignedPacket::builder()
            .record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("x".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap_err();
        assert!(matches!(err, SignedPacketError::NameRequired));
    }

    #[test]
    fn name_roundtrip_via_wire() {
        let name = test_name();
        let wire = name.to_wire_bytes();
        let decoded = Name::from_wire_bytes(&wire);
        assert_eq!(decoded, name);
    }

    #[test]
    fn custom_timestamp() {
        let ts = Timestamp::from(42);
        let signed = SignedPacket::builder()
            .name(test_name())
            .timestamp(ts)
            .sign(&test_keypair())
            .unwrap();
        assert_eq!(signed.timestamp(), ts);
    }

    #[test]
    fn from_too_large_packet() {
        let keypair = test_keypair();
        let mut builder = SignedPacket::builder().name(test_name());
        // Each TXT record is ~30 bytes; 100 of them exceeds MAX_ENCODED_PACKET_BYTES (1024).
        for _ in 0..100 {
            builder = builder.record(ResourceRecord::new(
                DnsName::new_unchecked("_foo").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("x".try_into().unwrap()),
            ));
        }
        let err = builder.sign(&keypair).unwrap_err();
        assert!(matches!(err, SignedPacketError::PacketTooLarge(_)));
    }

    #[test]
    fn wildcards_multi_label() {
        let keypair = test_keypair();
        let origin = test_name();
        let signed = SignedPacket::builder()
            .name(origin)
            .record(ResourceRecord::new(
                DnsName::new_unchecked("bar.foo.").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("match".try_into().unwrap()),
            ))
            .record(ResourceRecord::new(
                DnsName::new_unchecked("x.bar.foo.").into_owned(),
                CLASS::IN,
                30,
                RData::TXT("no-match".try_into().unwrap()),
            ))
            .record(ResourceRecord::new(
                DnsName::new_unchecked("foo.").into_owned(),
                CLASS::IN,
                60,
                RData::TXT("apex".try_into().unwrap()),
            ))
            .sign(&keypair)
            .unwrap();

        // `*.foo.` should only match `bar.foo.<origin>` (single label before `foo`)
        assert_eq!(signed.resource_records("*.foo").len(), 1);
    }

    #[test]
    fn compressed_names() {
        let keypair = test_keypair();
        let origin = test_name();
        let signed = SignedPacket::builder()
            .name(origin)
            .record(ResourceRecord::new(
                DnsName::new_unchecked(".").into_owned(),
                CLASS::IN,
                30,
                RData::CNAME(DnsName::new_unchecked("foobar").into()),
            ))
            .record(ResourceRecord::new(
                DnsName::new_unchecked(".").into_owned(),
                CLASS::IN,
                30,
                RData::CNAME(DnsName::new_unchecked("foobar").into()),
            ))
            .sign(&keypair)
            .unwrap();

        let records = signed.resource_records("@");
        assert_eq!(records.len(), 2);
        for r in records {
            assert!(matches!(r.rdata, RData::CNAME(_)));
        }
    }

    #[test]
    fn test_normalize_record_name_complex() {
        let zone: Name = "mokomedu-tasosuna".parse().unwrap();

        // 1. Bare relative queries
        assert_eq!(
            normalize_record_name(zone, "@"),
            "mokomedu-tasosuna.mns.alt"
        );
        assert_eq!(
            normalize_record_name(zone, "_foo"),
            "_foo.mokomedu-tasosuna.mns.alt"
        );

        // 2. Standard queries matching various suffixes
        assert_eq!(
            normalize_record_name(zone, "_foo.mokomedu-tasosuna.mns.alt"),
            "_foo.mokomedu-tasosuna.mns.alt"
        );
        assert_eq!(
            normalize_record_name(zone, "_foo.mokomedu-tasosuna.mns.mlkut.org"),
            "_foo.mokomedu-tasosuna.mns.alt"
        );
        assert_eq!(
            normalize_record_name(zone, "mokomedu-tasosuna.mns"),
            "mokomedu-tasosuna.mns.alt"
        );

        // 3. Vulnerability Edge Case: Multiple `.mns` or deceptive subdomains
        // A subdomain containing ".mns" before the true apex
        assert_eq!(
            normalize_record_name(zone, "mns-service.foo.mokomedu-tasosuna.mns.mlkut.org"),
            "mns-service.foo.mokomedu-tasosuna.mns.alt"
        );

        // Another valid name structure acting as a subdomain prefix
        assert_eq!(
            normalize_record_name(zone, "sikuteby-natubeku.mns.foo.mokomedu-tasosuna.mns.alt"),
            "sikuteby-natubeku.mns.foo.mokomedu-tasosuna.mns.alt"
        );
        assert_eq!(
            normalize_record_name(zone, "_foo.mokomedu-tasosuna.mns.mnsfoo.mlkut.org"),
            "_foo.mokomedu-tasosuna.mns.alt"
        );

        // 4. Invalid Name before the right most mns.
        // shorter than 17 characters
        assert_eq!(
            normalize_record_name(zone, "_foo.mokomedu-tasosuna.mns.tasosuna.mns.mlkut.org"),
            "_foo.mokomedu-tasosuna.mns.alt"
        );
        // longer than 17 characters
        assert_eq!(
            normalize_record_name(
                zone,
                "_foo.mokomedu-tasosuna.mns.mokomedu-tasosunatasosuna.mns.mlkut.org"
            ),
            "_foo.mokomedu-tasosuna.mns.alt"
        );
        // no dash in the middle
        assert_eq!(
            normalize_record_name(
                zone,
                "_foo.mokomedu-tasosuna.mns.mokomedu_tasosuna.mns.mlkut.org"
            ),
            "_foo.mokomedu-tasosuna.mns.alt"
        );
        // not valid name
        assert_eq!(
            normalize_record_name(
                zone,
                "_foo.mokomedu-tasosuna.mns.mokomedu-tasofoob.mns.mlkut.org"
            ),
            "_foo.mokomedu-tasosuna.mns.alt"
        );
    }
}
