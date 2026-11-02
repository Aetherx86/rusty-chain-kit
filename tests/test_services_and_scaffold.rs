use rusty_chain_kit::scaffold::{new_bot, new_contract, new_script};
use rusty_chain_kit::services::{DevnetOpts, Kind, NodeOpts, ServiceState, alive, log_file};
use rusty_chain_kit::tools;
use rusty_chain_kit::workflows::{AuditReport, CompileResult};
use rusty_chain_kit::workspace::Workspace;

#[test]
fn test_service_kind_properties() {
    assert_eq!(Kind::Devnet.name(), "devnet");
    assert_eq!(Kind::Devnet.alias(), "local");
    assert_eq!(Kind::Devnet.bin(), "anvil");
    assert_eq!(Kind::Devnet.default_port(), 8545);
    assert_eq!(Kind::Devnet.default_url(), "http://127.0.0.1:8545");

    assert_eq!(Kind::Node.name(), "node");
    assert_eq!(Kind::Node.alias(), "node");
    assert_eq!(Kind::Node.bin(), "reth");
    assert_eq!(Kind::Node.default_port(), 8645);
    assert_eq!(Kind::Node.default_url(), "http://127.0.0.1:8645");
}

#[test]
fn test_service_kind_parse() {
    assert_eq!(Kind::parse("devnet"), Some(Kind::Devnet));
    assert_eq!(Kind::parse("anvil"), Some(Kind::Devnet));
    assert_eq!(Kind::parse("local"), Some(Kind::Devnet));
    assert_eq!(Kind::parse("node"), Some(Kind::Node));
    assert_eq!(Kind::parse("reth"), Some(Kind::Node));
    assert_eq!(Kind::parse("unknown"), None);
}

#[test]
fn test_devnet_opts_defaults() {
    let opts = DevnetOpts::default();
    assert!(opts.port.is_none());
    assert!(opts.fork.is_none());
    assert!(opts.fork_block.is_none());
    assert!(opts.chain_id.is_none());
    assert!(opts.block_time.is_none());
    assert!(!opts.persist);
    assert!(opts.extra.is_empty());
}

#[test]
fn test_node_opts_defaults() {
    let opts = NodeOpts::default();
    assert!(opts.port.is_none());
    assert!(opts.block_time.is_none());
    assert!(!opts.fresh);
    assert!(opts.extra.is_empty());
}

#[test]
fn test_service_state_serialization() {
    let st = ServiceState {
        kind: Kind::Devnet,
        pid: 12345,
        port: 8545,
        url: "http://127.0.0.1:8545".into(),
        cmd: vec!["anvil".into(), "--port".into(), "8545".into()],
        started_at: "2026-10-31T00:00:00Z".into(),
        log: "/tmp/devnet.log".into(),
        fork: Some("mainnet".into()),
    };
    let json = serde_json::to_string(&st).unwrap();
    let loaded: ServiceState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.kind, Kind::Devnet);
    assert_eq!(loaded.pid, 12345);
    assert_eq!(loaded.port, 8545);
    assert_eq!(loaded.fork, Some("mainnet".into()));
}

#[test]
fn test_service_log_file_paths() {
    let ws = Workspace::discover().unwrap();
    let dev_log = log_file(&ws, Kind::Devnet);
    let node_log = log_file(&ws, Kind::Node);
    assert!(dev_log.ends_with(".para/logs/devnet.log"));
    assert!(node_log.ends_with(".para/logs/node.log"));
}

#[test]
fn test_process_alive_detection() {
    let current_pid = std::process::id();
    assert!(alive(current_pid));
    // Extremely high non-existent PID
    assert!(!alive(999_999_999));
}

#[test]
fn test_scaffold_new_bot_creates_files() {
    let ws = Workspace::discover().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("my-test-bot");

    let created = new_bot(&ws, "my-test-bot", Some(target.clone())).unwrap();
    assert_eq!(created, target);
    assert!(target.join("Cargo.toml").is_file());
    assert!(target.join("README.md").is_file());
    assert!(target.join("src/main.rs").is_file());
    assert!(target.join("src/engine.rs").is_file());
    assert!(target.join("src/strategy.rs").is_file());

    let cargo_content = std::fs::read_to_string(target.join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains("my-test-bot"));
}

#[test]
fn test_scaffold_new_bot_rejects_existing_dir() {
    let ws = Workspace::discover().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("existing-bot");
    std::fs::create_dir_all(&target).unwrap();

    let res = new_bot(&ws, "existing-bot", Some(target));
    assert!(res.is_err());
}

#[test]
fn test_scaffold_new_script_creates_files() {
    let ws = Workspace::discover().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("my-viem-script");

    let created = new_script(&ws, "my-viem-script", Some(target.clone())).unwrap();
    assert_eq!(created, target);
    assert!(target.join("package.json").is_file());
    assert!(target.join("tsconfig.json").is_file());
    assert!(target.join("src/index.ts").is_file());

    let pkg_content = std::fs::read_to_string(target.join("package.json")).unwrap();
    assert!(pkg_content.contains("my-viem-script"));
    assert!(pkg_content.contains("viem"));
}

#[test]
fn test_scaffold_new_script_rejects_existing_dir() {
    let ws = Workspace::discover().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("existing-script");
    std::fs::create_dir_all(&target).unwrap();

    let res = new_script(&ws, "existing-script", Some(target));
    assert!(res.is_err());
}

#[test]
fn test_scaffold_new_contract_creates_files() {
    let ws = Workspace::discover().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("my-foundry-proj");

    let created = new_contract(&ws, "my-foundry-proj", Some(target.clone())).unwrap();
    assert_eq!(created, target);
    assert!(target.join("foundry.toml").is_file());
    assert!(target.join("src").is_dir());
    assert!(target.join("test").is_dir());
    assert!(target.join("script").is_dir());
}

#[test]
fn test_scaffold_new_contract_rejects_existing_dir() {
    let ws = Workspace::discover().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("existing-contract");
    std::fs::create_dir_all(&target).unwrap();

    let res = new_contract(&ws, "existing-contract", Some(target));
    assert!(res.is_err());
}

#[test]
fn test_workflow_compile_result_serialization() {
    let res = CompileResult {
        compiler: "solar".into(),
        success: true,
        output: "Compiled 1 file".into(),
        artifacts: vec!["Counter.abi".into()],
    };
    let json = serde_json::to_value(&res).unwrap();
    assert_eq!(json["compiler"], "solar");
    assert_eq!(json["success"], true);
}

#[test]
fn test_workflow_audit_report_serialization() {
    let rep = AuditReport {
        tool: "aderyn".into(),
        summary: "0 High, 1 Low".into(),
        findings_count: 1,
        raw_output: "Report details".into(),
    };
    let json = serde_json::to_value(&rep).unwrap();
    assert_eq!(json["tool"], "aderyn");
    assert_eq!(json["findings_count"], 1);
}

#[test]
fn test_tools_list_and_resolve() {
    let ws = Workspace::discover().unwrap();
    let list = tools::list(&ws, false);
    assert!(!list.is_empty());
    assert!(list.iter().any(|t| t.bin == "forge"));
    assert!(list.iter().any(|t| t.bin == "solar"));
    assert!(list.iter().any(|t| t.bin == "cryo"));
}

#[test]
fn test_tools_resolve_nonexistent() {
    let ws = Workspace::discover().unwrap();
    assert!(tools::resolve(&ws, "nonexistent_tool_binary_xyz").is_none());
}
