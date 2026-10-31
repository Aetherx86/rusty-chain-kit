//! Build repos from source and link their binaries into `<root>/.para/bin`.

use crate::ui;
use crate::workspace::{BuildRecord, BuildSpec, Repo, Workspace, now};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct BuildOpts {
    pub jobs: Option<usize>,
    pub profile: Option<String>,
    /// Skip repos whose recorded build matches HEAD and whose binaries still exist.
    pub if_stale: bool,
}

pub fn default_jobs(ws: &Workspace) -> usize {
    ws.manifest.workspace.build_jobs.unwrap_or_else(|| {
        std::thread::available_parallelism().map(|n| n.get().saturating_sub(2).max(1)).unwrap_or(4)
    })
}

pub fn is_fresh(ws: &Workspace, repo: &Repo) -> bool {
    let st = ws.load_state();
    let head = crate::git::head(&ws.repo_dir(repo));
    match (st.builds.get(&repo.name), head) {
        (Some(b), Some(h)) if b.ok && b.commit == h => repo.bins().iter().all(|bin| ws.bin_dir().join(bin).exists()),
        _ => false,
    }
}

pub fn build(ws: &Workspace, repo: &Repo, opts: &BuildOpts) -> Result<()> {
    let Some(spec) = &repo.build else {
        ui::info(format!("{}: nothing to build ({})", repo.name, repo.category));
        return Ok(());
    };
    let dir = ws.repo_dir(repo);
    if !dir.exists() {
        bail!("{} is not cloned (run `para repos sync {}`)", repo.name, repo.name);
    }
    if opts.if_stale && is_fresh(ws, repo) {
        ui::ok(format!("{}: up to date, skipping", repo.name));
        return Ok(());
    }
    let head = crate::git::head(&dir).unwrap_or_default();
    ui::header(format!("building {} @ {}", repo.name, &head.chars().take(10).collect::<String>()));
    let started = std::time::Instant::now();

    let linked = match spec {
        BuildSpec::Cargo { packages, bins, manifest_dir, features, profile, args } => {
            let profile = opts.profile.clone().unwrap_or_else(|| profile.clone());
            let cdir = manifest_dir.as_ref().map(|m| dir.join(m)).unwrap_or_else(|| dir.clone());
            let jobs = opts.jobs.unwrap_or_else(|| default_jobs(ws));
            let mut base: Vec<String> = vec!["build".into(), "--profile".into(), profile.clone(), "-j".into(), jobs.to_string()];
            for p in packages {
                base.push("-p".into());
                base.push(p.clone());
            }
            for b in bins {
                base.push("--bin".into());
                base.push(b.clone());
            }
            if !features.is_empty() {
                base.push("--features".into());
                base.push(features.join(","));
            }
            base.extend(args.iter().cloned());

            let locked_ok = cdir.join("Cargo.lock").exists() && run_cargo(&cdir, &[&base[..], &["--locked".to_string()]].concat())?;
            if !locked_ok {
                if cdir.join("Cargo.lock").exists() {
                    ui::warn("--locked build failed; retrying without --locked (lockfile may be stale)");
                }
                if !run_cargo(&cdir, &base)? {
                    record(ws, repo, &head, vec![], false)?;
                    bail!("cargo build failed for {}", repo.name);
                }
            }
            let target = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| cdir.join("target"));
            let pdir = target.join(if profile == "dev" { "debug" } else { profile.as_str() });
            bins.iter().map(|b| (b.clone(), pdir.join(b))).collect::<Vec<_>>()
        }
        BuildSpec::Command { cmd, bins } => {
            let (prog, rest) = cmd.split_first().context("empty build cmd")?;
            let status = Command::new(prog).args(rest).current_dir(&dir).status()?;
            if !status.success() {
                record(ws, repo, &head, vec![], false)?;
                bail!("build command failed for {}", repo.name);
            }
            bins.iter().map(|(b, p)| (b.clone(), dir.join(p))).collect()
        }
    };

    let mut names = vec![];
    for (bin, src) in &linked {
        link(&ws.bin_dir(), bin, src)?;
        names.push(bin.clone());
    }
    record(ws, repo, &head, names.clone(), true)?;
    ui::ok(format!(
        "{} built in {:.0?} → {} linked into {}",
        repo.name,
        started.elapsed(),
        names.join(", "),
        ws.bin_dir().display()
    ));
    Ok(())
}

fn run_cargo(dir: &Path, args: &[String]) -> Result<bool> {
    ui::info(format!("cargo {}", args.join(" ")));
    let status = Command::new("cargo").args(args).current_dir(dir).status().context("failed to run cargo")?;
    Ok(status.success())
}

fn link(bin_dir: &Path, name: &str, src: &Path) -> Result<()> {
    if !src.exists() {
        bail!("expected binary {} was not produced", src.display());
    }
    let dst = bin_dir.join(name);
    let _ = std::fs::remove_file(&dst);
    #[cfg(unix)]
    std::os::unix::fs::symlink(src, &dst)?;
    #[cfg(not(unix))]
    std::fs::copy(src, &dst)?;
    Ok(())
}

fn record(ws: &Workspace, repo: &Repo, head: &str, bins: Vec<String>, ok: bool) -> Result<()> {
    let mut st = ws.load_state();
    st.builds.insert(repo.name.clone(), BuildRecord { commit: head.to_string(), at: now(), bins, ok });
    ws.save_state(&st)
}
