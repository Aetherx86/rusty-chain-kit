//! RPC endpoint resolution shared by every command and the GUI.
//!
//! Order: explicit `--rpc` → `$PARA_RPC` / `$ETH_RPC_URL` → `.para/config.toml` default → MESC default
//! → a running para devnet. Names are looked up in para's config, then built-ins (`local`, `node`),
//! then MESC (endpoint name / network / chain id query).

use crate::services;
use crate::workspace::Workspace;
use anyhow::{Result, bail};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Endpoint {
    pub name: String,
    pub url: String,
    pub source: String,
    pub chain_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    pub url: String,
    pub via: String,
}

fn is_url(s: &str) -> bool {
    ["http://", "https://", "ws://", "wss://"].iter().any(|p| s.starts_with(p)) || s.ends_with(".ipc")
}

fn builtins(ws: &Workspace) -> Vec<Endpoint> {
    let mut v = vec![];
    for svc in [services::Kind::Devnet, services::Kind::Node] {
        let url = services::status(ws, svc).map(|s| s.url).unwrap_or_else(|| svc.default_url());
        v.push(Endpoint { name: svc.alias().into(), url, source: "builtin".into(), chain_id: None });
    }
    v
}

pub fn list(ws: &Workspace) -> Vec<Endpoint> {
    let cfg = ws.load_config();
    let mut out: Vec<Endpoint> = cfg
        .rpc
        .endpoints
        .iter()
        .map(|(n, u)| Endpoint { name: n.clone(), url: u.clone(), source: "para".into(), chain_id: None })
        .collect();
    out.extend(builtins(ws));
    if mesc::is_mesc_enabled() {
        if let Ok(eps) = mesc::find_endpoints(Default::default()) {
            out.extend(eps.into_iter().map(|e| Endpoint {
                chain_id: e.chain_id.as_ref().map(|c| c.to_string()),
                name: e.name,
                url: e.url,
                source: "mesc".into(),
            }));
        }
    }
    out
}

/// Resolve a name / URL / MESC query to a URL.
pub fn lookup(ws: &Workspace, q: &str) -> Result<Resolved> {
    if is_url(q) {
        return Ok(Resolved { url: q.to_string(), via: "url".into() });
    }
    let cfg = ws.load_config();
    if let Some(u) = cfg.rpc.endpoints.get(q) {
        return Ok(Resolved { url: u.clone(), via: format!("para:{q}") });
    }
    if let Some(e) = builtins(ws).into_iter().find(|e| e.name == q || (q == "anvil" && e.name == "local") || (q == "reth" && e.name == "node")) {
        return Ok(Resolved { url: e.url, via: format!("builtin:{}", e.name) });
    }
    if mesc::is_mesc_enabled() {
        if let Ok(Some(e)) = mesc::get_endpoint_by_query(q, Some("para")) {
            return Ok(Resolved { url: e.url, via: format!("mesc:{}", e.name) });
        }
    }
    bail!("unknown RPC endpoint `{q}` (see `para rpc list`, or pass a URL)")
}

pub fn resolve(ws: &Workspace, explicit: Option<&str>) -> Result<Resolved> {
    if let Some(q) = explicit.filter(|s| !s.is_empty()) {
        return lookup(ws, q);
    }
    for var in ["PARA_RPC", "ETH_RPC_URL"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() {
                let mut r = lookup(ws, &v)?;
                r.via = format!("${var} ({})", r.via);
                return Ok(r);
            }
        }
    }
    if let Some(d) = ws.load_config().rpc.default {
        if let Ok(r) = lookup(ws, &d) {
            return Ok(Resolved { url: r.url, via: format!("default ({})", r.via) });
        }
    }
    if mesc::is_mesc_enabled() {
        if let Ok(Some(e)) = mesc::get_default_endpoint(Some("para")) {
            return Ok(Resolved { url: e.url, via: format!("mesc default:{}", e.name) });
        }
    }
    if let Some(s) = services::status(ws, services::Kind::Devnet) {
        return Ok(Resolved { url: s.url, via: "running devnet".into() });
    }
    bail!("no RPC endpoint configured: pass --rpc <name|url>, set ETH_RPC_URL, or `para rpc default <name>`")
}

pub fn set_default(ws: &Workspace, name: Option<String>) -> Result<()> {
    if let Some(n) = &name {
        lookup(ws, n)?;
    }
    let mut cfg = ws.load_config();
    cfg.rpc.default = name;
    ws.save_config(&cfg)
}

pub fn add(ws: &Workspace, name: &str, url: &str) -> Result<()> {
    if !is_url(url) {
        bail!("`{url}` does not look like an RPC URL");
    }
    let mut cfg = ws.load_config();
    cfg.rpc.endpoints.insert(name.to_string(), url.to_string());
    ws.save_config(&cfg)
}

pub fn remove(ws: &Workspace, name: &str) -> Result<()> {
    let mut cfg = ws.load_config();
    if cfg.rpc.endpoints.remove(name).is_none() {
        bail!("no para endpoint named `{name}`");
    }
    if cfg.rpc.default.as_deref() == Some(name) {
        cfg.rpc.default = None;
    }
    ws.save_config(&cfg)
}
