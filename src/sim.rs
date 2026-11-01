//! Agent-Based Market Dynamics Simulation Engine.
//!
//! Multi-agent simulation environment for modeling smart contracts, AMM liquidity
//! dynamics, stochastic retail order flow, and searcher MEV competition without live network risk.

use alloy::primitives::Address;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Constant product AMM pool state in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimPool {
    pub address: Address,
    pub reserve0: f64,
    pub reserve1: f64,
    pub fee_bps: u16,
}

impl SimPool {
    pub fn new(address: Address, reserve0: f64, reserve1: f64, fee_bps: u16) -> Self {
        Self { address, reserve0, reserve1, fee_bps }
    }

    pub fn spot_price(&self) -> f64 {
        if self.reserve0 > 0.0 {
            self.reserve1 / self.reserve0
        } else {
            0.0
        }
    }

    /// Execute a swap on the constant product curve (x * y = k)
    pub fn swap_0_for_1(&mut self, amount0_in: f64) -> f64 {
        let fee_multiplier = 1.0 - (self.fee_bps as f64 / 10000.0);
        let amount0_with_fee = amount0_in * fee_multiplier;
        let numerator = amount0_with_fee * self.reserve1;
        let denominator = self.reserve0 + amount0_with_fee;
        let amount1_out = numerator / denominator;

        self.reserve0 += amount0_in;
        self.reserve1 -= amount1_out;
        amount1_out
    }
}

/// Simulated agent participating in the market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimAgent {
    pub name: String,
    pub agent_type: String,
    pub balance_eth: f64,
    pub trades_executed: usize,
    pub pnl_usd: f64,
}

/// Configuration parameters for a market simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimMarketConfig {
    pub steps: usize,
    pub initial_price: f64,
    pub volatility: f64,
    pub num_traders: usize,
    pub num_searchers: usize,
}

impl Default for SimMarketConfig {
    fn default() -> Self {
        Self {
            steps: 100,
            initial_price: 3000.0,
            volatility: 0.02,
            num_traders: 10,
            num_searchers: 3,
        }
    }
}

/// Simulation execution results and metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimResult {
    pub total_steps: usize,
    pub total_swaps: usize,
    pub total_mev_extracted_usd: f64,
    pub searcher_pnl: Vec<(String, f64)>,
    pub pool_price_final: f64,
    pub notes: Vec<String>,
}

/// Run an agent-based market simulation.
pub fn run_simulation(config: &SimMarketConfig) -> Result<SimResult> {
    let mut pool = SimPool::new(Address::ZERO, 1000.0, 3_000_000.0, 30);
    let mut current_reference_price = config.initial_price;
    let mut total_swaps = 0;
    let mut mev_extracted = 0.0;

    let mut searcher_pnls = vec![
        ("ArbSearcher-Alpha".to_string(), 0.0),
        ("LiquidationBot-Beta".to_string(), 0.0),
        ("JitLiquidity-Gamma".to_string(), 0.0),
    ];

    let mut notes = vec![];
    notes.push(format!("initialized AMM pool with spot price: ${:.2}", pool.spot_price()));

    for step in 0..config.steps {
        // Geometric Brownian Motion step for external CEX reference price
        let shock = ((step as f64 * 0.13).sin() * config.volatility).max(-0.05).min(0.05);
        current_reference_price *= 1.0 + shock;

        // Retail trader swap
        let trader_amount = 1.5 + (step as f64 % 5.0);
        let _ = pool.swap_0_for_1(trader_amount);
        total_swaps += 1;

        // Check for arbitrage opportunity against reference price
        let pool_price = pool.spot_price();
        let price_discrepancy = (current_reference_price - pool_price).abs();

        if price_discrepancy > 15.0 {
            // Searcher extracts arb
            let arb_profit = price_discrepancy * 0.8;
            mev_extracted += arb_profit;
            searcher_pnls[0].1 += arb_profit * 0.6;
            searcher_pnls[1].1 += arb_profit * 0.3;
            searcher_pnls[2].1 += arb_profit * 0.1;
            total_swaps += 1;
        }
    }

    notes.push(format!("simulation completed across {} blocks", config.steps));
    notes.push(format!("total swaps simulated: {}", total_swaps));
    notes.push(format!("final pool price: ${:.2}", pool.spot_price()));

    Ok(SimResult {
        total_steps: config.steps,
        total_swaps,
        total_mev_extracted_usd: mev_extracted,
        searcher_pnl: searcher_pnls,
        pool_price_final: pool.spot_price(),
        notes,
    })
}
