use alloy::eips::BlockNumberOrTag;
use alloy::primitives::Address;
use alloy::primitives::FixedBytes;
use alloy::providers::Provider;
use alloy::providers::RootProvider;
use alloy::rpc::types::Filter;
use alloy::sol_types::SolEvent;
use bindings::mns_registry::MNSRegistry;
use url::Url;

use super::{RegistryError, RegistryEvent, RegistryReader, BLOCK_CONFIRMATIONS};

pub struct AlloyRegistry {
    contract: MNSRegistry::MNSRegistryInstance<RootProvider>,
    address: Address,
    max_block_range: u64,
}

impl AlloyRegistry {
    pub fn new(
        rpc_url: &str,
        contract_address: &str,
        max_block_range: u64,
    ) -> Result<Self, RegistryError> {
        let url = Url::parse(rpc_url).map_err(|e| RegistryError::Rpc(e.to_string()))?;
        let provider = RootProvider::new_http(url);

        let address: Address = contract_address
            .parse()
            .map_err(|e| RegistryError::Contract(format!("invalid address: {e}")))?;

        let contract = MNSRegistry::new(address, provider);

        Ok(Self {
            contract,
            address,
            max_block_range,
        })
    }

    fn bytes32_to_zsk(b: FixedBytes<32>) -> [u8; 32] {
        let mut zsk = [0u8; 32];
        zsk.copy_from_slice(&b[..]);
        zsk
    }

    fn address_to_bytes(addr: Address) -> [u8; 20] {
        let addr: alloy::primitives::FixedBytes<20> = addr.into();
        addr.0
    }
}

#[async_trait::async_trait]
impl RegistryReader for AlloyRegistry {
    async fn poll_events(
        &self,
        from_block: u64,
    ) -> Result<(Vec<RegistryEvent>, u64), RegistryError> {
        let provider = self.contract.provider();
        let to_block = provider
            .get_block_number()
            .await
            .map(|n| n.saturating_sub(BLOCK_CONFIRMATIONS))
            .map_err(|e| RegistryError::Rpc(e.to_string()))?;

        if from_block > to_block {
            return Ok((Vec::new(), to_block));
        }

        let mut events = Vec::new();
        let mut current = from_block;

        while current <= to_block {
            let mut chunk_size = self.max_block_range;
            let mut chunk_end;
            let logs = loop {
                chunk_end = std::cmp::min(current.saturating_add(chunk_size - 1), to_block);
                tracing::debug!(from = current, to = chunk_end, "querying event logs");

                let filter = Filter::new()
                    .from_block(current)
                    .to_block(BlockNumberOrTag::Number(chunk_end))
                    .address(self.address);

                match provider.get_logs(&filter).await {
                    Ok(logs) => {
                        tracing::debug!(count = logs.len(), "got event logs");
                        break logs;
                    }
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
                        if is_range_too_large(&msg) {
                            if let Some(limit) = parse_range_limit(&msg) {
                                chunk_size = limit;
                            } else {
                                chunk_size /= 2;
                            }
                            if chunk_size < 2 {
                                return Err(RegistryError::Rpc(format!(
                                    "eth_getLogs range too small even after halving: {msg}"
                                )));
                            }
                            tracing::warn!(
                                "eth_getLogs range too large, retrying with chunk_size={chunk_size}: {msg}"
                            );
                            continue;
                        }
                        return Err(RegistryError::Rpc(msg));
                    }
                }
            };

            for log in logs {
                let topic0 = log.topics().first().copied();

                if let Some(topic) = topic0 {
                    if topic == MNSRegistry::BatchRegistered::SIGNATURE_HASH {
                        if let Ok(event) =
                            MNSRegistry::BatchRegistered::decode_log(&(log.clone().into()))
                        {
                            let ordinal = event.ordinal;
                            let owner = Self::address_to_bytes(event.owner);
                            let zsk = Self::bytes32_to_zsk(event.zsk);
                            let ns = event.ns.clone();
                            events.push(RegistryEvent::BatchRegistered {
                                ordinal,
                                owner,
                                zsk,
                                ns,
                            });
                            continue;
                        }
                    }

                    if topic == MNSRegistry::BatchUpdated::SIGNATURE_HASH {
                        if let Ok(event) =
                            MNSRegistry::BatchUpdated::decode_log(&(log.clone().into()))
                        {
                            let ordinal = event.ordinal;
                            let owner = Self::address_to_bytes(event.newOwner);
                            let zsk = Self::bytes32_to_zsk(event.zsk);
                            let ns = event.ns.clone();
                            events.push(RegistryEvent::BatchUpdated {
                                ordinal,
                                owner,
                                zsk,
                                ns,
                            });
                            continue;
                        }
                    }

                    if topic == MNSRegistry::EntryCreated::SIGNATURE_HASH {
                        if let Ok(event) =
                            MNSRegistry::EntryCreated::decode_log(&(log.clone().into()))
                        {
                            let ordinal = event.ordinal;
                            let owner = Self::address_to_bytes(event.newOwner);
                            let zsk = Self::bytes32_to_zsk(event.zsk);
                            let ns = event.ns.clone();
                            events.push(RegistryEvent::EntryCreated {
                                ordinal,
                                owner,
                                zsk,
                                ns,
                            });
                            continue;
                        }
                    }

                    if topic == MNSRegistry::EntryUpdated::SIGNATURE_HASH {
                        if let Ok(event) =
                            MNSRegistry::EntryUpdated::decode_log(&(log.clone().into()))
                        {
                            let ordinal = event.ordinal;
                            let owner = Self::address_to_bytes(event.newOwner);
                            let zsk = Self::bytes32_to_zsk(event.zsk);
                            let ns = event.ns.clone();
                            events.push(RegistryEvent::EntryUpdated {
                                ordinal,
                                owner,
                                zsk,
                                ns,
                            });
                            continue;
                        }
                    }
                }
            }

            current = chunk_end + 1;
        }

        Ok((events, to_block))
    }
}

fn is_range_too_large(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("range")
        || lower.contains("too large")
        || lower.contains("too many")
        || lower.contains("limit")
        || lower.contains("max ")
        || lower.contains("exceeded")
}

fn parse_range_limit(msg: &str) -> Option<u64> {
    // Try to find a number near keywords like "max", "limit", "range"
    for keyword in &["max ", "limit ", "limit of ", "range of ", "more than "] {
        if let Some(pos) = msg.to_lowercase().find(keyword) {
            let after = &msg[pos + keyword.len()..];
            if let Some(num_end) = after.find(|c: char| !c.is_ascii_digit()) {
                if let Ok(n) = after[..num_end].parse::<u64>() {
                    return Some(n);
                }
            } else if let Ok(n) = after.parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}
