//! Type-prefixed public keys, supporting multiple key algorithms.
//!
//! Pkarr (<https://github.com/nuhvi/pkarr>) hard-codes Ed25519: a public key
//! is always exactly 32 bytes, and the z-base32 encoding of those 32 bytes
//! *is* the identity/domain name. We don't want that assumption baked in,
//! since we won't be using Ed25519 exclusively going forward. The prefix
//! scheme exists so that a future post-quantum signature scheme (e.g. SLH-DSA,
//! ML-DSA) can be added without changing existing key wire formats.
//!
//! **This is purely offchain.** The on-chain MNS registry is agnostic to key
//! type — it stores an opaque `bytes` value for each name, and the offchain
//! resolver is responsible for interpreting the prefix byte.
//!
//! Instead, every key here is serialized as:
//!
//! ```text
//! <1 byte key type><key bytes, length depends on key type>
//! ```
//!
//! The type byte is read first, and it alone determines how many following
//! bytes make up the key. `KeyType::Ed25519 = 0` is the default and the only
//! variant implemented today; adding a second key type later means adding a
//! new match arm here, not changing the wire format of existing keys.
//!
//! Because different key types can have different native lengths, they can't
//! be used directly as a fixed-size identity/domain name the way Pkarr uses
//! the raw Ed25519 key. Instead we derive a **ZSK** (zone signing key hash):
//!
//! ```text
//! zsk = SHA256(key_type_byte || raw_key_bytes)
//! ```
//!
//! which gives every key type, regardless of its length, the same 32-byte
//! identity shape. [SignedPacket](crate::signed_packet::SignedPacket)
//! verification recomputes this hash from the embedded key and uses it as
//! the record-naming origin, so a party can't present one key while claiming
//! the identity of another.

use ed25519_dalek::{Signature, SignatureError, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Byte length of a ZSK (Zone Signing Key hash) — always 32 bytes (SHA-256 output).
///
/// The ZSK binds a public key to a DNS zone without assuming the key's algorithm
/// or length. It is computed as `SHA256(key_type_byte || raw_key_bytes)`, giving every
/// key type the same 32-byte identity shape regardless of its native key size.
pub const ZSK_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("unsupported key type byte: {0}")]
    UnsupportedKeyType(u8),

    #[error("key bytes too short: expected at least {expected} bytes, got {got}")]
    InvalidLength { expected: usize, got: usize },

    #[error(transparent)]
    Ed25519(#[from] SignatureError),
}

/// The algorithm a key belongs to. The discriminant is the wire-format
/// prefix byte, so `KeyType::Ed25519 as u8 == 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KeyType {
    /// Default key type. Raw public key is 32 bytes, signatures are 64 bytes.
    Ed25519 = 0,
}

impl KeyType {
    pub fn from_byte(byte: u8) -> Result<Self, KeyError> {
        match byte {
            0 => Ok(KeyType::Ed25519),
            other => Err(KeyError::UnsupportedKeyType(other)),
        }
    }

    pub fn as_byte(&self) -> u8 {
        *self as u8
    }

    pub fn public_key_len(&self) -> usize {
        match self {
            KeyType::Ed25519 => 32,
        }
    }

    pub fn signature_len(&self) -> usize {
        match self {
            KeyType::Ed25519 => 64,
        }
    }
}

/// A public key tagged with its [KeyType].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicKey {
    Ed25519(VerifyingKey),
}

impl PublicKey {
    pub fn key_type(&self) -> KeyType {
        match self {
            PublicKey::Ed25519(_) => KeyType::Ed25519,
        }
    }

    pub fn key_bytes(&self) -> Vec<u8> {
        match self {
            PublicKey::Ed25519(vk) => vk.to_bytes().to_vec(),
        }
    }

    pub fn to_prefixed_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.key_type().public_key_len());
        out.push(self.key_type().as_byte());
        out.extend(self.key_bytes());
        out
    }

    pub fn from_prefixed_bytes(bytes: &[u8]) -> Result<(Self, usize), KeyError> {
        let type_byte = *bytes.first().ok_or(KeyError::InvalidLength {
            expected: 1,
            got: 0,
        })?;
        let key_type = KeyType::from_byte(type_byte)?;

        let needed = 1 + key_type.public_key_len();
        if bytes.len() < needed {
            return Err(KeyError::InvalidLength {
                expected: needed,
                got: bytes.len(),
            });
        }

        match key_type {
            KeyType::Ed25519 => {
                let raw: [u8; 32] = bytes[1..33].try_into().expect("checked length above");
                let vk = VerifyingKey::from_bytes(&raw)?;
                Ok((PublicKey::Ed25519(vk), needed))
            }
        }
    }

    pub fn verify(&self, message: &[u8], signature_bytes: &[u8]) -> Result<(), KeyError> {
        match self {
            PublicKey::Ed25519(vk) => {
                let expected_len = KeyType::Ed25519.signature_len();
                let sig_bytes: [u8; 64] =
                    signature_bytes
                        .try_into()
                        .map_err(|_| KeyError::InvalidLength {
                            expected: expected_len,
                            got: signature_bytes.len(),
                        })?;
                let sig = Signature::from_bytes(&sig_bytes);
                vk.verify_strict(message, &sig)?;
                Ok(())
            }
        }
    }

    /// ZSK (Zone Signing Key hash) for this public key.
    ///
    /// See [`compute_zsk`] for details.
    pub fn zsk(&self) -> [u8; ZSK_LEN] {
        compute_zsk(self.key_type(), &self.key_bytes())
    }
}

/// Compute the ZSK (Zone Signing Key hash) for a given key type and raw key bytes.
///
/// The ZSK is defined as:
/// ```text
/// ZSK = SHA256(key_type_byte || raw_key_bytes)
/// ```
/// It always produces a 32-byte output regardless of the input key type or length.
/// This allows different cryptographic algorithms to share a uniform identity format
/// for DNS zone binding.
pub fn compute_zsk(key_type: KeyType, key_bytes: &[u8]) -> [u8; ZSK_LEN] {
    let mut hasher = Sha256::new();
    hasher.update([key_type.as_byte()]);
    hasher.update(key_bytes);
    hasher.finalize().into()
}

/// A keypair tagged with its [KeyType], used for signing.
pub enum Keypair {
    Ed25519(SigningKey),
}

impl Keypair {
    pub fn from_ed25519(signing_key: SigningKey) -> Self {
        Keypair::Ed25519(signing_key)
    }

    pub fn from_ed25519_bytes(seed: [u8; 32]) -> Self {
        Keypair::Ed25519(SigningKey::from_bytes(&seed))
    }

    pub fn key_type(&self) -> KeyType {
        match self {
            Keypair::Ed25519(_) => KeyType::Ed25519,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        match self {
            Keypair::Ed25519(sk) => PublicKey::Ed25519(sk.verifying_key()),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        match self {
            Keypair::Ed25519(sk) => {
                use ed25519_dalek::Signer;
                sk.sign(message).to_bytes().to_vec()
            }
        }
    }

    /// ZSK (Zone Signing Key hash) for this keypair.
    ///
    /// Delegates to [`PublicKey::zsk`].
    pub fn zsk(&self) -> [u8; ZSK_LEN] {
        self.public_key().zsk()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keypair() -> Keypair {
        Keypair::from_ed25519_bytes([7u8; 32])
    }

    #[test]
    fn prefixed_roundtrip() {
        let keypair = test_keypair();
        let public_key = keypair.public_key();

        let prefixed = public_key.to_prefixed_bytes();
        assert_eq!(prefixed.len(), 33);
        assert_eq!(prefixed[0], KeyType::Ed25519.as_byte());

        let (parsed, consumed) = PublicKey::from_prefixed_bytes(&prefixed).unwrap();
        assert_eq!(consumed, 33);
        assert_eq!(parsed, public_key);
    }

    #[test]
    fn rejects_unknown_key_type() {
        let mut bytes = vec![1u8];
        bytes.extend_from_slice(&[0u8; 32]);
        let err = PublicKey::from_prefixed_bytes(&bytes).unwrap_err();
        assert!(matches!(err, KeyError::UnsupportedKeyType(1)));
    }

    #[test]
    fn zsk_is_deterministic_and_type_bound() {
        let keypair = test_keypair();
        let zsk_a = keypair.zsk();
        let zsk_b = keypair.public_key().zsk();
        assert_eq!(zsk_a, zsk_b);

        let key_bytes = keypair.public_key().key_bytes();
        let zsk_ed25519 = compute_zsk(KeyType::Ed25519, &key_bytes);
        assert_eq!(zsk_ed25519, zsk_a);
    }

    #[test]
    fn signature_roundtrip() {
        let keypair = test_keypair();
        let message = b"mns test message";
        let signature = keypair.sign(message);
        assert_eq!(signature.len(), KeyType::Ed25519.signature_len());

        keypair.public_key().verify(message, &signature).unwrap();

        let mut tampered = message.to_vec();
        tampered.push(0);
        assert!(keypair.public_key().verify(&tampered, &signature).is_err());
    }
}
