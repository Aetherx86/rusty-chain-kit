//! Git plumbing: status, upstream remote management, and the sync engine that keeps your
//! `local_branch` up to date with `upstream/branch` (rebase / merge / ff) without losing work.

pub use crate::workspace::Strategy;
use crate::workspace::{Repo, Workspace};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

pub fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .output()
        .context("failed to run git (is it installed?)")?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

fn git_ok(dir: &Path, args: &[&str]) -> bool {
    git(dir, args).is_ok()
}

fn rev(dir: &Path, r: &str) -> Option<String> {
    git(
        dir,
        &["rev-parse", "--verify", "-q", &format!("{r}^{{commit}}")],
    )
    .ok()
    .filter(|s| !s.is_empty())
}

fn count(dir: &Path, range: &str) -> usize {
    git(dir, &["rev-list", "--count", range])
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn short(s: &str) -> String {
    s.chars().take(10).collect()
}

// ───────────────────────────── status ─────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct RepoStatus {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub category: String,
    pub description: String,
    pub upstream: String,
    pub upstream_ref: String,
    pub remote_ok: bool,
    pub local_branch: String,
    pub strategy: String,
    pub current_branch: String,
    pub head: String,
    pub head_subject: String,
    pub head_age: String,
    pub dirty: usize,
    /// Commits on local_branch that are not upstream (your patches).
    pub ahead: usize,
    /// Upstream commits not yet in local_branch (run sync).
    pub behind: usize,
    pub last_fetch: Option<String>,
    pub fork: Option<String>,
}

pub fn status(ws: &Workspace, repo: &Repo) -> RepoStatus {
    let dir = ws.repo_dir(repo);
    let mut st = RepoStatus {
        name: repo.name.clone(),
        path: repo.path.clone(),
        exists: dir.join(".git").exists(),
        category: repo.category.clone(),
        description: repo.description.clone(),
        upstream: repo.upstream.clone(),
        upstream_ref: repo.upstream_ref(),
        remote_ok: false,
        local_branch: repo.local_branch().to_string(),
        strategy: repo.strategy.to_string(),
        current_branch: String::new(),
        head: String::new(),
        head_subject: String::new(),
        head_age: String::new(),
        dirty: 0,
        ahead: 0,
        behind: 0,
        last_fetch: None,
        fork: repo.fork.clone(),
    };
    if !st.exists {
        return st;
    }
    st.remote_ok = git(&dir, &["remote", "get-url", &repo.remote])
        .map(|u| same_url(&u, &repo.upstream))
        .unwrap_or(false);
    st.current_branch = git(&dir, &["symbolic-ref", "--short", "-q", "HEAD"])
        .unwrap_or_else(|_| "(detached)".into());
    if let Ok(line) = git(&dir, &["log", "-1", "--format=%h%x00%s%x00%cr"]) {
        let mut parts = line.split('\0');
        st.head = parts.next().unwrap_or_default().into();
        st.head_subject = parts.next().unwrap_or_default().into();
        st.head_age = parts.next().unwrap_or_default().into();
    }
    st.dirty = git(&dir, &["status", "--porcelain", "--untracked-files=no"])
        .map(|s| s.lines().filter(|l| !l.is_empty()).count())
        .unwrap_or(0);
    let up = upstream_ref_for(&dir, repo);
    if let (Some(up), true) = (up, rev(&dir, repo.local_branch()).is_some()) {
        st.ahead = count(&dir, &format!("{up}..{}", repo.local_branch()));
        st.behind = count(&dir, &format!("{}..{up}", repo.local_branch()));
    }
    let fh = dir.join(".git").join("FETCH_HEAD");
    st.last_fetch = std::fs::metadata(fh)
        .and_then(|m| m.modified())
        .ok()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Local> = t.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        });
    st
}

/// `upstream/branch` if fetched, else fall back to `origin/branch` when origin is the upstream.
fn upstream_ref_for(dir: &Path, repo: &Repo) -> Option<String> {
    let primary = repo.upstream_ref();
    if rev(dir, &primary).is_some() {
        return Some(primary);
    }
    let origin_is_upstream = git(dir, &["remote", "get-url", "origin"])
        .map(|u| same_url(&u, &repo.upstream))
        .unwrap_or(false);
    let alt = format!("origin/{}", repo.branch);
    (origin_is_upstream && rev(dir, &alt).is_some()).then_some(alt)
}

pub fn same_url(a: &str, b: &str) -> bool {
    fn norm(u: &str) -> String {
        let u = u
            .trim()
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .to_lowercase();
        let u = u.replace("git@github.com:", "github.com/");
        u.trim_start_matches("https://")
            .trim_start_matches("ssh://git@")
            .to_string()
    }
    norm(a) == norm(b)
}

pub fn head(dir: &Path) -> Option<String> {
    rev(dir, "HEAD")
}

// ───────────────────────────── remotes / clone ─────────────────────────────

/// Make sure the configured remote exists and points at the upstream URL. Returns a note if changed.
pub fn ensure_remote(dir: &Path, repo: &Repo) -> Result<Option<String>> {
    match git(dir, &["remote", "get-url", &repo.remote]) {
        Ok(url) if same_url(&url, &repo.upstream) => Ok(None),
        Ok(url) => {
            git(dir, &["remote", "set-url", &repo.remote, &repo.upstream])?;
            Ok(Some(format!(
                "remote `{}` repointed {url} → {}",
                repo.remote, repo.upstream
            )))
        }
        Err(_) => {
            git(dir, &["remote", "add", &repo.remote, &repo.upstream])?;
            Ok(Some(format!(
                "added remote `{}` → {}",
                repo.remote, repo.upstream
            )))
        }
    }
}

pub fn clone(ws: &Workspace, repo: &Repo) -> Result<()> {
    let dir = ws.repo_dir(repo);
    if dir.exists() {
        bail!("{} already exists", dir.display());
    }
    let status = Command::new("git")
        .args([
            "clone",
            "--filter=blob:none",
            "-o",
            &repo.remote,
            "-b",
            &repo.branch,
            &repo.upstream,
        ])
        .arg(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()?;
    if !status.success() {
        bail!("git clone {} failed", repo.upstream);
    }
    if repo.local_branch() != repo.branch {
        git(
            &dir,
            &[
                "checkout",
                "-b",
                repo.local_branch(),
                "--track",
                &repo.upstream_ref(),
            ],
        )?;
    }
    Ok(())
}

pub fn fetch(dir: &Path, repo: &Repo) -> Result<()> {
    let refspec = format!(
        "+refs/heads/{0}:refs/remotes/{1}/{0}",
        repo.branch, repo.remote
    );
    git(dir, &["fetch", "--quiet", &repo.remote, &refspec])?;
    Ok(())
}

// ───────────────────────────── sync ─────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct SyncOpts {
    pub strategy: Option<Strategy>,
    pub no_stash: bool,
    pub dry_run: bool,
    pub push: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Cloned,
    UpToDate,
    Updated,
    WouldUpdate,
    Conflict,
    Diverged,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub name: String,
    pub outcome: Outcome,
    pub from: String,
    pub to: String,
    pub new_commits: usize,
    pub local_commits: usize,
    pub notes: Vec<String>,
}

impl SyncReport {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            outcome: Outcome::Error,
            from: String::new(),
            to: String::new(),
            new_commits: 0,
            local_commits: 0,
            notes: vec![],
        }
    }
}

pub fn sync(ws: &Workspace, repo: &Repo, opts: &SyncOpts) -> SyncReport {
    let mut rep = SyncReport::new(&repo.name);
    if let Err(e) = sync_inner(ws, repo, opts, &mut rep) {
        rep.outcome = Outcome::Error;
        rep.notes.push(format!("{e:#}"));
    }
    rep
}

fn sync_inner(ws: &Workspace, repo: &Repo, opts: &SyncOpts, rep: &mut SyncReport) -> Result<()> {
    let dir = ws.repo_dir(repo);
    if !dir.join(".git").exists() {
        if opts.dry_run {
            rep.outcome = Outcome::WouldUpdate;
            rep.notes.push("not cloned yet; would clone".into());
            return Ok(());
        }
        clone(ws, repo)?;
        rep.outcome = Outcome::Cloned;
        rep.to = head(&dir).map(|h| short(&h)).unwrap_or_default();
        return Ok(());
    }

    if let Some(note) = ensure_remote(&dir, repo)? {
        rep.notes.push(note);
    }
    fetch(&dir, repo)?;

    let strategy = opts.strategy.unwrap_or(repo.strategy);
    let local = repo.local_branch().to_string();
    let up_ref = repo.upstream_ref();
    let up_sha = rev(&dir, &up_ref).with_context(|| format!("{up_ref} missing after fetch"))?;
    let current = git(&dir, &["symbolic-ref", "--short", "-q", "HEAD"]).unwrap_or_default();

    // Create the local branch from upstream if it doesn't exist yet.
    let Some(local_sha) = rev(&dir, &local) else {
        if opts.dry_run {
            rep.outcome = Outcome::WouldUpdate;
            rep.notes
                .push(format!("would create branch {local} from {up_ref}"));
            return Ok(());
        }
        git(&dir, &["branch", "--track", &local, &up_ref])?;
        rep.notes
            .push(format!("created branch {local} tracking {up_ref}"));
        rep.outcome = Outcome::Updated;
        rep.to = short(&up_sha);
        return Ok(());
    };

    rep.from = short(&local_sha);
    rep.new_commits = count(&dir, &format!("{local}..{up_ref}"));
    rep.local_commits = count(&dir, &format!("{up_ref}..{local}"));

    if rep.new_commits == 0 {
        rep.outcome = Outcome::UpToDate;
        rep.to = rep.from.clone();
        maybe_push(&dir, repo, opts, rep)?;
        return Ok(());
    }
    if opts.dry_run {
        rep.outcome = Outcome::WouldUpdate;
        rep.notes.push(format!(
            "{} upstream commit(s) to apply via {strategy}; {} local commit(s) carried",
            rep.new_commits, rep.local_commits
        ));
        return Ok(());
    }

    let fast_forwardable = rep.local_commits == 0;

    // Fast path: local branch has no patches and is not checked out → move the ref, no checkout.
    if fast_forwardable && current != local {
        git(
            &dir,
            &[
                "update-ref",
                &format!("refs/heads/{local}"),
                &up_sha,
                &local_sha,
            ],
        )?;
        rep.outcome = Outcome::Updated;
        rep.to = short(&up_sha);
        maybe_push(&dir, repo, opts, rep)?;
        return Ok(());
    }
    if !fast_forwardable && strategy == Strategy::Ff {
        rep.outcome = Outcome::Diverged;
        rep.notes.push(format!(
            "{local} has {} local commit(s); strategy=ff cannot apply. Use --strategy rebase|merge",
            rep.local_commits
        ));
        return Ok(());
    }

    // Slow path: we need local checked out with a clean tree.
    let dirty = !git(&dir, &["status", "--porcelain", "--untracked-files=no"])?.is_empty();
    let mut stashed = false;
    if dirty {
        if opts.no_stash {
            bail!("working tree is dirty (commit or stash, or drop --no-stash)");
        }
        git(&dir, &["stash", "push", "-m", "para-sync-autostash"])?;
        stashed = true;
        rep.notes.push("stashed local changes".into());
    }
    let switched = current != local;
    if switched {
        git(&dir, &["checkout", "--quiet", &local])?;
    }

    let result = if fast_forwardable {
        git(&dir, &["merge", "--ff-only", "--quiet", &up_ref]).map(|_| ())
    } else {
        match strategy {
            Strategy::Rebase => git(&dir, &["rebase", &up_ref]).map(|_| ()),
            Strategy::Merge => git(&dir, &["merge", "--no-edit", &up_ref]).map(|_| ()),
            Strategy::Ff => unreachable!(),
        }
    };

    match result {
        Ok(()) => {
            rep.outcome = Outcome::Updated;
            rep.to = head(&dir).map(|h| short(&h)).unwrap_or_default();
        }
        Err(e) => {
            let conflicts =
                git(&dir, &["diff", "--name-only", "--diff-filter=U"]).unwrap_or_default();
            let _ = git(&dir, &["rebase", "--abort"]);
            let _ = git(&dir, &["merge", "--abort"]);
            rep.outcome = Outcome::Conflict;
            rep.to = rep.from.clone();
            rep.notes
                .push(format!("{strategy} aborted, branch left unchanged: {e}"));
            if !conflicts.is_empty() {
                rep.notes.push(format!(
                    "conflicting files: {}",
                    conflicts.lines().collect::<Vec<_>>().join(", ")
                ));
            }
            rep.notes.push(format!(
                "resolve manually: cd {} && git checkout {local} && git {strategy} {up_ref}",
                dir.display()
            ));
        }
    }

    if switched && !current.is_empty() {
        let _ = git(&dir, &["checkout", "--quiet", &current]);
    }
    if stashed {
        if git_ok(&dir, &["stash", "pop", "--quiet"]) {
            rep.notes.push("restored stashed changes".into());
        } else {
            rep.notes
                .push("could not auto-restore stash; run `git stash pop` manually".into());
        }
    }
    if rep.outcome == Outcome::Updated {
        maybe_push(&dir, repo, opts, rep)?;
    }
    Ok(())
}

fn maybe_push(dir: &Path, repo: &Repo, opts: &SyncOpts, rep: &mut SyncReport) -> Result<()> {
    // Safety enforcement: external sibling repositories must NEVER be pushed alongside rusty-chain-kit
    if repo.name != "rusty-chain-kit" {
        return Ok(());
    }
    let Some(fork) = &repo.fork else {
        return Ok(());
    };
    if !(repo.push || opts.push) {
        return Ok(());
    }
    match git(dir, &["remote", "get-url", "fork"]) {
        Ok(u) if same_url(&u, fork) => {}
        Ok(_) => {
            git(dir, &["remote", "set-url", "fork", fork])?;
        }
        Err(_) => {
            git(dir, &["remote", "add", "fork", fork])?;
        }
    }
    let local = repo.local_branch();
    match git(
        dir,
        &[
            "push",
            "--quiet",
            "--force-with-lease",
            "fork",
            &format!("{local}:{local}"),
        ],
    ) {
        Ok(_) => rep.notes.push(format!("pushed {local} → fork")),
        Err(e) => rep.notes.push(format!("push to fork failed: {e}")),
    }
    Ok(())
}
