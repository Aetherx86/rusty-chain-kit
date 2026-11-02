# Architecture and Design

Rusty Chain Kit is architected as a modular, high-throughput developer toolchain and local execution runtime for Ethereum and EVM systems. It is written in pure Rust (Edition 2024, Rust 1.94+) to ensure maximum execution efficiency, sub-millisecond CLI response times, and deterministic memory safety.

---

## 1. System Decomposition

The toolchain is organized into distinct, decoupled subsystems:

```
[ Developer Entrypoints ]
  |-- CLI (`para` / `rusty-chain-kit`)  --> Clap derive parser & subcommand dispatcher
  |-- Web GUI Dashboard (`para gui`)    --> Axum 0.8 async HTTP + WebSocket server
  |-- REST API Endpoints                --> `/api/*` json handlers

[ Workspace & State Layer ]
  |-- Workspace Discovery Engine       --> `para.toml` -> `para/para.toml` -> `foundry.toml`
  |-- State Directory Manager (`.para`) --> `bin/`, `logs/`, `run/`, `data/`
  |-- RPC Routing Engine (MESC)        --> Endpoint resolution, override precedence, trustless verification

[ Core Analytical & Security Engines ]
  |-- Symbolic Execution & SMT         --> Path condition extraction + Z3 SMT solver
  |-- Stateful Fuzzing Engine          --> Mutational call sequence synthesis + invariant verification
  |-- Storage & Collision Inspector    --> 32-byte slot packing + upgrade differential analysis
  |-- Exploit PoC Synthesizer          --> Static vulnerability diagnosis + Foundry test generation
  |-- Gas Profiling & Autopsy Engine   --> Opcode attribution + Speedscope flamegraph + revert decoder
  |-- Bytecode Intelligence Engine     --> Decompilation + ABI extraction + Proxy pattern detection
  |-- Forensics & Graph Engine         --> Directed multi-hop graph traversal + entity classifier
  |-- Vectorized SQL OLAP Engine       --> Block/transaction SQL executor over Parquet/CSV/JSON
  |-- Simulation & Market Dynamics     --> Discrete-event AMM engine + Geometric Brownian Motion orderflow
```

---

## 2. Workspace Discovery Lifecycle

Rusty Chain Kit enforces a zero-config developer experience. When executed, the toolchain resolves its workspace context using a multi-tier fallback protocol:

```
1. Environment Variable Check:
   Is `$PARA_MANIFEST` set?
   |-- Yes: Load explicit manifest path.
   +-- No: Proceed to step 2.

2. Upward Directory Traversal (from current working directory):
   Loop traversing parent directories:
   |-- Check for `para.toml`
   |-- Check for `para/para.toml`
   |-- Check for `rusty-chain-kit/para.toml`
   +-- Check for `foundry.toml` (Zero-Config Fallback)
       |-- If found: Dynamically construct virtual workspace rooted at Foundry project directory.
       +-- If not found: Move to parent directory.

3. Built-In Workspace Fallback:
   If traversal reaches root without finding a manifest, verify if built-in binary template exists.
```

### The State Directory (`.para/`)
Every workspace manages an isolated local state directory (`<workspace_root>/.para/`):
- `bin/`: Symlinked tool binaries and custom user scripts.
- `logs/`: Background service stdout and stderr logs (`devnet.log`, `node.log`).
- `run/`: Active process PID metadata files (`devnet.pid`, `node.pid`).
- `data/`: Local persistent state databases, simulated orderbook histories, and cached contract artifacts.

---

## 3. Unified Dispatch Model

Every feature in Rusty Chain Kit is implemented as an independent library module in `src/` that exposes pure Rust functions. Both the Command Line Interface (`cli.rs`) and the Web GUI Server (`server.rs`) invoke the same underlying engine APIs:

```
                        [ User Action ]
                               |
               +---------------+---------------+
               |                               |
        (CLI Invocation)                (Web GUI HTTP API)
        `para storage Vault.sol`        `POST /api/storage/inspect`
               |                               |
               v                               v
        `cli::handle_storage()`         `server::api_storage_inspect()`
               |                               |
               +---------------+---------------+
                               |
                               v
               [ `storage::inspect_storage()` ]
                               |
                   +-----------+-----------+
                   |                       |
            (Format Table)          (Serialize JSON)
            Terminal Output         HTTP Response
```

This guarantees 100% feature parity between CLI terminal usage and browser-based GUI interaction.

---

## 4. Daemon & Process Supervision Model

The local devnet (`para devnet`) and execution node daemons (`para node`) run as detached background processes monitored by the service manager (`services.rs`):

- **Health Probing**: Periodically checks process liveness using POSIX signal polling (`kill -0 <PID>`) or OS process table inspection.
- **Log Streaming**: Standard output and error streams are piped to dedicated log files in `.para/logs/`.
- **Clean Termination**: Shuts down processes gracefully using `SIGTERM`, falling back to `SIGKILL` only if the daemon fails to exit within a 3-second grace window.
- **PID File Guard**: Uses atomic filesystem locks on PID files in `.para/run/` to prevent port collisions or concurrent duplicate instances.

---

## 5. Security & Isolation Invariants

1. **Push Isolation Guard**: Under no circumstances does Rusty Chain Kit push commits to external companion or sibling checkouts. The git synchronization module explicitly restricts remote push operations to the repository itself (`rusty-chain-kit`), ensuring external developer repositories are treated as read-only upstream sources.
2. **Deterministic SMT Solving**: Formal invariant queries are dispatched to the Z3 solver with deterministic random seeds and explicit query timeouts, preventing infinite loops on complex non-linear arithmetic.
3. **Safe Memory Operations**: The entire codebase avoids unsafe raw pointer arithmetic, leveraging Rust's ownership and borrow checker to guarantee memory safety and thread safety.
