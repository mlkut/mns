pub mod alloy;

#[derive(Debug, Clone)]
pub struct ZoneConfig {
    pub zsk: [u8; 32],
    pub ns: String,
}

#[derive(Debug, Clone)]
pub enum RegistryEvent {
    BatchRegistered {
        ordinal: u64,
        zsk: [u8; 32],
        ns: String,
    },
    BatchUpdated {
        ordinal: u64,
        zsk: [u8; 32],
        ns: String,
    },
    EntryCreated {
        ordinal: u64,
        zsk: [u8; 32],
        ns: String,
    },
    EntryUpdated {
        ordinal: u64,
        zsk: [u8; 32],
        ns: String,
    },
}

impl RegistryEvent {
    pub fn ordinal(&self) -> u64 {
        match self {
            RegistryEvent::BatchRegistered { ordinal, .. }
            | RegistryEvent::BatchUpdated { ordinal, .. }
            | RegistryEvent::EntryCreated { ordinal, .. }
            | RegistryEvent::EntryUpdated { ordinal, .. } => *ordinal,
        }
    }

    pub fn zsk(&self) -> [u8; 32] {
        match self {
            RegistryEvent::BatchRegistered { zsk, .. }
            | RegistryEvent::BatchUpdated { zsk, .. }
            | RegistryEvent::EntryCreated { zsk, .. }
            | RegistryEvent::EntryUpdated { zsk, .. } => *zsk,
        }
    }

    pub fn ns(&self) -> &str {
        match self {
            RegistryEvent::BatchRegistered { ns, .. }
            | RegistryEvent::BatchUpdated { ns, .. }
            | RegistryEvent::EntryCreated { ns, .. }
            | RegistryEvent::EntryUpdated { ns, .. } => ns,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("contract error: {0}")]
    Contract(String),

    #[allow(dead_code)]
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

#[async_trait::async_trait]
pub trait RegistryReader: Send + Sync {
    /// Total registered ordinals (nextOrdinal).
    async fn next_ordinal(&self) -> Result<u64, RegistryError>;

    /// Zone config (ZSK + NS) for an ordinal, read at the safe block.
    async fn get_zone_config(&self, ordinal: u64) -> Result<Option<ZoneConfig>, RegistryError>;

    /// Poll for registry events since `from_block`, up to the current safe block.
    /// Returns (events, safe_block_number).
    async fn poll_events(
        &self,
        from_block: u64,
    ) -> Result<(Vec<RegistryEvent>, u64), RegistryError>;
}
