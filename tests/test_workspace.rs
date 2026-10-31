use rusty_chain_kit::workspace::{Config, Manifest, Strategy, Workspace};
use std::path::Path;

#[test]
fn test_workspace_discovery() {
    let ws = Workspace::discover().expect("Failed to discover workspace");
    assert!(ws.root.exists());
    assert!(!ws.manifest.repos.is_empty());
}

#[test]
fn test_manifest_repos_count_and_categories() {
    let ws = Workspace::discover().unwrap();
    let repo_names: Vec<&str> = ws.manifest.repos.iter().map(|r| r.name.as_str()).collect();
    
    // All 11 repos must be present
    assert!(repo_names.contains(&"foundry"));
    assert!(repo_names.contains(&"solar"));
    assert!(repo_names.contains(&"alloy"));
    assert!(repo_names.contains(&"cryo"));
    assert!(repo_names.contains(&"reth"));
    assert!(repo_names.contains(&"viem"));
    assert!(repo_names.contains(&"artemis"));
    assert!(repo_names.contains(&"mesc"));
    assert!(repo_names.contains(&"heimdall"));
    assert!(repo_names.contains(&"revm"));
    assert!(repo_names.contains(&"aderyn"));
    assert_eq!(ws.manifest.repos.len(), 11);
}

#[test]
fn test_manifest_repos_canonical_upstreams() {
    let ws = Workspace::discover().unwrap();
    let foundry = ws.repo("foundry").expect("foundry must exist");
    assert_eq!(foundry.upstream, "https://github.com/foundry-rs/foundry.git");
    assert_eq!(foundry.branch, "master");

    let solar = ws.repo("solar").expect("solar must exist");
    assert_eq!(solar.upstream, "https://github.com/paradigmxyz/solar.git");

    let reth = ws.repo("reth").expect("reth must exist");
    assert_eq!(reth.upstream, "https://github.com/paradigmxyz/reth.git");
}

#[test]
fn test_workspace_select_repos() {
    let ws = Workspace::discover().unwrap();
    
    // Select specific repos
    let selected = ws.select(&["foundry".into(), "solar".into()]).unwrap();
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].name, "foundry");
    assert_eq!(selected[1].name, "solar");

    // Empty list selects all repos
    let all = ws.select(&[]).unwrap();
    assert_eq!(all.len(), 11);

    // Invalid repo name returns error
    let err = ws.select(&["nonexistent_repo".into()]);
    assert!(err.is_err());
}

#[test]
fn test_workspace_bin_mappings() {
    let ws = Workspace::discover().unwrap();
    
    let forge_repo = ws.repo_for_bin("forge").expect("forge binary mapping missing");
    assert_eq!(forge_repo.name, "foundry");

    let solar_repo = ws.repo_for_bin("solar").expect("solar binary mapping missing");
    assert_eq!(solar_repo.name, "solar");

    let cryo_repo = ws.repo_for_bin("cryo").expect("cryo binary mapping missing");
    assert_eq!(cryo_repo.name, "cryo");

    let mesc_repo = ws.repo_for_bin("mesc").expect("mesc binary mapping missing");
    assert_eq!(mesc_repo.name, "mesc");

    let heimdall_repo = ws.repo_for_bin("heimdall").expect("heimdall binary mapping missing");
    assert_eq!(heimdall_repo.name, "heimdall");

    let all_bins = ws.all_bins();
    assert!(all_bins.iter().any(|(bin, repo)| bin == "forge" && repo == "foundry"));
    assert!(all_bins.iter().any(|(bin, repo)| bin == "cast" && repo == "foundry"));
    assert!(all_bins.iter().any(|(bin, repo)| bin == "anvil" && repo == "foundry"));
    assert!(all_bins.iter().any(|(bin, repo)| bin == "chisel" && repo == "foundry"));
    assert!(all_bins.iter().any(|(bin, repo)| bin == "solar" && repo == "solar"));
}

#[test]
fn test_workspace_directories() {
    let ws = Workspace::discover().unwrap();
    let bin_dir = ws.bin_dir();
    let logs_dir = ws.logs_dir();
    let run_dir = ws.run_dir();
    let data_dir = ws.data_dir();

    assert!(bin_dir.ends_with(".para/bin"));
    assert!(logs_dir.ends_with(".para/logs"));
    assert!(run_dir.ends_with(".para/run"));
    assert!(data_dir.ends_with(".para/data"));
}

#[test]
fn test_workspace_repo_directory() {
    let ws = Workspace::discover().unwrap();
    let foundry = ws.repo("foundry").unwrap();
    let dir = ws.repo_dir(foundry);
    assert_eq!(dir, ws.root.join("foundry"));
}

#[test]
fn test_manifest_parse_valid_toml() {
    let toml_str = r#"
[workspace]
root = "."
state_dir = ".para"

[[repo]]
name = "test-tool"
category = "testing"
path = "test-tool"
upstream = "https://github.com/example/test-tool.git"
branch = "main"
description = "A test tool"
"#;
    let manifest: Manifest = toml::from_str(toml_str).expect("Valid TOML must parse");
    assert_eq!(manifest.workspace.root, ".");
    assert_eq!(manifest.repos.len(), 1);
    assert_eq!(manifest.repos[0].name, "test-tool");
}

#[test]
fn test_manifest_parse_invalid_toml() {
    let malformed = "this is not valid toml [[[";
    let res: Result<Manifest, _> = toml::from_str(malformed);
    assert!(res.is_err());
}

#[test]
fn test_workspace_load_nonexistent_file() {
    let res = Workspace::load(Path::new("/nonexistent/path/para.toml"));
    assert!(res.is_err());
}

#[test]
fn test_strategy_variants() {
    assert_eq!(Strategy::Rebase.to_string(), "rebase");
    assert_eq!(Strategy::Merge.to_string(), "merge");
    assert_eq!(Strategy::Ff.to_string(), "ff");
}

#[test]
fn test_workspace_config_serialization() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.toml");

    let mut cfg = Config::default();
    cfg.rpc.default = Some("sepolia".into());
    cfg.rpc.endpoints.insert("custom_net".into(), "https://custom.rpc".into());

    let toml_str = toml::to_string_pretty(&cfg).unwrap();
    std::fs::write(&config_path, toml_str).unwrap();

    let loaded_str = std::fs::read_to_string(&config_path).unwrap();
    let loaded: Config = toml::from_str(&loaded_str).unwrap();

    assert_eq!(loaded.rpc.default, Some("sepolia".into()));
    assert_eq!(loaded.rpc.endpoints.get("custom_net").unwrap(), "https://custom.rpc");
}

#[test]
fn test_workspace_repo_lookup_unknown() {
    let ws = Workspace::discover().unwrap();
    assert!(ws.repo("random_nonexistent_name").is_err());
}

#[test]
fn test_workspace_repo_local_branch_default() {
    let ws = Workspace::discover().unwrap();
    let foundry = ws.repo("foundry").unwrap();
    assert_eq!(foundry.local_branch(), "master");
}

#[test]
fn test_workspace_repo_bins_method() {
    let ws = Workspace::discover().unwrap();
    let foundry = ws.repo("foundry").unwrap();
    let bins = foundry.bins();
    assert!(bins.contains(&"forge".to_string()));
    assert!(bins.contains(&"cast".to_string()));
    assert!(bins.contains(&"anvil".to_string()));
    assert!(bins.contains(&"chisel".to_string()));
}
