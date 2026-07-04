use std::path::Path;
use std::sync::Arc;

use mns::Name;
use mns::ZSK_LEN;
use sneed::codec::SerdeWincode;
use sneed::env::OpenOptions;
use sneed::DatabaseUnique;
use sneed::Env;
use tokio::sync::Mutex;

use super::{StoreError, ZoneStore};

const DB_NAME_ZSK: &str = "name-zsk";
const DB_NAME_NS: &str = "name-ns";
const DB_NAME_PACKET: &str = "name-packet";
const DB_NAME_META: &str = "meta";
const META_KEY_CHECKPOINT: &str = "checkpoint";

fn name_key(name: &Name) -> [u8; 5] {
    name.to_wire_bytes()
}

type ZskDb = DatabaseUnique<SerdeWincode<[u8; 5]>, SerdeWincode<[u8; ZSK_LEN]>>;
type NsDb = DatabaseUnique<SerdeWincode<[u8; 5]>, SerdeWincode<String>>;
type PacketDb = DatabaseUnique<SerdeWincode<[u8; 5]>, SerdeWincode<Vec<u8>>>;
type MetaDb = DatabaseUnique<SerdeWincode<String>, SerdeWincode<Vec<u8>>>;

pub struct LmdbStore {
    env: Arc<Env>,
    zsk_db: ZskDb,
    ns_db: NsDb,
    packet_db: PacketDb,
    meta_db: MetaDb,
    write_lock: Arc<Mutex<()>>,
}

impl LmdbStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        std::fs::create_dir_all(path).map_err(|e| StoreError::Db(e.to_string()))?;

        let mut opts = OpenOptions::new();
        opts.max_dbs(4);
        let env = unsafe { Env::open(&opts, path).map_err(|e| StoreError::Db(e.to_string()))? };
        let env = Arc::new(env);

        let mut wtxn = env.write_txn().map_err(|e| StoreError::Db(e.to_string()))?;

        let zsk_db = ZskDb::create(&env, &mut wtxn, DB_NAME_ZSK)
            .map_err(|e| StoreError::Db(e.to_string()))?;

        let ns_db =
            NsDb::create(&env, &mut wtxn, DB_NAME_NS).map_err(|e| StoreError::Db(e.to_string()))?;

        let packet_db = PacketDb::create(&env, &mut wtxn, DB_NAME_PACKET)
            .map_err(|e| StoreError::Db(e.to_string()))?;

        let meta_db = MetaDb::create(&env, &mut wtxn, DB_NAME_META)
            .map_err(|e| StoreError::Db(e.to_string()))?;

        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;

        Ok(Self {
            env,
            zsk_db,
            ns_db,
            packet_db,
            meta_db,
            write_lock: Arc::new(Mutex::new(())),
        })
    }
}

#[async_trait::async_trait]
impl ZoneStore for LmdbStore {
    async fn get_zsk(&self, name: &Name) -> Result<Option<[u8; ZSK_LEN]>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = name_key(name);
        self.zsk_db
            .try_get(&rtxn, &key)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn set_zsk(&self, name: &Name, zsk: [u8; ZSK_LEN]) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = name_key(name);
        self.zsk_db
            .put(&mut wtxn, &key, &zsk)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    async fn get_ns(&self, name: &Name) -> Result<Option<String>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = name_key(name);
        self.ns_db
            .try_get(&rtxn, &key)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn set_ns(&self, name: &Name, ns: &str) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = name_key(name);
        self.ns_db
            .put(&mut wtxn, &key, &ns.to_string())
            .map_err(|e| StoreError::Db(e.to_string()))?;
        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    async fn get_signed_packet(&self, name: &Name) -> Result<Option<Vec<u8>>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = name_key(name);
        self.packet_db
            .try_get(&rtxn, &key)
            .map_err(|e| StoreError::Db(e.to_string()))
    }

    async fn set_signed_packet(&self, name: &Name, packet: &[u8]) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = name_key(name);
        self.packet_db
            .put(&mut wtxn, &key, &packet.to_vec())
            .map_err(|e| StoreError::Db(e.to_string()))?;
        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    async fn get_checkpoint(&self) -> Result<Option<u64>, StoreError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = META_KEY_CHECKPOINT.to_string();
        let result: Option<Vec<u8>> = self
            .meta_db
            .try_get(&rtxn, &key)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        match result {
            Some(bytes) => {
                let arr: [u8; 8] = bytes[..8]
                    .try_into()
                    .map_err(|_| StoreError::Serialization("invalid checkpoint".into()))?;
                Ok(Some(u64::from_be_bytes(arr)))
            }
            None => Ok(None),
        }
    }

    async fn set_checkpoint(&self, block: u64) -> Result<(), StoreError> {
        let _lock = self.write_lock.lock().await;
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StoreError::Db(e.to_string()))?;
        let key = META_KEY_CHECKPOINT.to_string();
        let bytes = block.to_be_bytes().to_vec();
        self.meta_db
            .put(&mut wtxn, &key, &bytes)
            .map_err(|e| StoreError::Db(e.to_string()))?;
        wtxn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }
}
