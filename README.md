<p align="center">
  <img src="assets/icon.png" width="160" alt="Rusty Chain Kit Logo" />
</p>

# Rusty Chain Kit

Rusty Chain Kit is an all-in-one developer command center and unified Ethereum toolchain. It provides both a high-performance Command Line Interface (CLI) and an interactive Web Graphical User Interface (GUI) dashboard for smart contract compilation, static security auditing, EVM bytecode decompilation, high-throughput blockchain data extraction, local devnet orchestration, and on-chain RPC inspection.

---

## Architecture Overview

```
+-------------------------------------------------------------------------+
|                            RUSTY CHAIN KIT                              |
|                   CLI (`rusty-chain-kit` / `para`)                      |
|                   GUI (Web Dashboard on localhost)                      |
+-------------------------------------------------------------------------+
                                     |
       +-----------------------------+-----------------------------+
       |                             |                             |
       v                             v                             v
[ Smart Contracts ]          [ Chain & Data ]             [ Environments ]
- Solar Compiler             - Cryo Data Extractor        - Anvil Devnet Daemon
- Forge Compiler Runner      - Alloy RPC Client           - Reth Execution Node
- Aderyn Vulnerability Audit - MESC Endpoint Routing      - Local Multi-Account
- Heimdall EVM Decompiler    - Live Block Streaming       - State Persistence
- Cast Bytecode Inspector    - Transaction Inspector      - Custom Gas/Forking
```

---

## Core Capabilities

### 1. High-Performance Solidity Compilation
Compile Solidity smart contracts using either Solar (next-generation Rust-native compiler) or Solc/Forge. Inspect emitted ABIs, runtime bytecode, and compiler diagnostics in sub-second build times.

### 2. Automated Static Security Auditing
Scan Solidity source trees for common vulnerabilities, reentrancy vectors, access control flaws, and security pitfalls using integrated Aderyn static analysis. Export structured reports in Markdown or JSON format.

### 3. EVM Bytecode Decompilation and Disassembly
Reverse engineer deployed contracts or raw bytecode into readable Solidity representations and control-flow graphs using Heimdall. Analyze deployed protocols directly via RPC address lookups without needing original source code.

### 4. High-Throughput On-Chain Data Extraction
Extract millions of blockchain records (blocks, transactions, logs, traces, state diffs) into query-ready Parquet, JSON, or CSV files using Cryo. Built for high-speed indexing, MEV analysis, and quantitative research.

### 5. Local Devnet and Node Management
Spin up, monitor, and manage local Ethereum execution environments in background daemons. Supports instant fork testing against live networks, configurable block intervals, state persistence across restarts, and built-in account key management.

### 6. Multi-Network RPC Routing (MESC Integration)
Manage RPC endpoints across dozens of EVM chains using Multiple Endpoint Standard Configuration (MESC) and custom aliases. Features automatic latency benchmarking, endpoint resolution, and chain ID lookups.

### 7. Interactive On-Chain Explorer and Inspector
Query account balances, view nonces, inspect deployed bytecode size and code hashes, decode transaction selectors, calculate EIP-1559 gas prices, and stream newly mined blocks in real time.

### 8. Interactive Web GUI Dashboard
A standalone web interface embedding all command center capabilities: real-time devnet controls, visual blockchain explorer, live Solidity compiler workbench, and security auditing interface.

### 9. Project Scaffolding
Generate production-ready starter templates for MEV searcher bots, Alloy automation scripts, and Foundry contract suites with a single command.

---

## Quick Start

### Installation

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

Link the binaries to your path:

```bash
mkdir -p ~/.local/bin
ln -sf $(pwd)/target/release/rusty-chain-kit ~/.local/bin/rusty-chain-kit
ln -sf $(pwd)/target/release/para ~/.local/bin/para
```

---

## CLI Reference

Commands can be invoked with either `rusty-chain-kit` or `para`.

### Compilation and Security Analysis

```bash
# Compile Solidity contract using Solar
para compile contracts/Counter.sol

# Compile using Forge Solc pipeline
para compile contracts/Counter.sol --forge

# Run static vulnerability audit on a contract directory
para audit contracts/

# Decompile deployed contract bytecode via RPC
para decompile 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --rpc https://eth.llamarpc.com
```

### Blockchain Data Extraction

```bash
# Extract logs across block range to Parquet
para extract logs -b 19000000:19001000 -o ./data -f parquet --rpc mainnet

# Extract block headers to JSON
para extract blocks -b 19000000:19000050 -o ./data -f json
```

### Local Devnet Management

```bash
# Start background Anvil devnet daemon on port 8545
para devnet up

# Start Anvil with live mainnet fork and 2-second block intervals
para devnet up --port 8555 --fork https://eth.llamarpc.com --block-time 2

# Inspect devnet status and process PID
para devnet status

# Show pre-funded development accounts, balances, and private keys
para devnet accounts

# Tail live devnet logs
para devnet logs -n 50

# Stop devnet daemon
para devnet down
```

### RPC Endpoint Management

```bash
# List all configured RPC endpoints (MESC + custom)
para rpc list

# Resolve network name or chain ID to RPC URL
para rpc resolve mainnet
para rpc resolve 8453
para rpc resolve arbitrum

# Set default RPC endpoint
para rpc default sepolia

# Add custom RPC endpoint
para rpc add local-node http://127.0.0.1:8545
```

### On-Chain Inspection

```bash
# Query chain state and gas metrics
para chain info

# Query block metadata
para chain block latest
para chain block 19500000

# Inspect transaction details and decoded selector
para chain tx 0x5c504ed432cb51138b309d2ed9413f733230d63a07bd1c5b06baac64cd55f241

# Query address balance, nonce, and contract status
para chain address 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Retrieve deployed contract runtime bytecode
para chain code 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Stream live blocks as they are mined
para chain watch
```

### Project Scaffolding

```bash
# Scaffold an MEV multi-strategy searcher bot
para new bot my-searcher ./bots

# Scaffold an Alloy TypeScript/Rust automation script
para new script deploy-script

# Scaffold a Foundry smart contract suite
para new contract TokenVault
```

### Web GUI Dashboard

Launch the browser-based graphical user interface:

```bash
para gui --port 3333
```

---

## Web GUI Interface Layout

```
+-------------------------------------------------------------------------+
| [Icon] Rusty Chain Kit   v0.1.0   para            [ Devnet: Active:8545 ] |
+-------------------------------------------------------------------------+
| Navigation:  [Overview]  [Devnet]  [Explorer]  [Compiler]  [Decompiler] |
+-------------------------------------------------------------------------+
|                                                                         |
|  +---------------------------+   +-----------------------------------+  |
|  | Devnet Controller         |   | Live Blockchain Explorer          |  |
|  | - Status: Running         |   | - Latest Block: #19520412         |  |
|  | - Port: 8545              |   | - Base Fee: 12.4 Gwei             |  |
|  | - Chain ID: 31337         |   | - Gas Price: 14.1 Gwei            |  |
|  | [Stop] [Restart] [Logs]   |   | - Latency: 18 ms                  |  |
|  +---------------------------+   +-----------------------------------+  |
|                                                                         |
|  +-------------------------------------------------------------------+  |
|  | Pre-funded Accounts (10,000 ETH each)                             |  |
|  | 0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266                     |  |
|  | 1: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8                     |  |
|  +-------------------------------------------------------------------+  |
|                                                                         |
+-------------------------------------------------------------------------+
```

---

## Automated Universal Updater

To keep all integrated tools synchronized with upstream releases and rebuild the unified binary:

```bash
# Check for updates without modifying local trees
./update-all.sh --check

# Synchronize updates and rebuild binaries
./update-all.sh

# Force recompile all binaries
./update-all.sh --rebuild
```

---

## Testing

Run the test suite:

```bash
cargo test
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
