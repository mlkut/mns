use mns::Name;
use mns::ZSK_LEN;

pub mod lmdb;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}

#[async_trait::async_trait]
pub trait ZoneStore: Send + Sync {
    async fn get_zsk(&self, name: &Name) -> Result<Option<[u8; ZSK_LEN]>, StoreError>;
    async fn set_zsk(&self, name: &Name, zsk: [u8; ZSK_LEN]) -> Result<(), StoreError>;

    async fn get_ns(&self, name: &Name) -> Result<Option<String>, StoreError>;
    async fn set_ns(&self, name: &Name, ns: &str) -> Result<(), StoreError>;

    async fn get_signed_packet(&self, name: &Name) -> Result<Option<Vec<u8>>, StoreError>;
    async fn set_signed_packet(&self, name: &Name, packet: &[u8]) -> Result<(), StoreError>;

    async fn get_checkpoint(&self) -> Result<Option<u64>, StoreError>;
    async fn set_checkpoint(&self, block: u64) -> Result<(), StoreError>;
}
