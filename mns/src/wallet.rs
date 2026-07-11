use hmac::{Hmac, Mac};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::elliptic_curve::PrimeField;
use k256::SecretKey;
use sha2::{Digest, Sha256};
use tiny_keccak::Hasher;

use crate::keys::{compute_zsk, KeyType};

type HmacSha512 = Hmac<sha2_10::Sha512>;

/// RSK coin type for BIP44.
const RSK_COIN_TYPE: u32 = 37310;

/// Full keys derived from a BIP39 mnemonic.
#[derive(Debug, Clone)]
pub struct WalletInfo {
    /// RSKIP-60 checksummed Rootstock address.
    pub address: String,
    /// secp256k1 private key (32 bytes).
    pub private_key: [u8; 32],
    /// ed25519 ZSK public key (32 bytes, displayed as hex).
    pub zsk_pub: [u8; 32],
    /// ZSK commitment for on-chain storage: SHA256(0x00 || zsk_pub).
    pub zsk_commitment: [u8; 32],
}

/// Generate a new 12-word BIP39 mnemonic.
pub fn generate_mnemonic() -> String {
    let mut entropy = [0u8; 16];
    getrandom::getrandom(&mut entropy).expect("getrandom should work");
    let m = bip39::Mnemonic::from_entropy(&entropy).expect("valid entropy");
    m.to_string()
}

/// Validate a BIP39 mnemonic phrase.
pub fn validate_mnemonic(phrase: &str) -> bool {
    bip39::Mnemonic::parse_in_normalized(bip39::Language::English, phrase).is_ok()
}

/// Derive a full [`WalletInfo`] from a BIP39 mnemonic.
///
/// Derivation paths:
/// - Rootstock secp256k1: `m/44'/37310'/0'/0/0` (BIP44)
/// - ZSK ed25519: `SHA256(seed || "mns-zsk")` → ed25519 public key
/// - ZSK commitment: `SHA256(0x00 || zsk_pub)` (stored on-chain)
pub fn derive_wallet(phrase: &str) -> Result<WalletInfo, &'static str> {
    let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, phrase)
        .map_err(|_| "invalid mnemonic")?;
    let seed = mnemonic.to_seed("");

    // ── BIP32/SLIP-0010 HD derivation for secp256k1 ──
    let path: [(u32, bool); 5] = [
        (44, true),
        (RSK_COIN_TYPE, true),
        (0, true),
        (0, false),
        (0, false),
    ];

    let mut key = [0u8; 32];
    let mut chain_code = [0u8; 32];

    // Master key from BIP39 seed
    {
        let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed")
            .map_err(|_| "hmac init failed")?;
        mac.update(&seed);
        let result = mac.finalize().into_bytes();
        key.copy_from_slice(&result[..32]);
        chain_code.copy_from_slice(&result[32..]);
    }

    for (index, hardened) in path {
        let child_index = if hardened { index + 0x80000000 } else { index };
        let mut mac = HmacSha512::new_from_slice(&chain_code)
            .map_err(|_| "hmac init failed")?;
        if hardened {
            mac.update(&[0x00]);
            mac.update(&key);
        } else {
            let sk = SecretKey::from_slice(&key).map_err(|_| "invalid key")?;
            let pubkey = sk.public_key().to_encoded_point(true);
            mac.update(pubkey.as_bytes());
        }
        mac.update(&child_index.to_be_bytes());
        let result = mac.finalize().into_bytes();

        // child_key = (IL + parent_key) mod n
        let il_bytes: [u8; 32] = result[..32].try_into().unwrap();
        let il_scalar = k256::Scalar::from_repr_vartime(il_bytes.into())
            .ok_or("invalid scalar")?;
        let parent_scalar = k256::Scalar::from_repr_vartime(key.into())
            .ok_or("invalid scalar")?;
        let child_scalar = il_scalar + parent_scalar;
        key = child_scalar.to_bytes().into();
        chain_code.copy_from_slice(&result[32..]);
    }

    // ── Address from secp256k1 public key ──
    let sk = SecretKey::from_slice(&key).map_err(|_| "invalid derived key")?;
    let pubkey = sk.public_key().to_encoded_point(false);
    let pubkey_bytes = pubkey.as_bytes(); // 65 bytes uncompressed
    let mut hasher = tiny_keccak::Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(&pubkey_bytes[1..]); // skip 0x04 prefix
    hasher.finalize(&mut hash);
    let address_bytes = &hash[12..]; // last 20 bytes
    let address = to_checksum_rsk(address_bytes, 31);

    // ── ZSK: ed25519 from SHA256(seed || "mns-zsk") ──
    let mut zsk_input = Vec::with_capacity(64 + 7);
    zsk_input.extend_from_slice(&seed);
    zsk_input.extend_from_slice(b"mns-zsk");
    let zsk_seed = Sha256::digest(&zsk_input);
    let seed_bytes: [u8; 32] = zsk_seed.into();
    let signing_key =
        ed25519_dalek::SigningKey::from_bytes(&seed_bytes);
    let verifying_key = signing_key.verifying_key();
    let zsk_pub: [u8; 32] = verifying_key.to_bytes().into();

    // ZSK commitment: SHA256(0x00 || pubkey)
    let zsk_commitment = compute_zsk(KeyType::Ed25519, &zsk_pub);

    Ok(WalletInfo {
        address,
        private_key: key,
        zsk_pub,
        zsk_commitment,
    })
}

/// RSKIP-60 checksummed address encoding.
pub fn to_checksum_rsk(addr: &[u8], chain_id: u64) -> String {
    let lower = hex::encode(addr);
    let prefix = format!("{}0x", chain_id);
    let mut hasher = tiny_keccak::Keccak::v256();
    let mut hash = [0u8; 32];
    let mut input = Vec::new();
    input.extend_from_slice(prefix.as_bytes());
    input.extend_from_slice(lower.as_bytes());
    hasher.update(&input);
    hasher.finalize(&mut hash);

    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for (i, c) in lower.chars().enumerate() {
        if c.is_ascii_hexdigit() {
            let h = hash[i / 2];
            let nibble = if i % 2 == 0 { h >> 4 } else { h & 0x0f };
            if nibble >= 8 {
                out.push(c.to_ascii_uppercase());
            } else {
                out.push(c.to_ascii_lowercase());
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_derivation() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let w1 = derive_wallet(phrase).unwrap();
        let w2 = derive_wallet(phrase).unwrap();
        assert_eq!(w1.address, w2.address);
        assert_eq!(w1.private_key, w2.private_key);
        assert_eq!(w1.zsk_pub, w2.zsk_pub);
        assert_eq!(w1.zsk_commitment, w2.zsk_commitment);
    }

    #[test]
    fn address_starts_with_0x() {
        let w = derive_wallet("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap();
        assert!(w.address.starts_with("0x"));
        assert_eq!(w.address.len(), 42);
    }

    #[test]
    fn zsk_commitment_matches_compute_zsk() {
        let w = derive_wallet("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap();
        let expected = compute_zsk(KeyType::Ed25519, &w.zsk_pub);
        assert_eq!(w.zsk_commitment, expected);
    }
}
