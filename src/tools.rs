//! Tool registry: resolve binaries (locally built first, then $PATH), passthrough exec, versions.

use crate::workspace::Workspace;
use anyhow::{Result, bail};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub bin: String,
    pub repo: String,
    pub path: Option<String>,
    /// "workspace" (built from our checkout), "path" (found on $PATH) or "missing".
    pub source: String,
    pub version: Option<String>,
    pub built_commit: Option<String>,
    pub repo_head: Option<String>,
    pub stale: bool,
}

pub fn resolve(ws: &Workspace, bin: &str) -> Option<(PathBuf, &'static str)> {
    let local = ws.bin_dir().join(bin);
    if local.exists() {
        return Some((local, "workspace"));
    }
    // Avoid resolving to ourselves through a `para` symlink in PATH.
    which::which(bin).ok().map(|p| (p, "path"))
}

pub fn require(ws: &Workspace, bin: &str) -> Result<PathBuf> {
    match resolve(ws, bin) {
        Some((p, _)) => Ok(p),
        None => {
            let hint = match ws.repo_for_bin(bin) {
                Some(r) => format!("build it with `para build {}`", r.name),
                None => "install it and make sure it is on $PATH".into(),
            };
            bail!("`{bin}` not found; {hint}")
        }
    }
}

pub fn version(path: &PathBuf) -> Option<String> {
    if let Ok(out) = Command::new(path).arg("--version").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(if out.stdout.is_empty() {
                &out.stderr
            } else {
                &out.stdout
            })
            .to_string();
            if let Some(line) = s.lines().find(|l| !l.trim().is_empty()) {
                return Some(line.trim().to_string());
            }
        }
    }
    if let Ok(out) = Command::new(path).arg("--help").output() {
        let s = String::from_utf8_lossy(if out.stdout.is_empty() {
            &out.stderr
        } else {
            &out.stdout
        })
        .to_string();
        if let Some(line) = s.lines().find(|l| !l.trim().is_empty()) {
            return Some(line.trim().to_string());
        }
    }
    None
}

pub fn list(ws: &Workspace, with_versions: bool) -> Vec<ToolInfo> {
    let state = ws.load_state();
    ws.all_bins()
        .into_iter()
        .map(|(bin, repo_name)| {
            let repo = ws.repo(&repo_name).ok();
            let resolved = resolve(ws, &bin);
            let built = state.builds.get(&repo_name).filter(|b| b.ok);
            let repo_head = repo.and_then(|r| crate::git::head(&ws.repo_dir(r)));
            let stale = match (built, &repo_head, &resolved) {
                (Some(b), Some(h), Some((_, "workspace"))) => &b.commit != h,
                _ => false,
            };
            ToolInfo {
                version: if with_versions {
                    resolved.as_ref().and_then(|(p, _)| version(p))
                } else {
                    None
                },
                path: resolved.as_ref().map(|(p, _)| p.display().to_string()),
                source: resolved
                    .as_ref()
                    .map(|(_, s)| s.to_string())
                    .unwrap_or_else(|| "missing".into()),
                built_commit: built.map(|b| b.commit.chars().take(10).collect()),
                repo_head: repo_head.map(|h| h.chars().take(10).collect()),
                stale,
                repo: repo_name,
                bin,
            }
        })
        .collect()
}

/// Replace this process with the tool (unix) so signals / TTY behave exactly like the real binary.
pub fn exec(ws: &Workspace, bin: &str, args: &[String]) -> Result<()> {
    let path = require(ws, bin)?;
    let mut cmd = Command::new(&path);
    cmd.args(args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        bail!("failed to exec {}: {err}", path.display());
    }
    #[cfg(not(unix))]
    {
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

/// Run a tool to completion, inheriting stdio. Errors on non-zero exit.
#[allow(dead_code)]
pub fn run(
    ws: &Workspace,
    bin: &str,
    args: &[String],
    cwd: Option<&std::path::Path>,
) -> Result<()> {
    let path = require(ws, bin)?;
    let mut cmd = Command::new(&path);
    cmd.args(args);
    if let Some(d) = cwd {
        cmd.current_dir(d);
    }
    let status = cmd.status()?;
    if !status.success() {
        bail!(
            "{bin} exited with {}",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".into())
        );
    }
    Ok(())
}
