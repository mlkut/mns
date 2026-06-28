mod config;
mod key;

use alloy::providers::{Provider, ProviderBuilder};
use bindings::mns_registry::MNSRegistry;
use clap::{Parser, Subcommand};
use mns::Name;
use url::Url;

const CONTRACT_ADDRESS: &str = "0xe916a48de922e8964542f4c4c66ec4837bbe3445";

#[derive(Parser)]
#[command(name = "mns", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Register {
        zsk: String,
        ns: String,
    },
    List {
        ordinal: u64,
    },
    Update {
        ordinal: u64,
        new_owner: String,
        zsk: String,
        ns: String,
    },
    UpdateBatch {
        ordinal: u64,
        new_owner: String,
        zsk: String,
        ns: String,
    },
}

fn rpc_url(cfg: &config::Config) -> String {
    cfg.testnet_rpc_url()
}

async fn build_provider(
    signer: alloy::signers::local::PrivateKeySigner,
    cfg: &config::Config,
) -> Result<impl alloy::providers::Provider, Box<dyn std::error::Error>> {
    let url = Url::parse(&rpc_url(cfg))?;
    let provider = ProviderBuilder::new()
        .with_chain_id(31)
        .wallet(signer)
        .connect_http(url);
    Ok(provider)
}

fn build_readonly_provider(cfg: &config::Config) -> impl alloy::providers::Provider {
    let url = Url::parse(&rpc_url(cfg)).unwrap();
    ProviderBuilder::new().with_chain_id(31).connect_http(url)
}

fn parse_bytes32(s: &str) -> Result<alloy::primitives::FixedBytes<32>, Box<dyn std::error::Error>> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)?;
    if bytes.len() != 32 {
        return Err("bytes32 must be 32 bytes (64 hex chars)".into());
    }
    Ok(alloy::primitives::FixedBytes::<32>::from_slice(&bytes))
}

fn parse_address(s: &str) -> Result<alloy::primitives::Address, Box<dyn std::error::Error>> {
    Ok(s.parse()?)
}

fn ordinal_to_name(ordinal: u64) -> String {
    Name::from_ordinal(ordinal).to_string()
}

fn format_batch(ordinal: u64, batch: &MNSRegistry::Batch, has_entry: &[bool]) {
    let batch_number = ordinal / 256;
    let start = batch_number * 256;
    let end = start + 255;
    println!("Batch #{} (ordinals {}-{})", batch_number, start, end);
    println!("  Owner: {:?}", batch.owner);
    println!("  ZSK: 0x{}", hex::encode(batch.zone.zsk));
    println!("  NS: {}", batch.zone.ns);
    println!();
    for (i, &taken) in has_entry.iter().enumerate() {
        if taken {
            continue;
        }
        let o = start + i as u64;
        let name = ordinal_to_name(o);
        println!("  {} -> {}", o, name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let signer = key::get_or_create_key()?;
            let address = signer.address();
            let mut cfg = config::Config::load();
            if cfg.testnet.is_none() {
                cfg.testnet = Some(config::NetworkConfig {
                    api_key: Some(String::new()),
                });
            }
            if cfg.mainnet.is_none() {
                cfg.mainnet = Some(config::NetworkConfig {
                    api_key: Some(String::new()),
                });
            }
            cfg.save()?;
            println!("RSK address: 0x{:x}", address);
            println!("Config: {}", config::Config::path().display());
            println!("RPC: {}", cfg.testnet_rpc_url());
        }
        Commands::Register { zsk, ns } => {
            let cfg = config::Config::load();
            let signer = key::get_key()?;
            let provider = build_provider(signer, &cfg).await?;
            let address: alloy::primitives::Address = CONTRACT_ADDRESS.parse()?;
            let contract = MNSRegistry::new(address, &provider);
            let zsk_bytes = parse_bytes32(&zsk)?;
            let gas_price = provider.get_gas_price().await?;
            let tx = contract
                .register(zsk_bytes, ns.clone())
                .gas_price(gas_price)
                .send()
                .await?;
            let receipt = tx.get_receipt().await?;
            println!("Registered! Tx: {:?}", receipt.transaction_hash);
        }
        Commands::List { ordinal } => {
            let cfg = config::Config::load();
            let provider = build_readonly_provider(&cfg);
            let address: alloy::primitives::Address = CONTRACT_ADDRESS.parse()?;
            let contract = MNSRegistry::new(address, &provider);
            let batch = contract.getBatch(ordinal).call().await?;
            let batch_number = ordinal / 256;
            let start = batch_number * 256;
            let end = start + 255;
            let mut has_entry = Vec::with_capacity(256);
            for o in start..=end {
                has_entry.push(contract.hasEntry(o).call().await?);
            }
            format_batch(ordinal, &batch, &has_entry);
        }
        Commands::Update {
            ordinal,
            new_owner,
            zsk,
            ns,
        } => {
            let cfg = config::Config::load();
            let signer = key::get_key()?;
            let provider = build_provider(signer, &cfg).await?;
            let address: alloy::primitives::Address = CONTRACT_ADDRESS.parse()?;
            let contract = MNSRegistry::new(address, &provider);
            let owner = parse_address(&new_owner)?;
            let zsk_bytes = parse_bytes32(&zsk)?;
            let gas_price = provider.get_gas_price().await?;
            let tx = contract
                .update(ordinal, owner, zsk_bytes, ns.clone())
                .gas_price(gas_price)
                .send()
                .await?;
            let receipt = tx.get_receipt().await?;
            println!("Updated! Tx: {:?}", receipt.transaction_hash);
        }
        Commands::UpdateBatch {
            ordinal,
            new_owner,
            zsk,
            ns,
        } => {
            let cfg = config::Config::load();
            let signer = key::get_key()?;
            let provider = build_provider(signer, &cfg).await?;
            let address: alloy::primitives::Address = CONTRACT_ADDRESS.parse()?;
            let contract = MNSRegistry::new(address, &provider);
            let owner = parse_address(&new_owner)?;
            let zsk_bytes = parse_bytes32(&zsk)?;
            let gas_price = provider.get_gas_price().await?;
            let tx = contract
                .updateBatch(ordinal, owner, zsk_bytes, ns.clone())
                .gas_price(gas_price)
                .send()
                .await?;
            let receipt = tx.get_receipt().await?;
            println!("Batch updated! Tx: {:?}", receipt.transaction_hash);
        }
    }

    Ok(())
}
