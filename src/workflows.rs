//! Multi-repo workflows: compile, audit, decompile, extract, and inspect.
//! Bridges Foundry, Solar, Heimdall, Cryo, Aderyn, and Alloy into unified operations.

use crate::chain;
use crate::tools;
use crate::ui;
use crate::workspace::Workspace;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

// ───────────────────────────── Compile Workflow ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub compiler: String,
    pub success: bool,
    pub output: String,
    pub artifacts: Vec<String>,
}

pub fn compile(ws: &Workspace, target: &Path, prefer_solar: bool) -> Result<CompileResult> {
    let has_solar = tools::resolve(ws, "solar").is_some();
    let has_forge = tools::resolve(ws, "forge").is_some();

    if prefer_solar && has_solar {
        ui::header(format!("Compiling {} with Solar (blazingly fast)", target.display()));
        let solar = tools::require(ws, "solar")?;
        let out = Command::new(solar)
            .arg(target)
            .arg("--emit")
            .arg("abi,bin,bin-runtime")
            .output()
            .context("failed to run solar")?;

        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}").trim().to_string();

        return Ok(CompileResult {
            compiler: "solar".into(),
            success: out.status.success(),
            output: combined,
            artifacts: vec![],
        });
    }

    if has_forge {
        ui::header(format!("Compiling {} with Forge", target.display()));
        let forge = tools::require(ws, "forge")?;
        let mut cmd = Command::new(forge);
        cmd.arg("build");
        if target.is_dir() {
            cmd.current_dir(target);
        }
        let out = cmd.output().context("failed to run forge")?;
        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ).trim().to_string();

        return Ok(CompileResult {
            compiler: "forge".into(),
            success: out.status.success(),
            output: combined,
            artifacts: vec![],
        });
    }

    bail!("neither `solar` nor `forge` is available. Build them with `para build`");
}

// ───────────────────────────── Audit Workflow ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub tool: String,
    pub summary: String,
    pub findings_count: usize,
    pub raw_output: String,
}

pub fn audit(ws: &Workspace, path: &Path) -> Result<AuditReport> {
    if let Some((aderyn, _)) = tools::resolve(ws, "aderyn") {
        ui::header(format!("Running Aderyn static security audit on {}", path.display()));
        let out = Command::new(aderyn)
            .arg("--root")
            .arg(path)
            .arg("--output")
            .arg("report.json")
            .output()
            .context("failed to run aderyn")?;

        let raw = format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        return Ok(AuditReport {
            tool: "aderyn".into(),
            summary: "Aderyn static analysis finished".into(),
            findings_count: 0,
            raw_output: raw,
        });
    }

    if let Some((solar, _)) = tools::resolve(ws, "solar") {
        ui::header(format!("Running Solar linter on {}", path.display()));
        let out = Command::new(solar)
            .arg(path)
            .arg("--stop-after")
            .arg("analysis")
            .output()
            .context("failed to run solar linter")?;

        let raw = format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        return Ok(AuditReport {
            tool: "solar-lint".into(),
            summary: "Solar lint analysis completed".into(),
            findings_count: 0,
            raw_output: raw,
        });
    }

    bail!("no audit tool available (build `aderyn` or `solar`)");
}

// ───────────────────────────── Decompile / Bytecode Inspection ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompileResult {
    pub address: Option<String>,
    pub bytecode_len: usize,
    pub source: String,
    pub disassembly: Option<String>,
}

pub async fn decompile(ws: &Workspace, rpc_url: Option<&str>, target: &str) -> Result<DecompileResult> {
    let bytecode: String = if target.starts_with("0x") && target.len() == 42 {
        let rpc = crate::rpc::resolve(ws, rpc_url)?;
        ui::info(format!("Fetching onchain bytecode for {target} via {}", rpc.url));
        chain::code(&rpc.url, target).await?
    } else if target.starts_with("0x") || target.chars().all(|c| c.is_ascii_hexdigit()) {
        target.to_string()
    } else {
        std::fs::read_to_string(target).with_context(|| format!("reading bytecode file {target}"))?
    };

    let clean_hex = bytecode.trim().trim_start_matches("0x");
    let byte_len = clean_hex.len() / 2;

    if byte_len == 0 {
        bail!("empty bytecode for target `{target}`");
    }

    let heimdall = tools::resolve(ws, "heimdall");
    let mut decompiled = String::new();
    let mut disasm = None;

    if let Some((h_bin, _)) = heimdall {
        ui::header(format!("Decompiling {byte_len} bytes using Heimdall"));
        let out = Command::new(&h_bin)
            .args(["decompile", clean_hex, "--default"])
            .output();

        if let Ok(o) = out {
            decompiled = String::from_utf8_lossy(&o.stdout).to_string();
        }

        let dis = Command::new(&h_bin)
            .args(["disassemble", clean_hex])
            .output();
        if let Ok(d) = dis {
            disasm = Some(String::from_utf8_lossy(&d.stdout).to_string());
        }
    } else if let Some((cast, _)) = tools::resolve(ws, "cast") {
        ui::header(format!("Disassembling {byte_len} bytes using Cast"));
        let dis = Command::new(cast)
            .args(["disassemble", clean_hex])
            .output();
        if let Ok(d) = dis {
            disasm = Some(String::from_utf8_lossy(&d.stdout).to_string());
        }
        decompiled = "// Heimdall not built yet. Disassembly provided via Cast.\n// Build heimdall with `para build heimdall` for full Solidity recovery.\n".into();
    } else {
        decompiled = format!("// Bytecode length: {} bytes\n// Build heimdall with `para build heimdall` to decompile.\n", byte_len);
    }

    Ok(DecompileResult {
        address: (target.len() == 42).then(|| target.to_string()),
        bytecode_len: byte_len,
        source: decompiled,
        disassembly: disasm,
    })
}

// ───────────────────────────── Data Extraction (Cryo) ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractOpts {
    pub datatype: String,
    pub blocks: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub format: Option<String>,
}

pub fn extract(ws: &Workspace, opts: &ExtractOpts) -> Result<String> {
    let cryo = tools::require(ws, "cryo")?;
    let rpc = crate::rpc::resolve(ws, opts.rpc_url.as_deref())?;
    let out_dir = opts.output_dir.clone().unwrap_or_else(|| ws.data_dir().join("cryo"));
    std::fs::create_dir_all(&out_dir)?;

    ui::header(format!("Extracting dataset `{}` via Cryo to {}", opts.datatype, out_dir.display()));

    let mut cmd = Command::new(cryo);
    cmd.arg(&opts.datatype)
        .arg("--rpc")
        .arg(&rpc.url)
        .arg("--output-dir")
        .arg(&out_dir);

    if let Some(blocks) = &opts.blocks {
        cmd.arg("-b").arg(blocks);
    }
    if let Some(fmt) = &opts.format {
        cmd.arg("--format").arg(fmt);
    }

    let output = cmd.output().context("failed to run cryo")?;
    let text = format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        bail!("cryo failed: {}", text.trim());
    }
    ui::ok("Extraction completed successfully");
    Ok(text)
}
