//! The artemis engine, ported from ethers-rs to alloy.
//!
//! Collectors turn external sources into a stream of events, strategies turn events into actions,
//! executors carry actions out. Everything is wired with tokio broadcast channels.

use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::BoxStream;
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::task::JoinSet;

/// A stream of events emitted by a [`Collector`].
pub type CollectorStream<'a, E> = BoxStream<'a, E>;

/// A source of events (new blocks, pending txs, logs, offchain orders, ...).
#[async_trait]
pub trait Collector<E>: Send + Sync {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, E>>;
}

/// Core opportunity logic.
#[async_trait]
pub trait Strategy<E, A>: Send + Sync {
    /// Sync initial state (usually onchain data) before events start flowing.
    async fn sync_state(&mut self) -> Result<()>;
    /// Process an event and return zero or more actions.
    async fn process_event(&mut self, event: E) -> Vec<A>;
}

/// Carries out actions (submit tx, send bundle, alert, ...).
#[async_trait]
pub trait Executor<A>: Send + Sync {
    async fn execute(&self, action: A) -> Result<()>;
}

pub struct Engine<E, A> {
    collectors: Vec<Box<dyn Collector<E>>>,
    strategies: Vec<Box<dyn Strategy<E, A>>>,
    executors: Vec<Box<dyn Executor<A>>>,
    event_capacity: usize,
    action_capacity: usize,
}

impl<E, A> Default for Engine<E, A> {
    fn default() -> Self {
        Self { collectors: vec![], strategies: vec![], executors: vec![], event_capacity: 512, action_capacity: 512 }
    }
}

impl<E, A> Engine<E, A>
where
    E: Send + Sync + Clone + 'static,
    A: Send + Sync + Clone + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_collector(mut self, c: Box<dyn Collector<E>>) -> Self {
        self.collectors.push(c);
        self
    }

    pub fn with_strategy(mut self, s: Box<dyn Strategy<E, A>>) -> Self {
        self.strategies.push(s);
        self
    }

    pub fn with_executor(mut self, x: Box<dyn Executor<A>>) -> Self {
        self.executors.push(x);
        self
    }

    /// Spawn every component; returns the task set so the caller can await it.
    pub async fn run(self) -> Result<JoinSet<()>> {
        let (event_tx, _) = broadcast::channel::<E>(self.event_capacity);
        let (action_tx, _) = broadcast::channel::<A>(self.action_capacity);
        let mut set = JoinSet::new();

        for executor in self.executors {
            let mut rx = action_tx.subscribe();
            set.spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(action) => {
                            if let Err(e) = executor.execute(action).await {
                                tracing::error!("executor error: {e:#}");
                            }
                        }
                        Err(RecvError::Lagged(n)) => tracing::warn!("executor lagged by {n} actions"),
                        Err(RecvError::Closed) => break,
                    }
                }
            });
        }

        for mut strategy in self.strategies {
            strategy.sync_state().await?;
            let mut rx = event_tx.subscribe();
            let action_tx = action_tx.clone();
            set.spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            for action in strategy.process_event(event).await {
                                let _ = action_tx.send(action);
                            }
                        }
                        Err(RecvError::Lagged(n)) => tracing::warn!("strategy lagged by {n} events"),
                        Err(RecvError::Closed) => break,
                    }
                }
            });
        }

        for collector in self.collectors {
            let event_tx = event_tx.clone();
            set.spawn(async move {
                let mut stream = match collector.get_event_stream().await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("collector failed to start: {e:#}");
                        return;
                    }
                };
                while let Some(event) = stream.next().await {
                    let _ = event_tx.send(event);
                }
            });
        }

        Ok(set)
    }
}
