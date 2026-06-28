mod config;
mod key;

use alloy::providers::{Provider, ProviderBuilder};
use bindings::mns_registry::MNSRegistry;
use clap::{Parser, Subcommand};
use mns::Name;
use url::Url;

const RPC_URL: &str = "https://public-node.testnet.rsk.co";
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

async fn build_provider(
    signer: alloy::signers::local::PrivateKeySigner,
) -> Result<impl alloy::providers::Provider, Box<dyn std::error::Error>> {
    let url = Url::parse(RPC_URL)?;
    let provider = ProviderBuilder::new()
        .with_chain_id(31)
        .wallet(signer)
        .connect_http(url);
    Ok(provider)
}

fn build_readonly_provider() -> impl alloy::providers::Provider {
    let url = Url::parse(RPC_URL).unwrap();
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

fn format_batch(ordinal: u64, batch: &MNSRegistry::Batch) {
    let batch_number = ordinal / 256;
    let start = batch_number * 256;
    let end = start + 255;
    println!("Batch #{} (ordinals {}-{})", batch_number, start, end);
    println!("  Owner: {:?}", batch.owner);
    println!("  ZSK: 0x{}", hex::encode(batch.zone.zsk));
    println!("  NS: {}", batch.zone.ns);
    println!();
    for o in start..=end {
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
            if cfg.api_key.is_none() {
                cfg.api_key = Some(String::new());
                cfg.save()?;
            }
            println!("RSK address: 0x{:x}", address);
            println!("Config: {}", config::Config::path().display());
        }
        Commands::Register { zsk, ns } => {
            let signer = key::get_key()?;
            let provider = build_provider(signer).await?;
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
            let provider = build_readonly_provider();
            let address: alloy::primitives::Address = CONTRACT_ADDRESS.parse()?;
            let contract = MNSRegistry::new(address, &provider);
            let batch = contract.getBatch(ordinal).call().await?;
            format_batch(ordinal, &batch);
        }
        Commands::Update {
            ordinal,
            new_owner,
            zsk,
            ns,
        } => {
            let signer = key::get_key()?;
            let provider = build_provider(signer).await?;
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
            let signer = key::get_key()?;
            let provider = build_provider(signer).await?;
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
