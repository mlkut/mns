use std::path::Path;
use std::sync::Arc;

use fallible_iterator::FallibleIterator;
use mns::Name;
use mns::ZSK_LEN;
use sneed::codec::SerdeWincode;
use sneed::env::OpenOptions;
use sneed::DatabaseUnique;
use sneed::Env;
use tokio::sync::Mutex;

use super::{StoreError, StoredConfig, ZoneStore};

const BATCH_SIZE: u64 = 256;

const LAST_SYNC_BLOCK_NUMBER_KEY: &str = "last-sync-block-number";

fn batch_start_ordinal(ordinal: u64) -> u64 {
    ordinal - (ordinal % BATCH_SIZE)
}

fn ordinal_key(ordinal: u64) -> [u8; 8] {
    ordinal.to_be_bytes()
}

fn name_key(name: &Name) -> [u8; 5] {
    name.to_wire_bytes()
}

type BatchesDb = DatabaseUnique<SerdeWincode<[u8; 8]>, SerdeWincode<StoredConfig>>;
type EntriesDb = DatabaseUnique<SerdeWincode<[u8; 8]>, SerdeWincode<StoredConfig>>;
type PacketDb = DatabaseUnique<SerdeWincode<[u8; 5]>, SerdeWincode<Vec<u8>>>;
type MetaDb = DatabaseUnique<SerdeWincode<String>, SerdeWincode<Vec<u8>>>;
type OwnerBatchesDb = DatabaseUnique<SerdeWincode<[u8; 20]>, SerdeWincode<Vec<u64>>>;
type OwnerEntriesDb = DatabaseUnique<SerdeWincode<[u8; 20]>, SerdeWincode<Vec<u64>>>;
type NsDb = DatabaseUnique<SerdeWincode<String>, SerdeWincode<u64>>;

pub struct LmdbStore {
    env: Arc<Env>,
    batches_db: BatchesDb,
    entries_db: EntriesDb,
    packet_db: PacketDb,
    meta_db: MetaDb,
    owner_batches_db: OwnerBatchesDb,
    owner_entries_db: OwnerEntriesDb,
    ns_db: NsDb,
    write_lock: Arc<Mutex<()>>,
}

impl LmdbStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        std::fs::create_dir_all(path).map_err(|e| StoreError::Db(e.to_string()))?;

        let mut opts = OpenOptions::new();
        opts.max_dbs(8);
        let env = unsafe { Env::open(&opts, path).map_err(|e| StoreError::Db(e.to_string()))? };
        let env = Arc::new(env);

        let mut wtxn = env.write_txn().map_err(|e| StoreError::Db(e.to_string()))?;

        let batches_db = BatchesDb::create(&env, &mut wtxn, "batches")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let entries_db = EntriesDb::create(&env, &mut wtxn, "entries")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let packet_db = PacketDb::create(&env, &mut wtxn, "name-packet")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let meta_db =
            MetaDb::create(&env, &mut wtxn, "meta").map_err(|e| StoreError::Db(e.to_string()))?;
        let owner_batches_db = OwnerBatchesDb::create(&env, &mut wtxn, "owner-batches")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let owner_entries_db = OwnerEntriesDb::create(&env, &mut wtxn, "owner-entries")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let ns_db =
            NsDb::create(&env, &mut wtxn, "ns").map_err(|e| StoreError::Db(e.to_string()))?;

        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;

        Ok(Self {
            env,
            batches_db,
            entries_db,
            packet_db,
            meta_db,
            owner_batches_db,
            owner_entries_db,
            ns_db,
            write_lock: Arc::new(Mutex::new(())),
        })
    }
}

#[async_trait::async_trait]
impl ZoneStore for LmdbStore {
    // ── Batches ──

    async fn get_batch(&self, ordinal: u64) -> Result<Option<StoredConfig>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.batches_db
            .try_get(&rtxn, &ordinal_key(ordinal))
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn set_batch(&self, ordinal: u64, config: &StoredConfig) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;

        let old = self
            .batches_db
            .try_get(&wtxn, &ordinal_key(ordinal))
            .map_err(|e| StoreError::Db(e.to_string()))?;

        // NS refcount tracking
        match &old {
            Some(old_config) if old_config.ns != config.ns => {
                let rc = self
                    .ns_db
                    .try_get(&wtxn, &old_config.ns)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or(0);
                if rc <= 1 {
                    self.ns_db
                        .delete(&mut wtxn, &old_config.ns)
                        .map_err(|e| StoreError::Db(format!("delete ns: {e}")))?;
                } else {
                    self.ns_db
                        .put(&mut wtxn, &old_config.ns, &(rc - 1))
                        .map_err(|e| StoreError::Db(e.to_string()))?;
                }
                let rc = self
                    .ns_db
                    .try_get(&wtxn, &config.ns)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or(0);
                self.ns_db
                    .put(&mut wtxn, &config.ns, &(rc + 1))
                    .map_err(|e| StoreError::Db(e.to_string()))?;
            }
            None => {
                let rc = self
                    .ns_db
                    .try_get(&wtxn, &config.ns)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or(0);
                self.ns_db
                    .put(&mut wtxn, &config.ns, &(rc + 1))
                    .map_err(|e| StoreError::Db(e.to_string()))?;
            }
            _ => {}
        }

        // Handle owner transfer
        if let Some(ref old_config) = old {
            if old_config.owner != config.owner {
                // Remove from old owner
                let mut list: Vec<u64> = self
                    .owner_batches_db
                    .try_get(&wtxn, &old_config.owner)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or_default();
                list.retain(|&b| b != ordinal);
                if list.is_empty() {
                    self.owner_batches_db
                        .delete(&mut wtxn, &old_config.owner)
                        .map_err(|e| StoreError::Db(format!("delete: {e}")))?;
                } else {
                    self.owner_batches_db
                        .put(&mut wtxn, &old_config.owner, &list)
                        .map_err(|e| StoreError::Db(e.to_string()))?;
                }
                // Add to new owner
                let mut list: Vec<u64> = self
                    .owner_batches_db
                    .try_get(&wtxn, &config.owner)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or_default();
                if !list.contains(&ordinal) {
                    list.push(ordinal);
                }
                self.owner_batches_db
                    .put(&mut wtxn, &config.owner, &list)
                    .map_err(|e| StoreError::Db(e.to_string()))?;
            }
        } else {
            // New batch: track owner
            let mut list: Vec<u64> = self
                .owner_batches_db
                .try_get(&wtxn, &config.owner)
                .map_err(|e| StoreError::Db(e.to_string()))?
                .unwrap_or_default();
            if !list.contains(&ordinal) {
                list.push(ordinal);
            }
            self.owner_batches_db
                .put(&mut wtxn, &config.owner, &list)
                .map_err(|e| StoreError::Db(e.to_string()))?;
        }

        self.batches_db
            .put(&mut wtxn, &ordinal_key(ordinal), config)
            .map_err(|e| StoreError::Db(e.to_string()))?;

        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // ── Entries ──

    async fn get_entry(&self, ordinal: u64) -> Result<Option<StoredConfig>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.entries_db
            .try_get(&rtxn, &ordinal_key(ordinal))
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn set_entry(&self, ordinal: u64, config: &StoredConfig) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;

        let old = self
            .entries_db
            .try_get(&wtxn, &ordinal_key(ordinal))
            .map_err(|e| StoreError::Db(e.to_string()))?;

        // NS refcount tracking
        match &old {
            Some(old_config) if old_config.ns != config.ns => {
                let rc = self
                    .ns_db
                    .try_get(&wtxn, &old_config.ns)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or(0);
                if rc <= 1 {
                    self.ns_db
                        .delete(&mut wtxn, &old_config.ns)
                        .map_err(|e| StoreError::Db(format!("delete ns: {e}")))?;
                } else {
                    self.ns_db
                        .put(&mut wtxn, &old_config.ns, &(rc - 1))
                        .map_err(|e| StoreError::Db(e.to_string()))?;
                }
                let rc = self
                    .ns_db
                    .try_get(&wtxn, &config.ns)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or(0);
                self.ns_db
                    .put(&mut wtxn, &config.ns, &(rc + 1))
                    .map_err(|e| StoreError::Db(e.to_string()))?;
            }
            None => {
                let rc = self
                    .ns_db
                    .try_get(&wtxn, &config.ns)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or(0);
                self.ns_db
                    .put(&mut wtxn, &config.ns, &(rc + 1))
                    .map_err(|e| StoreError::Db(e.to_string()))?;
            }
            _ => {}
        }

        // Handle owner transfer
        if let Some(ref old_config) = old {
            if old_config.owner != config.owner {
                // Remove from old owner
                let mut list: Vec<u64> = self
                    .owner_entries_db
                    .try_get(&wtxn, &old_config.owner)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or_default();
                list.retain(|&e| e != ordinal);
                if list.is_empty() {
                    self.owner_entries_db
                        .delete(&mut wtxn, &old_config.owner)
                        .map_err(|e| StoreError::Db(format!("delete: {e}")))?;
                } else {
                    self.owner_entries_db
                        .put(&mut wtxn, &old_config.owner, &list)
                        .map_err(|e| StoreError::Db(e.to_string()))?;
                }
                // Add to new owner
                let mut list: Vec<u64> = self
                    .owner_entries_db
                    .try_get(&wtxn, &config.owner)
                    .map_err(|e| StoreError::Db(e.to_string()))?
                    .unwrap_or_default();
                if !list.contains(&ordinal) {
                    list.push(ordinal);
                }
                self.owner_entries_db
                    .put(&mut wtxn, &config.owner, &list)
                    .map_err(|e| StoreError::Db(e.to_string()))?;
            }
        } else {
            // New entry: track owner
            let mut list: Vec<u64> = self
                .owner_entries_db
                .try_get(&wtxn, &config.owner)
                .map_err(|e| StoreError::Db(e.to_string()))?
                .unwrap_or_default();
            if !list.contains(&ordinal) {
                list.push(ordinal);
            }
            self.owner_entries_db
                .put(&mut wtxn, &config.owner, &list)
                .map_err(|e| StoreError::Db(e.to_string()))?;
        }

        self.entries_db
            .put(&mut wtxn, &ordinal_key(ordinal), config)
            .map_err(|e| StoreError::Db(e.to_string()))?;

        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // ── Signed packets ──

    async fn get_signed_packet(&self, name: &Name) -> Result<Option<Vec<u8>>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.packet_db
            .try_get(&rtxn, &name_key(name))
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn set_signed_packet(&self, name: &Name, packet: &[u8]) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.packet_db
            .put(&mut wtxn, &name_key(name), &packet.to_vec())
            .map_err(|e| StoreError::Db(e.to_string()))?;
        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // ── Last sync block number ──

    async fn get_last_sync_block_number(&self) -> Result<Option<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = LAST_SYNC_BLOCK_NUMBER_KEY.to_string();
        let result: Option<Vec<u8>> = self
            .meta_db
            .try_get(&rtxn, &key)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        match result {
            Some(bytes) => {
                let arr: [u8; 8] = bytes[..8].try_into().map_err(|_| {
                    StoreError::Serialization("invalid LAST_SYNC_BLOCK_NUMBER_KEY".into())
                })?;
                Ok(Some(u64::from_be_bytes(arr)))
            }
            None => Ok(None),
        }
    }

    async fn set_last_sync_block_number(&self, block: u64) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = LAST_SYNC_BLOCK_NUMBER_KEY.to_string();
        let bytes = block.to_be_bytes().to_vec();
        self.meta_db
            .put(&mut wtxn, &key, &bytes)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // ── Owner indexes ──

    async fn get_owner_batches(&self, owner: &[u8; 20]) -> Result<Vec<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.owner_batches_db
            .try_get(&rtxn, owner)
            .map_err(|e| StoreError::Db(e.to_string()))
            .map(|opt| opt.unwrap_or_default())
    }

    async fn get_owner_entries(&self, owner: &[u8; 20]) -> Result<Vec<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.owner_entries_db
            .try_get(&rtxn, owner)
            .map_err(|e| StoreError::Db(e.to_string()))
            .map(|opt| opt.unwrap_or_default())
    }

    // ── Convenience: layered lookup (entry → batch) ──

    async fn get_zsk(&self, name: &Name) -> Result<Option<[u8; ZSK_LEN]>, StoreError> {
        let ordinal = name.to_ordinal();
        if let Some(entry) = self.get_entry(ordinal).await? {
            return Ok(Some(entry.zsk));
        }
        let batch_start = batch_start_ordinal(ordinal);
        if let Some(batch) = self.get_batch(batch_start).await? {
            return Ok(Some(batch.zsk));
        }
        Ok(None)
    }

    async fn get_ns(&self, name: &Name) -> Result<Option<String>, StoreError> {
        let ordinal = name.to_ordinal();
        if let Some(entry) = self.get_entry(ordinal).await? {
            return Ok(Some(entry.ns));
        }
        let batch_start = batch_start_ordinal(ordinal);
        if let Some(batch) = self.get_batch(batch_start).await? {
            return Ok(Some(batch.ns));
        }
        Ok(None)
    }

    async fn get_name_owner(&self, name: &Name) -> Result<Option<[u8; 20]>, StoreError> {
        let ordinal = name.to_ordinal();
        if let Some(entry) = self.get_entry(ordinal).await? {
            return Ok(Some(entry.owner));
        }
        let batch_start = batch_start_ordinal(ordinal);
        if let Some(batch) = self.get_batch(batch_start).await? {
            return Ok(Some(batch.owner));
        }
        Ok(None)
    }

    // ── Owner enumeration (union of owner-batches + owner-entries keys) ──

    async fn all_owners(&self) -> Result<Vec<[u8; 20]>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let mut seen = std::collections::BTreeSet::new();

        let mut iter = self
            .owner_batches_db
            .iter_keys(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        while let Some(addr) = iter.next().map_err(|e| StoreError::Db(e.to_string()))? {
            seen.insert(addr);
        }

        let mut iter = self
            .owner_entries_db
            .iter_keys(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        while let Some(addr) = iter.next().map_err(|e| StoreError::Db(e.to_string()))? {
            seen.insert(addr);
        }

        Ok(seen.into_iter().collect())
    }

    async fn total_owners(&self) -> Result<u64, StoreError> {
        let owners = self.all_owners().await?;
        Ok(owners.len() as u64)
    }

    async fn total_batches(&self) -> Result<u64, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.batches_db
            .len(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn total_entries(&self) -> Result<u64, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.entries_db
            .len(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn total_packets(&self) -> Result<u64, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.packet_db
            .len(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn total_ns(&self) -> Result<u64, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.ns_db
            .len(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn all_ns(&self) -> Result<Vec<String>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let mut iter = self
            .ns_db
            .iter_keys(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let mut result = Vec::new();
        while let Some(ns) = iter.next().map_err(|e| StoreError::Db(e.to_string()))? {
            result.push(ns);
        }
        Ok(result)
    }
}
