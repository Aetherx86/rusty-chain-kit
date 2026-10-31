//! Rusty Chain Kit (`rusty-chain-kit` / `para`)
//!
//! An all-in-one developer command center (CLI + GUI) uniting the Paradigm
//! and Ethereum Rust/TS ecosystem: Foundry, Reth, Alloy, Solar, Cryo, MESC,
//! Heimdall, Revm, Aderyn, Artemis, and Viem.

pub mod builder;
pub mod chain;
pub mod cli;
pub mod git;
pub mod rpc;
pub mod scaffold;
pub mod server;
pub mod services;
pub mod tools;
pub mod ui;
pub mod workflows;
pub mod workspace;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
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
                    st.dirty.to_string(),
                    format!("{} ({})", st.head, st.head_age),
                ]);
            }
            println!("{t}");
        }
        cli::ReposSubcommand::Sync { repos, strategy, no_stash, dry_run, push } => {
            let targets = ws.select(&repos)?;
            let opts = git::SyncOpts { strategy, no_stash, dry_run, push };
            ui::header(format!("Syncing {} repo(s) with canonical upstreams", targets.len()));
            for r in targets {
                let rep = git::sync(ws, r, &opts);
                match rep.outcome {
                    git::Outcome::UpToDate => ui::ok(format!("{}: up to date ({})", r.name, rep.to)),
                    git::Outcome::Updated => ui::ok(format!(
                        "{}: updated {} -> {} (+{} commits)",
                        r.name, rep.from, rep.to, rep.new_commits
                    )),
                    git::Outcome::WouldUpdate => ui::info(format!("{}: would update ({:?})", r.name, rep.notes)),
                    git::Outcome::Cloned => ui::ok(format!("{}: cloned at {}", r.name, rep.to)),
                    git::Outcome::Conflict => ui::fail(format!("{}: conflict! {:?}", r.name, rep.notes)),
                    git::Outcome::Diverged => ui::warn(format!("{}: diverged ({:?})", r.name, rep.notes)),
                    git::Outcome::Error => ui::fail(format!("{}: failed ({:?})", r.name, rep.notes)),
                }
            }
        }
        cli::ReposSubcommand::Fetch { repos } => {
            let targets = ws.select(&repos)?;
            for r in targets {
                let dir = ws.repo_dir(r);
                if dir.join(".git").exists() {
                    git::fetch(&dir, r)?;
                    ui::ok(format!("{}: fetched from {}", r.name, r.upstream));
                }
            }
        }
    }
    Ok(())
}

pub fn handle_build(ws: &Workspace, args: cli::BuildArgs) -> Result<()> {
    let opts = builder::BuildOpts {
        jobs: args.jobs,
        profile: args.profile,
        if_stale: args.stale,
    };
    if let Some(name) = args.repo {
        let repo = ws.repo(&name)?;
        builder::build(ws, repo, &opts)?;
    } else {
        ui::header("Building all workspace repositories");
        for repo in &ws.manifest.repos {
            if repo.build.is_some() {
                let _ = builder::build(ws, repo, &opts);
            }
        }
    }
    Ok(())
}

pub fn handle_tools(ws: &Workspace, args: cli::ToolsArgs) -> Result<()> {
    match args.action.unwrap_or(cli::ToolsSubcommand::List) {
        cli::ToolsSubcommand::List => {
            ui::header("Tools & Executables (.para/bin & PATH)");
            let mut t = ui::table(&["Binary", "Providing Repo", "Source", "Version", "Path"]);
            for tool in tools::list(ws, true) {
                t.add_row(vec![
                    tool.bin,
                    tool.repo,
                    tool.source,
                    tool.version.unwrap_or_else(|| "-".into()),
                    tool.path.unwrap_or_else(|| "Not found".into()),
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
        cli::ChainSubcommand::Tx { hash } => {
            let (tx, _, _) = chain::tx(&rpc.url, &hash).await?;
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
        }
        cli::ChainSubcommand::Address { address } => {
            let a = chain::address(&rpc.url, &address).await?;
            ui::header(format!("Address {}", a.address));
            ui::kv("Balance", format!("{} ETH ({} wei)", a.balance_eth, a.balance_wei));
            ui::kv("Nonce", a.nonce);
            ui::kv("Type", if a.is_contract { "Smart Contract" } else { "EOA (Externally Owned Account)" });
            ui::kv("Bytecode Size", format!("{} bytes", a.code_size));
            if let Some(h) = a.code_hash {
                ui::kv("Code Hash", h);
            }
        }
        cli::ChainSubcommand::Code { address } => {
            let c = chain::code(&rpc.url, &address).await?;
            println!("{c}");
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
    let res = workflows::audit(ws, &args.path)?;
    println!("{}", res.raw_output);
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
    }
    Ok(())
}
