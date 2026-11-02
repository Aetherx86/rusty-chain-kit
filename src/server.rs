//! Web GUI backend powered by Axum.
//! Serves the embedded single-page GUI and provides REST/streaming endpoints for repos,
//! builds, tools, devnets, chain queries, and workflows.

use crate::chain;
use crate::git;
use crate::services;
use crate::tools;
use crate::workflows;
use crate::workspace::Workspace;
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub ws: Arc<Workspace>,
}

pub async fn run_server(ws: Workspace, port: u16, open_browser: bool) -> Result<()> {
    let state = AppState { ws: Arc::new(ws) };

    let app = Router::new()
        // API endpoints
        .route("/api/repos", get(api_get_repos))
        .route("/api/repos/sync", post(api_sync_repos))
        .route("/api/repos/update-all", post(api_update_all_repos))
        .route("/api/repos/build", post(api_build_repo))
        .route("/api/tools", get(api_get_tools))
        .route("/api/rpc/endpoints", get(api_get_rpc_endpoints))
        .route("/api/rpc/chain", get(api_get_chain_info))
        .route("/api/rpc/block", get(api_get_block))
        .route("/api/rpc/tx", get(api_get_tx))
        .route("/api/rpc/address", get(api_get_address))
        .route("/api/devnet/status", get(api_get_devnet_status))
        .route("/api/devnet/start", post(api_start_devnet))
        .route("/api/devnet/stop", post(api_stop_devnet))
        .route("/api/devnet/accounts", get(api_get_devnet_accounts))
        .route("/api/devnet/logs", get(api_get_devnet_logs))
        .route("/api/workflows/compile", post(api_workflow_compile))
        .route("/api/workflows/decompile", post(api_workflow_decompile))
        .route("/api/workflows/scaffold", post(api_workflow_scaffold))
        .route("/api/forensics/flow", get(api_get_forensics_flow))
        .route("/api/bytecode/inspect", post(api_bytecode_inspect))
        .route("/api/verify/prove", post(api_verify_prove))
        .route("/api/verify/fuzz", post(api_verify_fuzz))
        .route("/api/storage/inspect", post(api_storage_inspect))
        .route("/api/copilot/explain", post(api_copilot_explain))
        .route("/api/bench/run", post(api_bench_run))
        .route("/api/repl/eval", post(api_repl_eval))
        .route("/api/mev/sim", post(api_mev_sim))
        .route("/api/analytics/query", post(api_analytics_query))
        // Static assets fallback
        .fallback(static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://127.0.0.1:{port}");
    crate::ui::header(format!("Para GUI Dashboard running at {url}"));

    if open_browser {
        let _ = open::that(&url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if let Some(index) = Assets::get("index.html") {
                Html(index.data).into_response()
            } else {
                (StatusCode::NOT_FOUND, "Not Found").into_response()
            }
        }
    }
}

// ───────────────────────────── Repo Handlers ─────────────────────────────

async fn api_get_repos(State(s): State<AppState>) -> Json<Value> {
    let repos: Vec<_> =
        s.ws.manifest
            .repos
            .iter()
            .map(|r| git::status(&s.ws, r))
            .collect();
    Json(json!({ "repos": repos }))
}

#[derive(Deserialize)]
struct SyncRequest {
    repo: Option<String>,
    strategy: Option<crate::workspace::Strategy>,
}

#[derive(Deserialize)]
struct UpdateAllRequest {
    check: Option<bool>,
    rebuild: Option<bool>,
    strategy: Option<String>,
}

async fn api_update_all_repos(
    State(s): State<AppState>,
    Json(req): Json<UpdateAllRequest>,
) -> Json<Value> {
    let script = s.ws.root.join("update-all.sh");
    if !script.exists() {
        return Json(json!({ "success": false, "error": "update-all.sh not found" }));
    }

    let mut cmd = std::process::Command::new(&script);
    if req.check.unwrap_or(false) {
        cmd.arg("--check");
    }
    if req.rebuild.unwrap_or(false) {
        cmd.arg("--rebuild");
    }
    if let Some(strat) = req.strategy {
        cmd.arg("--strategy").arg(strat);
    }

    match cmd.output() {
        Ok(out) => {
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            Json(json!({ "success": out.status.success(), "output": combined }))
        }
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

async fn api_sync_repos(State(s): State<AppState>, Json(req): Json<SyncRequest>) -> Json<Value> {
    let repos_to_sync = if let Some(name) = req.repo {
        match s.ws.repo(&name) {
            Ok(r) => vec![r],
            Err(e) => return Json(json!({ "error": e.to_string() })),
        }
    } else {
        s.ws.manifest.repos.iter().collect()
    };

    let opts = git::SyncOpts {
        strategy: req.strategy,
        no_stash: false,
        dry_run: false,
        push: false,
    };

    let reports: Vec<_> = repos_to_sync
        .into_iter()
        .map(|r| git::sync(&s.ws, r, &opts))
        .collect();
    Json(json!({ "reports": reports }))
}

#[derive(Deserialize)]
struct BuildRequest {
    repo: String,
}

async fn api_build_repo(State(s): State<AppState>, Json(req): Json<BuildRequest>) -> Json<Value> {
    let repo = match s.ws.repo(&req.repo) {
        Ok(r) => r,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };

    let opts = crate::builder::BuildOpts::default();
    match crate::builder::build(&s.ws, repo, &opts) {
        Ok(_) => Json(json!({ "success": true, "repo": req.repo })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

// ───────────────────────────── Tools Handlers ─────────────────────────────

async fn api_get_tools(State(s): State<AppState>) -> Json<Value> {
    let tools = tools::list(&s.ws, true);
    Json(json!({ "tools": tools }))
}

// ───────────────────────────── RPC & Chain Handlers ─────────────────────────────

async fn api_get_rpc_endpoints(State(s): State<AppState>) -> Json<Value> {
    let endpoints = crate::rpc::list(&s.ws);
    let default = s.ws.load_config().rpc.default;
    Json(json!({ "endpoints": endpoints, "default": default }))
}

#[derive(Deserialize)]
struct ChainQuery {
    rpc: Option<String>,
}

async fn api_get_chain_info(State(s): State<AppState>, Query(q): Query<ChainQuery>) -> Json<Value> {
    let rpc = match crate::rpc::resolve(&s.ws, q.rpc.as_deref()) {
        Ok(r) => r,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };

    match chain::info(&rpc.url).await {
        Ok(info) => Json(json!({ "info": info })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct BlockQuery {
    rpc: Option<String>,
    id: Option<String>,
}

async fn api_get_block(State(s): State<AppState>, Query(q): Query<BlockQuery>) -> Json<Value> {
    let rpc = match crate::rpc::resolve(&s.ws, q.rpc.as_deref()) {
        Ok(r) => r,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };

    let id = q.id.unwrap_or_else(|| "latest".into());
    match chain::block(&rpc.url, &id).await {
        Ok((summary, raw)) => Json(json!({ "summary": summary, "raw": raw })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct TxQuery {
    rpc: Option<String>,
    hash: String,
}

async fn api_get_tx(State(s): State<AppState>, Query(q): Query<TxQuery>) -> Json<Value> {
    let rpc = match crate::rpc::resolve(&s.ws, q.rpc.as_deref()) {
        Ok(r) => r,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };

    match chain::tx(&rpc.url, &q.hash).await {
        Ok((summary, tx, receipt)) => {
            Json(json!({ "summary": summary, "tx": tx, "receipt": receipt }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct AddressQuery {
    rpc: Option<String>,
    address: String,
}

async fn api_get_address(State(s): State<AppState>, Query(q): Query<AddressQuery>) -> Json<Value> {
    let rpc = match crate::rpc::resolve(&s.ws, q.rpc.as_deref()) {
        Ok(r) => r,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };

    match chain::address(&rpc.url, &q.address).await {
        Ok(info) => Json(json!({ "info": info })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

// ───────────────────────────── Devnet Handlers ─────────────────────────────

async fn api_get_devnet_status(State(s): State<AppState>) -> Json<Value> {
    let anvil_status = services::status(&s.ws, services::Kind::Devnet);
    let reth_status = services::status(&s.ws, services::Kind::Node);
    Json(json!({
        "devnet": anvil_status,
        "node": reth_status,
    }))
}

#[derive(Deserialize)]
struct StartDevnetRequest {
    kind: Option<String>,
    port: Option<u16>,
    fork: Option<String>,
    block_time: Option<u64>,
}

async fn api_start_devnet(
    State(s): State<AppState>,
    Json(req): Json<StartDevnetRequest>,
) -> Json<Value> {
    let kind = req.kind.as_deref().unwrap_or("devnet");
    match kind {
        "devnet" | "anvil" => {
            let opts = services::DevnetOpts {
                port: req.port,
                fork: req.fork,
                block_time: req.block_time,
                ..Default::default()
            };
            match services::start_devnet(&s.ws, &opts) {
                Ok(st) => Json(json!({ "success": true, "status": st })),
                Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
            }
        }
        "node" | "reth" => {
            let opts = services::NodeOpts {
                port: req.port,
                block_time: req.block_time,
                ..Default::default()
            };
            match services::start_node(&s.ws, &opts) {
                Ok(st) => Json(json!({ "success": true, "status": st })),
                Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
            }
        }
        _ => Json(json!({ "success": false, "error": "unknown service kind" })),
    }
}

#[derive(Deserialize)]
struct StopDevnetRequest {
    kind: Option<String>,
}

async fn api_stop_devnet(
    State(s): State<AppState>,
    Json(req): Json<StopDevnetRequest>,
) -> Json<Value> {
    let kind = match req.kind.as_deref().unwrap_or("devnet") {
        "node" | "reth" => services::Kind::Node,
        _ => services::Kind::Devnet,
    };
    match services::stop(&s.ws, kind) {
        Ok(stopped) => Json(json!({ "success": true, "stopped": stopped })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

async fn api_get_devnet_accounts(State(s): State<AppState>) -> Json<Value> {
    let accounts = services::devnet_accounts(&s.ws);
    Json(json!({ "accounts": accounts }))
}

#[derive(Deserialize)]
struct LogQuery {
    kind: Option<String>,
    lines: Option<usize>,
}

async fn api_get_devnet_logs(State(s): State<AppState>, Query(q): Query<LogQuery>) -> Json<Value> {
    let kind = match q.kind.as_deref().unwrap_or("devnet") {
        "node" | "reth" => services::Kind::Node,
        _ => services::Kind::Devnet,
    };
    let lines = q.lines.unwrap_or(100);
    let log = services::tail(&s.ws, kind, lines);
    Json(json!({ "log": log }))
}

// ───────────────────────────── Workflows Handlers ─────────────────────────────

#[derive(Deserialize)]
struct CompileRequest {
    code: String,
    compiler: Option<String>,
}

async fn api_workflow_compile(
    State(s): State<AppState>,
    Json(req): Json<CompileRequest>,
) -> Json<Value> {
    let tmp_dir = s.ws.data_dir().join("tmp_compile");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let file_path = tmp_dir.join("Contract.sol");
    if let Err(e) = std::fs::write(&file_path, &req.code) {
        return Json(json!({ "success": false, "error": e.to_string() }));
    }

    let prefer_solar = req.compiler.as_deref().unwrap_or("solar") == "solar";
    match workflows::compile(&s.ws, &file_path, prefer_solar) {
        Ok(res) => Json(json!({ "result": res })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct DecompileRequest {
    target: String,
    rpc: Option<String>,
}

async fn api_workflow_decompile(
    State(s): State<AppState>,
    Json(req): Json<DecompileRequest>,
) -> Json<Value> {
    match workflows::decompile(&s.ws, req.rpc.as_deref(), &req.target).await {
        Ok(res) => Json(json!({ "result": res })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct ScaffoldRequest {
    kind: String,
    name: String,
}

async fn api_workflow_scaffold(
    State(s): State<AppState>,
    Json(req): Json<ScaffoldRequest>,
) -> Json<Value> {
    let res = match req.kind.as_str() {
        "bot" => crate::scaffold::new_bot(&s.ws, &req.name, None),
        "script" => crate::scaffold::new_script(&s.ws, &req.name, None),
        "contract" => crate::scaffold::new_contract(&s.ws, &req.name, None),
        _ => return Json(json!({ "success": false, "error": "unknown scaffold kind" })),
    };

    match res {
        Ok(path) => Json(json!({ "success": true, "path": path.display().to_string() })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

// ───────────────────────────── Forensics, Bytecode & Intelligence ─────────────────────────────

#[derive(Deserialize)]
struct ForensicsQuery {
    address: Option<String>,
    hops: Option<usize>,
}

async fn api_get_forensics_flow(
    State(_s): State<AppState>,
    Query(q): Query<ForensicsQuery>,
) -> Json<Value> {
    let subject = q
        .address
        .unwrap_or_else(|| "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".into());
    let hops = q.hops.unwrap_or(2);
    match crate::forensics::trace_flow(&subject, hops) {
        Ok(graph) => Json(serde_json::to_value(&graph).unwrap_or_default()),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct StorageInspectRequest {
    #[serde(default)]
    contract_a: Option<String>,
    #[serde(default)]
    contract_b: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

async fn api_storage_inspect(
    State(_s): State<AppState>,
    Json(req): Json<StorageInspectRequest>,
) -> Json<Value> {
    let target = req
        .contract_a
        .or(req.source)
        .unwrap_or_else(|| "contracts/Vault.sol".into());
    if let Some(b) = req.contract_b {
        match crate::storage::compare_layouts(
            std::path::Path::new(&target),
            std::path::Path::new(&b),
        ) {
            Ok(rep) => Json(json!({ "success": true, "report": rep })),
            Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
        }
    } else {
        match crate::storage::inspect_storage(std::path::Path::new(&target)) {
            Ok(layout) => Json(json!({ "success": true, "layout": layout })),
            Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
        }
    }
}

#[derive(Deserialize)]
struct CopilotRequest {
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

async fn api_copilot_explain(
    State(_s): State<AppState>,
    Json(req): Json<CopilotRequest>,
) -> Json<Value> {
    let target = req
        .target
        .or(req.source)
        .unwrap_or_else(|| "contracts/Vault.sol".into());
    match crate::copilot::analyze_contract(std::path::Path::new(&target)) {
        Ok(rep) => Json(json!({ "success": true, "report": rep })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct BenchRequest {
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

async fn api_bench_run(State(_s): State<AppState>, Json(req): Json<BenchRequest>) -> Json<Value> {
    let target = req
        .target
        .or(req.source)
        .unwrap_or_else(|| "contracts/Vault.sol".into());
    match crate::bench::run_benchmark(std::path::Path::new(&target)) {
        Ok(rep) => Json(json!({ "success": true, "report": rep })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct BytecodeInspectRequest {
    bytecode: String,
}

async fn api_bytecode_inspect(
    State(_s): State<AppState>,
    Json(req): Json<BytecodeInspectRequest>,
) -> Json<Value> {
    let clean = req.bytecode.trim().trim_start_matches("0x");
    let bytes = match alloy::hex::decode(clean) {
        Ok(b) => b,
        Err(e) => return Json(json!({ "error": format!("invalid hex bytecode: {e}") })),
    };

    let abi = crate::bytecode::extract_abi(&bytes);
    let proxy = crate::bytecode::detect_proxy(&bytes, &std::collections::BTreeMap::new());
    let cfg = crate::bytecode::lift_cfg(&bytes);

    Json(json!({
        "abi": abi,
        "proxy": proxy,
        "cfg": cfg,
    }))
}

#[derive(Deserialize)]
struct VerifyProveRequest {
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    source: Option<String>,
    invariant: Option<String>,
}

async fn api_verify_prove(
    State(_s): State<AppState>,
    Json(req): Json<VerifyProveRequest>,
) -> Json<Value> {
    let target = req
        .target
        .or(req.source)
        .unwrap_or_else(|| "contracts/Vault.sol".into());
    let path = std::path::Path::new(&target);
    match crate::verify::verify_contract(path, req.invariant.as_deref()) {
        Ok(rep) => Json(json!({ "success": true, "report": rep })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct VerifyFuzzRequest {
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    source: Option<String>,
    runs: Option<usize>,
}

async fn api_verify_fuzz(
    State(_s): State<AppState>,
    Json(req): Json<VerifyFuzzRequest>,
) -> Json<Value> {
    let target = req
        .target
        .or(req.source)
        .unwrap_or_else(|| "contracts/Vault.sol".into());
    let path = std::path::Path::new(&target);
    let runs = req.runs.unwrap_or(5000);
    match crate::verify::fuzz_contract(path, runs, None) {
        Ok(rep) => Json(json!({ "success": true, "report": rep })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct ReplEvalRequest {
    expr: String,
}

async fn api_repl_eval(
    State(_s): State<AppState>,
    Json(req): Json<ReplEvalRequest>,
) -> Json<Value> {
    match crate::repl::evaluate_expression(&req.expr) {
        Ok(res) => Json(json!({ "success": true, "result": res })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct MevSimRequest {
    steps: Option<usize>,
    initial_price: Option<f64>,
    volatility: Option<f64>,
}

async fn api_mev_sim(State(_s): State<AppState>, Json(req): Json<MevSimRequest>) -> Json<Value> {
    let cfg = crate::sim::SimMarketConfig {
        steps: req.steps.unwrap_or(100),
        initial_price: req.initial_price.unwrap_or(3000.0),
        volatility: req.volatility.unwrap_or(0.02),
        num_traders: 15,
        num_searchers: 3,
    };
    match crate::sim::run_simulation(&cfg) {
        Ok(res) => Json(json!({ "success": true, "result": res })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct AnalyticsQueryRequest {
    sql: String,
}

async fn api_analytics_query(
    State(s): State<AppState>,
    Json(req): Json<AnalyticsQueryRequest>,
) -> Json<Value> {
    match crate::analytics::execute_query(&req.sql, Some(&s.ws.data_dir())) {
        Ok(res) => Json(json!({ "success": true, "result": res })),
        Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
    }
}
