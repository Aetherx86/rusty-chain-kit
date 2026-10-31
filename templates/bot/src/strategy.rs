//! Collectors, a sample strategy, and executors. Replace `GasWatch` with your own logic.

use crate::engine::{Collector, CollectorStream, Executor, Strategy};
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::{DynProvider, Provider};
use alloy::rpc::types::TransactionRequest;
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;

// ───────────────────────────── events & actions ─────────────────────────────

#[derive(Debug, Clone)]
pub enum Event {
    NewBlock { number: u64, hash: B256, base_fee: Option<u64>, tx_count: usize },
    PendingTx(B256),
}

#[derive(Debug, Clone)]
pub enum Action {
    Alert(String),
    SubmitTx(Box<TransactionRequest>),
}

// ───────────────────────────── collectors ─────────────────────────────

/// New blocks via polling (works on any HTTP or WS endpoint).
pub struct BlockCollector {
    pub provider: DynProvider,
}

#[async_trait]
impl Collector<Event> for BlockCollector {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Event>> {
        let poller = self.provider.watch_blocks().await?;
        let provider = self.provider.clone();
        let stream = poller
            .into_stream()
            .flat_map(futures::stream::iter)
            .then(move |hash| {
                let provider = provider.clone();
                async move { provider.get_block_by_hash(hash).await.ok().flatten() }
            })
            .filter_map(|b| async move {
                b.map(|b| Event::NewBlock {
                    number: b.header.number,
                    hash: b.header.hash,
                    base_fee: b.header.base_fee_per_gas,
                    tx_count: b.transactions.len(),
                })
            });
        Ok(Box::pin(stream))
    }
}

/// Pending transaction hashes (requires a ws:// endpoint).
pub struct MempoolCollector {
    pub provider: DynProvider,
}

#[async_trait]
impl Collector<Event> for MempoolCollector {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Event>> {
        let sub = self.provider.subscribe_pending_transactions().await?;
        Ok(Box::pin(sub.into_stream().map(Event::PendingTx)))
    }
}

// ───────────────────────────── strategy ─────────────────────────────

/// Example strategy: alert when the base fee drops below a threshold, and (optionally) send a
/// self-transfer "ping" tx when it does. Shows how to emit both alert and transaction actions.
pub struct GasWatch {
    pub threshold_gwei: f64,
    pub ping_from: Option<Address>,
    pending_seen: u64,
}

impl GasWatch {
    pub fn new(threshold_gwei: f64, ping_from: Option<Address>) -> Self {
        Self { threshold_gwei, ping_from, pending_seen: 0 }
    }
}

#[async_trait]
impl Strategy<Event, Action> for GasWatch {
    async fn sync_state(&mut self) -> Result<()> {
        tracing::info!("GasWatch ready (threshold {} gwei)", self.threshold_gwei);
        Ok(())
    }

    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::PendingTx(_) => {
                self.pending_seen += 1;
                vec![]
            }
            Event::NewBlock { number, hash, base_fee, tx_count } => {
                let gwei = base_fee.map(|b| b as f64 / 1e9).unwrap_or(0.0);
                tracing::info!(
                    "block {number} {hash:.10} txs={tx_count} base_fee={gwei:.3} gwei mempool_seen={}",
                    self.pending_seen
                );
                self.pending_seen = 0;
                if base_fee.is_none() || gwei >= self.threshold_gwei {
                    return vec![];
                }
                let mut actions = vec![Action::Alert(format!("cheap gas at block {number}: {gwei:.3} gwei"))];
                if let Some(me) = self.ping_from {
                    let tx = TransactionRequest::default().with_to(me).with_value(U256::ZERO);
                    actions.push(Action::SubmitTx(Box::new(tx)));
                }
                actions
            }
        }
    }
}

// ───────────────────────────── executors ─────────────────────────────

pub struct AlertExecutor;

#[async_trait]
impl Executor<Action> for AlertExecutor {
    async fn execute(&self, action: Action) -> Result<()> {
        if let Action::Alert(msg) = action {
            tracing::warn!("ALERT: {msg}");
        }
        Ok(())
    }
}

/// Submits transactions with a wallet-enabled provider; logs only in dry-run mode.
pub struct MempoolExecutor {
    pub provider: Option<DynProvider>,
}

#[async_trait]
impl Executor<Action> for MempoolExecutor {
    async fn execute(&self, action: Action) -> Result<()> {
        let Action::SubmitTx(tx) = action else { return Ok(()) };
        match &self.provider {
            None => tracing::info!("dry-run: would submit {:?}", tx.to),
            Some(p) => {
                let pending = p.send_transaction(*tx).await?;
                tracing::info!("submitted {}", pending.tx_hash());
            }
        }
        Ok(())
    }
}
