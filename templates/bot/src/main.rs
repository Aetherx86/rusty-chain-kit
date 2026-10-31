mod engine;
mod strategy;

use alloy::network::EthereumWallet;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use clap::Parser;
use engine::Engine;
use strategy::{Action, AlertExecutor, BlockCollector, Event, GasWatch, MempoolCollector, MempoolExecutor};

#[derive(Parser)]
#[command(about = "{{name}}: artemis-style bot on alloy (scaffolded by para)")]
struct Args {
    /// RPC endpoint (ws:// enables the mempool collector).
    #[arg(long, env = "ETH_RPC_URL", default_value = "http://127.0.0.1:8545")]
    rpc: String,
    /// Alert when the base fee falls below this many gwei.
    #[arg(long, default_value_t = 5.0)]
    threshold_gwei: f64,
    /// Private key for the executor. Without it (or without --execute) the bot runs dry.
    #[arg(long, env = "PRIVATE_KEY", hide_env_values = true)]
    private_key: Option<String>,
    /// Actually submit transactions.
    #[arg(long)]
    execute: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let args = Args::parse();

    let read = ProviderBuilder::new().connect(&args.rpc).await?.erased();
    tracing::info!("connected to chain {}", read.get_chain_id().await?);

    let signer: Option<PrivateKeySigner> = args.private_key.as_deref().map(str::parse).transpose()?;
    let ping_from = signer.as_ref().map(|s| s.address());
    let write = match (&signer, args.execute) {
        (Some(s), true) => Some(
            ProviderBuilder::new().wallet(EthereumWallet::from(s.clone())).connect(&args.rpc).await?.erased(),
        ),
        _ => None,
    };

    let mut engine: Engine<Event, Action> = Engine::new()
        .with_collector(Box::new(BlockCollector { provider: read.clone() }))
        .with_strategy(Box::new(GasWatch::new(args.threshold_gwei, ping_from)))
        .with_executor(Box::new(AlertExecutor))
        .with_executor(Box::new(MempoolExecutor { provider: write }));
    if args.rpc.starts_with("ws") {
        engine = engine.with_collector(Box::new(MempoolCollector { provider: read.clone() }));
    }

    let mut set = engine.run().await?;
    while let Some(res) = set.join_next().await {
        if let Err(e) = res {
            tracing::error!("task failed: {e}");
        }
    }
    Ok(())
}
