use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: String,
    pub rpc_url: String,
    pub contract_address: String,
    pub db_path: PathBuf,
    pub poll_interval_secs: u64,
    pub reorg_lookback: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:3000".into(),
            rpc_url: "https://public-node.testnet.rsk.co".into(),
            contract_address: "0xe916a48de922e8964542f4c4c66ec4837bbe3445".into(),
            db_path: PathBuf::from(".mns-server/db"),
            poll_interval_secs: 12,
            reorg_lookback: 100,
        }
    }
}
