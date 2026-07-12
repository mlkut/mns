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
    CLIENT.with(|c| {
        c.borrow()
            .clone()
            .ok_or_else(|| JsError::new("call init_client() first"))
    })
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
    zsk_privkey_hex: String,
    zsk_key_type: u8,
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
    pub fn zsk_privkey(&self) -> String {
        self.zsk_privkey_hex.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn zsk_key_type(&self) -> u8 {
        self.zsk_key_type
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
    let key_type = mns::KeyType::Ed25519;
    let mut rsk_key = mns::generate_random_bytes();
    let mut zsk_key = mns::generate_random_bytes();

    let (address, zsk_commitment_hex) = derive_from_hex_keys(&rsk_key, key_type, &zsk_key)?;

    let result = Ok(JsWalletKeys {
        rsk_privkey_hex: hex::encode(&rsk_key),
        zsk_privkey_hex: hex::encode(&zsk_key),
        zsk_key_type: key_type.as_byte(),
        address,
        zsk_commitment_hex,
    });
    rsk_key.zeroize();
    zsk_key.zeroize();
    result
}

/// Derive address + zsk_commitment from a hex private key (for unlock).
/// Input: (rsk_privkey_hex, key_type, zsk_privkey_hex). Returns [address, zsk_commitment_hex].
#[wasm_bindgen]
pub fn derive_wallet_from_hex(
    rsk_hex: &str,
    key_type: u8,
    zsk_hex: &str,
) -> Result<Vec<String>, JsError> {
    let rsk_bytes = parse_fixed_32(rsk_hex, "rsk private key")?;
    let key_type = mns::KeyType::from_byte(key_type)?;
    let zsk_bytes = parse_fixed_len(zsk_hex, "zsk private key", key_type.public_key_len())?;
    let (address, zsk_commitment_hex) = derive_from_hex_keys(&rsk_bytes, key_type, &zsk_bytes)?;
    Ok(vec![address, zsk_commitment_hex])
}

fn derive_from_hex_keys(
    rsk_key: &[u8; 32],
    key_type: mns::KeyType,
    zsk_key: &[u8],
) -> Result<(String, String), JsError> {
    // secp256k1 → Rootstock address
    let sk = k256::SecretKey::from_slice(rsk_key).map_err(|e| JsError::new(&e.to_string()))?;
    let pubkey = sk.public_key().to_encoded_point(false);
    let mut hasher = tiny_keccak::Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(&pubkey.as_bytes()[1..]);
    hasher.finalize(&mut hash);
    let address = to_checksum_rsk(&hash[12..], 31);

    // zsk key → ZSK commitment based on key type
    let zsk_pub: [u8; 32] = match key_type {
        mns::KeyType::Ed25519 => {
            let key_arr: [u8; 32] = zsk_key
                .try_into()
                .map_err(|_| JsError::new("Ed25519 key must be 32 bytes"))?;
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_arr);
            signing_key.verifying_key().to_bytes().into()
        }
    };
    let zsk_commitment = mns::compute_zsk(key_type, &zsk_pub);

    Ok((address, hex::encode(zsk_commitment)))
}

/// Convert a 32-byte hex key to a 24-word BIP39 mnemonic (for export).
#[wasm_bindgen]
pub fn key_to_mnemonic(hex_key: &str) -> Result<String, JsError> {
    let bytes = parse_fixed_32(hex_key, "key")?;
    let mnemonic = bip39::Mnemonic::from_entropy(bytes.as_slice())
        .map_err(|e| JsError::new(&e.to_string()))?;
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
        let il_scalar = k256::Scalar::from_repr_vartime(il_bytes.into())
            .ok_or_else(|| JsError::new("invalid scalar"))?;
        let parent_scalar = k256::Scalar::from_repr_vartime(key.into())
            .ok_or_else(|| JsError::new("invalid scalar"))?;
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
pub async fn register(private_key_hex: &str, zsk_hex: &str, ns: &str) -> Result<String, JsError> {
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

// ── Signed Packet ──

/// Create a signed packet from an Ed25519 private key, MNS name, and records JSON.
///
/// records_json is an array of objects:
///   { "type": "A", "ttl": 300, "address": "1.2.3.4" }
///   { "type": "AAAA", "ttl": 300, "address": "::1" }
///   { "type": "NS", "ttl": 3600, "target": "ns1.example.com" }
///   { "type": "CNAME", "ttl": 300, "target": "example.com" }
///   { "type": "MX", "ttl": 300, "preference": 10, "exchange": "mail.example.com" }
///   { "type": "TXT", "ttl": 300, "txt": "v=spf1 ..." }
///   { "type": "HTTPS", "ttl": 300, "priority": 1, "target": ".", "alpn": "h2,h3", "port": 443 }
///   { "type": "SVCB", "ttl": 300, "priority": 1, "target": "example.com", "alpn": "h2", "port": 443 }
///   { "type": "SRV", "ttl": 300, "priority": 10, "weight": 0, "port": 443, "target": "example.com" }
///   { "type": "SOA", "ttl": 300, "mname": "ns1.example.com", "rname": "admin.example.com",
///     "serial": 2024010101, "refresh": 3600, "retry": 600, "expire": 604800, "minimum": 86400 }
///   { "type": "PTR", "ttl": 300, "target": "example.com" }
///
/// Returns base64-encoded signed packet bytes.
#[wasm_bindgen]
pub fn create_signed_packet(
    key_type: u8,
    privkey_hex: &str,
    name_str: &str,
    records_json: &str,
) -> Result<String, JsError> {
    use base64::Engine;
    use ntimestamp::Timestamp;
    use simple_dns::rdata::{RData, HTTPS, MX, SOA, SRV, SVCB};
    use simple_dns::{CharacterString, Name as DnsName, ResourceRecord, CLASS};
    use zeroize::Zeroize;

    let key_type = mns::KeyType::from_byte(key_type)?;
    let key_len = key_type.public_key_len();
    let mut key_bytes = parse_fixed_len(privkey_hex, "private key", key_len)?;
    let name: mns::Name = name_str.parse().map_err(|e: &str| JsError::new(e))?;

    let records: Vec<serde_json::Value> = serde_json::from_str(records_json)
        .map_err(|e| JsError::new(&format!("invalid records JSON: {e}")))?;

    let mut resource_records = Vec::new();

    for rec in &records {
        let rtype = rec["type"].as_str().unwrap_or("A");
        let ttl = rec["ttl"].as_u64().unwrap_or(300) as u32;
        let record_name = rec["name"].as_str().unwrap_or("");

        let rdata = match rtype {
            "A" => {
                let addr: std::net::Ipv4Addr = rec["address"]
                    .as_str()
                    .ok_or_else(|| JsError::new("A record: missing address"))?
                    .parse()
                    .map_err(|e| JsError::new(&format!("A record: invalid address: {e}")))?;
                RData::A(addr.into())
            }
            "AAAA" => {
                let addr: std::net::Ipv6Addr = rec["address"]
                    .as_str()
                    .ok_or_else(|| JsError::new("AAAA record: missing address"))?
                    .parse()
                    .map_err(|e| JsError::new(&format!("AAAA record: invalid address: {e}")))?;
                RData::AAAA(addr.into())
            }
            "NS" => {
                let target = rec["target"]
                    .as_str()
                    .ok_or_else(|| JsError::new("NS record: missing target"))?;
                RData::NS(simple_dns::rdata::NS(DnsName::new(target).map_err(
                    |e| JsError::new(&format!("NS record: invalid name: {e}")),
                )?))
            }
            "CNAME" => {
                let target = rec["target"]
                    .as_str()
                    .ok_or_else(|| JsError::new("CNAME record: missing target"))?;
                RData::CNAME(simple_dns::rdata::CNAME(DnsName::new(target).map_err(
                    |e| JsError::new(&format!("CNAME record: invalid name: {e}")),
                )?))
            }
            "MX" => {
                let pref = rec["preference"].as_u64().unwrap_or(10) as u16;
                let exchange = rec["exchange"]
                    .as_str()
                    .ok_or_else(|| JsError::new("MX record: missing exchange"))?;
                RData::MX(MX {
                    preference: pref,
                    exchange: DnsName::new(exchange)
                        .map_err(|e| JsError::new(&format!("MX record: invalid name: {e}")))?,
                })
            }
            "TXT" => {
                let txt_str = rec["txt"]
                    .as_str()
                    .ok_or_else(|| JsError::new("TXT record: missing txt"))?;
                let txt: simple_dns::rdata::TXT = txt_str
                    .try_into()
                    .map_err(|e| JsError::new(&format!("TXT record: {e}")))?;
                RData::TXT(txt)
            }
            "SRV" => {
                let priority = rec["priority"].as_u64().unwrap_or(10) as u16;
                let weight = rec["weight"].as_u64().unwrap_or(0) as u16;
                let port = rec["port"].as_u64().unwrap_or(443) as u16;
                let target = rec["target"]
                    .as_str()
                    .ok_or_else(|| JsError::new("SRV record: missing target"))?;
                RData::SRV(SRV {
                    priority,
                    weight,
                    port,
                    target: DnsName::new(target)
                        .map_err(|e| JsError::new(&format!("SRV record: invalid name: {e}")))?,
                })
            }
            "HTTPS" | "SVCB" => {
                let priority = rec["priority"].as_u64().unwrap_or(1) as u16;
                let target_str = rec["target"].as_str().unwrap_or(".");
                let target = DnsName::new(target_str)
                    .map_err(|e| JsError::new(&format!("{rtype} record: invalid target: {e}")))?;

                let mut svcb = SVCB::new(priority, target);

                if let Some(alpn_str) = rec["alpn"].as_str() {
                    if !alpn_str.is_empty() {
                        let alpn_ids: Vec<CharacterString> = alpn_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|s| CharacterString::try_from(s))
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|e| {
                                JsError::new(&format!("{rtype} record: invalid ALPN: {e}"))
                            })?;
                        svcb.set_alpn(&alpn_ids);
                    }
                }
                if let Some(port) = rec["port"].as_u64() {
                    svcb.set_port(port as u16);
                }

                match rtype {
                    "HTTPS" => RData::HTTPS(HTTPS(svcb)),
                    _ => RData::SVCB(svcb),
                }
            }
            "SOA" => {
                let mname = rec["mname"]
                    .as_str()
                    .ok_or_else(|| JsError::new("SOA record: missing mname"))?;
                let rname = rec["rname"]
                    .as_str()
                    .ok_or_else(|| JsError::new("SOA record: missing rname"))?;
                RData::SOA(SOA {
                    mname: DnsName::new(mname)
                        .map_err(|e| JsError::new(&format!("SOA record: invalid mname: {e}")))?,
                    rname: DnsName::new(rname)
                        .map_err(|e| JsError::new(&format!("SOA record: invalid rname: {e}")))?,
                    serial: rec["serial"].as_u64().unwrap_or(2024010101) as u32,
                    refresh: rec["refresh"].as_i64().unwrap_or(3600) as i32,
                    retry: rec["retry"].as_i64().unwrap_or(600) as i32,
                    expire: rec["expire"].as_i64().unwrap_or(604800) as i32,
                    minimum: rec["minimum"].as_u64().unwrap_or(86400) as u32,
                })
            }
            "PTR" => {
                let target = rec["target"]
                    .as_str()
                    .ok_or_else(|| JsError::new("PTR record: missing target"))?;
                RData::PTR(simple_dns::rdata::PTR(DnsName::new(target).map_err(
                    |e| JsError::new(&format!("PTR record: invalid name: {e}")),
                )?))
            }
            other => {
                return Err(JsError::new(&format!("unsupported record type: {other}")));
            }
        };

        let rr_name = if record_name.is_empty() {
            DnsName::new_unchecked("")
        } else {
            DnsName::new(record_name).map_err(|e| JsError::new(&format!("record name: {e}")))?
        };

        resource_records.push(ResourceRecord::new(rr_name, CLASS::IN, ttl, rdata));
    }

    let keypair = match key_type {
        mns::KeyType::Ed25519 => {
            let key_arr: [u8; 32] = key_bytes[..]
                .try_into()
                .map_err(|_| JsError::new("Ed25519 key must be 32 bytes"))?;
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_arr);
            mns::Keypair::from_ed25519(signing_key)
        }
    };

    let packet = mns::SignedPacket::new(&keypair, name, &resource_records, Timestamp::now())
        .map_err(|e| JsError::new(&format!("failed to create signed packet: {e}")))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(packet.as_bytes());
    key_bytes.zeroize();
    Ok(b64)
}

// ── Helpers ──

fn parse_fixed_32(hex_str: &str, label: &str) -> Result<FixedBytes<32>, JsError> {
    let bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
        .map_err(|e| JsError::new(&format!("{label}: bad hex: {e}")))?;
    FixedBytes::<32>::try_from(bytes.as_slice())
        .map_err(|_| JsError::new(&format!("{label} must be 32 bytes")))
}

fn parse_fixed_len(hex_str: &str, label: &str, expected_len: usize) -> Result<Vec<u8>, JsError> {
    let bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
        .map_err(|e| JsError::new(&format!("{label}: bad hex: {e}")))?;
    if bytes.len() != expected_len {
        return Err(JsError::new(&format!(
            "{label} must be {expected_len} bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn parse_address(hex_str: &str) -> Result<Address, JsError> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes =
        hex::decode(hex_str).map_err(|e| JsError::new(&format!("address: bad hex: {e}")))?;
    Address::try_from(bytes.as_slice()).map_err(|_| JsError::new("address must be 20 bytes"))
}
