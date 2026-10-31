//! Command line interface definitions for `para`.

use crate::workspace::Strategy;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "para",
    author = "Paradigm Suite",
    version = "0.1.0",
    about = "All-in-one command center (CLI + GUI) for the Paradigm / Ethereum Rust + TS toolchain",
    long_about = "para unites Foundry, Reth, Alloy, Solar, Cryo, Viem, Artemis, MESC, and Heimdall into a single unified CLI and Web GUI."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Direct passthrough command if no built-in subcommand matched
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub passthrough: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage tracked repositories and sync with canonical upstreams
    Repos(ReposArgs),

    /// Synchronize all repositories from upstreams and streamline the entire toolchain build
    Update(UpdateArgs),

    /// Build repository binaries and link them into .para/bin
    Build(BuildArgs),

    /// Tool registry and binary runner (.para/bin and $PATH)
    Tools(ToolsArgs),

    /// Multiple Endpoint Shared Configuration (MESC) & RPC endpoint management
    Rpc(RpcArgs),

    /// Anvil local devnet management (background daemon)
    Devnet(DevnetArgs),

    /// Reth modular execution client management (--dev mode)
    Node(NodeArgs),

    /// Direct chain queries via Alloy (info, blocks, txs, addresses)
    Chain(ChainArgs),

    /// Compile Solidity contracts using Solar (Rust compiler) or Forge
    Compile(CompileArgs),

    /// Run static security audit on Solidity code (Aderyn / Solar linter)
    Audit(AuditArgs),

    /// Decompile EVM bytecode to Solidity via Heimdall or disassemble via Cast
    Decompile(DecompileArgs),

    /// Extract blockchain datasets (blocks, txs, logs) via Cryo
    Extract(ExtractArgs),

    /// Scaffold new projects (Artemis bot, Viem script, Foundry/Solar contract)
    New(NewArgs),

    /// Launch the embedded Web GUI Dashboard
    Gui(GuiArgs),

    // ───────────────── Direct Tool Passthrough Subcommands ─────────────────
    /// Invoke `forge` (Foundry build/test/deploy tool)
    Forge {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `cast` (Foundry EVM swiss-army knife)
    Cast {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `anvil` (Foundry local node)
    Anvil {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `chisel` (Foundry Solidity REPL)
    Chisel {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `solar` (Fast Solidity compiler)
    Solar {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `cryo` (Blockchain data extractor)
    Cryo {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `reth` (Ethereum execution client)
    Reth {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `mesc` (Multiple Endpoint Shared Config)
    Mesc {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `heimdall` (EVM decompiler & analyzer)
    Heimdall {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `aderyn` (Static analyzer for Solidity)
    Aderyn {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Invoke `revme` (REVM test runner & debugger)
    Revme {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

// ───────────────────────────── Repos ─────────────────────────────

#[derive(Args)]
pub struct UpdateArgs {
    /// Check for upstream updates without modifying working trees
    #[arg(long, short)]
    pub check: bool,

    /// Force rebuilding all tools even if commits haven't changed
    #[arg(long, short)]
    pub rebuild: bool,

    /// Only sync repositories, skip building binaries
    #[arg(long, short = 's')]
    pub skip_build: bool,

    /// Git integration strategy: rebase (default), merge, or ff
    #[arg(long, short = 'S')]
    pub strategy: Option<Strategy>,
}

#[derive(Args)]
pub struct ReposArgs {
    #[command(subcommand)]
    pub action: Option<ReposSubcommand>,
}

#[derive(Subcommand)]
pub enum ReposSubcommand {
    /// List all configured repositories
    List,
    /// Show detailed git status, tracking branches, and commit freshness
    Status,
    /// Sync repositories with canonical upstream remotes
    Sync {
        /// Names of repos to sync (empty = sync all)
        repos: Vec<String>,
        /// Strategy to apply upstream changes: rebase (default), merge, or ff
        #[arg(long, short)]
        strategy: Option<Strategy>,
        /// Don't stash uncommitted local changes before sync
        #[arg(long)]
        no_stash: bool,
        /// Show what would be synced without modifying repository state
        #[arg(long)]
        dry_run: bool,
        /// Push updated local branch to your fork if configured
        #[arg(long)]
        push: bool,
    },
    /// Fetch latest commits from upstream remotes without modifying working trees
    Fetch {
        repos: Vec<String>,
    },
}

// ───────────────────────────── Build ─────────────────────────────

#[derive(Args)]
pub struct BuildArgs {
    /// Name of repo to build (empty = build all configured repos)
    pub repo: Option<String>,
    /// Build profile: release (default) or dev
    #[arg(long, short)]
    pub profile: Option<String>,
    /// Number of parallel cargo jobs
    #[arg(long, short = 'j')]
    pub jobs: Option<usize>,
    /// Only build if source commits have changed since last build
    #[arg(long)]
    pub stale: bool,
}

// ───────────────────────────── Tools ─────────────────────────────

#[derive(Args)]
pub struct ToolsArgs {
    #[command(subcommand)]
    pub action: Option<ToolsSubcommand>,
}

#[derive(Subcommand)]
pub enum ToolsSubcommand {
    /// List all binaries and their resolved sources (.para/bin or PATH)
    List,
    /// Run a binary with arguments
    Run {
        bin: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

// ───────────────────────────── RPC ─────────────────────────────

#[derive(Args)]
pub struct RpcArgs {
    #[command(subcommand)]
    pub action: Option<RpcSubcommand>,
}

#[derive(Subcommand)]
pub enum RpcSubcommand {
    /// List all available RPC endpoints from config, MESC, and running services
    List,
    /// Resolve an alias or MESC query to its full URL
    Resolve {
        query: String,
    },
    /// Get or set the default RPC endpoint
    Default {
        name: Option<String>,
    },
    /// Add a custom RPC endpoint alias
    Add {
        name: String,
        url: String,
    },
    /// Remove an RPC endpoint alias
    Remove {
        name: String,
    },
}

// ───────────────────────────── Devnet ─────────────────────────────

#[derive(Args)]
pub struct DevnetArgs {
    #[command(subcommand)]
    pub action: Option<DevnetSubcommand>,
}

#[derive(Subcommand)]
pub enum DevnetSubcommand {
    /// Start Anvil devnet as a background daemon
    Up {
        #[arg(long, short, default_value_t = 8545)]
        port: u16,
        /// Fork a remote chain (pass name or URL, e.g. --fork mainnet)
        #[arg(long)]
        fork: Option<String>,
        /// Fork at a specific block number
        #[arg(long)]
        fork_block: Option<u64>,
        /// Custom chain id (default: 31337)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Block time in seconds (0 = mine on demand)
        #[arg(long, default_value_t = 0)]
        block_time: u64,
        /// Persist chain state to disk across restarts
        #[arg(long)]
        persist: bool,
    },
    /// Stop running Anvil devnet daemon
    Down,
    /// Check running status of Anvil devnet
    Status,
    /// Show pre-funded development accounts and private keys
    Accounts,
    /// Tail live Anvil daemon logs
    Logs {
        #[arg(long, short = 'n', default_value_t = 50)]
        lines: usize,
    },
}

// ───────────────────────────── Node (Reth) ─────────────────────────────

#[derive(Args)]
pub struct NodeArgs {
    #[command(subcommand)]
    pub action: Option<NodeSubcommand>,
}

#[derive(Subcommand)]
pub enum NodeSubcommand {
    /// Start Reth in --dev mode
    Up {
        #[arg(long, short, default_value_t = 8645)]
        port: u16,
        /// Block interval in seconds (default: 1s)
        #[arg(long, default_value_t = 1)]
        block_time: u64,
        /// Wipe dev datadir before starting
        #[arg(long)]
        fresh: bool,
    },
    /// Stop running Reth node
    Down,
    /// Check Reth status
    Status,
    /// Tail Reth node logs
    Logs {
        #[arg(long, short = 'n', default_value_t = 50)]
        lines: usize,
    },
}

// ───────────────────────────── Chain ─────────────────────────────

#[derive(Args)]
pub struct ChainArgs {
    /// RPC endpoint or alias to target (defaults to configured default / devnet)
    #[arg(long, short, env = "ETH_RPC_URL", global = true)]
    pub rpc: Option<String>,

    #[command(subcommand)]
    pub action: ChainSubcommand,
}

#[derive(Subcommand)]
pub enum ChainSubcommand {
    /// Display chain summary (chain ID, block, gas price, client version)
    Info,
    /// Inspect block details by number, hash, or 'latest'
    Block {
        id: Option<String>,
    },
    /// Inspect transaction details, status, and receipt
    Tx {
        hash: String,
    },
    /// Inspect account address (balance, nonce, code size, contract status)
    Address {
        address: String,
    },
    /// Fetch runtime bytecode for a contract address
    Code {
        address: String,
    },
    /// Stream new blocks as they are mined
    Watch,
}

// ───────────────────────────── Workflows ─────────────────────────────

#[derive(Args)]
pub struct CompileArgs {
    /// Solidity file or project directory to compile
    pub path: PathBuf,
    /// Use Forge instead of Solar
    #[arg(long)]
    pub forge: bool,
}

#[derive(Args)]
pub struct AuditArgs {
    /// Path to project to audit
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct DecompileArgs {
    /// Contract address (0x...) or bytecode hex or file path
    pub target: String,
    /// RPC endpoint to fetch bytecode if target is an address
    #[arg(long, short)]
    pub rpc: Option<String>,
}

#[derive(Args)]
pub struct ExtractArgs {
    /// Cryo dataset name (blocks, transactions, logs, traces, etc.)
    pub dataset: String,
    /// Block range (e.g. -100:latest, or 18000000:18000100)
    #[arg(long, short)]
    pub blocks: Option<String>,
    /// Output format: parquet (default), csv, or json
    #[arg(long, short)]
    pub format: Option<String>,
    /// Destination directory
    #[arg(long, short)]
    pub out: Option<PathBuf>,
    /// RPC endpoint
    #[arg(long, short)]
    pub rpc: Option<String>,
}

#[derive(Args)]
pub struct NewArgs {
    #[command(subcommand)]
    pub kind: NewKind,
}

#[derive(Subcommand)]
pub enum NewKind {
    /// Scaffold an Artemis-style MEV bot (powered by Alloy)
    Bot {
        name: String,
        out: Option<PathBuf>,
    },
    /// Scaffold a modern Viem TypeScript script
    Script {
        name: String,
        out: Option<PathBuf>,
    },
    /// Scaffold a Solidity contract project with Foundry and Solar
    Contract {
        name: String,
        out: Option<PathBuf>,
    },
}

#[derive(Args)]
pub struct GuiArgs {
    /// Port to listen on (default: 3333)
    #[arg(long, short, default_value_t = 3333)]
    pub port: u16,
    /// Do not automatically open the browser
    #[arg(long)]
    pub no_open: bool,
}
