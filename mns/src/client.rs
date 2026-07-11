use alloy::consensus::{SignableTransaction, TxEnvelope, TxLegacy};
use alloy::eips::Encodable2718;
use alloy::primitives::{Address, Bytes, FixedBytes, TxHash, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol_types::SolCall;
use bindings::mns_registry::MNSRegistry::MNSRegistryInstance;

/// Client for interacting with the MNS registry contract.
///
/// Stores connection config. Write operations take a private key at call time.
#[derive(Clone)]
pub struct MnsClient {
    rpc_url: String,
    registry_address: Address,
}

impl MnsClient {
    pub fn new(rpc_url: &str, registry_address: Address) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            registry_address,
        }
    }

    fn provider(
        &self,
    ) -> Result<impl Provider<alloy::network::Ethereum>, Box<dyn std::error::Error>> {
        Ok(ProviderBuilder::new().connect_http(self.rpc_url.parse()?))
    }

    async fn send_raw_tx(
        &self,
        private_key: &[u8; 32],
        calldata: Vec<u8>,
    ) -> Result<B256, Box<dyn std::error::Error>> {
        let signer = PrivateKeySigner::from_bytes(private_key.into())?;
        let from = signer.address();
        let provider = self.provider()?;

        let nonce = provider.get_transaction_count(from).await?;
        let gas_price = provider.get_gas_price().await?;
        let chain_id = provider.get_chain_id().await?;

        let tx = TxLegacy {
            nonce,
            gas_price,
            gas_limit: 300_000,
            to: self.registry_address.into(),
            value: U256::ZERO,
            input: Bytes::from(calldata),
            chain_id: Some(chain_id),
        };

        let sig = signer.sign_hash(&tx.signature_hash()).await?;
        let signed = tx.into_signed(sig);
        let envelope = TxEnvelope::Legacy(signed);

        let mut encoded = Vec::new();
        Encodable2718::encode_2718(&envelope, &mut encoded);

        let rlp_hex = format!("0x{}", hex::encode(&encoded));
        let tx_hash: TxHash = provider
            .raw_request("eth_sendRawTransaction".into(), (rlp_hex,))
            .await?;

        Ok(tx_hash)
    }

    // ── Write operations (require private key) ──

    pub async fn register(
        &self,
        private_key: &[u8; 32],
        zsk: FixedBytes<32>,
        ns: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use bindings::mns_registry::MNSRegistry::registerCall;
        let calldata = registerCall::new((zsk, ns)).abi_encode();
        let tx_hash = self.send_raw_tx(private_key, calldata).await?;
        Ok(format!("0x{:x}", tx_hash))
    }

    pub async fn update_batch(
        &self,
        private_key: &[u8; 32],
        ordinal: u64,
        new_owner: Address,
        new_zsk: FixedBytes<32>,
        new_ns: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use bindings::mns_registry::MNSRegistry::updateBatchCall;
        let calldata = updateBatchCall::new((ordinal, new_owner, new_zsk, new_ns)).abi_encode();
        let tx_hash = self.send_raw_tx(private_key, calldata).await?;
        Ok(format!("0x{:x}", tx_hash))
    }

    // ── Read-only operations ──

    pub async fn get_balance(
        &self,
        address: Address,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let provider = self.provider()?;
        let balance = provider.get_balance(address).await?;
        let wei = balance.to_string().parse::<f64>().unwrap_or(0.0);
        Ok(format!("{:.6} RBTC", wei / 1e18))
    }

    pub async fn get_zone_config(
        &self,
        ordinal: u64,
    ) -> Result<(FixedBytes<32>, String), Box<dyn std::error::Error>> {
        let provider = self.provider()?;
        let registry = MNSRegistryInstance::new(self.registry_address, provider);
        let result = registry.getZoneConfig(ordinal).call().await?;
        Ok((result.zsk, result.ns))
    }

    pub async fn get_owner(&self, ordinal: u64) -> Result<Address, Box<dyn std::error::Error>> {
        let provider = self.provider()?;
        let registry = MNSRegistryInstance::new(self.registry_address, provider);
        let result = registry.getOwner(ordinal).call().await?;
        Ok(result)
    }

    pub async fn next_ordinal(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let provider = self.provider()?;
        let registry = MNSRegistryInstance::new(self.registry_address, provider);
        let result = registry.nextOrdinal().call().await?;
        Ok(result)
    }
}
