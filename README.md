<p align="center">
  <img src="assets/icon.png" width="160" alt="Rusty Chain Kit Logo" />
</p>

# Rusty Chain Kit

Rusty Chain Kit (v0.1.1 beta) is an all-in-one developer command center and unified Ethereum toolchain. It provides a high-performance Command Line Interface (`para` / `rusty-chain-kit`) and an interactive Web Graphical User Interface dashboard uniting smart contract compilation, formal invariant verification, stateful dynamic fuzzing, EVM bytecode decompilation, trustless light-client state root verification, zero-knowledge virtual machine (zkVM) proving, agent-based market dynamic simulation, high-performance MEV mempool execution, native Account Abstraction (EIP-7702 / ERC-4337), cycle-accurate gas flamegraphs, revert autopsies, vectorized blockchain SQL analytics, and interactive multi-hop fund flow forensic visualization.

---

## Architecture Overview

```
+---------------------------------------------------------------------------------------------------------+
|                                             RUSTY CHAIN KIT                                             |
|                                     CLI (`rusty-chain-kit` / `para`)                                    |
|                                     GUI (Interactive Web Dashboard)                                     |
+---------------------------------------------------------------------------------------------------------+
                                                     |
       +-------------------------+-------------------+--------------------+------------------------+
       |                         |                   |                    |                        |
       v                         v                   v                    v                        v
[ Smart Contracts ]     [ Security & Proof ]   [ MEV & Simulation ]  [ Chain Intelligence ]   [ Environments ]
- Solidity Compiler     - Symbolic SMT Proof   - Mempool Streaming   - Forensics Flow Graph   - Local Devnet
- EVM Disassembler      - Stateful Invariant   - Strategy Pipeline   - Trustless Light Client - Modular Execution
- Decompiler Engine       Fuzzing Engine       - Multi-Agent AMM     - Proxy Detector (1967,    Node Daemon
- Fast ABI Extraction   - RISC-V zkVM Prover     Market Simulation     1167, 1822, 2535, 7702)- State Persistence
- SSA CFG Lifter        - Revert Autopsy       - Priority Gas Bundle - Vectorized SQL OLAP    - Multi-Account
- Live Solidity REPL    - Gas Flamegraphs        Atomic Dispatch       Query Runner             Funder
```

---

## Multi-Hop Forensic Flow & Transaction Graph Architecture

```
                   [ Subject Account (0xd8dA6...6045) ]
                                    |
        +---------------------------+---------------------------+
        | (Hop 1: 15 ETH)           | (Hop 1: 50 ETH)           | (Hop 1: 25 USDC)
        v                           v                           v
[ DEX Swap Router ]         [ Bridge Contract ]        [ Aggregator Router ]
        |                                                       |
        | (Hop 2: 10 WETH)                                      | (Hop 2: 24.8 USDT)
        v                                                       v
[ Mixer / Privacy Pool ]                                 [ Exchange Deposit ]
  (Risk: Flagged High)                                     (Risk: Verified Clean)
```

---

## Symbolic Path Verification & SMT Invariant Prover

```
               [ Contract Method Entry Point: deposit(uint256) ]
                                      |
                           (Symbolic Branch Split)
                                      |
                 +--------------------+--------------------+
                 |                                         |
         [ Path Condition A ]                      [ Path Condition B ]
         balance >= amount                         balance < amount
                 |                                         |
     (State Equation Update)                     (Branch Revert Path)
     balance' = balance - amount                 require(false, "insufficient")
                 |                                         |
     (Safety Invariant Query)                    (SMT Solver Verification)
     assert(balance' + amount == balance)                  |
                 |                                         v
                 v                                   [ SAT Check ]
          [ UNSAT Proof ]                    Counterexample Generated or
     Invariant Formally Proved!                     Revert Confirmed
```

---

## Core Capabilities

### 1. High-Performance Solidity Compilation
Compile Solidity smart contracts with ultra-fast compilation speeds. Inspect emitted ABIs, runtime bytecode, storage layouts, and compiler diagnostics in sub-second build times.

### 2. Formal Invariant Verification & Symbolic SMT Execution
Mathematically prove protocol safety properties, solvency conditions, and state transition equations using symbolic execution. Encodes contract memory, storage equations, and path conditions into SMT-LIB2 formulas solved natively with Z3 to find concrete counterexample transaction inputs or prove invariants across all reachable execution states.

### 3. Stateful Snapshot Fuzzing Engine
Rapid mutational fuzzing suite that tests smart contracts with thousands of multi-transaction call sequences per second. Automatically discovers invariant violations, shrinks call graphs to minimal reproduction steps, and detects edge cases missed by static scanners.

### 4. Native Account Abstraction & EIP-7702 Tooling
First-class tooling for next-generation EVM account delegation and smart accounts:
- Generate and sign EIP-7702 authorization tuples (chain ID, target address, nonce, y-parity, r, s) using local secp256k1 signers.
- Simulate ERC-4337 UserOperation bundles, validating verification gas limits, pre-verification gas, paymaster sponsorship thresholds, and max fee structures.
- Decode Type 4 (EIP-7702) transaction envelopes and parse EntryPoint `handleOps` calldata into structured UserOperation summaries.

### 5. Cycle-Accurate Gas Profiling & Revert Autopsy
Deep execution introspection for EVM transactions:
- Profile gas consumption per opcode (`SLOAD`, `SSTORE`, `KECCAK256`, `LOG3`, `CALL`).
- Generate hierarchical Flamegraph JSON compatible with Speedscope and D3 visualizers.
- Revert Autopsy: Automatically dissects revert data into standard `Error(string)`, custom Solidity error selectors, and exact `Panic(uint256)` codes (assertion failure, arithmetic overflow/underflow, division by zero, array out-of-bounds, pop on empty array, out-of-memory). Pinpoints failing contract addresses and provides actionable remediation hints.

### 6. Fast Bytecode Intelligence & Proxy Pattern Resolver
Inspect unverified or deployed EVM runtime bytecode in sub-milliseconds:
- Extract function selectors, state mutability (`pure`, `view`, `nonpayable`, `payable`), and inferred argument signatures using `evmole`.
- Automatically identify proxy architectures: EIP-1967 transparent/beacon slots, EIP-1167 minimal proxy clones, EIP-1822 UUPS, EIP-2535 Diamond multi-facets, and EIP-7702 code delegation prefixes (`0xef0100`).
- Lift raw opcodes into Basic Block Control Flow Graphs (CFG) resolving jump destinations and terminal blocks.

### 7. Interactive Solidity & EVM REPL
A live read-eval-print-loop to experiment with Solidity expressions, ABI encoding, hashing algorithms (`keccak256`), bitwise manipulations, and opcode sequences without writing and deploying test harness contracts. Supports piped single-expression evaluation with instant gas estimates.

### 8. Trustless Light-Client Verification
Verify blockchain state without placing blind trust in centralized RPC providers. Validates consensus sync committee block headers, state roots, and account storage Merkle-Patricia Trie proofs (`eth_getProof`), guaranteeing trustless on-chain inspections.

### 9. Zero-Knowledge Virtual Machine (zkVM) Pipeline
Bridge EVM development with zero-knowledge execution:
- Scaffold complete RISC-V zkVM projects with decoupled guest programs and host prover scripts.
- Execute compilation and generate verifiable zero-knowledge proofs (STARK/SNARK wrappers) with cycle execution metrics.

### 10. Multi-Agent Market Dynamics & AMM Simulation
Discrete-event multi-agent simulation engine for smart contract economics:
- Simulate constant-product AMM pools with configurable fee structures.
- Model stochastic retail order flows using Geometric Brownian Motion price processes and Poisson arrival distributions.
- Quantify MEV searcher competition, priority gas auctions, slippage, and PnL distributions across hundreds of blocks before deploying capital.

### 11. High-Performance MEV Engine & Mempool Pipeline
Modular pipeline for searchers and arbitrageurs:
- Stream pending transactions and blocks over WebSocket connections.
- Evaluate arbitrage cycles, sandwich opportunities, and liquidation triggers.
- Construct atomic multi-transaction bundles and verify local bundle simulation.

### 12. Vectorized SQL Blockchain Analytics
Query extracted blockchain datasets (Cryo Parquet, JSON, CSV files) using standard SQL queries (`SELECT`, `WHERE`, `GROUP BY`, `ORDER BY`, `LIMIT`). Filter transactions, aggregate gas metrics, and export results into formatted terminal tables, CSV, or JSON.

### 13. Blockchain Forensic Visualizer (Web GUI)
An interactive directed graph canvas embedding Chainalysis-style entity clustering, multi-hop fund transfers, mixer and privacy pool alerts, and address dossiers in the browser dashboard.

---

## Installation

Prerequisites: Rust 1.94+ (`cargo`, `rustc`) and Git.

```bash
# Clone the repository
git clone https://github.com/Aetherx86/rusty-chain-kit.git
cd rusty-chain-kit

# Build release binaries
cargo build --release

# Both binaries are produced:
# - target/release/rusty-chain-kit
# - target/release/para (convenience alias)
```

Link binaries to your user path:

```bash
mkdir -p ~/.local/bin
ln -sf $(pwd)/target/release/rusty-chain-kit ~/.local/bin/rusty-chain-kit
ln -sf $(pwd)/target/release/para ~/.local/bin/para
```

---

## CLI Reference

Commands can be invoked with either `rusty-chain-kit` or `para`.

### Account Abstraction & EIP-7702

```bash
# Generate and sign an EIP-7702 authorization tuple
para aa generate-auth \
  --chain-id 1 \
  --address 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 \
  --nonce 0 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Simulate an ERC-4337 UserOperation bundle
para aa bundle --json

# Decode EIP-7702 authorizations and ERC-4337 EntryPoint calldata from an on-chain transaction
para chain tx 0x3b89... --decode-aa
```

### Verification & Fuzzing

```bash
# Formally prove smart contract invariants using symbolic SMT constraint solving
para verify contracts/Vault.sol --invariant solvency

# Run stateful snapshot fuzzer across 10,000 transaction sequences
para audit contracts/ --fuzz --runs 10000

# Combine static auditing with formal proof validation
para audit contracts/ --prove
```

### Trace, Gas Profiling & Revert Autopsy

```bash
# Profile cycle-accurate opcode gas consumption and export a flamegraph
para profile 0x9a8b... --flamegraph profile.json

# Detailed execution trace of an on-chain transaction
para trace 0x9a8b...

# Autopsy a reverted transaction or raw revert hex data
para autopsy 0x4e487b710000000000000000000000000000000000000000000000000000000000000011
```

### Bytecode Intelligence & Proxy Inspection

```bash
# Extract ABI function selectors and inferred signatures from runtime bytecode
para chain code 0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984 --abi

# Detect proxy architecture and extract implementation address
para chain code 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --proxy

# Lift bytecode into basic block Control Flow Graph (CFG)
para chain code 0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984 --cfg

# Comprehensive deep inspection of an account address
para chain address 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --inspect
```

### Interactive Solidity REPL

```bash
# Launch interactive REPL session
para repl

# Piped single-expression evaluation with gas estimates
para repl --eval "keccak256('hello')"
para repl --eval "type(uint256).max"
```

### MEV Engine & Market Simulation

```bash
# Run multi-agent AMM market simulation
para sim --steps 200 --initial-price 3000.0 --volatility 0.02

# Run MEV searcher strategy pipeline in simulation mode
para mev run strategies/arbitrage.rs --rpc ws://127.0.0.1:8545 --dry-run

# Backtest an MEV strategy over a historical block range
para mev backtest strategies/arbitrage.rs --start-block 19000000 --end-block 19000500
```

### Vectorized SQL Blockchain Analytics

```bash
# Execute analytical SQL queries against extracted blockchain datasets
para query "SELECT number, gas_used, tx_count FROM blocks WHERE gas_used > 15000000 ORDER BY gas_used DESC LIMIT 10"

# Format output as JSON
para query "SELECT * FROM transactions WHERE value_eth > 5.0" --json
```

### Trustless Light Client State Inspection

```bash
# Verify account balance and storage proof trustlessly against consensus state roots
para rpc trustless 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --block 20000000
```

### zkVM Project Scaffolding & Proving

```bash
# Scaffold a new RISC-V zkVM project layout (guest program + host prover script)
para new zk my-prover

# Compile guest program and generate execution proof
para zk prove --path my-prover/
```

### Web GUI Dashboard

```bash
# Launch the local web graphical user interface
para gui

# Custom port without auto-opening default browser
para gui --port 8080 --no-open
```

### Devnet & Node Daemons

```bash
# Start background Anvil local devnet
para devnet up --port 8545 --block-time 1

# Fork remote mainnet state
para devnet up --fork mainnet --persist

# View pre-funded development accounts
para devnet accounts

# Stop running devnet
para devnet down

# Start local execution client in dev mode
para node up --port 8645
```

---

## Testing & Quality Assurance

Rusty Chain Kit includes an extensive integration test suite verifying every subsystem, CLI flag, proxy detector, and cryptographic proof:

```bash
# Execute all test suites
cargo test

# Execute specific test suites
cargo test --test test_v011_features
cargo test --test test_cli
cargo test --test test_chain
cargo test --test test_rpc
cargo test --test test_git
cargo test --test test_services_and_scaffold
cargo test --test test_workspace
```

---

## License

MIT License

Copyright (c) 2026 Jeong Hoon Kim (Aetherx86)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
