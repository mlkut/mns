use alloy::eips::BlockId;
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::Address;
use alloy::primitives::FixedBytes;
use alloy::providers::Provider;
use alloy::providers::RootProvider;
use alloy::rpc::types::Filter;
use alloy::sol_types::SolEvent;
use bindings::mns_registry::MNSRegistry;
use url::Url;

use super::{RegistryError, RegistryEvent, RegistryReader, ZoneConfig};

pub struct AlloyRegistry {
    contract: MNSRegistry::MNSRegistryInstance<RootProvider>,
    address: Address,
}

impl AlloyRegistry {
    pub fn new(rpc_url: &str, contract_address: &str) -> Result<Self, RegistryError> {
        let url = Url::parse(rpc_url).map_err(|e| RegistryError::Rpc(e.to_string()))?;
        let provider = RootProvider::new_http(url);

        let address: Address = contract_address
            .parse()
            .map_err(|e| RegistryError::Contract(format!("invalid address: {e}")))?;

        let contract = MNSRegistry::new(address, provider);

        Ok(Self { contract, address })
    }

    fn bytes32_to_zsk(b: FixedBytes<32>) -> [u8; 32] {
        let mut zsk = [0u8; 32];
        zsk.copy_from_slice(&b[..]);
        zsk
    }
}

#[async_trait::async_trait]
impl RegistryReader for AlloyRegistry {
    async fn next_ordinal(&self) -> Result<u64, RegistryError> {
        self.contract
            .nextOrdinal()
            .call()
            .await
            .map_err(|e| RegistryError::Contract(e.to_string()))
    }

    async fn get_zone_config(&self, ordinal: u64) -> Result<Option<ZoneConfig>, RegistryError> {
        let result = self
            .contract
            .getZoneConfig(ordinal)
            .block(BlockId::Number(BlockNumberOrTag::Latest))
            .call()
            .await;

        match result {
            Ok(config) => Ok(Some(ZoneConfig {
                zsk: Self::bytes32_to_zsk(config.zsk),
                ns: config.ns,
            })),
            Err(e) => {
                if e.as_decoded_error::<MNSRegistry::OrdinalNotRegistered>()
                    .is_some()
                {
                    return Ok(None);
                }
                Err(RegistryError::Contract(e.to_string()))
            }
        }
    }

    async fn poll_events(
        &self,
        from_block: u64,
    ) -> Result<(Vec<RegistryEvent>, u64), RegistryError> {
        let provider = self.contract.provider();
        let to_block = provider
            .get_block_number()
            .await
            .map(|n| n.saturating_sub(1))
            .map_err(|e| RegistryError::Rpc(e.to_string()))?;

        if to_block <= from_block {
            return Ok((Vec::new(), from_block));
        }

        let filter = Filter::new()
            .from_block(from_block + 1)
            .to_block(BlockNumberOrTag::Number(to_block))
            .address(self.address);

        let logs = match provider.get_logs(&filter).await {
            Ok(logs) => logs,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("does not exist/is not available")
                    || msg.contains("method not found")
                    || msg.contains("-32601")
                {
                    tracing::warn!(
                        "eth_getLogs not supported by this RPC provider; event polling disabled"
                    );
                    return Ok((Vec::new(), to_block));
                }
                return Err(RegistryError::Rpc(msg));
            }
        };

        let mut events = Vec::new();

        for log in logs {
            let topic0 = log.topics().first().copied();

            if let Some(topic) = topic0 {
                if topic == MNSRegistry::BatchRegistered::SIGNATURE_HASH {
                    if let Ok(event) =
                        MNSRegistry::BatchRegistered::decode_log(&(log.clone().into()))
                    {
                        let ordinal = event.ordinal;
                        let zsk = Self::bytes32_to_zsk(event.zsk);
                        let ns = event.ns.clone();
                        events.push(RegistryEvent::BatchRegistered { ordinal, zsk, ns });
                        continue;
                    }
                }

                if topic == MNSRegistry::BatchUpdated::SIGNATURE_HASH {
                    if let Ok(event) = MNSRegistry::BatchUpdated::decode_log(&(log.clone().into()))
                    {
                        let ordinal = event.ordinal;
                        let zsk = Self::bytes32_to_zsk(event.zsk);
                        let ns = event.ns.clone();
                        events.push(RegistryEvent::BatchUpdated { ordinal, zsk, ns });
                        continue;
                    }
                }

                if topic == MNSRegistry::EntryCreated::SIGNATURE_HASH {
                    if let Ok(event) = MNSRegistry::EntryCreated::decode_log(&(log.clone().into()))
                    {
                        let ordinal = event.ordinal;
                        let zsk = Self::bytes32_to_zsk(event.zsk);
                        let ns = event.ns.clone();
                        events.push(RegistryEvent::EntryCreated { ordinal, zsk, ns });
                        continue;
                    }
                }

                if topic == MNSRegistry::EntryUpdated::SIGNATURE_HASH {
                    if let Ok(event) = MNSRegistry::EntryUpdated::decode_log(&(log.clone().into()))
                    {
                        let ordinal = event.ordinal;
                        let zsk = Self::bytes32_to_zsk(event.zsk);
                        let ns = event.ns.clone();
                        events.push(RegistryEvent::EntryUpdated { ordinal, zsk, ns });
                        continue;
                    }
                }
            }
        }

        Ok((events, to_block))
    }
}
