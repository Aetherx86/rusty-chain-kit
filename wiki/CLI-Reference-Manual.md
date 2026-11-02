# CLI Reference Manual

The Rusty Chain Kit Command Line Interface can be invoked using either binary alias: `para` or `rusty-chain-kit`.

---

## Global Options

```
para [OPTIONS] <COMMAND>
```

- `-h, --help`: Print help information.
- `-V, --version`: Print version information (`v0.1.1-beta`).
- `--verbose`: Enable detailed operational logs and diagnostic traces.

---

## Command Reference

### 1. `para compile`
Compile Solidity contracts with sub-second compiler speeds.
```bash
para compile [PATH] [OPTIONS]
```
- `PATH`: File or directory to compile (default: `contracts/` or `src/`).
- `--opt-level <N>`: Optimizer runs (default: 200).
- `--via-ir`: Compile via Yul IR pipeline.
- `--json`: Output compilation artifacts and ABI diagnostics as JSON.

---

### 2. `para storage`
Inspect state variable slot packing and evaluate proxy upgrade storage collisions.
```bash
para storage <CONTRACT_A> [CONTRACT_B] [OPTIONS]
```
- `CONTRACT_A`: Target contract Solidity file (or implementation V1).
- `CONTRACT_B`: Optional implementation V2 to perform differential collision checks.
- `--json`: Output storage schema and collision report as structured JSON.

---

### 3. `para copilot`
Diagnose vulnerabilities and synthesize executable Foundry PoC exploit test harnesses.
```bash
para copilot <TARGET> [OPTIONS]
```
- `TARGET`: Target contract Solidity file or bytecode.
- `--poc`: Synthesize complete Foundry test contract (`contract ExploitTest is Test`).
- `--out <FILE>`: Save generated exploit PoC harness to a specific file path.
- `--json`: Output vulnerability diagnostic report as JSON.

---

### 4. `para bench`
Run cross-compiler differential benchmarks comparing optimization profiles and gas usage.
```bash
para bench <TARGET> [OPTIONS]
```
- `TARGET`: Solidity contract file to benchmark.
- `--profile <NAME>`: Custom optimization profile (`standard`, `aggressive`, `size`).
- `--json`: Output benchmark comparison matrix as JSON.

---

### 5. `para flow`
Trace multi-hop fund flows and screen counterparties against sanctions registries.
```bash
para flow <ADDRESS> [OPTIONS]
```
- `ADDRESS`: Subject Ethereum address (`0x...`) or transaction hash to trace.
- `--hops <N>`: Maximum graph traversal hops (default: 2, range: 1 to 4).
- `--json`: Output complete transaction flow graph as JSON.
- `--csv <FILE>`: Export graph edges and nodes to a CSV file.

---

### 6. `para verify`
Formally prove protocol safety invariants using symbolic SMT constraint solving.
```bash
para verify <CONTRACT> [OPTIONS]
```
- `CONTRACT`: Solidity contract file containing formal invariants.
- `--invariant <NAME>`: Specific invariant function name to prove (e.g., `solvency`, `constant_product`).
- `--timeout <MS>`: SMT solver timeout per branch in milliseconds (default: 5000).

---

### 7. `para audit`
Audit smart contracts using static analysis, stateful fuzzing, and exploit synthesis.
```bash
para audit <PATH> [OPTIONS]
```
- `PATH`: Directory or contract file to audit.
- `--fuzz`: Execute stateful snapshot fuzzer across multi-transaction call sequences.
- `--runs <N>`: Number of fuzzing iterations (default: 5000).
- `--prove`: Verify contract invariants using symbolic execution alongside static auditing.
- `--poc`: Synthesize executable exploit PoC test harnesses for detected vulnerabilities.
- `--explain`: Generate remediation instructions for identified flaws.

---

### 8. `para profile`
Profile cycle-accurate opcode gas consumption and export execution flamegraphs.
```bash
para profile <TX_HASH> [OPTIONS]
```
- `TX_HASH`: Transaction hash on-chain (`0x...`).
- `--flamegraph <FILE>`: Export hierarchical flamegraph JSON compatible with Speedscope.

---

### 9. `para trace`
Display a detailed step-by-step execution trace of an on-chain transaction.
```bash
para trace <TX_HASH>
```
- `TX_HASH`: Transaction hash to inspect.

---

### 10. `para autopsy`
Analyze and dissect reverted transaction data into human-readable error reasons.
```bash
para autopsy <REVERT_HEX>
```
- `REVERT_HEX`: Raw revert calldata hex string or transaction hash.
- Decodes `Error(string)`, custom Solidity error selectors, and exact `Panic(uint256)` codes.

---

### 11. `para aa`
Next-generation Account Abstraction and EIP-7702 tooling.
```bash
para aa <SUBCOMMAND>
```
- `generate-auth`: Generate and sign an EIP-7702 authorization tuple.
  - `--chain-id <ID>`: Target chain ID.
  - `--address <ADDR>`: Delegation contract address (`0x...`).
  - `--nonce <N>`: Signer account nonce.
  - `--private-key <HEX>`: Signer private key hex string.
- `bundle`: Simulate an ERC-4337 UserOperation bundle.
  - `--json`: Output bundle simulation validation results as JSON.

---

### 12. `para chain`
Inspect live blockchain state, blocks, transactions, and deployed bytecode.
```bash
para chain <SUBCOMMAND>
```
- `info`: Display current connected network name, chain ID, and latest block number.
- `block [NUMBER]`: Display block header summary, gas usage, and transaction count.
- `tx <HASH> [--decode-aa]`: Inspect transaction details and decode EIP-7702 / ERC-4337 envelopes.
- `address <ADDR> [--inspect]`: Query address balance, nonce, and code state.
- `code <ADDR> [--abi] [--proxy] [--cfg]`:
  - `--abi`: Extract function selectors and inferred argument signatures.
  - `--proxy`: Detect proxy pattern (EIP-1967, EIP-1167, EIP-1822, EIP-2535, EIP-7702).
  - `--cfg`: Lift runtime bytecode into a Basic Block Control Flow Graph.

---

### 13. `para repl`
Interactive Solidity and EVM expression read-eval-print loop.
```bash
para repl [OPTIONS]
```
- `--eval <EXPR>`: Evaluate a single Solidity expression non-interactively and exit (e.g., `keccak256("test")`).

---

### 14. `para sim`
Discrete-event multi-agent simulation for AMM pools and market dynamics.
```bash
para sim [OPTIONS]
```
- `--steps <N>`: Number of simulation steps/blocks (default: 100).
- `--initial-price <F>`: Starting pool price (default: 3000.0).
- `--volatility <F>`: Geometric Brownian Motion price volatility (default: 0.02).
- `--seed <N>`: Deterministic PRNG seed for reproducible simulations.

---

### 15. `para mev`
High-performance MEV engine and mempool execution framework.
```bash
para mev <SUBCOMMAND>
```
- `run <STRATEGY>`: Stream mempool and execute an arbitrage or searcher strategy.
  - `--rpc <URL>`: WebSocket RPC endpoint (e.g., `ws://127.0.0.1:8545`).
  - `--dry-run`: Simulate transactions locally without broadcasting.
- `backtest <STRATEGY>`: Backtest strategy execution against historical block ranges.
  - `--start-block <N>`: Initial block number.
  - `--end-block <N>`: Terminal block number.

---

### 16. `para zk`
Zero-Knowledge Virtual Machine project management and proof generation.
```bash
para zk <SUBCOMMAND>
```
- `prove`: Compile guest program and generate execution proof.
  - `--path <DIR>`: Path to zkVM project directory.

---

### 17. `para query`
Vectorized SQL analytics engine for querying blockchain datasets.
```bash
para query <SQL> [OPTIONS]
```
- `SQL`: Standard SQL query string (`SELECT ... FROM blocks WHERE ...`).
- `--json`: Output analytical query result set as JSON.

---

### 18. `para devnet`
Manage the local persistent background development network.
```bash
para devnet <SUBCOMMAND>
```
- `up`: Start local devnet in the background.
  - `--port <N>`: Listening port (default: 8545).
  - `--block-time <N>`: Automated block production interval in seconds.
  - `--fork <NETWORK>`: Fork remote state from a network or RPC URL.
  - `--fork-block <N>`: Specific historical block number to fork from.
  - `--persist`: Persist state database across restarts.
- `down`: Stop the running devnet process.
- `status`: Check if the devnet daemon is currently running.
- `logs`: Tail devnet standard output and error streams.
- `accounts`: Display pre-funded development accounts and private keys.

---

### 19. `para node`
Manage a full execution node daemon in development mode.
```bash
para node <SUBCOMMAND>
```
- `up`: Start local execution client daemon.
  - `--port <N>`: RPC listening port (default: 8645).
  - `--fresh`: Wipe data directory before starting.
- `down`: Stop the running node daemon.
- `status`: Check node daemon liveness.
- `logs`: Tail node logs.

---

### 20. `para rpc`
Manage RPC endpoints and verify state proofs trustlessly.
```bash
para rpc <SUBCOMMAND>
```
- `list`: Display configured RPC endpoints from `para.toml`.
- `add <NAME> <URL>`: Add or update an RPC endpoint.
- `remove <NAME>`: Remove an RPC endpoint.
- `default [NAME]`: View or set the default RPC network.
- `resolve <NAME>`: Resolve the canonical endpoint URL for a network.
- `trustless <ADDRESS>`: Verify account balance and storage root against consensus state roots.

---

### 21. `para gui`
Launch the interactive web graphical command center.
```bash
para gui [OPTIONS]
```
- `--port <N>, -p <N>`: Port to bind dashboard server (default: 3333).
- `--no-open`: Start the server without automatically opening the default web browser.

---

### 22. `para new`
Scaffold new projects from production-grade templates.
```bash
para new <TEMPLATE> <NAME>
```
- `contract <NAME>`: Scaffold a new Foundry-compatible smart contract project.
- `bot <NAME>`: Scaffold a high-performance Rust MEV searcher bot project.
- `script <NAME>`: Scaffold a TypeScript Viem automation script project.
- `zk <NAME>`: Scaffold a decoupled RISC-V zkVM guest and prover project.
