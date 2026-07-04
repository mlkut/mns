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
use tracing::info;

use crate::config::Config;
use crate::store::lmdb::LmdbStore;

#[derive(Parser)]
#[command(name = "mns-server", version)]
struct Cli {
    /// Listen address (default: 127.0.0.1:3000)
    #[arg(long, default_value = "127.0.0.1:3000")]
    listen: String,

    /// Rootstock RPC URL
    #[arg(long, default_value = "https://public-node.testnet.rsk.co")]
    rpc_url: String,

    /// MNS Registry contract address
    #[arg(long, default_value = "0xe916a48de922e8964542f4c4c66ec4837bbe3445")]
    contract: String,

    /// Path to LMDB database directory
    #[arg(long, default_value = ".mns-server/db")]
    db_path: String,

    /// Poll interval in seconds for contract events
    #[arg(long, default_value_t = 12)]
    poll_interval: u64,

    /// Reorg lookback in blocks
    #[arg(long, default_value_t = 100)]
    reorg_lookback: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let cfg = Config {
        listen: cli.listen,
        rpc_url: cli.rpc_url,
        contract_address: cli.contract,
        db_path: cli.db_path.into(),
        poll_interval_secs: cli.poll_interval,
        reorg_lookback: cli.reorg_lookback,
    };

    info!("opening database at {}", cfg.db_path.display());
    let store = Arc::new(LmdbStore::open(&cfg.db_path)?);

    info!("connecting to Rootstock RPC: {}", cfg.rpc_url);
    let registry = Arc::new(crate::registry::alloy::AlloyRegistry::new(
        &cfg.rpc_url,
        &cfg.contract_address,
    )?);

    // Initial sync from contract
    let syncer = crate::syncer::Syncer::new(
        registry.clone(),
        store.clone(),
        Duration::from_secs(cfg.poll_interval_secs),
        cfg.reorg_lookback,
    );

    info!("starting initial sync from registry");
    if let Err(e) = syncer.initial_sync().await {
        tracing::error!("initial sync failed: {e}");
    }

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
