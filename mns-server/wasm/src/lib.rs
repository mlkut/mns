use std::cell::RefCell;

use alloy::primitives::{Address, FixedBytes};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use tiny_keccak::Hasher;
use wasm_bindgen::prelude::*;

use mns::client::MnsClient;

thread_local! {
    static CLIENT: RefCell<Option<MnsClient>> = const { RefCell::new(None) };
}

fn take_client() -> Result<MnsClient, JsError> {
    CLIENT.with(|c| c.borrow().clone().ok_or_else(|| JsError::new("call init_client() first")))
}

// ── Init ──

const REGISTRY: &str = "e916A48dE922E8964542F4c4c66Ec4837bBE3445";

#[wasm_bindgen]
pub fn init_client(rpc_url: &str) -> Result<(), JsError> {
    let registry = parse_address(REGISTRY)?;
    let client = MnsClient::new(rpc_url, registry);
    CLIENT.with(|c| *c.borrow_mut() = Some(client));
    Ok(())
}

// ── Wallet ──

#[wasm_bindgen]
pub struct JsWalletKeys {
    rsk_privkey_hex: String,
    ed_privkey_hex: String,
    address: String,
    zsk_commitment_hex: String,
}

#[wasm_bindgen]
impl JsWalletKeys {
    #[wasm_bindgen(getter)]
    pub fn rsk_privkey(&self) -> String {
        self.rsk_privkey_hex.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn ed_privkey(&self) -> String {
        self.ed_privkey_hex.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn zsk_commitment(&self) -> String {
        self.zsk_commitment_hex.clone()
    }
}

/// Generate two random private keys and derive wallet from them.
#[wasm_bindgen]
pub fn generate_wallet() -> Result<JsWalletKeys, JsError> {
    use zeroize::Zeroize;
    let mut rsk_key = mns::generate_random_bytes();
    let mut ed_key = mns::generate_random_bytes();

    let (address, zsk_commitment_hex) = derive_from_hex_keys(&rsk_key, &ed_key)?;

    let result = Ok(JsWalletKeys {
        rsk_privkey_hex: hex::encode(&rsk_key),
        ed_privkey_hex: hex::encode(&ed_key),
        address,
        zsk_commitment_hex,
    });
    rsk_key.zeroize();
    ed_key.zeroize();
    result
}

/// Derive address + zsk_commitment from two hex private keys (for unlock).
/// Input: (rsk_privkey_hex, ed_privkey_hex). Returns (address, zsk_commitment_hex).
#[wasm_bindgen]
pub fn derive_wallet_from_hex(
    rsk_hex: &str,
    ed_hex: &str,
) -> Result<Vec<String>, JsError> {
    let rsk_bytes = parse_fixed_32(rsk_hex, "rsk private key")?;
    let ed_bytes = parse_fixed_32(ed_hex, "ed private key")?;
    let (address, zsk_commitment_hex) = derive_from_hex_keys(&rsk_bytes, &ed_bytes)?;
    Ok(vec![address, zsk_commitment_hex])
}

fn derive_from_hex_keys(
    rsk_key: &[u8; 32],
    ed_key: &[u8; 32],
) -> Result<(String, String), JsError> {
    // secp256k1 → Rootstock address
    let sk = k256::SecretKey::from_slice(rsk_key)
        .map_err(|e| JsError::new(&e.to_string()))?;
    let pubkey = sk.public_key().to_encoded_point(false);
    let mut hasher = tiny_keccak::Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(&pubkey.as_bytes()[1..]);
    hasher.finalize(&mut hash);
    let address = to_checksum_rsk(&hash[12..], 31);

    // ed25519 → ZSK commitment
    let signing_key = ed25519_dalek::SigningKey::from_bytes(ed_key);
    let zsk_pub: [u8; 32] = signing_key.verifying_key().to_bytes().into();
    let zsk_commitment = mns::compute_zsk(mns::KeyType::Ed25519, &zsk_pub);

    Ok((address, hex::encode(zsk_commitment)))
}

/// Convert a 32-byte hex key to a 24-word BIP39 mnemonic (for export).
#[wasm_bindgen]
pub fn key_to_mnemonic(hex_key: &str) -> Result<String, JsError> {
    let bytes = parse_fixed_32(hex_key, "key")?;
    let mnemonic =
        bip39::Mnemonic::from_entropy(bytes.as_slice()).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(mnemonic.to_string())
}

/// Migrate from an old BIP39 mnemonic to the new key format.
/// Returns [rsk_privkey_hex, ed_privkey_hex, address, zsk_commitment_hex].
#[wasm_bindgen]
pub fn migrate_from_mnemonic(phrase: &str) -> Result<Vec<String>, JsError> {
    use hmac::{Hmac, Mac};
    use k256::elliptic_curve::PrimeField;
    use sha2::{Digest, Sha256};
    use zeroize::Zeroize;

    type HmacSha512 = Hmac<sha2_10::Sha512>;

    let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, phrase)
        .map_err(|_| JsError::new("invalid mnemonic"))?;
    let mut seed = mnemonic.to_seed("");

    // BIP44 secp256k1: m/44'/37310'/0'/0/0
    const RSK_COIN_TYPE: u32 = 37310;
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
        let mut mac =
            HmacSha512::new_from_slice(b"Bitcoin seed").map_err(|_| JsError::new("hmac init"))?;
        mac.update(&seed);
        let result = mac.finalize().into_bytes();
        key.copy_from_slice(&result[..32]);
        chain_code.copy_from_slice(&result[32..]);
    }

    // Derive child keys
    for (index, hardened) in path {
        let child_index = if hardened { index + 0x80000000 } else { index };
        let mut mac =
            HmacSha512::new_from_slice(&chain_code).map_err(|_| JsError::new("hmac init"))?;
        if hardened {
            mac.update(&[0x00]);
            mac.update(&key);
        } else {
            let sk = k256::SecretKey::from_slice(&key).map_err(|_| JsError::new("invalid key"))?;
            let pubkey = sk.public_key().to_encoded_point(true);
            mac.update(pubkey.as_bytes());
        }
        mac.update(&child_index.to_be_bytes());
        let result = mac.finalize().into_bytes();

        let il_bytes: [u8; 32] = result[..32].try_into().unwrap();
        let il_scalar =
            k256::Scalar::from_repr_vartime(il_bytes.into()).ok_or_else(|| JsError::new("invalid scalar"))?;
        let parent_scalar =
            k256::Scalar::from_repr_vartime(key.into()).ok_or_else(|| JsError::new("invalid scalar"))?;
        let child_scalar = il_scalar + parent_scalar;
        key = child_scalar.to_bytes().into();
        chain_code.copy_from_slice(&result[32..]);
    }

    // Address from secp256k1 public key
    let sk = k256::SecretKey::from_slice(&key).map_err(|_| JsError::new("invalid derived key"))?;
    let pubkey = sk.public_key().to_encoded_point(false);
    let mut hasher = tiny_keccak::Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(&pubkey.as_bytes()[1..]);
    hasher.finalize(&mut hash);
    let address = to_checksum_rsk(&hash[12..], 31);

    // ZSK: ed25519 from SHA256(seed || "mns-zsk")
    let mut zsk_input = Vec::with_capacity(64 + 7);
    zsk_input.extend_from_slice(&seed);
    zsk_input.extend_from_slice(b"mns-zsk");
    let zsk_seed = Sha256::digest(&zsk_input);
    let mut ed_seed: [u8; 32] = zsk_seed.into();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&ed_seed);
    let zsk_pub: [u8; 32] = signing_key.verifying_key().to_bytes().into();
    let zsk_commitment = mns::compute_zsk(mns::KeyType::Ed25519, &zsk_pub);

    let result = Ok(vec![
        hex::encode(key),
        hex::encode(ed_seed),
        address,
        hex::encode(zsk_commitment),
    ]);
    key.zeroize();
    ed_seed.zeroize();
    seed.zeroize();
    chain_code.zeroize();
    result
}

/// RSKIP-60 checksummed address encoding.
fn to_checksum_rsk(addr: &[u8], chain_id: u64) -> String {
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

// ── Avatar ──

#[wasm_bindgen]
pub fn render_avatar_svg(name: &str) -> Result<String, JsError> {
    let name: mns::Name = name.parse().map_err(|e: &str| JsError::new(e))?;
    Ok(name.render_avatar_svg())
}

// ── RPC ──

#[wasm_bindgen]
pub async fn get_balance(address_hex: &str) -> Result<String, JsError> {
    let address = parse_address(address_hex)?;
    let client = take_client()?;
    let balance = client
        .get_balance(address)
        .await
        .map_err(|e| JsError::new(&e.to_string()))?;
    let wei = balance.to_string().parse::<f64>().unwrap_or(0.0);
    Ok(format!("{:.6} RBTC", wei / 1e18))
}

#[wasm_bindgen]
pub async fn register(
    private_key_hex: &str,
    zsk_hex: &str,
    ns: &str,
) -> Result<String, JsError> {
    use zeroize::Zeroize;
    let mut pk = parse_fixed_32(private_key_hex, "private key")?;
    let zsk = parse_fixed_32(zsk_hex, "zsk")?;
    let client = take_client()?;
    let tx_hash = client
        .register(&pk, zsk, ns.to_string())
        .await
        .map_err(|e| JsError::new(&e.to_string()))?;
    pk.0.zeroize();
    Ok(format!("{tx_hash}"))
}

#[wasm_bindgen]
pub async fn update_batch(
    private_key_hex: &str,
    ordinal: u64,
    owner_hex: &str,
    zsk_hex: &str,
    ns: &str,
) -> Result<String, JsError> {
    use zeroize::Zeroize;
    let mut pk = parse_fixed_32(private_key_hex, "private key")?;
    let owner = parse_address(owner_hex)?;
    let zsk = parse_fixed_32(zsk_hex, "zsk")?;
    let client = take_client()?;
    let tx_hash = client
        .update_batch(&pk, ordinal, owner, zsk, ns.to_string())
        .await
        .map_err(|e| JsError::new(&e.to_string()))?;
    pk.0.zeroize();
    Ok(format!("{tx_hash}"))
}

// ── Helpers ──

fn parse_fixed_32(hex_str: &str, label: &str) -> Result<FixedBytes<32>, JsError> {
    let bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
        .map_err(|e| JsError::new(&format!("{label}: bad hex: {e}")))?;
    FixedBytes::<32>::try_from(bytes.as_slice())
        .map_err(|_| JsError::new(&format!("{label} must be 32 bytes")))
}

fn parse_address(hex_str: &str) -> Result<Address, JsError> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str).map_err(|e| JsError::new(&format!("address: bad hex: {e}")))?;
    Address::try_from(bytes.as_slice()).map_err(|_| JsError::new("address must be 20 bytes"))
}
