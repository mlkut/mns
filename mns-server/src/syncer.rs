use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mns::Name;
use tracing::{error, info};

use crate::registry::RegistryReader;
use crate::store::{StoredConfig, ZoneStore};

pub struct Syncer<R: RegistryReader, S: ZoneStore> {
    registry: Arc<R>,
    store: Arc<S>,
    poll_interval: Duration,
    deployment_block: u64,
}

impl<R: RegistryReader, S: ZoneStore> Syncer<R, S> {
    pub fn new(
        registry: Arc<R>,
        store: Arc<S>,
        poll_interval: Duration,
        deployment_block: u64,
    ) -> Self {
        Self {
            registry,
            store,
            poll_interval,
            deployment_block,
        }
    }

    /// Run the ongoing poll loop, processing new events.
    pub async fn run(&self) {
        loop {
            let from = match self.store.get_last_sync_block_number().await {
                Ok(Some(block)) => block,
                Ok(None) => {
                    info!(
                        "no checkpoint found, starting event sync from deployment block {}",
                        self.deployment_block
                    );
                    if let Err(e) = self
                        .store
                        .set_last_sync_block_number(self.deployment_block)
                        .await
                    {
                        error!("failed to store deployment block: {e}");
                    }
                    self.deployment_block
                }
                Err(e) => {
                    error!("failed to read last sync block: {e}");
                    tokio::time::sleep(self.poll_interval).await;
                    continue;
                }
            };

            match self.registry.poll_events(from).await {
                Ok((events, safe_block)) => {
                    for event in &events {
                        let ordinal = event.ordinal();
                        let zsk = event.zsk();
                        let ns = event.ns().to_string();
                        let owner = event.owner();
                        info!(
                            "processing event: ordinal={ordinal} owner={} zsk={}",
                            hex::encode(owner),
                            hex::encode(zsk)
                        );

                        match event {
                            crate::registry::RegistryEvent::BatchRegistered { .. }
                            | crate::registry::RegistryEvent::BatchUpdated { .. } => {
                                let stored = StoredConfig { zsk, ns, owner };
                                if let Err(e) = self.store.set_batch(ordinal, &stored).await {
                                    error!(
                                        "failed to store batch config for ordinal {ordinal}: {e}"
                                    );
                                }
                            }
                            crate::registry::RegistryEvent::EntryCreated { .. }
                            | crate::registry::RegistryEvent::EntryUpdated { .. } => {
                                let stored = StoredConfig { zsk, ns, owner };
                                if let Err(e) = self.store.set_entry(ordinal, &stored).await {
                                    error!(
                                        "failed to store entry config for ordinal {ordinal}: {e}"
                                    );
                                }
                                // Evict any cached signed packet
                                let name = Name::from_ordinal(ordinal);
                                if let Err(e) = self.store.remove_signed_packet(&name).await {
                                    error!("failed to evict packet for ordinal {ordinal}: {e}");
                                }
                            }
                        }
                    }

                    if !events.is_empty() {
                        info!("processed {} events, sync_block={safe_block}", events.len());
                    }

                    if let Err(e) = self.store.set_last_sync_block_number(safe_block).await {
                        error!("failed to store last sync block: {e}");
                    }
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if let Err(e) = self.store.set_last_sync_block_time(now).await {
                        error!("failed to store last sync time: {e}");
                    }
                }
                Err(e) => {
                    error!("event poll failed: {e}");
                }
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
