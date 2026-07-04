use std::sync::Arc;
use std::time::Duration;

use mns::Name;
use tracing::{error, info};

use crate::registry::RegistryReader;
use crate::store::ZoneStore;

pub struct Syncer<R: RegistryReader, S: ZoneStore> {
    registry: Arc<R>,
    store: Arc<S>,
    poll_interval: Duration,
    reorg_lookback: u64,
}

impl<R: RegistryReader, S: ZoneStore> Syncer<R, S> {
    pub fn new(
        registry: Arc<R>,
        store: Arc<S>,
        poll_interval: Duration,
        reorg_lookback: u64,
    ) -> Self {
        Self {
            registry,
            store,
            poll_interval,
            reorg_lookback,
        }
    }

    fn ordinal_to_name(ordinal: u64) -> Name {
        Name::from_ordinal(ordinal)
    }

    async fn upsert_zsk(&self, ordinal: u64, zsk: [u8; 32], ns: &str) {
        let name = Self::ordinal_to_name(ordinal);
        if let Err(e) = self.store.set_zsk(&name, zsk).await {
            error!("failed to store ZSK for ordinal {ordinal}: {e}");
        }
        if let Err(e) = self.store.set_ns(&name, ns).await {
            error!("failed to store NS for ordinal {ordinal}: {e}");
        }
    }

    /// Initial full sync: iterate all registered batches and fetch their zone configs.
    pub async fn initial_sync(&self) -> Result<(), crate::registry::RegistryError> {
        info!("starting initial sync from registry");

        let total = self.registry.next_ordinal().await?;
        info!("total registered ordinals: {total}");

        let batch_size: u64 = 256;
        let mut ordinal = 0u64;
        while ordinal < total {
            if let Some(config) = self.registry.get_zone_config(ordinal).await? {
                self.upsert_zsk(ordinal, config.zsk, &config.ns).await;
            }
            ordinal += batch_size;
        }

        info!("initial sync complete");
        Ok(())
    }

    /// Run the ongoing poll loop, processing new events.
    pub async fn run(&self) {
        loop {
            let from = match self.store.get_checkpoint().await {
                Ok(Some(block)) => block.saturating_sub(self.reorg_lookback),
                Ok(None) => 0,
                Err(e) => {
                    error!("failed to read checkpoint: {e}");
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
                        info!(
                            "processing event: ordinal={ordinal} zsk={}",
                            hex::encode(zsk)
                        );
                        self.upsert_zsk(ordinal, zsk, &ns).await;

                        // If the ZSK changed, also evict any cached signed packet for this name.
                        let name = Self::ordinal_to_name(ordinal);
                        if let Err(e) = self.store.set_signed_packet(&name, &[]).await {
                            error!("failed to evict packet for ordinal {ordinal}: {e}");
                        }
                    }

                    if !events.is_empty() {
                        info!("processed {} events, checkpoint={safe_block}", events.len());
                    }

                    if let Err(e) = self.store.set_checkpoint(safe_block).await {
                        error!("failed to store checkpoint: {e}");
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
