//! Long-running local services managed by para: an anvil devnet (optionally forking any chain)
//! and a reth `--dev` node. Processes are detached into their own process group, tracked with a
//! JSON state file under `.para/run`, and log to `.para/logs`.

use crate::tools;
use crate::workspace::{Workspace, now};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// anvil devnet (foundry)
    Devnet,
    /// reth --dev node
    Node,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Devnet => "devnet",
            Kind::Node => "node",
        }
    }
    pub fn alias(self) -> &'static str {
        match self {
            Kind::Devnet => "local",
            Kind::Node => "node",
        }
    }
    pub fn bin(self) -> &'static str {
        match self {
            Kind::Devnet => "anvil",
            Kind::Node => "reth",
        }
    }
    pub fn default_port(self) -> u16 {
        match self {
            Kind::Devnet => 8545,
            Kind::Node => 8645,
        }
    }
    pub fn default_url(self) -> String {
        format!("http://127.0.0.1:{}", self.default_port())
    }
    #[allow(dead_code)]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "devnet" | "anvil" | "local" => Some(Kind::Devnet),
            "node" | "reth" => Some(Kind::Node),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub kind: Kind,
    pub pid: u32,
    pub port: u16,
    pub url: String,
    pub cmd: Vec<String>,
    pub started_at: String,
    pub log: String,
    #[serde(default)]
    pub fork: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevnetOpts {
    pub port: Option<u16>,
    /// RPC name/url to fork (resolved through `rpc::lookup`).
    pub fork: Option<String>,
    pub fork_block: Option<u64>,
    pub chain_id: Option<u64>,
    pub block_time: Option<u64>,
    pub accounts: Option<u32>,
    pub balance: Option<u64>,
    /// Persist chain state to `.para/data/devnet-state.json` across restarts.
    #[serde(default)]
    pub persist: bool,
    #[serde(default)]
    pub extra: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeOpts {
    pub port: Option<u16>,
    /// Seconds between dev blocks (default: mine on transaction).
    pub block_time: Option<u64>,
    /// Wipe the dev datadir before starting.
    #[serde(default)]
    pub fresh: bool,
    #[serde(default)]
    pub extra: Vec<String>,
}

fn state_file(ws: &Workspace, kind: Kind) -> PathBuf {
    ws.run_dir().join(format!("{}.json", kind.name()))
}

pub fn log_file(ws: &Workspace, kind: Kind) -> PathBuf {
    ws.logs_dir().join(format!("{}.log", kind.name()))
}

pub fn alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Current state if the service is running (cleans up stale state files).
pub fn status(ws: &Workspace, kind: Kind) -> Option<ServiceState> {
    let path = state_file(ws, kind);
    let st: ServiceState = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
    if alive(st.pid) {
        Some(st)
    } else {
        let _ = std::fs::remove_file(path);
        None
    }
}

pub fn start_devnet(ws: &Workspace, o: &DevnetOpts) -> Result<ServiceState> {
    let port = o.port.unwrap_or(Kind::Devnet.default_port());
    let mut args: Vec<String> = vec!["--host".into(), "127.0.0.1".into(), "--port".into(), port.to_string()];
    let mut fork_label = None;
    if let Some(f) = o.fork.as_deref().filter(|s| !s.is_empty()) {
        let r = crate::rpc::lookup(ws, f)?;
        args.extend(["--fork-url".into(), r.url.clone()]);
        fork_label = Some(format!("{f} ({})", r.url));
        if let Some(b) = o.fork_block {
            args.extend(["--fork-block-number".into(), b.to_string()]);
        }
    }
    if let Some(c) = o.chain_id {
        args.extend(["--chain-id".into(), c.to_string()]);
    }
    if let Some(t) = o.block_time.filter(|t| *t > 0) {
        args.extend(["--block-time".into(), t.to_string()]);
    }
    if let Some(a) = o.accounts {
        args.extend(["--accounts".into(), a.to_string()]);
    }
    if let Some(b) = o.balance {
        args.extend(["--balance".into(), b.to_string()]);
    }
    if o.persist {
        let p = ws.data_dir().join("devnet-state.json");
        args.extend(["--state".into(), p.display().to_string()]);
    }
    args.extend(o.extra.iter().cloned());
    spawn(ws, Kind::Devnet, port, args, fork_label)
}

pub fn start_node(ws: &Workspace, o: &NodeOpts) -> Result<ServiceState> {
    let port = o.port.unwrap_or(Kind::Node.default_port());
    let datadir = ws.data_dir().join("reth-dev");
    if o.fresh && datadir.exists() {
        std::fs::remove_dir_all(&datadir)?;
    }
    let mut args: Vec<String> = [
        "node", "--dev", "--datadir", &datadir.display().to_string(),
        "--http", "--http.addr", "127.0.0.1", "--http.port", &port.to_string(), "--http.api", "all",
        "--ws", "--ws.addr", "127.0.0.1", "--ws.port", &(port + 1).to_string(),
        "--authrpc.port", &(port + 6).to_string(),
        "--port", "30313", "--disable-discovery",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    if let Some(t) = o.block_time.filter(|t| *t > 0) {
        args.extend(["--dev.block-time".into(), format!("{t}s")]);
    }
    args.extend(o.extra.iter().cloned());
    spawn(ws, Kind::Node, port, args, None)
}

fn spawn(ws: &Workspace, kind: Kind, port: u16, args: Vec<String>, fork: Option<String>) -> Result<ServiceState> {
    if let Some(s) = status(ws, kind) {
        bail!("{} already running (pid {}, {}); stop it first", kind.name(), s.pid, s.url);
    }
    if std::net::TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_millis(200)).is_ok() {
        bail!("port {port} is already in use");
    }
    let bin = tools::require(ws, kind.bin())?;
    let log_path = log_file(ws, kind);
    let log = std::fs::File::create(&log_path).with_context(|| format!("creating {}", log_path.display()))?;
    let mut cmd = Command::new(&bin);
    cmd.args(&args).stdin(Stdio::null()).stdout(log.try_clone()?).stderr(log);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().with_context(|| format!("spawning {}", bin.display()))?;
    let st = ServiceState {
        kind,
        pid: child.id(),
        port,
        url: format!("http://127.0.0.1:{port}"),
        cmd: std::iter::once(bin.display().to_string()).chain(args).collect(),
        started_at: now(),
        log: log_path.display().to_string(),
        fork,
    };
    std::fs::write(state_file(ws, kind), serde_json::to_string_pretty(&st)?)?;

    // Wait for the RPC port to accept connections.
    let deadline = Instant::now() + Duration::from_secs(if kind == Kind::Node { 60 } else { 30 });
    loop {
        if let Ok(Some(code)) = child.try_wait() {
            let _ = std::fs::remove_file(state_file(ws, kind));
            bail!("{} exited early ({code}). Last log lines:\n{}", kind.name(), tail(ws, kind, 15));
        }
        if std::net::TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_millis(200)).is_ok() {
            break;
        }
        if Instant::now() > deadline {
            bail!("{} did not open port {port} in time; see {}", kind.name(), st.log);
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    // Detach: the child keeps running after a CLI invocation exits. In the long-lived GUI server we
    // still reap it when it dies so `kill -0` doesn't report a zombie as alive.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(st)
}

pub fn stop(ws: &Workspace, kind: Kind) -> Result<bool> {
    let Some(st) = status(ws, kind) else { return Ok(false) };
    let _ = Command::new("kill").args(["-TERM", &st.pid.to_string()]).status();
    let deadline = Instant::now() + Duration::from_secs(15);
    while alive(st.pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(200));
    }
    if alive(st.pid) {
        let _ = Command::new("kill").args(["-KILL", &st.pid.to_string()]).status();
    }
    let _ = std::fs::remove_file(state_file(ws, kind));
    Ok(true)
}

pub fn tail(ws: &Workspace, kind: Kind, n: usize) -> String {
    let text = std::fs::read_to_string(log_file(ws, kind)).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

#[derive(Debug, Clone, Serialize)]
pub struct DevAccount {
    pub index: usize,
    pub address: String,
    pub private_key: Option<String>,
}

/// Parse anvil's startup banner for the dev accounts and their (well-known, test-only) keys.
pub fn devnet_accounts(ws: &Workspace) -> Vec<DevAccount> {
    let text = std::fs::read_to_string(log_file(ws, Kind::Devnet)).unwrap_or_default();
    let mut accounts: Vec<DevAccount> = vec![];
    let mut section = "";
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("Available Accounts") {
            section = "acc";
            continue;
        }
        if l.starts_with("Private Keys") {
            section = "key";
            continue;
        }
        if l.starts_with("Wallet") || l.starts_with("Chain ID") || l.starts_with("Listening") {
            section = "";
        }
        let Some(rest) = l.strip_prefix('(') else { continue };
        let Some((idx, val)) = rest.split_once(')') else { continue };
        let Ok(idx) = idx.trim().parse::<usize>() else { continue };
        let val = val.split_whitespace().next().unwrap_or_default().to_string();
        match section {
            "acc" => accounts.push(DevAccount { index: idx, address: val, private_key: None }),
            "key" => {
                if let Some(a) = accounts.iter_mut().find(|a| a.index == idx) {
                    a.private_key = Some(val);
                }
            }
            _ => {}
        }
    }
    accounts
}
