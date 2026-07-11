use std::num::NonZeroUsize;
use std::sync::Mutex;

use lru::LruCache;
use mns::{Name, ZSK_LEN};

const STAGING_CAPACITY: usize = 100_000;

pub struct StagedPacket {
    pub zsk: [u8; ZSK_LEN],
    pub packet: Vec<u8>,
}

pub struct PacketStaging {
    cache: Mutex<LruCache<Name, StagedPacket>>,
}

impl PacketStaging {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(STAGING_CAPACITY).unwrap())),
        }
    }

    /// Stage a packet whose ZSK doesn't yet match the on-chain registry.
    /// Returns true if staged, false if the cache is full.
    pub fn stage(&self, name: Name, zsk: [u8; ZSK_LEN], packet: Vec<u8>) -> bool {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= STAGING_CAPACITY && !cache.contains(&name) {
            return false;
        }
        cache.put(name, StagedPacket { zsk, packet });
        true
    }

    /// Promote a staged packet for `name` if its embedded ZSK matches
    /// `expected_zsk`. Returns the packet bytes and removes it from staging.
    pub fn promote(&self, name: &Name, expected_zsk: &[u8; ZSK_LEN]) -> Option<Vec<u8>> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(staged) = cache.peek(name) {
            if &staged.zsk == expected_zsk {
                return cache.pop(name).map(|s| s.packet);
            }
        }
        None
    }

    /// Drop a staged packet for a name (e.g. if it turns out to be bogus).
    pub fn remove(&self, name: &Name) {
        let mut cache = self.cache.lock().unwrap();
        cache.pop(name);
    }
}
