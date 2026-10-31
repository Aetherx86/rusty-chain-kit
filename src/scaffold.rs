//! Project scaffolding: `para new bot`, `para new contract`, `para new script`.

use crate::ui;
use crate::workspace::Workspace;
use anyhow::{Result, bail};
use std::fs;
use std::path::PathBuf;

pub fn new_bot(ws: &Workspace, name: &str, target_dir: Option<PathBuf>) -> Result<PathBuf> {
    let out = target_dir.unwrap_or_else(|| ws.root.join(name));
    if out.exists() {
        bail!("directory `{}` already exists", out.display());
    }

    ui::header(format!("Scaffolding Artemis-style bot `{name}` (powered by alloy)"));
    fs::create_dir_all(out.join("src"))?;

    // Determine alloy dependency path or crates.io version
    let alloy_rel = pathdiff::diff_paths(ws.repo_dir(ws.repo("alloy")?), &out)
        .map(|p| format!("path = \"{}/crates/alloy\"", p.display()))
        .unwrap_or_else(|| "version = \"0.1\"".into());

    let cargo_toml = include_str!("../templates/bot/Cargo.toml.tmpl")
        .replace("{{name}}", name)
        .replace("{{alloy_dep}}", &alloy_rel);

    fs::write(out.join("Cargo.toml"), cargo_toml)?;
    fs::write(out.join("README.md"), include_str!("../templates/bot/README.md").replace("{{name}}", name))?;
    fs::write(out.join("src/main.rs"), include_str!("../templates/bot/src/main.rs").replace("{{name}}", name))?;
    fs::write(out.join("src/engine.rs"), include_str!("../templates/bot/src/engine.rs"))?;
    fs::write(out.join("src/strategy.rs"), include_str!("../templates/bot/src/strategy.rs"))?;

    ui::ok(format!("Created bot at {}", out.display()));
    ui::info("Next steps:");
    ui::info(format!("  cd {}", out.display()));
    ui::info("  para devnet up          # start local chain");
    ui::info("  cargo run               # run in dry-run mode");
    Ok(out)
}

pub fn new_script(ws: &Workspace, name: &str, target_dir: Option<PathBuf>) -> Result<PathBuf> {
    let out = target_dir.unwrap_or_else(|| ws.root.join(name));
    if out.exists() {
        bail!("directory `{}` already exists", out.display());
    }

    ui::header(format!("Scaffolding viem TypeScript script `{name}`"));
    fs::create_dir_all(out.join("src"))?;

    let viem_ver = "2.21.0";
    let pkg_json = include_str!("../templates/script/package.json.tmpl")
        .replace("{{name}}", name)
        .replace("{{viem_version}}", viem_ver);

    fs::write(out.join("package.json"), pkg_json)?;
    fs::write(out.join("tsconfig.json"), include_str!("../templates/script/tsconfig.json"))?;
    fs::write(out.join("src/index.ts"), include_str!("../templates/script/src/index.ts").replace("{{name}}", name))?;

    ui::ok(format!("Created viem script at {}", out.display()));
    ui::info("Next steps:");
    ui::info(format!("  cd {}", out.display()));
    ui::info("  npm install");
    ui::info("  npm start");
    Ok(out)
}

pub fn new_contract(ws: &Workspace, name: &str, target_dir: Option<PathBuf>) -> Result<PathBuf> {
    let out = target_dir.unwrap_or_else(|| ws.root.join(name));
    if out.exists() {
        bail!("directory `{}` already exists", out.display());
    }

    ui::header(format!("Scaffolding Solidity project `{name}` (Foundry + Solar)"));
    fs::create_dir_all(out.join("src"))?;
    fs::create_dir_all(out.join("test"))?;
    fs::create_dir_all(out.join("script"))?;

    let foundry_toml = r#"[profile.default]
src = "src"
out = "out"
libs = ["lib"]
solc_version = "0.8.28"

# Configure solar as the alternative fast compiler:
# solar = true
"#;
    fs::write(out.join("foundry.toml"), foundry_toml)?;

    let contract_sol = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

contract {name} {{
    uint256 public value;
    address public owner;

    event ValueChanged(uint256 indexed oldValue, uint256 indexed newValue);

    constructor(uint256 initialValue) {{
        owner = msg.sender;
        value = initialValue;
    }}

    function setValue(uint256 newValue) external {{
        require(msg.sender == owner, "Only owner");
        emit ValueChanged(value, newValue);
        value = newValue;
    }}
}}
"#
    );
    fs::write(out.join(format!("src/{name}.sol")), contract_sol)?;

    let test_sol = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {{Test, console}} from "forge-std/Test.sol";
import {{"../src/{name}.sol"}};

contract {name}Test is Test {{
    {name} public instance;

    function setUp() public {{
        instance = new {name}(42);
    }}

    function test_InitialValue() public view {{
        assertEq(instance.value(), 42);
    }}

    function test_SetValue() public {{
        instance.setValue(100);
        assertEq(instance.value(), 100);
    }}

    function testRevert_SetValueUnauthorized() public {{
        vm.prank(address(0xdead));
        vm.expectRevert("Only owner");
        instance.setValue(999);
    }}
}}
"#
    );
    fs::write(out.join(format!("test/{name}.t.sol")), test_sol)?;

    ui::ok(format!("Created contract project at {}", out.display()));
    ui::info("Next steps:");
    ui::info(format!("  cd {}", out.display()));
    ui::info("  para solar src/         # fast compile with Solar");
    ui::info("  para forge test         # run tests with Foundry");
    Ok(out)
}
