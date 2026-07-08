use mns::Name;
use mns::ZSK_LEN;
use serde::Deserialize;
use serde::Serialize;

pub mod lmdb;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConfig {
    pub zsk: [u8; ZSK_LEN],
    pub ns: String,
    pub owner: [u8; 20],
}

#[async_trait::async_trait]
pub trait ZoneStore: Send + Sync {
    /// Batch config for a batch-start ordinal.
    async fn get_batch(&self, ordinal: u64) -> Result<Option<StoredConfig>, StoreError>;
    async fn set_batch(&self, ordinal: u64, config: &StoredConfig) -> Result<(), StoreError>;

    /// Entry config for an entry ordinal.
    async fn get_entry(&self, ordinal: u64) -> Result<Option<StoredConfig>, StoreError>;
    async fn set_entry(&self, ordinal: u64, config: &StoredConfig) -> Result<(), StoreError>;

    /// Cached signed packet per name.
    async fn get_signed_packet(&self, name: &Name) -> Result<Option<Vec<u8>>, StoreError>;
    async fn set_signed_packet(&self, name: &Name, packet: &[u8]) -> Result<(), StoreError>;

    /// Event sync block (meta).
    async fn get_last_sync_block_number(&self) -> Result<Option<u64>, StoreError>;
    async fn set_last_sync_block_number(&self, block: u64) -> Result<(), StoreError>;

    /// Batch-start ordinals owned by this address.
    async fn get_owner_batches(&self, owner: &[u8; 20]) -> Result<Vec<u64>, StoreError>;
    /// Entry ordinals owned by this address.
    async fn get_owner_entries(&self, owner: &[u8; 20]) -> Result<Vec<u64>, StoreError>;

    /// Convenience: resolve ZSK for a name — checks entries, then falls back to batches.
    async fn get_zsk(&self, name: &Name) -> Result<Option<[u8; ZSK_LEN]>, StoreError>;
    /// Convenience: resolve NS for a name — checks entries, then falls back to batches.
    async fn get_ns(&self, name: &Name) -> Result<Option<String>, StoreError>;
    /// Convenience: resolve owner for a name — checks entries, then falls back to batches.
    async fn get_name_owner(&self, name: &Name) -> Result<Option<[u8; 20]>, StoreError>;

    /// All unique owners (registry keys — O(number of owners), no dedup scan).
    async fn all_owners(&self) -> Result<Vec<[u8; 20]>, StoreError>;
    /// Total unique owner count (O(1)).
    async fn total_owners(&self) -> Result<u64, StoreError>;

    /// Total number of batch registrations.
    async fn total_batches(&self) -> Result<u64, StoreError>;
    /// Total number of entry (individual name) overrides.
    async fn total_entries(&self) -> Result<u64, StoreError>;
    /// Total number of signed packets stored.
    async fn total_packets(&self) -> Result<u64, StoreError>;

    /// Total unique name servers across all batches and entries.
    async fn total_ns(&self) -> Result<u64, StoreError>;
    /// All unique name server strings.
    async fn all_ns(&self) -> Result<Vec<String>, StoreError>;

    /// Total unique ZSKs across all batches and entries (O(1)).
    async fn total_zsks(&self) -> Result<u64, StoreError>;
    /// Batch-start ordinals whose config uses this ZSK.
    async fn get_zsk_batches(&self, zsk: &[u8; ZSK_LEN]) -> Result<Vec<u64>, StoreError>;
    /// Entry ordinals whose config uses this ZSK.
    async fn get_zsk_entries(&self, zsk: &[u8; ZSK_LEN]) -> Result<Vec<u64>, StoreError>;
}
