use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: String,
    pub rpc_url: String,
    pub contract_address: String,
    pub deployment_block: u64,
    pub db_path: PathBuf,
    pub poll_interval_secs: u64,
    pub max_block_range: u64,
    pub public_rpc_url: String,
    pub chain_id: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:3000".into(),
            rpc_url: "https://public-node.testnet.rsk.co".into(),
            contract_address: "0xe916a48de922e8964542f4c4c66ec4837bbe3445".into(),
            deployment_block: 7797344,
            db_path: PathBuf::from(".mns-server/db"),
            poll_interval_secs: 15,
            max_block_range: 2000,
            public_rpc_url: "https://public-node.testnet.rsk.co".into(),
            chain_id: 31,
        }
    }
}
