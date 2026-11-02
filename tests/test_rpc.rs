use rusty_chain_kit::rpc::{Endpoint, Resolved, add, list, lookup, remove, resolve, set_default};
use rusty_chain_kit::workspace::Workspace;

#[test]
fn test_rpc_list_not_empty() {
    let ws = Workspace::discover().unwrap();
    let endpoints = list(&ws);
    assert!(!endpoints.is_empty());
}

#[test]
fn test_rpc_list_contains_builtin_sources() {
    let ws = Workspace::discover().unwrap();
    let endpoints = list(&ws);
    assert!(
        endpoints
            .iter()
            .any(|e| e.source == "builtin" || e.source == "para" || e.source == "mesc")
    );
}

#[test]
fn test_rpc_lookup_direct_http() {
    let ws = Workspace::discover().unwrap();
    let res = lookup(&ws, "https://eth.llamarpc.com").unwrap();
    assert_eq!(res.url, "https://eth.llamarpc.com");
    assert_eq!(res.via, "url");
}

#[test]
fn test_rpc_lookup_direct_ws() {
    let ws = Workspace::discover().unwrap();
    let res = lookup(&ws, "ws://127.0.0.1:8545").unwrap();
    assert_eq!(res.url, "ws://127.0.0.1:8545");
    assert_eq!(res.via, "url");
}

#[test]
fn test_rpc_lookup_direct_wss() {
    let ws = Workspace::discover().unwrap();
    let res = lookup(&ws, "wss://eth-mainnet.ws.alchemyapi.io").unwrap();
    assert_eq!(res.url, "wss://eth-mainnet.ws.alchemyapi.io");
    assert_eq!(res.via, "url");
}

#[test]
fn test_rpc_lookup_direct_ipc() {
    let ws = Workspace::discover().unwrap();
    let res = lookup(&ws, "/tmp/anvil.ipc").unwrap();
    assert_eq!(res.url, "/tmp/anvil.ipc");
    assert_eq!(res.via, "url");
}

#[test]
fn test_rpc_lookup_builtin_local() {
    let ws = Workspace::discover().unwrap();
    let res = lookup(&ws, "local").unwrap();
    assert!(res.url.contains("8545"));
    assert_eq!(res.via, "builtin:local");
}

#[test]
fn test_rpc_lookup_builtin_node() {
    let ws = Workspace::discover().unwrap();
    let res = lookup(&ws, "node").unwrap();
    assert!(res.url.contains("8545") || res.url.contains("8645"));
    assert_eq!(res.via, "builtin:node");
}

fn isolated_ws() -> (tempfile::TempDir, Workspace) {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("para.toml");
    std::fs::write(&p, "[workspace]\nroot = \".\"\nstate_dir = \".para\"\n").unwrap();
    let ws = Workspace::load(&p).unwrap();
    (tmp, ws)
}

#[test]
fn test_rpc_add_lookup_and_remove() {
    let (_tmp, ws) = isolated_ws();
    let alias = "testnet-temp-suite";
    let target = "http://127.0.0.1:7777";

    // Add
    add(&ws, alias, target).unwrap();

    // Verify lookup
    let res = lookup(&ws, alias).unwrap();
    assert_eq!(res.url, target);
    assert_eq!(res.via, format!("para:{alias}"));

    // Remove
    remove(&ws, alias).unwrap();
    let removed = ws.load_config().rpc.endpoints.contains_key(alias);
    assert!(!removed);
}

#[test]
fn test_rpc_set_and_clear_default() {
    let (_tmp, ws) = isolated_ws();

    // Set default to mainnet URL
    let target = "https://rpc.ankr.com/eth";
    set_default(&ws, Some(target.into())).unwrap();
    assert_eq!(ws.load_config().rpc.default, Some(target.into()));

    // Clear default
    set_default(&ws, None).unwrap();
    assert!(ws.load_config().rpc.default.is_none());
}

#[test]
fn test_rpc_resolve_explicit() {
    let ws = Workspace::discover().unwrap();
    let res = resolve(&ws, Some("https://arbitrum.llamarpc.com")).unwrap();
    assert_eq!(res.url, "https://arbitrum.llamarpc.com");
    assert_eq!(res.via, "url");
}

#[test]
fn test_rpc_resolve_fallback() {
    let ws = Workspace::discover().unwrap();
    let res = resolve(&ws, None).unwrap();
    assert!(!res.url.is_empty());
}

#[test]
fn test_rpc_lookup_nonexistent_fails() {
    let ws = Workspace::discover().unwrap();
    let res = lookup(&ws, "this_alias_definitely_does_not_exist_9999");
    assert!(res.is_err());
}

#[test]
fn test_rpc_endpoint_struct() {
    let ep = Endpoint {
        name: "test".into(),
        url: "http://localhost:8545".into(),
        source: "custom".into(),
        chain_id: Some("1".into()),
    };
    assert_eq!(ep.name, "test");
    assert_eq!(ep.url, "http://localhost:8545");
    assert_eq!(ep.source, "custom");
    assert_eq!(ep.chain_id, Some("1".into()));
}

#[test]
fn test_rpc_resolved_struct() {
    let r = Resolved {
        url: "https://ethereum.org".into(),
        via: "custom".into(),
    };
    assert_eq!(r.url, "https://ethereum.org");
    assert_eq!(r.via, "custom");
}
