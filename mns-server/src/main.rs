mod config;
mod content_type;
mod html;
mod registry;
mod resolver;
mod server;
mod store;
mod syncer;

use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use dotenvy::dotenv;
use tracing::{info, warn};

use crate::config::Config;
use crate::store::lmdb::LmdbStore;

#[derive(Parser)]
#[command(name = "mns-server", version)]
struct Cli {
    /// Listen address (or just a port, e.g. "3000")
    #[arg(long, default_value = "3000")]
    listen: String,

    /// Rootstock RPC URL
    #[arg(long, default_value = "https://rpc.testnet.rootstock.io")]
    rpc_url: String,

    /// Rootstock RPC API key (env: RSK_RPC_API_KEY). If set, appended as a
    /// path to --rpc-url: `{rpc_url}/{api_key}`.
    #[arg(long)]
    rpc_api_key: Option<String>,

    /// Network (testnet)
    #[arg(long, default_value = "testnet")]
    network: String,

    /// Max blocks per eth_getLogs query
    #[arg(long, default_value_t = 2000)]
    max_block_range: u64,

    /// Path to LMDB database directory
    #[arg(long, default_value = ".mns-server/db")]
    db_path: String,

    /// Poll interval in seconds for contract events
    #[arg(long, default_value_t = 12)]
    poll_interval: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let cli = Cli::parse();

    let api_key = cli.rpc_api_key.or_else(|| {
        std::env::var("RSK_RPC_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
    });

    if api_key.is_none() {
        warn!(
            "no RSK_RPC_API_KEY configured — event polling requires an API key; \
             set it via --rpc-api-key or the RSK_RPC_API_KEY env var"
        );
    }

    let network_info = mns::networks::resolve(&cli.network)
        .unwrap_or_else(|| panic!("unknown network '{}'", cli.network));

    info!(
        "network: {} (contract={}, deployment_block={})",
        cli.network, network_info.contract_address, network_info.deployment_block
    );

    let rpc_url = match &api_key {
        Some(key) => format!("{}/{}", cli.rpc_url.trim_end_matches('/'), key),
        None => cli.rpc_url.clone(),
    };

    let listen = if cli.listen.contains(':') {
        cli.listen
    } else {
        format!("0.0.0.0:{}", cli.listen)
    };

    let cfg = Config {
        listen,
        rpc_url,
        contract_address: network_info.contract_address.to_string(),
        deployment_block: network_info.deployment_block,
        db_path: cli.db_path.into(),
        poll_interval_secs: cli.poll_interval,
        max_block_range: cli.max_block_range,
    };

    info!("opening database at {}", cfg.db_path.display());
    let store = Arc::new(LmdbStore::open(&cfg.db_path)?);

    info!(
        "connecting to Rootstock RPC: {} with an api key",
        &cli.rpc_url
    );
    let registry = Arc::new(crate::registry::alloy::AlloyRegistry::new(
        &cfg.rpc_url,
        &cfg.contract_address,
        cfg.max_block_range,
    )?);

    let syncer = crate::syncer::Syncer::new(
        registry.clone(),
        store.clone(),
        Duration::from_secs(cfg.poll_interval_secs),
        cfg.deployment_block,
    );

    // Start background poll loop
    tokio::spawn(async move {
        syncer.run().await;
    });

    // Build HTTP server
    let app = crate::server::build_router(store);
    let listener = tokio::net::TcpListener::bind(&cfg.listen).await?;
    info!("listening on {}", cfg.listen);

    axum::serve(listener, app).await?;

    Ok(())
}
