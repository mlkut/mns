use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub api_key: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub testnet: Option<NetworkConfig>,
    pub mainnet: Option<NetworkConfig>,
}

impl Config {
    pub fn path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("mns").join("config.toml")
    }

    pub fn load() -> Self {
        let p = Self::path();
        fs::read_to_string(&p)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&p, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn testnet_rpc_url(&self) -> String {
        if let Some(ref net) = self.testnet {
            if let Some(ref key) = net.api_key {
                if !key.is_empty() {
                    return format!("https://rpc.rootstock.io/{}/testnet", key);
                }
            }
        }
        "https://public-node.testnet.rsk.co".to_string()
    }
}
