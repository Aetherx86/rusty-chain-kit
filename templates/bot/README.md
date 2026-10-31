# {{name}}

An [artemis](https://github.com/paradigmxyz/artemis)-style MEV/automation bot, ported to
[alloy](https://github.com/alloy-rs/alloy). Scaffolded by `para new bot`.

```
Collectors ──events──▶ Strategies ──actions──▶ Executors
(blocks, mempool)      (GasWatch)              (alert, mempool tx)
```

```bash
para devnet up                       # local anvil on :8545
cargo run -- --threshold-gwei 2      # dry run against it
ETH_RPC_URL=wss://... cargo run      # ws endpoint also streams pending txs
PRIVATE_KEY=0x... cargo run -- --execute   # actually submit transactions
```

- `src/engine.rs`: the artemis engine (Collector / Strategy / Executor traits + broadcast wiring)
- `src/strategy.rs`: events, actions, collectors, the sample `GasWatch` strategy, executors
