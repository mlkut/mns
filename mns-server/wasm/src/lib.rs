use std::cell::RefCell;

use alloy::primitives::{Address, FixedBytes};
use wasm_bindgen::prelude::*;

use mns::client::MnsClient;
use mns::wallet;

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
pub struct JsWalletInfo {
    address: String,
    private_key_hex: String,
    zsk_pub_hex: String,
    zsk_commitment_hex: String,
}

#[wasm_bindgen]
impl JsWalletInfo {
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn private_key(&self) -> String {
        self.private_key_hex.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn zsk_pub(&self) -> String {
        self.zsk_pub_hex.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn zsk_commitment(&self) -> String {
        self.zsk_commitment_hex.clone()
    }
}

#[wasm_bindgen]
pub fn generate_mnemonic() -> String {
    wallet::generate_mnemonic()
}

#[wasm_bindgen]
pub fn validate_mnemonic(phrase: &str) -> bool {
    wallet::validate_mnemonic(phrase)
}

#[wasm_bindgen]
pub fn derive_keys(phrase: &str) -> Result<JsWalletInfo, JsError> {
    let w = wallet::derive_wallet(phrase).map_err(JsError::new)?;
    Ok(JsWalletInfo {
        address: w.address,
        private_key_hex: hex::encode(w.private_key),
        zsk_pub_hex: hex::encode(w.zsk_pub),
        zsk_commitment_hex: hex::encode(w.zsk_commitment),
    })
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
    client.get_balance(address).await.map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub async fn register(private_key_hex: &str, zsk_hex: &str, ns: &str) -> Result<String, JsError> {
    let pk = parse_fixed_32(private_key_hex, "private key")?;
    let zsk = parse_fixed_32(zsk_hex, "zsk")?;
    let client = take_client()?;
    client.register(&pk, zsk, ns.to_string()).await.map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub async fn update_batch(
    private_key_hex: &str,
    ordinal: u64,
    owner_hex: &str,
    zsk_hex: &str,
    ns: &str,
) -> Result<String, JsError> {
    let pk = parse_fixed_32(private_key_hex, "private key")?;
    let owner = parse_address(owner_hex)?;
    let zsk = parse_fixed_32(zsk_hex, "zsk")?;
    let client = take_client()?;
    client
        .update_batch(&pk, ordinal, owner, zsk, ns.to_string())
        .await
        .map_err(|e| JsError::new(&e.to_string()))
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
    let bytes = hex::decode(hex_str)
        .map_err(|e| JsError::new(&format!("address: bad hex: {e}")))?;
    Address::try_from(bytes.as_slice())
        .map_err(|_| JsError::new("address must be 20 bytes"))
}
