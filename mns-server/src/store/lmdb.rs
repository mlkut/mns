use std::path::Path;
use std::sync::Arc;

use fallible_iterator::FallibleIterator;
use mns::Name;
use mns::ZSK_LEN;
use sneed::codec::SerdeWincode;
use sneed::env::OpenOptions;
use sneed::DatabaseUnique;
use sneed::Env;
use sneed::RwTxn;
use tokio::sync::Mutex;

use super::{StoreError, StoredConfig, ZoneStore};

const BATCH_SIZE: u64 = 256;

const LAST_SYNC_BLOCK_NUMBER_KEY: &str = "last-sync-block-number";
const LAST_SYNC_BLOCK_TIME_KEY: &str = "last-sync-block-time";
const OWNER_NEXT_ID_KEY: &str = "owner-next-id";
const ZSK_NEXT_ID_KEY: &str = "zsk-next-id";

fn batch_start_ordinal(ordinal: u64) -> u64 {
    ordinal - (ordinal % BATCH_SIZE)
}

fn ordinal_key(ordinal: u64) -> [u8; 8] {
    ordinal.to_be_bytes()
}

fn name_key(name: &Name) -> [u8; 5] {
    name.to_wire_bytes()
}

// ── Databases ──────────────────────────────────────────────────────────────
//
// Owners and ZSKs each get a compact registry: a `key -> u64 id` map that is
// the canonical set of that entity. `.len()` on the registry is an O(1) unique
// count, and iterating its keys lists every entity — no full-table scan. The
// per-entity reverse indexes (id -> ordinals) are keyed by the compact id.

type BatchesDb = DatabaseUnique<SerdeWincode<[u8; 8]>, SerdeWincode<StoredConfig>>;
type EntriesDb = DatabaseUnique<SerdeWincode<[u8; 8]>, SerdeWincode<StoredConfig>>;
type PacketDb = DatabaseUnique<SerdeWincode<[u8; 5]>, SerdeWincode<Vec<u8>>>;
type MetaDb = DatabaseUnique<SerdeWincode<String>, SerdeWincode<Vec<u8>>>;

// Owner registry: address -> compact owner id.
type OwnerIndexDb = DatabaseUnique<SerdeWincode<[u8; 20]>, SerdeWincode<u64>>;
// Reverse indexes keyed by compact owner id.
type OwnerBatchesDb = DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>;
type OwnerEntriesDb = DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>;

type NsDb = DatabaseUnique<SerdeWincode<String>, SerdeWincode<u64>>;

// ZSK registry: zsk hash -> compact zsk id.
type ZskIndexDb = DatabaseUnique<SerdeWincode<[u8; ZSK_LEN]>, SerdeWincode<u64>>;
// Reverse indexes keyed by compact zsk id.
type ZskBatchesDb = DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>;
type ZskEntriesDb = DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>;

pub struct LmdbStore {
    env: Arc<Env>,
    batches_db: BatchesDb,
    entries_db: EntriesDb,
    packet_db: PacketDb,
    meta_db: MetaDb,
    owner_index_db: OwnerIndexDb,
    owner_batches_db: OwnerBatchesDb,
    owner_entries_db: OwnerEntriesDb,
    ns_db: NsDb,
    zsk_index_db: ZskIndexDb,
    zsk_batches_db: ZskBatchesDb,
    zsk_entries_db: ZskEntriesDb,
    write_lock: Arc<Mutex<()>>,
}

impl LmdbStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        std::fs::create_dir_all(path).map_err(|e| StoreError::Db(e.to_string()))?;

        let mut opts = OpenOptions::new();
        opts.max_dbs(12);
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
        let owner_index_db = OwnerIndexDb::create(&env, &mut wtxn, "owner-index")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let owner_batches_db = OwnerBatchesDb::create(&env, &mut wtxn, "owner-batches")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let owner_entries_db = OwnerEntriesDb::create(&env, &mut wtxn, "owner-entries")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let ns_db =
            NsDb::create(&env, &mut wtxn, "ns").map_err(|e| StoreError::Db(e.to_string()))?;
        let zsk_index_db = ZskIndexDb::create(&env, &mut wtxn, "zsk-index")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let zsk_batches_db = ZskBatchesDb::create(&env, &mut wtxn, "zsk-batches")
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let zsk_entries_db = ZskEntriesDb::create(&env, &mut wtxn, "zsk-entries")
            .map_err(|e| StoreError::Db(e.to_string()))?;

        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;

        Ok(Self {
            env,
            batches_db,
            entries_db,
            packet_db,
            meta_db,
            owner_index_db,
            owner_batches_db,
            owner_entries_db,
            ns_db,
            zsk_index_db,
            zsk_batches_db,
            zsk_entries_db,
            write_lock: Arc::new(Mutex::new(())),
        })
    }

    // ── Meta counter helpers ──

    fn meta_get_u64(&self, wtxn: &RwTxn, key: &str) -> Result<u64, StoreError> {
        let bytes: Option<Vec<u8>> = self
            .meta_db
            .try_get(wtxn, &key.to_string())
            .map_err(|e| StoreError::Db(e.to_string()))?;
        match bytes {
            Some(b) => {
                let arr: [u8; 8] = b[..8]
                    .try_into()
                    .map_err(|_| StoreError::Serialization(format!("invalid u64 for {key}")))?;
                Ok(u64::from_be_bytes(arr))
            }
            None => Ok(0),
        }
    }

    fn meta_put_u64(&self, wtxn: &mut RwTxn, key: &str, value: u64) -> Result<(), StoreError> {
        self.meta_db
            .put(wtxn, &key.to_string(), &value.to_be_bytes().to_vec())
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    // ── Registry helpers (address/zsk -> compact id) ──
    //
    // Resolve an existing compact id for a key, or allocate the next monotonic id
    // (never reused). The registry key exists iff the entity is live, so
    // `registry.len()` is an O(1) unique count. serde's `Serialize` isn't
    // implemented for a generic `[u8; N]`, so these are spelled out per registry.

    fn owner_id_lookup(&self, wtxn: &RwTxn, owner: &[u8; 20]) -> Result<Option<u64>, StoreError> {
        self.owner_index_db
            .try_get(wtxn, owner)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    fn owner_id_get_or_alloc(&self, wtxn: &mut RwTxn, owner: &[u8; 20]) -> Result<u64, StoreError> {
        if let Some(id) = self.owner_id_lookup(wtxn, owner)? {
            return Ok(id);
        }
        let id = self.meta_get_u64(wtxn, OWNER_NEXT_ID_KEY)?;
        self.owner_index_db
            .put(wtxn, owner, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.meta_put_u64(wtxn, OWNER_NEXT_ID_KEY, id + 1)?;
        Ok(id)
    }

    fn zsk_id_lookup(&self, wtxn: &RwTxn, zsk: &[u8; ZSK_LEN]) -> Result<Option<u64>, StoreError> {
        self.zsk_index_db
            .try_get(wtxn, zsk)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    fn zsk_id_get_or_alloc(
        &self,
        wtxn: &mut RwTxn,
        zsk: &[u8; ZSK_LEN],
    ) -> Result<u64, StoreError> {
        if let Some(id) = self.zsk_id_lookup(wtxn, zsk)? {
            return Ok(id);
        }
        let id = self.meta_get_u64(wtxn, ZSK_NEXT_ID_KEY)?;
        self.zsk_index_db
            .put(wtxn, zsk, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.meta_put_u64(wtxn, ZSK_NEXT_ID_KEY, id + 1)?;
        Ok(id)
    }

    // ── Reverse-index helpers (compact id -> Vec<ordinal>) ──

    fn index_add(
        &self,
        db: &DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>,
        wtxn: &mut RwTxn,
        id: u64,
        ordinal: u64,
    ) -> Result<(), StoreError> {
        let mut list: Vec<u64> = db
            .try_get(wtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?
            .unwrap_or_default();
        if !list.contains(&ordinal) {
            list.push(ordinal);
        }
        db.put(wtxn, &id, &list)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    /// Remove `ordinal` from id's list. Returns true if the list became empty
    /// (and was deleted), signalling the entity now holds no records.
    fn index_remove(
        &self,
        db: &DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>,
        wtxn: &mut RwTxn,
        id: u64,
        ordinal: u64,
    ) -> Result<bool, StoreError> {
        let mut list: Vec<u64> = db
            .try_get(wtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?
            .unwrap_or_default();
        list.retain(|&x| x != ordinal);
        if list.is_empty() {
            db.delete(wtxn, &id)
                .map_err(|e| StoreError::Db(e.to_string()))?;
            Ok(true)
        } else {
            db.put(wtxn, &id, &list)
                .map_err(|e| StoreError::Db(e.to_string()))?;
            Ok(false)
        }
    }

    // ── NS refcount helper ──

    fn ns_inc(&self, wtxn: &mut RwTxn, ns: &str) -> Result<(), StoreError> {
        let rc = self
            .ns_db
            .try_get(wtxn, &ns.to_string())
            .map_err(|e| StoreError::Db(e.to_string()))?
            .unwrap_or(0);
        self.ns_db
            .put(wtxn, &ns.to_string(), &(rc + 1))
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    fn ns_dec(&self, wtxn: &mut RwTxn, ns: &str) -> Result<(), StoreError> {
        let rc = self
            .ns_db
            .try_get(wtxn, &ns.to_string())
            .map_err(|e| StoreError::Db(e.to_string()))?
            .unwrap_or(0);
        if rc <= 1 {
            self.ns_db
                .delete(wtxn, &ns.to_string())
                .map_err(|e| StoreError::Db(format!("delete ns: {e}")))?;
        } else {
            self.ns_db
                .put(wtxn, &ns.to_string(), &(rc - 1))
                .map_err(|e| StoreError::Db(e.to_string()))?;
        }
        Ok(())
    }

    /// Apply the owner/zsk/ns index maintenance for a config write.
    ///
    /// `record_db` selects which reverse index (batches vs entries) to update.
    /// Handles the four cases uniformly for owner and zsk registries plus the
    /// ns refcount: new record, unchanged, or a change of owner/zsk/ns.
    #[allow(clippy::too_many_arguments)]
    fn apply_indexes(
        &self,
        wtxn: &mut RwTxn,
        ordinal: u64,
        old: Option<&StoredConfig>,
        config: &StoredConfig,
        owner_record_db: &DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>,
        zsk_record_db: &DatabaseUnique<SerdeWincode<u64>, SerdeWincode<Vec<u64>>>,
    ) -> Result<(), StoreError> {
        // ── NS refcount ──
        match old {
            Some(o) if o.ns != config.ns => {
                self.ns_dec(wtxn, &o.ns)?;
                self.ns_inc(wtxn, &config.ns)?;
            }
            None => self.ns_inc(wtxn, &config.ns)?,
            _ => {}
        }

        // ── Owner registry + reverse index ──
        let owner_changed = match old {
            Some(o) => o.owner != config.owner,
            None => true,
        };
        if owner_changed {
            if let Some(o) = old {
                if let Some(old_id) = self.owner_id_lookup(wtxn, &o.owner)? {
                    let emptied = self.index_remove(owner_record_db, wtxn, old_id, ordinal)?;
                    if emptied {
                        self.maybe_retire_owner(wtxn, old_id, &o.owner)?;
                    }
                }
            }
            let new_id = self.owner_id_get_or_alloc(wtxn, &config.owner)?;
            self.index_add(owner_record_db, wtxn, new_id, ordinal)?;
        }

        // ── ZSK registry + reverse index ──
        let zsk_changed = match old {
            Some(o) => o.zsk != config.zsk,
            None => true,
        };
        if zsk_changed {
            if let Some(o) = old {
                if let Some(old_id) = self.zsk_id_lookup(wtxn, &o.zsk)? {
                    let emptied = self.index_remove(zsk_record_db, wtxn, old_id, ordinal)?;
                    if emptied {
                        self.maybe_retire_zsk(wtxn, old_id, &o.zsk)?;
                    }
                }
            }
            let new_id = self.zsk_id_get_or_alloc(wtxn, &config.zsk)?;
            self.index_add(zsk_record_db, wtxn, new_id, ordinal)?;
        }

        Ok(())
    }

    /// Delete the owner registry entry if the owner holds no batches and no
    /// entries anymore. IDs are never reused.
    fn maybe_retire_owner(
        &self,
        wtxn: &mut RwTxn,
        id: u64,
        owner: &[u8; 20],
    ) -> Result<(), StoreError> {
        let has_batches = self
            .owner_batches_db
            .try_get(wtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?
            .is_some();
        let has_entries = self
            .owner_entries_db
            .try_get(wtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?
            .is_some();
        if !has_batches && !has_entries {
            self.owner_index_db
                .delete(wtxn, owner)
                .map_err(|e| StoreError::Db(format!("delete owner-index: {e}")))?;
        }
        Ok(())
    }

    /// Delete the zsk registry entry if the zsk is used by no batches and no
    /// entries anymore. IDs are never reused.
    fn maybe_retire_zsk(
        &self,
        wtxn: &mut RwTxn,
        id: u64,
        zsk: &[u8; ZSK_LEN],
    ) -> Result<(), StoreError> {
        let has_batches = self
            .zsk_batches_db
            .try_get(wtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?
            .is_some();
        let has_entries = self
            .zsk_entries_db
            .try_get(wtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))?
            .is_some();
        if !has_batches && !has_entries {
            self.zsk_index_db
                .delete(wtxn, zsk)
                .map_err(|e| StoreError::Db(format!("delete zsk-index: {e}")))?;
        }
        Ok(())
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

        // Clone the DB handles we need so we don't hold an immutable borrow of
        // `self` across the &mut wtxn calls inside apply_indexes.
        self.apply_indexes(
            &mut wtxn,
            ordinal,
            old.as_ref(),
            config,
            &self.owner_batches_db,
            &self.zsk_batches_db,
        )?;

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

        self.apply_indexes(
            &mut wtxn,
            ordinal,
            old.as_ref(),
            config,
            &self.owner_entries_db,
            &self.zsk_entries_db,
        )?;

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

    // ── Last sync block time ──

    async fn get_last_sync_block_time(&self) -> Result<Option<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = LAST_SYNC_BLOCK_TIME_KEY.to_string();
        let result: Option<Vec<u8>> = self
            .meta_db
            .try_get(&rtxn, &key)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        match result {
            Some(bytes) => {
                let arr: [u8; 8] = bytes[..8].try_into().map_err(|_| {
                    StoreError::Serialization("invalid LAST_SYNC_BLOCK_TIME_KEY".into())
                })?;
                Ok(Some(u64::from_be_bytes(arr)))
            }
            None => Ok(None),
        }
    }

    async fn set_last_sync_block_time(&self, ts: u64) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = LAST_SYNC_BLOCK_TIME_KEY.to_string();
        let bytes = ts.to_be_bytes().to_vec();
        self.meta_db
            .put(&mut wtxn, &key, &bytes)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // ── Owner indexes (public API keyed by address; resolves via registry) ──

    async fn get_owner_batches(&self, owner: &[u8; 20]) -> Result<Vec<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let id = match self
            .owner_index_db
            .try_get(&rtxn, owner)
            .map_err(|e| StoreError::Db(e.to_string()))?
        {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };
        self.owner_batches_db
            .try_get(&rtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))
            .map(|opt| opt.unwrap_or_default())
    }

    async fn get_owner_entries(&self, owner: &[u8; 20]) -> Result<Vec<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let id = match self
            .owner_index_db
            .try_get(&rtxn, owner)
            .map_err(|e| StoreError::Db(e.to_string()))?
        {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };
        self.owner_entries_db
            .try_get(&rtxn, &id)
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

    // ── Owner enumeration (registry keys — no scan/union needed) ──

    async fn all_owners(&self) -> Result<Vec<[u8; 20]>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let mut iter = self
            .owner_index_db
            .iter_keys(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let mut result = Vec::new();
        while let Some(addr) = iter.next().map_err(|e| StoreError::Db(e.to_string()))? {
            result.push(addr);
        }
        Ok(result)
    }

    async fn total_owners(&self) -> Result<u64, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.owner_index_db
            .len(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))
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

    async fn total_zsks(&self) -> Result<u64, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        self.zsk_index_db
            .len(&rtxn)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn get_zsk_batches(&self, zsk: &[u8; ZSK_LEN]) -> Result<Vec<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let id = match self
            .zsk_index_db
            .try_get(&rtxn, zsk)
            .map_err(|e| StoreError::Db(e.to_string()))?
        {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };
        self.zsk_batches_db
            .try_get(&rtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))
            .map(|opt| opt.unwrap_or_default())
    }

    async fn get_zsk_entries(&self, zsk: &[u8; ZSK_LEN]) -> Result<Vec<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let id = match self
            .zsk_index_db
            .try_get(&rtxn, zsk)
            .map_err(|e| StoreError::Db(e.to_string()))?
        {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };
        self.zsk_entries_db
            .try_get(&rtxn, &id)
            .map_err(|e| StoreError::Db(e.to_string()))
            .map(|opt| opt.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (LmdbStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = LmdbStore::open(dir.path()).unwrap();
        (store, dir)
    }

    fn cfg(zsk: u8, ns: &str, owner: u8) -> StoredConfig {
        StoredConfig {
            zsk: [zsk; ZSK_LEN],
            ns: ns.to_string(),
            owner: [owner; 20],
        }
    }

    fn addr(b: u8) -> [u8; 20] {
        [b; 20]
    }

    fn zk(b: u8) -> [u8; ZSK_LEN] {
        [b; ZSK_LEN]
    }

    #[tokio::test]
    async fn empty_store_has_zero_counts() {
        let (s, _d) = store();
        assert_eq!(s.total_owners().await.unwrap(), 0);
        assert_eq!(s.total_zsks().await.unwrap(), 0);
        assert_eq!(s.total_batches().await.unwrap(), 0);
        assert_eq!(s.total_entries().await.unwrap(), 0);
        assert_eq!(s.total_ns().await.unwrap(), 0);
        assert!(s.all_owners().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn single_batch_registers_owner_and_zsk() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();

        assert_eq!(s.total_owners().await.unwrap(), 1);
        assert_eq!(s.total_zsks().await.unwrap(), 1);
        assert_eq!(s.total_batches().await.unwrap(), 1);
        assert_eq!(s.total_ns().await.unwrap(), 1);
        assert_eq!(s.all_owners().await.unwrap(), vec![addr(1)]);
        assert_eq!(s.get_owner_batches(&addr(1)).await.unwrap(), vec![0]);
        assert_eq!(s.get_zsk_batches(&zk(1)).await.unwrap(), vec![0]);
    }

    #[tokio::test]
    async fn duplicate_owner_and_zsk_counted_once() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
        s.set_batch(256, &cfg(1, "ns1", 1)).await.unwrap();

        assert_eq!(s.total_owners().await.unwrap(), 1);
        assert_eq!(s.total_zsks().await.unwrap(), 1);
        assert_eq!(s.total_ns().await.unwrap(), 1);
        assert_eq!(s.total_batches().await.unwrap(), 2);
        let mut b = s.get_owner_batches(&addr(1)).await.unwrap();
        b.sort();
        assert_eq!(b, vec![0, 256]);
        let mut z = s.get_zsk_batches(&zk(1)).await.unwrap();
        z.sort();
        assert_eq!(z, vec![0, 256]);
    }

    #[tokio::test]
    async fn distinct_owners_and_zsks() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
        s.set_batch(256, &cfg(2, "ns2", 2)).await.unwrap();

        assert_eq!(s.total_owners().await.unwrap(), 2);
        assert_eq!(s.total_zsks().await.unwrap(), 2);
        assert_eq!(s.total_ns().await.unwrap(), 2);
        let mut owners = s.all_owners().await.unwrap();
        owners.sort();
        assert_eq!(owners, vec![addr(1), addr(2)]);
    }

    #[tokio::test]
    async fn owner_transfer_retires_old_owner() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
        // transfer ordinal 0 from owner 1 to owner 2 (same zsk/ns)
        s.set_batch(0, &cfg(1, "ns1", 2)).await.unwrap();

        assert_eq!(s.total_owners().await.unwrap(), 1);
        assert_eq!(s.all_owners().await.unwrap(), vec![addr(2)]);
        assert!(s.get_owner_batches(&addr(1)).await.unwrap().is_empty());
        assert_eq!(s.get_owner_batches(&addr(2)).await.unwrap(), vec![0]);
        // zsk unchanged
        assert_eq!(s.total_zsks().await.unwrap(), 1);
        assert_eq!(s.get_zsk_batches(&zk(1)).await.unwrap(), vec![0]);
    }

    #[tokio::test]
    async fn zsk_change_retires_old_zsk() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
        // change zsk from 1 to 2 (same owner/ns)
        s.set_batch(0, &cfg(2, "ns1", 1)).await.unwrap();

        assert_eq!(s.total_zsks().await.unwrap(), 1);
        assert!(s.get_zsk_batches(&zk(1)).await.unwrap().is_empty());
        assert_eq!(s.get_zsk_batches(&zk(2)).await.unwrap(), vec![0]);
        // owner unchanged
        assert_eq!(s.total_owners().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn entry_contributes_to_owner_and_zsk_counts() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
        // entry override with a different owner and zsk
        s.set_entry(5, &cfg(2, "ns2", 2)).await.unwrap();

        assert_eq!(s.total_owners().await.unwrap(), 2);
        assert_eq!(s.total_zsks().await.unwrap(), 2);
        assert_eq!(s.total_ns().await.unwrap(), 2);
        assert_eq!(s.total_entries().await.unwrap(), 1);
        assert_eq!(s.get_owner_entries(&addr(2)).await.unwrap(), vec![5]);
        assert_eq!(s.get_zsk_entries(&zk(2)).await.unwrap(), vec![5]);
        // batch side still intact
        assert_eq!(s.get_zsk_batches(&zk(1)).await.unwrap(), vec![0]);
    }

    #[tokio::test]
    async fn zsk_shared_between_batch_and_entry_counted_once() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(9, "ns1", 1)).await.unwrap();
        s.set_entry(5, &cfg(9, "ns1", 1)).await.unwrap();

        // same zsk (9) used by a batch and an entry -> unique count 1
        assert_eq!(s.total_zsks().await.unwrap(), 1);
        assert_eq!(s.get_zsk_batches(&zk(9)).await.unwrap(), vec![0]);
        assert_eq!(s.get_zsk_entries(&zk(9)).await.unwrap(), vec![5]);
        // same owner (1) across both -> unique count 1
        assert_eq!(s.total_owners().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn owner_not_retired_while_holding_entry() {
        let (s, _d) = store();
        // owner 1 holds a batch and an entry
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
        s.set_entry(5, &cfg(1, "ns1", 1)).await.unwrap();
        assert_eq!(s.total_owners().await.unwrap(), 1);

        // transfer only the batch to owner 2; owner 1 still holds the entry
        s.set_batch(0, &cfg(1, "ns1", 2)).await.unwrap();

        assert_eq!(s.total_owners().await.unwrap(), 2);
        let mut owners = s.all_owners().await.unwrap();
        owners.sort();
        assert_eq!(owners, vec![addr(1), addr(2)]);
        assert_eq!(s.get_owner_entries(&addr(1)).await.unwrap(), vec![5]);
        assert!(s.get_owner_batches(&addr(1)).await.unwrap().is_empty());
        assert_eq!(s.get_owner_batches(&addr(2)).await.unwrap(), vec![0]);
    }

    #[tokio::test]
    async fn zsk_not_retired_while_used_by_entry() {
        let (s, _d) = store();
        s.set_batch(0, &cfg(7, "ns1", 1)).await.unwrap();
        s.set_entry(5, &cfg(7, "ns1", 1)).await.unwrap();
        assert_eq!(s.total_zsks().await.unwrap(), 1);

        // change only the batch's zsk; entry still uses zsk 7
        s.set_batch(0, &cfg(8, "ns1", 1)).await.unwrap();

        assert_eq!(s.total_zsks().await.unwrap(), 2);
        assert_eq!(s.get_zsk_entries(&zk(7)).await.unwrap(), vec![5]);
        assert!(s.get_zsk_batches(&zk(7)).await.unwrap().is_empty());
        assert_eq!(s.get_zsk_batches(&zk(8)).await.unwrap(), vec![0]);
    }

    #[tokio::test]
    async fn ids_not_reused_after_retire() {
        let (s, _d) = store();
        // create then fully retire owner 1 and zsk 1
        s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
        s.set_batch(0, &cfg(2, "ns1", 2)).await.unwrap(); // moves to owner 2, zsk 2
        assert_eq!(s.total_owners().await.unwrap(), 1);
        assert_eq!(s.total_zsks().await.unwrap(), 1);

        // bring back owner 1 / zsk 1 on a new ordinal; counts should be exact
        s.set_batch(256, &cfg(1, "ns1", 1)).await.unwrap();
        assert_eq!(s.total_owners().await.unwrap(), 2);
        assert_eq!(s.total_zsks().await.unwrap(), 2);
        assert_eq!(s.get_owner_batches(&addr(1)).await.unwrap(), vec![256]);
        assert_eq!(s.get_zsk_batches(&zk(1)).await.unwrap(), vec![256]);
    }

    #[tokio::test]
    async fn reopen_persists_counts_and_indexes() {
        let dir = tempfile::tempdir().unwrap();
        {
            let s = LmdbStore::open(dir.path()).unwrap();
            s.set_batch(0, &cfg(1, "ns1", 1)).await.unwrap();
            s.set_entry(5, &cfg(2, "ns2", 2)).await.unwrap();
        }
        // reopen the same path
        let s = LmdbStore::open(dir.path()).unwrap();
        assert_eq!(s.total_owners().await.unwrap(), 2);
        assert_eq!(s.total_zsks().await.unwrap(), 2);
        assert_eq!(s.get_owner_batches(&addr(1)).await.unwrap(), vec![0]);
        assert_eq!(s.get_zsk_entries(&zk(2)).await.unwrap(), vec![5]);
    }
}
