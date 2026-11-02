# Rusty Chain Kit Wiki

Welcome to the official documentation and technical knowledge base for Rusty Chain Kit (`para` / `rusty-chain-kit`), version v0.1.1-beta.

Rusty Chain Kit is an all-in-one developer command center and unified Ethereum toolchain. It unites smart contract compilation, formal invariant verification, stateful dynamic fuzzing, EVM bytecode decompilation, trustless light-client state root verification, zero-knowledge virtual machine (zkVM) proving, agent-based market dynamic simulation, high-performance MEV mempool execution, native Account Abstraction (EIP-7702 / ERC-4337), cycle-accurate gas flamegraphs, revert autopsies, vectorized blockchain SQL analytics, interactive multi-hop fund flow forensic visualization, storage layout and proxy upgrade collision inspection, automated vulnerability diagnosis with executable PoC synthesis, and cross-compiler differential benchmarking into a single binary and web dashboard.

---

## Architectural Blueprint

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

## Documentation Index

### Core Architecture & Environment
- [[Architecture and Design]]: System architecture, workspace discovery lifecycle, execution layers, and memory safety model.
- [[CLI Reference Manual]]: Exhaustive command dictionary detailing all subcommands, arguments, flags, environment variables, and exit codes.
- [[Web GUI Command Center]]: Guide to the browser-based dashboard, REST API endpoints, real-time transaction graph canvas, and headless server configuration.
- [[Deployment and CI-CD Guide]]: Production deployment procedures, multi-platform binary compilation, Docker container registry usage, and GitHub Actions automation.

### Smart Contracts & Development Tooling
- [[Interactive Solidity and EVM REPL]]: Read-eval-print loop usage, piped expression evaluation, gas profiling, and ABI encoding.
- [[Storage Layout and Upgrade Inspector]]: State variable slot packing, sub-32-byte alignment, wasted byte padding, and differential upgrade collision checks.
- [[Cross-Compiler Benchmarking]]: Differential optimization profiling comparing standard versus aggressive `via-ir` compiler runs, deployment gas, and execution footprints.

### Security, Auditing & Formal Methods
- [[Formal Verification and SMT]]: Symbolic EVM execution, path condition generation, Z3 SMT constraint solving, and protocol safety proofs.
- [[Stateful Fuzzing Engine]]: Stateful mutational snapshot fuzzing, invariant checking, call sequence synthesis, and minimal reproduction shrinking.
- [[Vulnerability Diagnosis and Exploit PoC Synthesizer]]: Automated detection of critical vulnerability patterns and synthesis of executable Foundry test harnesses (`contract ExploitTest is Test`).
- [[Gas Profiling and Revert Autopsy]]: Cycle-accurate opcode gas attribution, Speedscope flamegraph generation, and comprehensive revert decoding (`Error`, custom selectors, `Panic`).

### On-Chain Intelligence, Forensics & Analytics
- [[Forensic Flow and Graph Intelligence]]: Directed multi-hop fund flow analysis, counterparty classification (`DEX`, `Bridge`, `Mixer`, `Exchange`, `Lending`), sanctions screening, and graph export.
- [[Account Abstraction and EIP-7702]]: Secp256k1 EIP-7702 authorization tuple signing, ERC-4337 UserOperation bundle simulation, and calldata decoding.
- [[Vectorized SQL Blockchain Analytics]]: SQL analytics engine querying extracted blocks, transactions, and event logs from Parquet, CSV, and JSON files.
- [[Trustless Light Client Verification]]: Verification of state roots, consensus sync committee headers, and account Merkle-Patricia Trie proofs (`eth_getProof`).

### Simulation & Mempool Execution
- [[MEV Mempool and Simulation Engine]]: Real-time WebSocket mempool streaming, arbitrage strategy execution, and multi-agent discrete-event market simulation.
- [[Zero-Knowledge VM Pipeline]]: Scaffolding RISC-V zkVM projects, guest program compilation, and execution proof generation.

---

## Quick Start Matrix

### Installation

Prerequisites: Rust 1.94+ (`cargo`, `rustc`) and Git.

```bash
# Clone the repository
git clone https://github.com/Aetherx86/rusty-chain-kit.git
cd rusty-chain-kit

# Compile release binaries
cargo build --release

# Binaries generated:
# target/release/rusty-chain-kit
# target/release/para
```

### Docker Container

```bash
# Pull container from GitHub Container Registry
docker pull ghcr.io/aetherx86/rusty-chain-kit:latest

# Run CLI command
docker run --rm ghcr.io/aetherx86/rusty-chain-kit:latest --help

# Run web GUI dashboard
docker run -p 3333:3333 -p 8545:8545 ghcr.io/aetherx86/rusty-chain-kit:latest gui --no-open
```

### Essential Commands at a Glance

| Command | Purpose |
| :--- | :--- |
| `para compile [path]` | Sub-second compilation of Solidity contracts |
| `para storage <v1.sol> [v2.sol]` | Inspect slot packing and check upgrade collisions |
| `para copilot <contract.sol> --poc` | Diagnose vulnerabilities and generate executable exploit PoC |
| `para bench <contract.sol>` | Differential optimization benchmark (`via-ir` vs standard) |
| `para flow <address> --hops 3` | Multi-hop forensic flow graph with sanctions screening |
| `para verify <contract.sol>` | Formal invariant verification using Z3 SMT solver |
| `para audit <path> --fuzz` | Stateful snapshot fuzzing with invariant checks |
| `para profile <tx_hash>` | Cycle-accurate gas flamegraph profiling |
| `para autopsy <revert_hex>` | Decode exact revert reason and Panic codes |
| `para aa generate-auth` | Generate and sign EIP-7702 authorization tuple |
| `para repl` | Interactive Solidity and EVM expression REPL |
| `para sim --steps 200` | Discrete-event multi-agent AMM market simulation |
| `para query "<sql>"` | Vectorized SQL query over blockchain datasets |
| `para devnet up` | Background local devnet daemon with state persistence |
| `para gui` | Launch interactive web graphical command center |
