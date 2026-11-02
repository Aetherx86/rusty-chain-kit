# Web GUI Command Center

Rusty Chain Kit provides an interactive Web Graphical User Interface dashboard served by a lightweight, embedded asynchronous web server (`server.rs` powered by Axum 0.8 and Tokio).

The GUI command center provides full visual access to every toolchain capability, including real-time graph visualization, contract auditing, and devnet monitoring.

---

## 1. Launching the GUI

Start the local web dashboard:
```bash
para gui
```
By default, the server binds to `127.0.0.1:3333` and automatically opens your default web browser.

### Custom Options
- `--port <N>, -p <N>`: Specify a custom port (e.g., `para gui --port 8080`).
- `--no-open`: Run in headless mode without automatically launching a browser window (ideal for remote servers, Docker containers, and CI environments).

---

## 2. Dashboard Modules & Views

### 1. Overview & Service Daemon Center
- Real-time status cards for the background Devnet (`anvil`) and Execution Node (`reth`).
- One-click buttons to spawn, restart, fork from mainnet, or terminate background processes.
- Live streaming process output logs and PID monitoring.

### 2. Contract Studio
- Compile contracts and view emitted ABIs and bytecode.
- **Storage Inspector Tab**: View visual slot packing diagrams and wasted byte padding.
- **Upgrade Collision Tab**: Upload Implementation V1 and V2 to run instant differential checks.
- **Optimizer Benchmarker Tab**: View differential bytecode and deployment gas charts.

### 3. Forensic Flow Visualizer
- Interactive directed graph canvas rendering multi-hop transaction flows.
- Node coloring by entity type (`DEX`, `Bridge`, `Mixer`, `Exchange`, `Lending`).
- Risk badges with sanctions screening alerts.
- One-click export to PNG, JSON, or CSV edge lists.

### 4. Vulnerability Explainer & Exploit Synthesizer
- Paste contract source code or select a workspace file to diagnose vulnerability patterns.
- View step-by-step plain-language explanations of attack mechanics.
- Copy or download synthesized Foundry test harnesses (`contract ExploitTest is Test`).

### 5. Gas Profiler & Flamegraph Viewer
- Input transaction hashes to inspect cycle-accurate opcode gas attribution.
- Interactive hierarchical flamegraph viewer powered by Speedscope.

### 6. Interactive Solidity REPL
- Live terminal console in the browser to test Solidity expressions, ABI encoding, and hash functions with instant gas feedback.

### 7. SQL Analytics Explorer
- Interactive SQL editor with syntax highlighting and formatted tabular results.

---

## 3. REST API Specification

The GUI backend exposes a comprehensive JSON REST API:

| Endpoint | Method | Request Payload | Description |
| :--- | :--- | :--- | :--- |
| `/api/compile` | `POST` | `{"path": "..."}` or `{"source": "..."}` | Compile Solidity contract |
| `/api/verify/run` | `POST` | `{"contract": "...", "invariant": "..."}` | Run formal SMT verification |
| `/api/storage/inspect` | `POST` | `{"contract_a": "...", "contract_b": "..."}`| Inspect storage or compare layouts |
| `/api/copilot/explain` | `POST` | `{"target": "..."}` or `{"source": "..."}` | Diagnose vulnerabilities & PoC |
| `/api/bench/run` | `POST` | `{"target": "..."}` or `{"source": "..."}` | Run cross-compiler benchmark |
| `/api/forensics/flow` | `POST` | `{"address": "0x...", "hops": 2}` | Generate multi-hop flow graph |
| `/api/repl/eval` | `POST` | `{"expression": "keccak256('test')"}` | Evaluate single Solidity expression |
| `/api/analytics/query`| `POST` | `{"query": "SELECT ...", "limit": 10}` | Execute analytical SQL query |
| `/api/services/status` | `GET` | None | Query devnet and node daemon status |
