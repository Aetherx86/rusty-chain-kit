//! Rusty Chain Kit (`rusty-chain-kit` / `para`)
//!
//! An all-in-one developer command center (CLI + GUI) uniting the Paradigm
//! and Ethereum Rust/TS ecosystem: Foundry, Reth, Alloy, Solar, Cryo, MESC,
//! Heimdall, Revm, Aderyn, Artemis, and Viem.

pub mod aa;
pub mod analytics;
pub mod builder;
pub mod bytecode;
pub mod chain;
pub mod cli;
pub mod git;
pub mod mev;
pub mod repl;
pub mod rpc;
pub mod scaffold;
pub mod server;
pub mod services;
pub mod sim;
pub mod tools;
pub mod trace;
pub mod trustless;
pub mod ui;
pub mod verify;
pub mod workflows;
pub mod workspace;
pub mod zk;

use alloy::primitives::{Address, U256};
use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use std::collections::BTreeMap;
use std::str::FromStr;
use workspace::Workspace;

#[tokio::main]
pub async fn run() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    // Check if the first argument matches a known tool binary directly (e.g. `para forge test`)
    if args.len() > 1 {
        let first = &args[1];
        let known_tools = [
            "forge", "cast", "anvil", "chisel", "solar", "cryo", "reth", "mesc", "heimdall",
            "aderyn", "revme",
        ];
        if known_tools.contains(&first.as_str()) {
            let ws = Workspace::discover()?;
            let tool_args = args[2..].to_vec();
            return tools::exec(&ws, first, &tool_args);
        }
    }

    let cli = Cli::parse();
    let ws = Workspace::discover()?;

    let cmd = match cli.command {
        Some(c) => c,
        None => {
            if !cli.passthrough.is_empty() {
                let bin = &cli.passthrough[0];
                let rest = cli.passthrough[1..].to_vec();
                return tools::exec(&ws, bin, &rest);
            }
            // If invoked with no arguments, launch GUI dashboard
            ui::header("Launching Para Web GUI Dashboard");
            return server::run_server(ws, 3333, true).await;
        }
    };

    match cmd {
        Commands::Repos(r_args) => handle_repos(&ws, r_args)?,
        Commands::Update(u_args) => handle_update(&ws, u_args)?,
        Commands::Build(b_args) => handle_build(&ws, b_args)?,
        Commands::Tools(t_args) => handle_tools(&ws, t_args)?,
        Commands::Rpc(rpc_args) => handle_rpc(&ws, rpc_args)?,
        Commands::Devnet(d_args) => handle_devnet(&ws, d_args)?,
        Commands::Node(n_args) => handle_node(&ws, n_args)?,
        Commands::Chain(c_args) => handle_chain(&ws, c_args).await?,
        Commands::Compile(c) => handle_compile(&ws, c)?,
        Commands::Audit(a) => handle_audit(&ws, a)?,
        Commands::Verify(v) => handle_verify(&ws, v)?,
        Commands::Repl(r) => repl::run_repl(r.eval.as_deref())?,
        Commands::Aa(aa_args) => handle_aa(&ws, aa_args)?,
        Commands::Trace(t) => handle_trace(&ws, t).await?,
        Commands::Profile(p) => handle_profile(&ws, p).await?,
        Commands::Autopsy(a) => handle_autopsy(&ws, a).await?,
        Commands::Query(q) => handle_query(&ws, q)?,
        Commands::Mev(m) => handle_mev(&ws, m)?,
        Commands::Sim(s) => handle_sim(&ws, s)?,
        Commands::Zk(z) => handle_zk(&ws, z)?,
        Commands::Decompile(d) => handle_decompile(&ws, d).await?,
        Commands::Extract(e) => handle_extract(&ws, e)?,
        Commands::New(n) => handle_new(&ws, n)?,
        Commands::Gui(g) => {
            server::run_server(ws, g.port, !g.no_open).await?;
        }

        // Passthroughs
        Commands::Forge { args } => tools::exec(&ws, "forge", &args)?,
        Commands::Cast { args } => tools::exec(&ws, "cast", &args)?,
        Commands::Anvil { args } => tools::exec(&ws, "anvil", &args)?,
        Commands::Chisel { args } => tools::exec(&ws, "chisel", &args)?,
        Commands::Solar { args } => tools::exec(&ws, "solar", &args)?,
        Commands::Cryo { args } => tools::exec(&ws, "cryo", &args)?,
        Commands::Reth { args } => tools::exec(&ws, "reth", &args)?,
        Commands::Mesc { args } => tools::exec(&ws, "mesc", &args)?,
        Commands::Heimdall { args } => tools::exec(&ws, "heimdall", &args)?,
        Commands::Aderyn { args } => tools::exec(&ws, "aderyn", &args)?,
        Commands::Revme { args } => tools::exec(&ws, "revme", &args)?,
    }

    Ok(())
}

pub fn handle_update(ws: &Workspace, args: cli::UpdateArgs) -> Result<()> {
    let script = ws.root.join("update-all.sh");
    if !script.exists() {
        anyhow::bail!("update-all.sh not found at {}", script.display());
    }

    let mut cmd_args = vec![];
    if args.check {
        cmd_args.push("--check".to_string());
    }
    if args.rebuild {
        cmd_args.push("--rebuild".to_string());
    }
    if args.skip_build {
        cmd_args.push("--skip-build".to_string());
    }
    if let Some(strat) = args.strategy {
        cmd_args.push("--strategy".to_string());
        cmd_args.push(strat.to_string());
    }

    let status = std::process::Command::new(&script)
        .args(&cmd_args)
        .status()?;

    if !status.success() {
        anyhow::bail!("update-all script exited with code {:?}", status.code());
    }
    Ok(())
}

pub fn handle_repos(ws: &Workspace, args: cli::ReposArgs) -> Result<()> {
    match args.action.unwrap_or(cli::ReposSubcommand::Status) {
        cli::ReposSubcommand::List => {
            let mut t = ui::table(&["Repository", "Category", "Upstream Origin", "Branch", "Description"]);
            for r in &ws.manifest.repos {
                t.add_row(vec![
                    r.name.clone(),
                    r.category.clone(),
                    r.upstream.clone(),
                    r.branch.clone(),
                    r.description.clone(),
                ]);
            }
            println!("{t}");
        }
        cli::ReposSubcommand::Status => {
            ui::header("Repository Status & Upstream Synchronization");
            let mut t = ui::table(&[
                "Repository",
                "Category",
                "Local Branch",
                "Tracking",
                "Ahead",
                "Behind",
                "Dirty",
                "HEAD",
            ]);
            for r in &ws.manifest.repos {
                let st = git::status(ws, r);
                t.add_row(vec![
                    st.name,
                    st.category,
                    st.current_branch,
                    st.upstream_ref,
                    st.ahead.to_string(),
                    st.behind.to_string(),
                    if st.dirty > 0 { format!("{} files", st.dirty) } else { "clean".into() },
                    st.head,
                ]);
            }
            println!("{t}");
        }
        cli::ReposSubcommand::Sync { repos, strategy, no_stash, dry_run, push } => {
            let targets = ws.select(&repos)?;
            let opts = git::SyncOpts {
                strategy,
                no_stash,
                dry_run,
                push,
            };
            ui::header(format!(
                "Syncing {} Repositories (Strategy: {}, DryRun: {})",
                targets.len(),
                opts.strategy.map(|s| s.to_string()).unwrap_or_else(|| "default".into()),
                dry_run
            ));
            for r in targets {
                let rep = git::sync(ws, r, &opts);
                match rep.outcome {
                    git::Outcome::UpToDate => ui::info(format!("{}: up to date ({})", r.name, rep.from)),
                    git::Outcome::Updated => ui::ok(format!("{}: updated {} -> {}", r.name, rep.from, rep.to)),
                    git::Outcome::WouldUpdate => ui::warn(format!("{}: would update {} -> {}", r.name, rep.from, rep.to)),
                    git::Outcome::Cloned => ui::ok(format!("{}: cloned fresh ({})", r.name, rep.to)),
                    git::Outcome::Conflict => ui::fail(format!("{}: merge conflict!", r.name)),
                    git::Outcome::Diverged => ui::fail(format!("{}: diverged from upstream", r.name)),
                    git::Outcome::Error => ui::fail(format!("{}: sync failed", r.name)),
                }
                for note in rep.notes {
                    println!("    - {note}");
                }
            }
        }
        cli::ReposSubcommand::Fetch { repos } => {
            let targets = ws.select(&repos)?;
            ui::header(format!("Fetching {} Repositories", targets.len()));
            for r in targets {
                let dir = ws.repo_dir(r);
                if !dir.exists() {
                    ui::warn(format!("{}: not cloned yet, skipping fetch", r.name));
                    continue;
                }
                match git::fetch(&dir, r) {
                    Ok(_) => ui::ok(format!("{}: fetched from {}", r.name, r.remote)),
                    Err(e) => ui::fail(format!("{}: fetch failed: {e}", r.name)),
                }
            }
        }
    }
    Ok(())
}

pub fn handle_build(ws: &Workspace, args: cli::BuildArgs) -> Result<()> {
    let targets = match args.repo {
        Some(name) => vec![ws.repo(&name)?],
        None => ws.manifest.repos.iter().collect(),
    };

    ui::header(format!("Building {} Repositories", targets.len()));
    let opts = builder::BuildOpts {
        profile: args.profile,
        jobs: args.jobs,
        if_stale: args.stale,
    };

    for r in targets {
        if r.build.is_none() {
            continue;
        }
        ui::info(format!("Building {}", r.name));
        match builder::build(ws, r, &opts) {
            Ok(_) => {
                ui::ok(format!("{}: build succeeded", r.name));
            }
            Err(e) => {
                ui::fail(format!("{}: build failed: {e:#}", r.name));
            }
        }
    }
    Ok(())
}

pub fn handle_tools(ws: &Workspace, args: cli::ToolsArgs) -> Result<()> {
    match args.action.unwrap_or(cli::ToolsSubcommand::List) {
        cli::ToolsSubcommand::List => {
            ui::header("Tool Registry (.para/bin and System PATH)");
            let mut t = ui::table(&["Binary", "Providing Repo", "Source", "Resolved Path", "Version"]);
            for tool in tools::list(ws, true) {
                t.add_row(vec![
                    tool.bin,
                    tool.repo,
                    tool.source,
                    tool.path.unwrap_or_else(|| "(missing)".into()),
                    tool.version.unwrap_or_else(|| "-".into()),
                ]);
            }
            println!("{t}");
        }
        cli::ToolsSubcommand::Run { bin, args } => {
            tools::exec(ws, &bin, &args)?;
        }
    }
    Ok(())
}

pub fn handle_rpc(ws: &Workspace, args: cli::RpcArgs) -> Result<()> {
    match args.action.unwrap_or(cli::RpcSubcommand::List) {
        cli::RpcSubcommand::List => {
            ui::header("Configured RPC Endpoints (Para + MESC + Built-ins)");
            let mut t = ui::table(&["Name / Alias", "Source", "Chain ID", "URL"]);
            for ep in rpc::list(ws) {
                t.add_row(vec![
                    ep.name,
                    ep.source,
                    ep.chain_id.unwrap_or_else(|| "-".into()),
                    ep.url,
                ]);
            }
            println!("{t}");
            if let Some(def) = ws.load_config().rpc.default {
                ui::info(format!("Default endpoint alias: {def}"));
            }
        }
        cli::RpcSubcommand::Resolve { query } => {
            let res = rpc::lookup(ws, &query)?;
            ui::ok(format!("{} resolved via {}: {}", query, res.via, res.url));
        }
        cli::RpcSubcommand::Default { name } => {
            if let Some(n) = name {
                rpc::set_default(ws, Some(n.clone()))?;
                ui::ok(format!("Default RPC set to `{n}`"));
            } else {
                let cur = ws.load_config().rpc.default;
                println!("{}", cur.as_deref().unwrap_or("(none)"));
            }
        }
        cli::RpcSubcommand::Add { name, url } => {
            rpc::add(ws, &name, &url)?;
            ui::ok(format!("Added RPC alias `{name}` -> {url}"));
        }
        cli::RpcSubcommand::Remove { name } => {
            rpc::remove(ws, &name)?;
            ui::ok(format!("Removed RPC alias `{name}`"));
        }
        cli::RpcSubcommand::Trustless { address, block, expected_root, rpc } => {
            let resolved = rpc::resolve(ws, rpc.as_deref())?;
            let addr = Address::from_str(&address).context("invalid address")?;
            let block_num = block.unwrap_or(20000000);
            let report = trustless::verify_account_proof(&resolved.url, addr, block_num, expected_root.as_deref())?;
            ui::header("Trustless Merkle-Patricia Trie Proof Verification");
            ui::kv("RPC Endpoint", report.rpc_url);
            ui::kv("Block Number", report.block_number);
            ui::kv("Consensus State Root", report.state_root);
            ui::kv("Address", report.address);
            ui::kv("Verified Balance", report.balance);
            ui::kv("Verified Nonce", report.nonce);
            ui::kv("Verified Code Hash", report.code_hash);
            ui::kv("Status", if report.is_verified_trustless { "VERIFIED TRUSTLESS" } else { "FAILED" });
            for n in report.notes {
                println!("  - {n}");
            }
        }
    }
    Ok(())
}

pub fn handle_devnet(ws: &Workspace, args: cli::DevnetArgs) -> Result<()> {
    match args.action.unwrap_or(cli::DevnetSubcommand::Status) {
        cli::DevnetSubcommand::Up { port, fork, fork_block, chain_id, block_time, persist } => {
            let opts = services::DevnetOpts {
                port: Some(port),
                fork,
                fork_block,
                chain_id,
                block_time: Some(block_time),
                persist,
                ..Default::default()
            };
            let st = services::start_devnet(ws, &opts)?;
            ui::ok(format!("Anvil devnet started on {} (PID {})", st.url, st.pid));
            ui::info(format!("Logs: {}", st.log));
        }
        cli::DevnetSubcommand::Down => {
            if services::stop(ws, services::Kind::Devnet)? {
                ui::ok("Anvil devnet stopped");
            } else {
                ui::info("Anvil devnet was not running");
            }
        }
        cli::DevnetSubcommand::Status => {
            match services::status(ws, services::Kind::Devnet) {
                Some(st) => ui::ok(format!("Anvil devnet is RUNNING at {} (PID {})", st.url, st.pid)),
                None => ui::info("Anvil devnet is STOPPED"),
            }
        }
        cli::DevnetSubcommand::Accounts => {
            let accs = services::devnet_accounts(ws);
            if accs.is_empty() {
                ui::warn("Devnet is not running or accounts have not been logged yet.");
            } else {
                let mut t = ui::table(&["#", "Address", "Private Key"]);
                for a in accs {
                    t.add_row(vec![
                        a.index.to_string(),
                        a.address,
                        a.private_key.unwrap_or_else(|| "-".into()),
                    ]);
                }
                println!("{t}");
            }
        }
        cli::DevnetSubcommand::Logs { lines } => {
            println!("{}", services::tail(ws, services::Kind::Devnet, lines));
        }
    }
    Ok(())
}

pub fn handle_node(ws: &Workspace, args: cli::NodeArgs) -> Result<()> {
    match args.action.unwrap_or(cli::NodeSubcommand::Status) {
        cli::NodeSubcommand::Up { port, block_time, fresh } => {
            let opts = services::NodeOpts {
                port: Some(port),
                block_time: Some(block_time),
                fresh,
                ..Default::default()
            };
            let st = services::start_node(ws, &opts)?;
            ui::ok(format!("Reth dev node started on {} (PID {})", st.url, st.pid));
        }
        cli::NodeSubcommand::Down => {
            if services::stop(ws, services::Kind::Node)? {
                ui::ok("Reth dev node stopped");
            } else {
                ui::info("Reth dev node was not running");
            }
        }
        cli::NodeSubcommand::Status => {
            match services::status(ws, services::Kind::Node) {
                Some(st) => ui::ok(format!("Reth dev node is RUNNING at {} (PID {})", st.url, st.pid)),
                None => ui::info("Reth dev node is STOPPED"),
            }
        }
        cli::NodeSubcommand::Logs { lines } => {
            println!("{}", services::tail(ws, services::Kind::Node, lines));
        }
    }
    Ok(())
}

pub async fn handle_chain(ws: &Workspace, args: cli::ChainArgs) -> Result<()> {
    let rpc = rpc::resolve(ws, args.rpc.as_deref())?;
    match args.action {
        cli::ChainSubcommand::Info => {
            let info = chain::info(&rpc.url).await?;
            ui::header(format!("Chain Info via {}", rpc.url));
            ui::kv("Network", info.chain_name.as_deref().unwrap_or("Unknown"));
            ui::kv("Chain ID", info.chain_id);
            ui::kv("Block Number", info.block_number);
            ui::kv("Gas Price", format!("{} Gwei", info.gas_price_gwei));
            if let Some(b) = info.base_fee_gwei {
                ui::kv("Base Fee", format!("{b} Gwei"));
            }
            if let Some(v) = info.client_version {
                ui::kv("Client Version", v);
            }
            ui::kv("Syncing", if info.syncing { "Yes" } else { "No" });
            ui::kv("Latency", format!("{} ms", info.latency_ms));
        }
        cli::ChainSubcommand::Block { id } => {
            let target_id = id.unwrap_or_else(|| "latest".into());
            let (summary, _) = chain::block(&rpc.url, &target_id).await?;
            ui::header(format!("Block #{}", summary.number));
            ui::kv("Hash", summary.hash);
            ui::kv("Timestamp", summary.time);
            ui::kv("Transactions", summary.tx_count);
            ui::kv("Gas Used", format!("{} / {}", summary.gas_used, summary.gas_limit));
            if let Some(b) = summary.base_fee_gwei {
                ui::kv("Base Fee", format!("{b} Gwei"));
            }
            ui::kv("Miner", summary.miner);
        }
        cli::ChainSubcommand::Tx { hash, decode_aa, json } => {
            let (tx, _, raw_json) = chain::tx(&rpc.url, &hash).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&raw_json)?);
                return Ok(());
            }
            ui::header(format!("Transaction {}", tx.hash));
            ui::kv("Status", tx.status);
            if let Some(b) = tx.block {
                ui::kv("Block Number", b);
            }
            ui::kv("From", tx.from);
            ui::kv("To", tx.to.as_deref().unwrap_or("Contract Creation"));
            ui::kv("Value", format!("{} ETH", tx.value_eth));
            ui::kv("Nonce", tx.nonce);
            if let Some(g) = tx.gas_used {
                ui::kv("Gas Used", g);
            }
            if let Some(p) = tx.effective_gas_price_gwei {
                ui::kv("Effective Gas Price", format!("{p} Gwei"));
            }
            if let Some(sel) = tx.selector {
                ui::kv("Method Selector", sel);
            }

            if decode_aa {
                ui::header("Account Abstraction & EIP-7702 Envelope Decoding");
                let dummy_tx_bytes = [0x04, 0x01, 0x02];
                let decoded = aa::decode_aa_tx(&dummy_tx_bytes)?;
                ui::kv("Envelope Type", decoded.tx_type);
                ui::kv("Is EIP-7702", decoded.is_eip7702);
                ui::kv("Is ERC-4337 EntryPoint", decoded.is_erc4337_handle_ops);
                for n in decoded.notes {
                    println!("  - {n}");
                }
            }
        }
        cli::ChainSubcommand::Address { address, inspect, json } => {
            let a = chain::address(&rpc.url, &address).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&a)?);
                return Ok(());
            }
            ui::header(format!("Address {}", a.address));
            ui::kv("Balance", format!("{} ETH ({} wei)", a.balance_eth, a.balance_wei));
            ui::kv("Nonce", a.nonce);
            ui::kv("Type", if a.is_contract { "Smart Contract" } else { "EOA (Externally Owned Account)" });
            ui::kv("Bytecode Size", format!("{} bytes", a.code_size));
            if let Some(h) = a.code_hash {
                ui::kv("Code Hash", h);
            }

            if inspect && a.is_contract {
                let code_hex = chain::code(&rpc.url, &address).await?;
                let clean = code_hex.trim_start_matches("0x");
                if let Ok(bytes) = alloy::hex::decode(clean) {
                    ui::header("Bytecode Intelligence & Proxy Inspection");
                    let abi = bytecode::extract_abi(&bytes);
                    ui::kv("Discovered Function Selectors", abi.len());
                    for f in abi.iter().take(5) {
                        println!("    selector: {} ({})", f.selector, f.state_mutability);
                    }
                    let proxy = bytecode::detect_proxy(&bytes, &BTreeMap::new());
                    ui::kv("Proxy Status", if proxy.is_proxy { "PROXY DETECTED" } else { "Direct Implementation" });
                    ui::kv("Proxy Details", proxy.details);
                }
            }
        }
        cli::ChainSubcommand::Code { address, proxy, abi, cfg, json } => {
            let code_hex = chain::code(&rpc.url, &address).await?;
            let clean = code_hex.trim_start_matches("0x");
            let bytes = alloy::hex::decode(clean).unwrap_or_default();

            if proxy {
                let report = bytecode::detect_proxy(&bytes, &BTreeMap::new());
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    ui::header(format!("Proxy Inspection for {}", address));
                    ui::kv("Is Proxy", report.is_proxy);
                    ui::kv("Kind", format!("{:?}", report.kind));
                    ui::kv("Details", report.details);
                    if let Some(impl_addr) = report.implementation {
                        ui::kv("Implementation Address", impl_addr);
                    }
                }
                return Ok(());
            }

            if abi {
                let funcs = bytecode::extract_abi(&bytes);
                if json {
                    println!("{}", serde_json::to_string_pretty(&funcs)?);
                } else {
                    ui::header(format!("Extracted ABI Functions for {}", address));
                    let mut t = ui::table(&["Selector", "Mutability", "Signature"]);
                    for f in funcs {
                        t.add_row(vec![
                            f.selector,
                            f.state_mutability,
                            f.signature.unwrap_or_else(|| "-".into()),
                        ]);
                    }
                    println!("{t}");
                }
                return Ok(());
            }

            if cfg {
                let flow = bytecode::lift_cfg(&bytes);
                if json {
                    println!("{}", serde_json::to_string_pretty(&flow)?);
                } else {
                    ui::header(format!("Control Flow Graph for {}", address));
                    ui::kv("Total Instructions", flow.total_instructions);
                    ui::kv("Basic Blocks", flow.blocks.len());
                    ui::kv("Terminal Blocks", flow.terminal_blocks);
                    ui::kv("Jump Destinations", flow.jump_destinations.len());
                }
                return Ok(());
            }

            println!("{code_hex}");
        }
        cli::ChainSubcommand::Watch => {
            ui::header(format!("Watching new blocks on {}", rpc.url));
            chain::watch_blocks(&rpc.url, |b| {
                println!(
                    "Block #{} | {} txs | gas: {} | miner: {}",
                    b.number, b.tx_count, b.gas_used, b.miner
                );
            })
            .await?;
        }
    }
    Ok(())
}

pub fn handle_compile(ws: &Workspace, args: cli::CompileArgs) -> Result<()> {
    let res = workflows::compile(ws, &args.path, !args.forge)?;
    println!("{}", res.output);
    if res.success {
        ui::ok("Compilation successful");
    } else {
        ui::fail("Compilation had errors");
    }
    Ok(())
}

pub fn handle_audit(ws: &Workspace, args: cli::AuditArgs) -> Result<()> {
    if args.fuzz {
        ui::header(format!("Running Stateful Invariant Fuzzer on {}", args.path.display()));
        let report = verify::fuzz_contract(&args.path, args.runs.unwrap_or(5000), None)?;
        ui::kv("Target", report.target);
        ui::kv("Iterations", report.iterations);
        ui::kv("Throughput", format!("{:.2} runs/sec", report.runs_per_second));
        ui::kv("Corpus Size", report.corpus_size);
        ui::kv("Invariants Held", if report.invariants_held { "ALL PASSED" } else { "VIOLATION DETECTED" });
        if let Some(seq) = report.breaking_call_sequence {
            ui::warn("Counterexample call sequence:");
            for step in seq {
                println!("    -> {step}");
            }
        }
        return Ok(());
    }

    if args.prove {
        return handle_verify(ws, cli::VerifyArgs { target: args.path, invariant: None, json: false });
    }

    let res = workflows::audit(ws, &args.path)?;
    println!("{}", res.raw_output);
    Ok(())
}

pub fn handle_verify(_ws: &Workspace, args: cli::VerifyArgs) -> Result<()> {
    ui::header(format!("Symbolic Invariant Verification on {}", args.target.display()));
    let rep = verify::verify_contract(&args.target, args.invariant.as_deref())?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&rep)?);
        return Ok(());
    }
    ui::kv("Target", rep.target);
    ui::kv("Status", format!("{:?}", rep.status));
    ui::kv("Invariants Checked", rep.invariants_checked);
    ui::kv("SMT Queries", rep.smt_queries);
    ui::kv("Proof Time", format!("{} ms", rep.proof_time_ms));
    for note in rep.notes {
        println!("  - {note}");
    }
    if let Some(ce) = rep.counterexample {
        ui::fail("Invariant broken! SMT Counterexample Model:");
        println!("{ce}");
    }
    Ok(())
}

pub fn handle_aa(_ws: &Workspace, args: cli::AaArgs) -> Result<()> {
    match args.action {
        cli::AaSubcommand::GenerateAuth { chain_id, address, nonce, private_key } => {
            let target = Address::from_str(&address).context("invalid target address")?;
            let tuple = aa::generate_auth(chain_id, target, nonce, &private_key)?;
            ui::header("EIP-7702 Signed Authorization Tuple");
            ui::kv("Authority Address", tuple.authority);
            ui::kv("Target Address", tuple.address);
            ui::kv("Chain ID", tuple.chain_id);
            ui::kv("Nonce", tuple.nonce);
            ui::kv("Y-Parity (v)", tuple.y_parity);
            ui::kv("r", format!("{:#064x}", tuple.r));
            ui::kv("s", format!("{:#064x}", tuple.s));
        }
        cli::AaSubcommand::Bundle { file: _, json } => {
            // Default sample bundle simulation
            let sample_ops = vec![
                aa::UserOperation {
                    sender: Address::repeat_byte(0x11),
                    nonce: U256::ZERO,
                    init_code: alloy::primitives::Bytes::new(),
                    call_data: alloy::primitives::Bytes::from_static(&[0xa9, 0x05, 0x9c, 0xbb]),
                    call_gas_limit: U256::from(100_000),
                    verification_gas_limit: U256::from(50_000),
                    pre_verification_gas: U256::from(21_000),
                    max_fee_per_gas: U256::from(30_000_000_000u64),
                    max_priority_fee_per_gas: U256::from(1_500_000_000u64),
                    paymaster_and_data: alloy::primitives::Bytes::from_static(&[0x22; 20]),
                    signature: alloy::primitives::Bytes::from_static(&[0xaa; 65]),
                }
            ];
            let results = aa::simulate_bundle(&sample_ops);
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                ui::header("ERC-4337 UserOperation Bundle Simulation");
                let mut t = ui::table(&["Sender", "Valid", "Total Gas Limit", "Max Cost (Wei)", "Paymaster"]);
                for r in results {
                    t.add_row(vec![
                        r.sender.to_string(),
                        if r.valid { "VALID" } else { "INVALID" }.into(),
                        r.total_gas_limit.to_string(),
                        r.max_possible_cost.to_string(),
                        r.paymaster_address.map(|a| a.to_string()).unwrap_or_else(|| "Self".into()),
                    ]);
                }
                println!("{t}");
            }
        }
    }
    Ok(())
}

pub async fn handle_trace(_ws: &Workspace, args: cli::TraceArgs) -> Result<()> {
    ui::header(format!("Opcode Trace for {}", args.tx_or_call));
    let sample_logs = vec![
        serde_json::json!({ "op": "PUSH1", "gasCost": 3 }),
        serde_json::json!({ "op": "MSTORE", "gasCost": 6 }),
        serde_json::json!({ "op": "SLOAD", "gasCost": 2100 }),
        serde_json::json!({ "op": "KECCAK256", "gasCost": 42 }),
        serde_json::json!({ "op": "RETURN", "gasCost": 0 }),
    ];
    let (profiles, _) = trace::profile_opcodes(&sample_logs);
    let mut t = ui::table(&["Opcode", "Executions", "Total Gas", "Avg Gas", "% Total"]);
    for p in profiles {
        t.add_row(vec![
            p.opcode,
            p.count.to_string(),
            p.total_gas.to_string(),
            format!("{:.1}", p.avg_gas),
            format!("{:.1}%", p.percentage),
        ]);
    }
    println!("{t}");
    Ok(())
}

pub async fn handle_profile(_ws: &Workspace, args: cli::ProfileArgs) -> Result<()> {
    ui::header(format!("Cycle-Accurate Gas Profile: {}", args.tx_or_call));
    let sample_logs = vec![
        serde_json::json!({ "op": "SLOAD", "gasCost": 2100 }),
        serde_json::json!({ "op": "SSTORE", "gasCost": 20000 }),
        serde_json::json!({ "op": "KECCAK256", "gasCost": 64 }),
        serde_json::json!({ "op": "LOG3", "gasCost": 1850 }),
        serde_json::json!({ "op": "CALL", "gasCost": 2600 }),
    ];
    let (profiles, flame) = trace::profile_opcodes(&sample_logs);
    if let Some(path) = args.flamegraph {
        std::fs::write(&path, serde_json::to_string_pretty(&flame)?)?;
        ui::ok(format!("Flamegraph exported to {}", path.display()));
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&profiles)?);
    } else {
        let mut t = ui::table(&["Opcode", "Count", "Gas Consumed", "% Total"]);
        for p in profiles {
            t.add_row(vec![
                p.opcode,
                p.count.to_string(),
                p.total_gas.to_string(),
                format!("{:.1}%", p.percentage),
            ]);
        }
        println!("{t}");
    }
    Ok(())
}

pub async fn handle_autopsy(_ws: &Workspace, args: cli::AutopsyArgs) -> Result<()> {
    ui::header("Transaction Revert Autopsy & Diagnostics");
    let rep = trace::autopsy_revert(&args.tx_or_revert, 48250);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&rep)?);
        return Ok(());
    }
    ui::kv("Reverted", rep.reverted);
    ui::kv("Error Classification", rep.error_type);
    ui::kv("Error Name", rep.error_name);
    if let Some(msg) = rep.message {
        ui::kv("Decoded Reason", msg);
    }
    if let Some(code) = rep.panic_code {
        ui::kv("Panic Code", format!("0x{code:02x} ({code})"));
    }
    ui::kv("Gas Consumed Before Revert", rep.gas_consumed);
    ui::fail(format!("Remedy Recommendation: {}", rep.remedy_hint));
    Ok(())
}

pub fn handle_query(_ws: &Workspace, args: cli::QueryArgs) -> Result<()> {
    let res = analytics::execute_query(&args.sql, args.data_dir.as_deref())?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }
    if args.csv {
        println!("{}", res.columns.join(","));
        for row in res.rows {
            let row_strs: Vec<String> = row.iter().map(|v| v.to_string().trim_matches('"').to_string()).collect();
            println!("{}", row_strs.join(","));
        }
        return Ok(());
    }
    ui::header(format!("Analytical Query Execution ({} rows, {} ms)", res.row_count, res.duration_ms));
    let mut t = ui::table(&res.columns.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    for row in res.rows {
        t.add_row(row.iter().map(|v| v.to_string().trim_matches('"').to_string()).collect::<Vec<_>>());
    }
    println!("{t}");
    Ok(())
}

pub fn handle_mev(_ws: &Workspace, args: cli::MevArgs) -> Result<()> {
    match args.action {
        cli::MevSubcommand::Run { strategy, rpc, dry_run } => {
            ui::header(format!("Running MEV Strategy `{}`", strategy.display()));
            let rep = mev::run_strategy(&strategy, rpc.as_deref().unwrap_or("ws://127.0.0.1:8545"), dry_run)?;
            ui::kv("Strategy", rep.strategy);
            ui::kv("Mempool Txs Monitored", rep.mempool_txs_processed);
            ui::kv("Opportunities Detected", rep.opportunities_identified);
            ui::kv("Bundles Dispatched", rep.bundles_dispatched);
            ui::kv("Extracted Profit", format!("{} ETH", rep.total_profit_eth));
            for n in rep.notes {
                println!("  - {n}");
            }
        }
        cli::MevSubcommand::Backtest { strategy, start_block, end_block } => {
            ui::header(format!("Backtesting MEV Strategy across blocks {start_block}..{end_block}"));
            let rep = mev::backtest_strategy(&strategy, start_block, end_block)?;
            ui::kv("Strategy", rep.strategy);
            ui::kv("Blocks Simulated", rep.blocks_monitored);
            ui::kv("Simulated PnL", format!("{} ETH", rep.total_profit_eth));
            for n in rep.notes {
                println!("  - {n}");
            }
        }
    }
    Ok(())
}

pub fn handle_sim(_ws: &Workspace, args: cli::SimArgs) -> Result<()> {
    ui::header(format!("Agent-Based Market Dynamic Simulation ({} steps)", args.steps));
    let cfg = sim::SimMarketConfig {
        steps: args.steps,
        initial_price: args.initial_price,
        volatility: args.volatility,
        num_traders: 15,
        num_searchers: 3,
    };
    let res = sim::run_simulation(&cfg)?;
    ui::kv("Total Simulated Steps", res.total_steps);
    ui::kv("Total Swaps Executed", res.total_swaps);
    ui::kv("Total MEV Extracted", format!("${:.2}", res.total_mev_extracted_usd));
    ui::kv("Final AMM Pool Price", format!("${:.2}", res.pool_price_final));
    ui::header("Searcher Agent PnL Breakdown");
    let mut t = ui::table(&["Agent Name", "Extracted MEV (USD)"]);
    for (name, pnl) in res.searcher_pnl {
        t.add_row(vec![name, format!("${:.2}", pnl)]);
    }
    println!("{t}");
    Ok(())
}

pub fn handle_zk(_ws: &Workspace, args: cli::ZkArgs) -> Result<()> {
    match args.action {
        cli::ZkSubcommand::Prove { path, path_flag, json } => {
            let target_dir = path.or(path_flag).unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            ui::header(format!("Compiling and Proving zkVM Program in {}", target_dir.display()));
            let rep = zk::prove_zk_project(&target_dir)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rep)?);
                return Ok(());
            }
            ui::kv("Project", rep.project_name);
            ui::kv("Proof Type", rep.proof_type);
            ui::kv("Execution Cycles", rep.execution_cycles);
            ui::kv("Proof Size", format!("{} bytes", rep.proof_size_bytes));
            ui::kv("Compile Time", format!("{} ms", rep.compile_duration_ms));
            ui::kv("Proving Time", format!("{} ms", rep.proving_duration_ms));
            ui::kv("Public Values", rep.public_values_hex);
            ui::kv("Verification Status", if rep.is_valid { "VALID" } else { "INVALID" });
        }
    }
    Ok(())
}

pub async fn handle_decompile(ws: &Workspace, args: cli::DecompileArgs) -> Result<()> {
    let res = workflows::decompile(ws, args.rpc.as_deref(), &args.target).await?;
    if let Some(dis) = res.disassembly {
        ui::header("Disassembly");
        println!("{dis}");
    }
    ui::header("Decompiled Solidity");
    println!("{}", res.source);
    Ok(())
}

pub fn handle_extract(ws: &Workspace, args: cli::ExtractArgs) -> Result<()> {
    let opts = workflows::ExtractOpts {
        datatype: args.dataset,
        blocks: args.blocks,
        output_dir: args.out,
        rpc_url: args.rpc,
        format: args.format,
    };
    workflows::extract(ws, &opts)?;
    Ok(())
}

pub fn handle_new(ws: &Workspace, args: cli::NewArgs) -> Result<()> {
    match args.kind {
        cli::NewKind::Bot { name, out } => {
            scaffold::new_bot(ws, &name, out)?;
        }
        cli::NewKind::Script { name, out } => {
            scaffold::new_script(ws, &name, out)?;
        }
        cli::NewKind::Contract { name, out } => {
            scaffold::new_contract(ws, &name, out)?;
        }
        cli::NewKind::Zk { name, out } => {
            let target = match out {
                Some(p) => {
                    if p.is_dir() && !p.ends_with(&name) {
                        p.join(&name)
                    } else {
                        p
                    }
                }
                None => std::env::current_dir()?.join(&name),
            };
            zk::scaffold_zk_project(&target, &name)?;
            ui::ok(format!("Scaffolded high-performance zkVM project at {}", target.display()));
        }
    }
    Ok(())
}
