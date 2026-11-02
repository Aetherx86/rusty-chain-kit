# MEV Mempool and Simulation Engine

Developing and stress-testing Maximum Extractable Value (MEV) strategies and DeFi protocols requires high-speed mempool processing and realistic market dynamic simulation before risking capital on mainnet liquidity.

Rusty Chain Kit integrates an asynchronous MEV execution pipeline (`mev.rs`) and a multi-agent discrete-event market simulation engine (`sim.rs`).

---

## 1. MEV Strategy Pipeline & Mempool Streaming

The MEV engine provides a modular framework for searchers, arbitrageurs, and liquidators:

```
[ Ethereum Execution Client (WebSocket) ]
                   |
     (Pending Transactions & New Blocks)
                   |
                   v
[ Stream Ingestion & Decoding Pipeline ]
                   |
     (Event Decoding & Pool State Sync)
                   |
                   v
       [ Strategy Engine (Rust) ]
   - Multi-Hop Arbitrage Discovery
   - Liquidation Trigger Evaluation
   - Sandwich Profit Calculation
                   |
                   v
   [ Atomic Bundle Dispatch / Dry-Run ]
   - Flashbots / Private Relay Bundles
   - Simulation & Local Gas Checks
```

### Strategy Scaffolding
Scaffold a complete Rust MEV bot structure:
```bash
para new bot my-searcher-bot
```
This generates a project with pre-configured WebSocket stream handlers, pool synchronization routines, and bundle construction helpers.

### CLI Execution
Run an MEV strategy pipeline in simulation mode:
```bash
para mev run strategies/arbitrage.rs --rpc ws://127.0.0.1:8545 --dry-run
```

Backtest a strategy over a historical block range:
```bash
para mev backtest strategies/arbitrage.rs --start-block 19000000 --end-block 19000500
```

---

## 2. Multi-Agent AMM Market Simulation

The simulation engine models complex market dynamics, liquidity provider returns, and searcher competition without deploying to a live blockchain:

```
[ Multi-Agent Simulation Engine ]
  |-- Constant-Product AMM Pool (x * y = k)
  |-- Stochastic Price Process (Geometric Brownian Motion)
  |-- Retail Trader Agents (Poisson Arrival Distribution)
  +-- Arbitrage Searcher Agents (Profit Maximization & Gas Auctions)
```

### Mathematical Foundations

1. **Stochastic Asset Pricing**: External reference prices evolve according to Geometric Brownian Motion:
   $$dS_t = \mu S_t dt + \sigma S_t dW_t$$
   where $\mu$ is drift, $\sigma$ is asset volatility, and $W_t$ is a Wiener process.

2. **Retail Swapper Flow**: Retail orders arrive following a Poisson process with arrival rate $\lambda$, executing market orders subject to pool slippage:
   $$\Delta y = \frac{y \cdot \Delta x}{x + \Delta x} \cdot (1 - \text{fee})$$

3. **Searcher Arbitrage Flow**: Whenever the pool's spot price diverges from the external reference price beyond the pool fee threshold, searcher agents execute backrunning arbitrage transactions to extract value and restore equilibrium.

---

## 3. Running Market Simulations

Execute a 200-step simulation with custom price volatility:
```bash
para sim --steps 200 --initial-price 3000.0 --volatility 0.02
```

Output:
```
[Market Dynamics Simulation]
Steps Simulated:    200 blocks
Initial Price:      $3,000.00
Terminal Price:     $3,084.50
Total Swaps:        1,420
Searcher PnL:       +14.82 ETH
Pool Fee Revenue:   +42.15 ETH
Max Slippage:       1.48%
```

Set a deterministic seed for reproducible economic experiments:
```bash
para sim --steps 500 --seed 42 --json
```
