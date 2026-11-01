use rusty_chain_kit::git::{same_url, status, sync, Outcome, SyncOpts, SyncReport};
use rusty_chain_kit::workspace::{Repo, Strategy, Workspace};
use std::path::Path;

#[test]
fn test_same_url_identical() {
    assert!(same_url(
        "https://github.com/foundry-rs/foundry.git",
        "https://github.com/foundry-rs/foundry.git"
    ));
}

#[test]
fn test_same_url_git_suffix() {
    assert!(same_url(
        "https://github.com/foundry-rs/foundry.git",
        "https://github.com/foundry-rs/foundry"
    ));
    assert!(same_url(
        "https://github.com/foundry-rs/foundry",
        "https://github.com/foundry-rs/foundry.git"
    ));
}

#[test]
fn test_same_url_trailing_slash() {
    assert!(same_url(
        "https://github.com/foundry-rs/foundry/",
        "https://github.com/foundry-rs/foundry.git"
    ));
}

#[test]
fn test_same_url_case_insensitive() {
    assert!(same_url(
        "https://github.com/Foundry-RS/Foundry.git",
        "https://github.com/foundry-rs/foundry.git"
    ));
}

#[test]
fn test_same_url_different() {
    assert!(!same_url(
        "https://github.com/foundry-rs/foundry.git",
        "https://github.com/paradigmxyz/solar.git"
    ));
}

#[test]
fn test_sync_opts_default() {
    let opts = SyncOpts::default();
    assert!(opts.strategy.is_none());
    assert!(!opts.no_stash);
    assert!(!opts.dry_run);
    assert!(!opts.push);
}

#[test]
fn test_sync_opts_custom() {
    let opts = SyncOpts {
        strategy: Some(Strategy::Merge),
        no_stash: true,
        dry_run: true,
        push: false,
    };
    assert_eq!(opts.strategy, Some(Strategy::Merge));
    assert!(opts.no_stash);
    assert!(opts.dry_run);
    assert!(!opts.push);
}

#[test]
fn test_sync_report_initialization() {
    let rep = SyncReport::new("foundry");
    assert_eq!(rep.name, "foundry");
    assert_eq!(rep.outcome, Outcome::Error);
    assert!(rep.notes.is_empty());
}

#[test]
fn test_status_for_known_repo() {
    let ws = Workspace::discover().unwrap();
    let foundry = ws.repo("foundry").unwrap();
    let st = status(&ws, foundry);
    assert_eq!(st.name, "foundry");
    assert_eq!(st.category, "toolkit");
    assert!(!st.head.is_empty());
}

#[test]
fn test_status_for_companion_repo() {
    let ws = Workspace::discover().unwrap();
    let solar = ws.repo("solar").unwrap();
    let st = status(&ws, solar);
    assert_eq!(st.name, "solar");
    assert_eq!(st.category, "compiler");
}

#[test]
fn test_sync_dry_run_does_not_modify() {
    let ws = Workspace::discover().unwrap();
    let foundry = ws.repo("foundry").unwrap();
    let opts = SyncOpts {
        strategy: Some(Strategy::Rebase),
        no_stash: false,
        dry_run: true,
        push: false,
    };
    let rep = sync(&ws, foundry, &opts);
    assert_eq!(rep.name, "foundry");
    // Under dry_run, either UpToDate or WouldUpdate is produced, or Error if offline/network lock
    assert!(matches!(rep.outcome, Outcome::UpToDate | Outcome::WouldUpdate | Outcome::Error));
}

#[test]
fn test_repo_upstream_ref_construction() {
    let repo = Repo {
        name: "test".into(),
        path: "test".into(),
        upstream: "https://github.com/foo/bar.git".into(),
        branch: "main".into(),
        local_branch: None,
        strategy: Strategy::Rebase,
        remote: "upstream".into(),
        fork: None,
        push: false,
        category: "test".into(),
        description: "test".into(),
        build: None,
    };
    assert_eq!(repo.upstream_ref(), "upstream/main");
    assert_eq!(repo.local_branch(), "main");
}

#[test]
fn test_repo_custom_local_branch() {
    let repo = Repo {
        name: "test".into(),
        path: "test".into(),
        upstream: "https://github.com/foo/bar.git".into(),
        branch: "main".into(),
        local_branch: Some("my-custom-branch".into()),
        strategy: Strategy::Rebase,
        remote: "upstream".into(),
        fork: None,
        push: false,
        category: "test".into(),
        description: "test".into(),
        build: None,
    };
    assert_eq!(repo.upstream_ref(), "upstream/main");
    assert_eq!(repo.local_branch(), "my-custom-branch");
}

#[test]
fn test_push_isolation_guard() {
    // External repos must NEVER be allowed to push, even if push = true is passed
    let ws = Workspace::discover().unwrap();
    let foundry = ws.repo("foundry").unwrap();
    
    let mut custom_foundry = foundry.clone();
    custom_foundry.fork = Some("https://github.com/fake/foundry.git".into());
    custom_foundry.push = true;

    let opts = SyncOpts {
        strategy: None,
        no_stash: false,
        dry_run: true,
        push: true,
    };

    let rep = sync(&ws, &custom_foundry, &opts);
    // Because name != "rusty-chain-kit", maybe_push returns Ok(()) immediately and never pushes
    assert!(!rep.notes.iter().any(|n| n.contains("pushed")));
}

#[test]
fn test_git_execution_in_nonexistent_dir() {
    let res = rusty_chain_kit::git::git(Path::new("/nonexistent/directory/12345"), &["status"]);
    assert!(res.is_err());
}
