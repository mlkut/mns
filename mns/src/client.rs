use alloy::primitives::{Address, FixedBytes};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use bindings::mns_registry::MNSRegistry::MNSRegistryInstance;

/// Client for interacting with the MNS registry contract.
///
/// Wraps an alloy provider and the `MNSRegistry` binding. All contract calls
/// and transactions flow through this single struct.
pub struct MnsClient<P> {
    registry: MNSRegistryInstance<P>,
}

impl MnsClient<()> {
    /// Connect to a registry at `rpc_url` with a wallet for signing.
    pub fn connect(
        rpc_url: &str,
        private_key: &[u8; 32],
        registry_address: Address,
    ) -> Result<
        MnsClient<impl Provider<alloy::network::Ethereum>>,
        Box<dyn std::error::Error>,
    > {
        let signer = PrivateKeySigner::from_bytes(private_key.into())?;
        let provider = ProviderBuilder::new()
            .wallet(signer)
            .connect_http(rpc_url.parse()?);
        Ok(MnsClient {
            registry: MNSRegistryInstance::new(registry_address, provider),
        })
    }
}

impl<P: Provider<alloy::network::Ethereum>> MnsClient<P> {
    /// Wrap an existing provider and registry address.
    pub fn new(provider: P, registry_address: Address) -> Self {
        Self {
            registry: MNSRegistryInstance::new(registry_address, provider),
        }
    }

    /// Access the inner `MNSRegistryInstance` for direct contract calls.
    pub fn registry(&self) -> &MNSRegistryInstance<P> {
        &self.registry
    }

    /// Access the inner provider.
    pub fn provider(&self) -> &P {
        self.registry.provider()
    }

    /// Register a new MNS name. Sends a transaction and returns the tx hash.
    pub async fn register(
        &self,
        zsk: FixedBytes<32>,
        ns: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let pending = self.registry.register(zsk, ns).send().await?;
        let receipt = pending.get_receipt().await?;
        Ok(format!("0x{:x}", receipt.transaction_hash))
    }

    /// Update a batch's owner, ZSK, and name. Sends a transaction and returns the tx hash.
    pub async fn update_batch(
        &self,
        ordinal: u64,
        new_owner: Address,
        new_zsk: FixedBytes<32>,
        new_ns: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let pending = self
            .registry
            .updateBatch(ordinal, new_owner, new_zsk, new_ns)
            .send()
            .await?;
        let receipt = pending.get_receipt().await?;
        Ok(format!("0x{:x}", receipt.transaction_hash))
    }

    /// Read the zone config (ZSK, NS) for a batch ordinal.
    pub async fn get_zone_config(
        &self,
        ordinal: u64,
    ) -> Result<(FixedBytes<32>, String), Box<dyn std::error::Error>> {
        let result = self.registry.getZoneConfig(ordinal).call().await?;
        Ok((result.zsk, result.ns))
    }

    /// Read the owner of a batch ordinal.
    pub async fn get_owner(&self, ordinal: u64) -> Result<Address, Box<dyn std::error::Error>> {
        let result = self.registry.getOwner(ordinal).call().await?;
        Ok(result)
    }

    /// Read the next available ordinal.
    pub async fn next_ordinal(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let result = self.registry.nextOrdinal().call().await?;
        Ok(result)
    }
}
