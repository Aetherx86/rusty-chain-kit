//! Workspace discovery, the `para.toml` manifest, and persistent state under `<root>/.para`.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ───────────────────────────── manifest ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub workspace: WorkspaceCfg,
    #[serde(rename = "repo", default)]
    pub repos: Vec<Repo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCfg {
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_state_dir")]
    pub state_dir: String,
    #[serde(default)]
    pub build_jobs: Option<usize>,
}

impl Default for WorkspaceCfg {
    fn default() -> Self {
        Self { root: default_root(), state_dir: default_state_dir(), build_jobs: None }
    }
}

fn default_root() -> String { "..".into() }
fn default_state_dir() -> String { ".para".into() }
fn default_remote() -> String { "upstream".into() }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Strategy {
    #[default]
    Rebase,
    Merge,
    Ff,
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Strategy::Rebase => "rebase",
            Strategy::Merge => "merge",
            Strategy::Ff => "ff",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub upstream: String,
    pub branch: String,
    #[serde(default)]
    pub local_branch: Option<String>,
    #[serde(default)]
    pub strategy: Strategy,
    #[serde(default = "default_remote")]
    pub remote: String,
    #[serde(default)]
    pub fork: Option<String>,
    #[serde(default)]
    pub push: bool,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub build: Option<BuildSpec>,
}

impl Repo {
    pub fn local_branch(&self) -> &str {
        self.local_branch.as_deref().unwrap_or(&self.branch)
    }
    pub fn upstream_ref(&self) -> String {
        format!("{}/{}", self.remote, self.branch)
    }
    pub fn bins(&self) -> Vec<String> {
        match &self.build {
            Some(BuildSpec::Cargo { bins, .. }) => bins.clone(),
            Some(BuildSpec::Command { bins, .. }) => bins.keys().cloned().collect(),
            None => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum BuildSpec {
    Cargo {
        packages: Vec<String>,
        bins: Vec<String>,
        /// Sub-directory holding the Cargo workspace (e.g. "rust" for mesc).
        #[serde(default)]
        manifest_dir: Option<String>,
        #[serde(default)]
        features: Vec<String>,
        #[serde(default = "default_profile")]
        profile: String,
        #[serde(default)]
        args: Vec<String>,
    },
    /// Arbitrary build command; `bins` maps binary name -> path relative to the repo.
    Command { cmd: Vec<String>, bins: BTreeMap<String, String> },
}

fn default_profile() -> String { "release".into() }

// ───────────────────────────── workspace ─────────────────────────────

#[derive(Debug, Clone)]
pub struct Workspace {
    #[allow(dead_code)]
    pub manifest_path: PathBuf,
    pub root: PathBuf,
    pub state: PathBuf,
    pub manifest: Manifest,
}

impl Workspace {
    /// Find the manifest: `$PARA_MANIFEST`, then walk up from cwd looking for `para.toml` or
    /// `para/para.toml`, falling back to `foundry.toml` discovery or built-in defaults.
    pub fn discover() -> Result<Self> {
        if let Some(p) = std::env::var_os("PARA_MANIFEST") {
            return Self::load(Path::new(&p));
        }
        if let Ok(mut dir) = std::env::current_dir() {
            loop {
                for cand in [dir.join("para.toml"), dir.join("para").join("para.toml"), dir.join("rusty-chain-kit").join("para.toml")] {
                    if cand.is_file() {
                        return Self::load(&cand);
                    }
                }
                let foundry_cand = dir.join("foundry.toml");
                if foundry_cand.is_file() {
                    return Self::from_foundry_toml(&foundry_cand);
                }
                if !dir.pop() {
                    break;
                }
            }
        }
        let built_in = Path::new(env!("CARGO_MANIFEST_DIR")).join("para.toml");
        if built_in.is_file() {
            return Self::load(&built_in);
        }
        bail!("could not find para.toml or foundry.toml (set PARA_MANIFEST or run inside the workspace)")
    }

    /// Automatically load a virtual workspace from an existing `foundry.toml` configuration.
    pub fn from_foundry_toml(path: &Path) -> Result<Self> {
        let manifest_path = path.canonicalize().with_context(|| format!("reading {}", path.display()))?;
        let root = manifest_path.parent().unwrap().to_path_buf();
        let state = root.join(".para");
        for sub in ["bin", "logs", "run", "data"] {
            std::fs::create_dir_all(state.join(sub)).ok();
        }
        let manifest = Manifest {
            workspace: WorkspaceCfg {
                root: ".".into(),
                state_dir: ".para".into(),
                build_jobs: None,
            },
            repos: vec![],
        };
        Ok(Self { manifest_path, root, state, manifest })
    }

    pub fn load(path: &Path) -> Result<Self> {
        let manifest_path = path.canonicalize().with_context(|| format!("reading {}", path.display()))?;
        let text = std::fs::read_to_string(&manifest_path)?;
        let manifest: Manifest =
            toml::from_str(&text).with_context(|| format!("parsing {}", manifest_path.display()))?;
        let base = manifest_path.parent().unwrap().to_path_buf();
        let root = normalize(&base.join(&manifest.workspace.root));
        let state = root.join(&manifest.workspace.state_dir);
        for sub in ["bin", "logs", "run", "data"] {
            std::fs::create_dir_all(state.join(sub)).ok();
        }
        Ok(Self { manifest_path, root, state, manifest })
    }

    pub fn repo(&self, name: &str) -> Result<&Repo> {
        self.manifest
            .repos
            .iter()
            .find(|r| r.name == name || r.path == name)
            .with_context(|| format!("unknown repo `{name}` (see `para repos list`)"))
    }

    /// Resolve a list of names (empty = all repos).
    pub fn select(&self, names: &[String]) -> Result<Vec<&Repo>> {
        if names.is_empty() {
            return Ok(self.manifest.repos.iter().collect());
        }
        names.iter().map(|n| self.repo(n)).collect()
    }

    pub fn repo_dir(&self, repo: &Repo) -> PathBuf { self.root.join(&repo.path) }
    pub fn bin_dir(&self) -> PathBuf { self.state.join("bin") }
    pub fn logs_dir(&self) -> PathBuf { self.state.join("logs") }
    pub fn run_dir(&self) -> PathBuf { self.state.join("run") }
    pub fn data_dir(&self) -> PathBuf { self.state.join("data") }
    #[allow(dead_code)]
    pub fn para_dir(&self) -> PathBuf { self.manifest_path.parent().unwrap().to_path_buf() }

    /// The repo that provides a given binary.
    pub fn repo_for_bin(&self, bin: &str) -> Option<&Repo> {
        self.manifest.repos.iter().find(|r| r.bins().iter().any(|b| b == bin))
    }

    pub fn all_bins(&self) -> Vec<(String, String)> {
        self.manifest
            .repos
            .iter()
            .flat_map(|r| r.bins().into_iter().map(move |b| (b, r.name.clone())))
            .collect()
    }

    pub fn load_state(&self) -> State {
        std::fs::read_to_string(self.state.join("state.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_state(&self, st: &State) -> Result<()> {
        std::fs::write(self.state.join("state.json"), serde_json::to_string_pretty(st)?)?;
        Ok(())
    }

    pub fn config_path(&self) -> PathBuf { self.state.join("config.toml") }

    pub fn load_config(&self) -> Config {
        let p = self.config_path();
        if let Some(cfg) = std::fs::read_to_string(&p).ok().and_then(|s| toml::from_str(&s).ok()) {
            return cfg;
        }
        let cfg = Config::default();
        let _ = self.save_config(&cfg);
        cfg
    }

    pub fn save_config(&self, cfg: &Config) -> Result<()> {
        let mut text = String::from(
            "# para user config (endpoints are also read from MESC, see `para rpc list`)\n",
        );
        text.push_str(&toml::to_string_pretty(cfg)?);
        std::fs::write(self.config_path(), text)?;
        Ok(())
    }
}

fn normalize(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

// ───────────────────────────── persistent state ─────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub builds: BTreeMap<String, BuildRecord>,
    #[serde(default)]
    pub syncs: BTreeMap<String, SyncRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRecord {
    pub commit: String,
    pub at: String,
    pub bins: Vec<String>,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub at: String,
    pub from: String,
    pub to: String,
    pub outcome: String,
}

// ───────────────────────────── user config ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rpc: RpcConfig,
}

impl Default for Config {
    fn default() -> Self { Self { rpc: RpcConfig::default() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// Default endpoint: an alias from `endpoints`, a built-in alias, a MESC query, or a URL.
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub endpoints: BTreeMap<String, String>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        let endpoints = [
            ("mainnet", "https://ethereum-rpc.publicnode.com"),
            ("sepolia", "https://ethereum-sepolia-rpc.publicnode.com"),
            ("base", "https://base-rpc.publicnode.com"),
            ("optimism", "https://optimism-rpc.publicnode.com"),
            ("arbitrum", "https://arbitrum-one-rpc.publicnode.com"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        Self { default: Some("mainnet".into()), endpoints }
    }
}

pub fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
