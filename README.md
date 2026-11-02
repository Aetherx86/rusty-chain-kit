<p align="center">
  <img src="assets/icon.png" width="160" alt="Rusty Chain Kit Logo" />
</p>

# Rusty Chain Kit

Rusty Chain Kit (v0.1.1 beta) is an all-in-one developer command center and unified Ethereum toolchain. It provides a high-performance Command Line Interface (`para` / `rusty-chain-kit`) and an interactive Web Graphical User Interface dashboard uniting smart contract compilation, formal invariant verification, stateful dynamic fuzzing, EVM bytecode decompilation, trustless light-client state root verification, zero-knowledge virtual machine (zkVM) proving, agent-based market dynamic simulation, high-performance MEV mempool execution, native Account Abstraction (EIP-7702 / ERC-4337), cycle-accurate gas flamegraphs, revert autopsies, vectorized blockchain SQL analytics, interactive multi-hop fund flow forensic visualization, storage layout and proxy upgrade collision inspection, automated vulnerability diagnosis with executable PoC synthesis, and cross-compiler differential benchmarking.

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
- Storage Inspector     - Vulnerability PoC    - Arbitrage Pipeline  - Storage Layout Diff    - Zero-Config
- Cross-Opt Benchmarker   Synthesizer Engine   - Market Backtester     & Collision Checker      Foundry Discovery
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

## Storage Layout & Upgrade Collision Inspector

```
[ Implementation V1 ]                                  [ Implementation V2 (Faulty) ]
+------------------------------------+                 +------------------------------------+
| Slot 0: [ owner (20B) ][ flag(1B) ]|                 | Slot 0: [ new_admin (20B) ]        | <-- COLLISION: SlotShift!
+------------------------------------+                 +------------------------------------+
| Slot 1: [ total_supply (32B) ]     |                 | Slot 1: [ owner (20B) ][ flag(1B) ]|
+------------------------------------+                 +------------------------------------+
| Slot 2: [ balances (mapping 32B) ] |                 | Slot 2: [ total_supply (uint128) ] | <-- MUTATION: TypeMutation!
+------------------------------------+                 +------------------------------------+
```

---

## Vulnerability Diagnosis & Executable PoC Synthesizer

```
[ Target Contract AST / Source ]
               |
    (Static & Dynamic Scan)
               |
               v
+-------------------------------+
| Vulnerability Diagnostic      |
| - Pattern: Reentrancy         |
| - Location: Vault.withdraw()  |
| - Severity: Critical          |
+-------------------------------+
               |
     (Executable PoC Synthesis)
               |
               v
[ Generated Foundry Test Harness: contract ExploitTest is Test ]
- Deploys Attacker & Vulnerable Target
- Executes Simulated Flash Loan / Reentrant Call Sequence
- Asserts Exploit Invariant: target.balance == 0
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

### 1. High-Performance Solidity Compilation & Zero-Config Workspaces
Compile Solidity smart contracts with sub-second build times. Automatically discovers existing projects via `foundry.toml` fallback without requiring manual workspace initialization. Reads single consolidated `para.toml` configurations for project options.

### 2. Storage Layout Packing & Proxy Upgrade Collision Inspector
Inspect contract state variable slot packing, calculate wasted byte padding, and run differential collision checks between proxy implementation versions:
- Detect `SlotShift` vulnerabilities where newly inserted variables offset existing storage slots.
- Identify `TypeMutation` breaking changes where variable bitwidths change across contract upgrades.
- Flag `VariableRemoved` gaps leading to unaligned storage layout corruption.

### 3. Automated Vulnerability Diagnosis & Executable PoC Exploit Synthesizer
Scan smart contracts for critical vulnerability patterns (reentrancy, spot reserve price manipulation, missing access control) and automatically synthesize executable Foundry test harnesses (`contract ExploitTest is Test`) proving vulnerability exploitability directly in developer environments.

### 4. Cross-Compiler Differential Optimization Benchmarking
Benchmark contract compilation across standard and aggressive `via-ir` optimization profiles:
- Compare runtime bytecode footprint reduction and deployment gas savings.
- Measure simulated execution gas differentials across contract entrypoints.
- Quantify compile time overhead to guide deployment configuration choices.

### 5. Multi-Hop Forensic Flow & Entity Graph Visualizer
Trace transaction graphs and multi-hop fund flows across Ethereum accounts:
- Classify counterparty entities into semantic labels (`DEX`, `Bridge`, `Exchange`, `Mixer`, `Lending`).
- Screen counterparties against sanctions registries and privacy pool address lists.
- Compute composite risk scores (`Clean`, `Low`, `Medium`, `High`, `Severe`) with CSV and JSON graph export.

### 6. Formal Invariant Verification & Symbolic SMT Execution
Mathematically prove protocol safety properties, solvency conditions, and state transition equations using symbolic execution. Encodes contract memory, storage equations, and path conditions into SMT-LIB2 formulas solved natively with Z3 to find concrete counterexample transaction inputs or prove invariants across all reachable execution states.

### 7. Stateful Snapshot Fuzzing Engine
Rapid mutational fuzzing suite that tests smart contracts with thousands of multi-transaction call sequences per second. Automatically discovers invariant violations, shrinks call graphs to minimal reproduction steps, and detects edge cases missed by static scanners.

### 8. Native Account Abstraction & EIP-7702 Tooling
First-class tooling for next-generation EVM account delegation and smart accounts:
- Generate and sign EIP-7702 authorization tuples (chain ID, target address, nonce, y-parity, r, s) using local secp256k1 signers.
- Simulate ERC-4337 UserOperation bundles, validating verification gas limits, pre-verification gas, paymaster sponsorship thresholds, and max fee structures.
- Decode Type 4 (EIP-7702) transaction envelopes and parse EntryPoint `handleOps` calldata into structured UserOperation summaries.

### 9. Cycle-Accurate Gas Profiling & Revert Autopsy
Deep execution introspection for EVM transactions:
- Profile gas consumption per opcode (`SLOAD`, `SSTORE`, `KECCAK256`, `LOG3`, `CALL`).
- Generate hierarchical Flamegraph JSON compatible with Speedscope and visualizers.
- Revert Autopsy: Automatically dissects revert data into standard `Error(string)`, custom Solidity error selectors, and exact `Panic(uint256)` codes (assertion failure, arithmetic overflow/underflow, division by zero, array out-of-bounds, pop on empty array, out-of-memory). Pinpoints failing contract addresses and provides actionable remediation hints.

### 10. Fast Bytecode Intelligence & Proxy Pattern Resolver
Inspect unverified or deployed EVM runtime bytecode in sub-milliseconds:
- Extract function selectors, state mutability (`pure`, `view`, `nonpayable`, `payable`), and inferred argument signatures.
- Automatically identify proxy architectures: EIP-1967 transparent/beacon slots, EIP-1167 minimal proxy clones, EIP-1822 UUPS, EIP-2535 Diamond multi-facets, and EIP-7702 code delegation prefixes (`0xef0100`).
- Lift raw opcodes into Basic Block Control Flow Graphs (CFG) resolving jump destinations and terminal blocks.

### 11. Interactive Solidity & EVM REPL
A live read-eval-print-loop to experiment with Solidity expressions, ABI encoding, hashing algorithms (`keccak256`), bitwise manipulations, and opcode sequences without writing and deploying test harness contracts. Supports piped single-expression evaluation with instant gas estimates.

### 12. Trustless Light-Client Verification
Verify blockchain state without placing blind trust in centralized RPC providers. Validates consensus sync committee block headers, state roots, and account storage Merkle-Patricia Trie proofs (`eth_getProof`), guaranteeing trustless on-chain inspections.

### 13. Zero-Knowledge Virtual Machine (zkVM) Pipeline
Bridge EVM development with zero-knowledge execution:
- Scaffold complete RISC-V zkVM projects with decoupled guest programs and host prover scripts.
- Execute compilation and generate verifiable zero-knowledge proofs (STARK/SNARK wrappers) with cycle execution metrics.

### 14. Multi-Agent Market Dynamics & AMM Simulation
Discrete-event multi-agent simulation engine for smart contract economics:
- Simulate constant-product AMM pools with configurable fee structures.
- Model stochastic retail order flows using Geometric Brownian Motion price processes and Poisson arrival distributions.
- Quantify MEV searcher competition, priority gas auctions, slippage, and PnL distributions across hundreds of blocks before deploying capital.

### 15. High-Performance MEV Engine & Mempool Pipeline
Modular pipeline for searchers and arbitrageurs:
- Stream pending transactions and blocks over WebSocket connections.
- Evaluate arbitrage cycles, sandwich opportunities, and liquidation triggers.
- Construct atomic multi-transaction bundles and verify local bundle simulation.

### 16. Vectorized SQL Blockchain Analytics
Query extracted blockchain datasets using standard SQL queries (`SELECT`, `WHERE`, `GROUP BY`, `ORDER BY`, `LIMIT`). Filter transactions, aggregate gas metrics, and export results into formatted terminal tables, CSV, or JSON.

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

### Multi-Hop Forensic Flow Analysis

```bash
# Trace 3-hop fund flows and screen counterparties against sanctions registries
para flow 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --hops 3

# Export forensic transaction graph as structured JSON
para flow 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --json

# Export transfer edges as CSV for external graphing
para flow 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --csv
```

### Storage Layout & Proxy Upgrade Inspector

```bash
# Inspect storage slot packing and wasted bytes for a contract
para storage contracts/VaultV1.sol

# Run differential collision inspection between V1 and V2 implementations
para storage contracts/VaultV1.sol contracts/VaultV2.sol

# Output storage layout schema in JSON format
para storage contracts/VaultV1.sol --json
```

### Automated Vulnerability Diagnosis & Exploit PoC Synthesis

```bash
# Diagnose vulnerabilities and explain remediation steps
para copilot contracts/Vault.sol

# Synthesize executable Foundry test harness proving exploitability
para copilot contracts/Vault.sol --poc

# Integrate static audit scan with exploit synthesis
para audit contracts/ --poc --explain
```

### Differential Optimization Benchmarking

```bash
# Compare standard vs aggressive via-ir compiler performance and gas usage
para bench contracts/Token.sol

# Custom optimization profile with JSON metrics export
para bench contracts/Token.sol --profile aggressive --json
```

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
# Start background local devnet
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

Rusty Chain Kit includes an extensive integration test suite verifying every subsystem, CLI flag, proxy detector, and cryptographic proof across 202 integration tests:

```bash
# Execute all test suites (202 tests passing)
cargo test

# Execute specific test suites
cargo test --test test_advanced_features
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
