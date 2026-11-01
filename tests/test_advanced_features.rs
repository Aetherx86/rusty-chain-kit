//! Advanced Subsystem Test Suite:
//! - Multi-Hop Forensic Flow & Entity Classification
//! - Storage Variable Layout & Proxy Upgrade Collision Inspector
//! - Vulnerability Diagnosis & PoC Exploit Synthesizer
//! - Cross-Compiler Differential Benchmarking
//! - Automatic Foundry.toml Fallback Workspace Discovery
//! - CLI Parsing & Real-World Protocol Scenarios

use clap::Parser;
use rusty_chain_kit::bench::run_benchmark;
use rusty_chain_kit::cli::{Cli, Commands, FlowArgs, StorageArgs, CopilotArgs, BenchArgs, AuditArgs};
use rusty_chain_kit::copilot::{analyze_contract, VulnerabilityClass};
use rusty_chain_kit::forensics::{lookup_known_entity, trace_flow, EntityCategory, RiskLevel};
use rusty_chain_kit::storage::{compare_layouts, get_type_byte_size, parse_storage_layout_from_source, CollisionKind};
use rusty_chain_kit::workspace::Workspace;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// Forensics Flow & Entity Intelligence Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_forensics_flow_basic_graph() {
    let graph = trace_flow("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", 2).unwrap();
    assert_eq!(graph.subject, "0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
    assert_eq!(graph.max_hops, 2);
    assert!(graph.nodes.len() >= 5);
    assert!(graph.edges.len() >= 4);
    assert_eq!(graph.nodes[0].entity_type, EntityCategory::Subject);
    assert_eq!(graph.nodes[0].hop, 0);
}

#[test]
fn test_forensics_flow_sanctioned_mixer_detection() {
    // Tornado Cash Router address
    let tornado_addr = "0xd90e2f925DA726b50C4Ed8D0Fb90Ad053324F31b";
    let (cat, label, risk, sanctioned) = lookup_known_entity(tornado_addr);
    assert_eq!(cat, EntityCategory::Mixer);
    assert!(label.contains("Tornado"));
    assert_eq!(risk, RiskLevel::Severe);
    assert!(sanctioned);

    let graph = trace_flow(tornado_addr, 1).unwrap();
    assert_eq!(graph.overall_risk_score, 100);
    assert_eq!(graph.overall_risk_level, RiskLevel::Severe);
    assert!(graph.dossier.is_sanctioned);
    assert!(graph.dossier.flags.contains(&"SANCTIONED_ENTITY".to_string()));
}

#[test]
fn test_forensics_flow_hops_clamping() {
    let g_low = trace_flow("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", 0).unwrap();
    assert_eq!(g_low.max_hops, 1);

    let g_high = trace_flow("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", 10).unwrap();
    assert_eq!(g_high.max_hops, 4);
}

#[test]
fn test_forensics_known_entities_mapping() {
    // Uniswap V3
    let (cat_dex, _, risk_dex, sanc_dex) = lookup_known_entity("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45");
    assert_eq!(cat_dex, EntityCategory::Dex);
    assert_eq!(risk_dex, RiskLevel::Clean);
    assert!(!sanc_dex);

    // Arbitrum Bridge
    let (cat_bridge, _, _, _) = lookup_known_entity("0x4dbd4fc535bd2916850c95d2c0b02bc450ff48ff");
    assert_eq!(cat_bridge, EntityCategory::Bridge);

    // Binance Hot Wallet
    let (cat_cex, _, _, _) = lookup_known_entity("0x3f5ce5fbfe3e9af3971dd833d26ba9b5c936f0be");
    assert_eq!(cat_cex, EntityCategory::Exchange);

    // Aave V3 Pool
    let (cat_lend, _, _, _) = lookup_known_entity("0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2");
    assert_eq!(cat_lend, EntityCategory::Lending);
}

// ─────────────────────────────────────────────────────────────────────────────
// Storage Layout & Upgrade Collision Inspector Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_storage_type_byte_sizes() {
    assert_eq!(get_type_byte_size("bool"), 1);
    assert_eq!(get_type_byte_size("uint8"), 1);
    assert_eq!(get_type_byte_size("uint16"), 2);
    assert_eq!(get_type_byte_size("uint32"), 4);
    assert_eq!(get_type_byte_size("uint64"), 8);
    assert_eq!(get_type_byte_size("uint128"), 16);
    assert_eq!(get_type_byte_size("address"), 20);
    assert_eq!(get_type_byte_size("bytes32"), 32);
    assert_eq!(get_type_byte_size("uint256"), 32);
}

#[test]
fn test_storage_layout_basic_packing() {
    let src = r#"
contract PackedVault {
    bool public isPaused;        // 1 byte (slot 0, offset 0)
    uint8 public feePct;         // 1 byte (slot 0, offset 1)
    address public owner;        // 20 bytes (slot 0, offset 2)
    uint256 public totalAssets;  // 32 bytes (slot 1, offset 0)
}
"#;
    let layout = parse_storage_layout_from_source(src, Some("PackedVault"));
    assert_eq!(layout.contract_name, "PackedVault");
    assert_eq!(layout.slots.len(), 4);

    // First 3 items fit in slot 0 (1 + 1 + 20 = 22 bytes <= 32 bytes)
    assert_eq!(layout.slots[0].slot, 0);
    assert_eq!(layout.slots[0].offset, 0);
    assert_eq!(layout.slots[0].size_bytes, 1);

    assert_eq!(layout.slots[1].slot, 0);
    assert_eq!(layout.slots[1].offset, 1);
    assert_eq!(layout.slots[1].size_bytes, 1);

    assert_eq!(layout.slots[2].slot, 0);
    assert_eq!(layout.slots[2].offset, 2);
    assert_eq!(layout.slots[2].size_bytes, 20);

    // 4th item uint256 is in slot 1
    assert_eq!(layout.slots[3].slot, 1);
    assert_eq!(layout.slots[3].offset, 0);
    assert_eq!(layout.slots[3].size_bytes, 32);

    assert_eq!(layout.total_slots_used, 2);
}

#[test]
fn test_storage_layout_wasted_bytes_detection() {
    let src = r#"
contract UnpackedVault {
    bool public isPaused;        // 1 byte, slot 0
    uint256 public totalAssets;  // 32 bytes -> forces slot 1, wasting 31 bytes in slot 0!
    uint8 public feePct;         // 1 byte, slot 2
}
"#;
    let layout = parse_storage_layout_from_source(src, Some("UnpackedVault"));
    assert_eq!(layout.total_slots_used, 3);
    assert!(layout.wasted_bytes_in_packed_slots >= 31);
}

#[test]
fn test_storage_upgrade_safe_append() {
    let v1 = r#"
contract VaultV1 {
    address public owner;
    uint256 public totalAssets;
}
"#;
    let v2 = r#"
contract VaultV2 {
    address public owner;
    uint256 public totalAssets;
    uint256 public newProtocolFee; // Appended at end
}
"#;
    let tmp = tempfile::tempdir().unwrap();
    let p1 = tmp.path().join("VaultV1.sol");
    let p2 = tmp.path().join("VaultV2.sol");
    std::fs::write(&p1, v1).unwrap();
    std::fs::write(&p2, v2).unwrap();

    let rep = compare_layouts(&p1, &p2).unwrap();
    assert!(rep.is_safe_to_upgrade);
    assert!(rep.collisions.is_empty());
}

#[test]
fn test_storage_upgrade_collision_slot_shift() {
    let v1 = r#"
contract TokenV1 {
    uint256 public totalSupply;
    mapping(address => uint256) public balances;
}
"#;
    let v2 = r#"
contract TokenV2 {
    address public feeRecipient; // Inserted at slot 0 -> shifts totalSupply to slot 1!
    uint256 public totalSupply;
    mapping(address => uint256) public balances;
}
"#;
    let tmp = tempfile::tempdir().unwrap();
    let p1 = tmp.path().join("TokenV1.sol");
    let p2 = tmp.path().join("TokenV2.sol");
    std::fs::write(&p1, v1).unwrap();
    std::fs::write(&p2, v2).unwrap();

    let rep = compare_layouts(&p1, &p2).unwrap();
    assert!(!rep.is_safe_to_upgrade);
    assert!(!rep.collisions.is_empty());

    let has_slot_shift = rep.collisions.iter().any(|c| matches!(c, CollisionKind::SlotShift { .. }));
    assert!(has_slot_shift);
}

#[test]
fn test_storage_upgrade_collision_type_mutation() {
    let v1 = r#"
contract PoolV1 {
    uint256 public rate;
}
"#;
    let v2 = r#"
contract PoolV2 {
    address public rate; // Type mutated in slot 0!
}
"#;
    let tmp = tempfile::tempdir().unwrap();
    let p1 = tmp.path().join("PoolV1.sol");
    let p2 = tmp.path().join("PoolV2.sol");
    std::fs::write(&p1, v1).unwrap();
    std::fs::write(&p2, v2).unwrap();

    let rep = compare_layouts(&p1, &p2).unwrap();
    assert!(!rep.is_safe_to_upgrade);

    let has_type_mutation = rep.collisions.iter().any(|c| matches!(c, CollisionKind::TypeMutation { .. }));
    assert!(has_type_mutation);
}

// ─────────────────────────────────────────────────────────────────────────────
// Copilot Vulnerability Diagnosis & PoC Synthesizer Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_copilot_reentrancy_detection() {
    let code = r#"
contract VulnerableBank {
    mapping(address => uint256) public balances;

    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount);
        (bool ok, ) = msg.sender.call{value: amount}("");
        require(ok);
        balances[msg.sender] -= amount; // Reentrancy! State updated AFTER external call
    }
}
"#;
    let rep = analyze_contract(Path::new(code)).unwrap();
    assert!(rep.total_findings >= 1);
    assert!(rep.critical_count >= 1);

    let reentrancy_finding = rep.findings.iter().find(|f| f.vulnerability_class == VulnerabilityClass::Reentrancy);
    assert!(reentrancy_finding.is_some());

    let f = reentrancy_finding.unwrap();
    assert_eq!(f.severity, "Critical");
    assert!(f.poc_test_code.contains("contract ExploitReentrancyTest"));
    assert!(f.poc_test_code.contains("testExploitReentrancy"));
}

#[test]
fn test_copilot_oracle_manipulation_detection() {
    let code = r#"
contract LendingProtocol {
    function getCollateralPrice() public view returns (uint256) {
        (uint112 reserve0, uint112 reserve1, ) = pool.getReserves();
        return (reserve1 * 1e18) / reserve0; // Spot reserve manipulation!
    }
}
"#;
    let rep = analyze_contract(Path::new(code)).unwrap();
    let oracle_finding = rep.findings.iter().find(|f| f.vulnerability_class == VulnerabilityClass::OraclePriceManipulation);
    assert!(oracle_finding.is_some());
}

#[test]
fn test_copilot_access_control_fallback() {
    let code = r#"
contract SafeVault {
    uint256 public totalAssets;
}
"#;
    let rep = analyze_contract(Path::new(code)).unwrap();
    assert_eq!(rep.total_findings, 1);
    assert_eq!(rep.findings[0].vulnerability_class, VulnerabilityClass::MissingAccessControl);
}

// ─────────────────────────────────────────────────────────────────────────────
// Differential Benchmarking Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_bench_differential_optimization_profile() {
    let code = r#"
contract OptimizedSwap {
    function deposit(uint256 amount) external {}
    function withdraw(uint256 amount) external {}
    function transfer(address to, uint256 amount) external {}
}
"#;
    let rep = run_benchmark(Path::new(code)).unwrap();
    assert!(rep.bytecode_size_delta_bytes < 0);
    assert!(rep.bytecode_size_reduction_pct > 0.0);
    assert!(rep.average_gas_savings_pct > 0.0);
    assert!(rep.compile_time_overhead_ms > 0);
    assert!(rep.analysis_notes.len() >= 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Foundry.toml Fallback Workspace Discovery
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_workspace_foundry_toml_fallback_discovery() {
    let tmp = tempfile::tempdir().unwrap();
    let foundry_toml = tmp.path().join("foundry.toml");
    std::fs::write(&foundry_toml, "[profile.default]\nsrc = 'src'\nout = 'out'\n").unwrap();

    let ws = Workspace::from_foundry_toml(&foundry_toml).unwrap();
    assert_eq!(ws.manifest_path, foundry_toml.canonicalize().unwrap());
    assert_eq!(ws.root, tmp.path().canonicalize().unwrap());
    assert!(ws.state.join("bin").is_dir());
}

// ─────────────────────────────────────────────────────────────────────────────
// CLI Parser Tests for New Commands
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_cli_flow_parser() {
    let cli = Cli::try_parse_from([
        "para", "flow", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "--hops", "3", "--json"
    ]).unwrap();

    match cli.command.unwrap() {
        Commands::Flow(FlowArgs { address, hops, json, export_csv }) => {
            assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
            assert_eq!(hops, 3);
            assert!(json);
            assert!(export_csv.is_none());
        }
        _ => panic!("Expected flow command"),
    }
}

#[test]
fn test_cli_storage_inspect_and_compare() {
    let cli_inspect = Cli::try_parse_from(["para", "storage", "contracts/Vault.sol", "--json"]).unwrap();
    match cli_inspect.command.unwrap() {
        Commands::Storage(StorageArgs { contract_a, contract_b, json }) => {
            assert_eq!(contract_a.to_str().unwrap(), "contracts/Vault.sol");
            assert!(contract_b.is_none());
            assert!(json);
        }
        _ => panic!("Expected storage command"),
    }

    let cli_diff = Cli::try_parse_from(["para", "storage", "VaultV1.sol", "VaultV2.sol"]).unwrap();
    match cli_diff.command.unwrap() {
        Commands::Storage(StorageArgs { contract_a, contract_b, .. }) => {
            assert_eq!(contract_a.to_str().unwrap(), "VaultV1.sol");
            assert_eq!(contract_b.unwrap().to_str().unwrap(), "VaultV2.sol");
        }
        _ => panic!("Expected storage diff command"),
    }
}

#[test]
fn test_cli_copilot_parser() {
    let cli = Cli::try_parse_from(["para", "copilot", "contracts/Vault.sol", "--out", "exploit.sol", "--json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Copilot(CopilotArgs { target, out, json }) => {
            assert_eq!(target.to_str().unwrap(), "contracts/Vault.sol");
            assert_eq!(out.unwrap().to_str().unwrap(), "exploit.sol");
            assert!(json);
        }
        _ => panic!("Expected copilot command"),
    }
}

#[test]
fn test_cli_bench_parser() {
    let cli = Cli::try_parse_from(["para", "bench", "contracts/Vault.sol", "--json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Bench(BenchArgs { target, json }) => {
            assert_eq!(target.to_str().unwrap(), "contracts/Vault.sol");
            assert!(json);
        }
        _ => panic!("Expected bench command"),
    }
}

#[test]
fn test_cli_audit_with_poc_and_explain_flags() {
    let cli = Cli::try_parse_from(["para", "audit", "contracts/", "--poc", "--explain"]).unwrap();
    match cli.command.unwrap() {
        Commands::Audit(AuditArgs { path, poc, explain, .. }) => {
            assert_eq!(path.to_str().unwrap(), "contracts/");
            assert!(poc);
            assert!(explain);
        }
        _ => panic!("Expected audit with poc/explain"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Real-World Protocol Scenarios
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_real_world_amm_constant_product_invariant() {
    let reserve_x = 1000.0f64;
    let reserve_y = 3_000_000.0f64;
    let k = reserve_x * reserve_y;

    let dx = 10.0f64; // Inflow 10 ETH
    let dy = (reserve_y * dx) / (reserve_x + dx); // Outflow USD

    let new_x = reserve_x + dx;
    let new_y = reserve_y - dy;
    let new_k = new_x * new_y;

    assert!((new_k - k).abs() < 1e-4, "Constant product invariant violated!");
    assert!(new_y < reserve_y);
}

#[test]
fn test_real_world_lending_health_factor_and_liquidation() {
    let collateral_eth = 10.0f64;
    let eth_price = 3000.0f64;
    let liquidation_threshold = 0.80f64; // 80% LTV
    let borrowed_usd = 25000.0f64;

    let collateral_usd = collateral_eth * eth_price;
    let health_factor = (collateral_usd * liquidation_threshold) / borrowed_usd;

    // 10 * 3000 * 0.8 / 25000 = 24000 / 25000 = 0.96 (< 1.0 -> Liquidatable!)
    assert!(health_factor < 1.0, "Position should be eligible for liquidation");

    // Liquidation bonus of 5%
    let bonus = 1.05f64;
    let debt_to_cover = 10000.0f64;
    let collateral_seized_usd = debt_to_cover * bonus;
    let collateral_seized_eth = collateral_seized_usd / eth_price;
    assert_eq!(collateral_seized_eth, 3.5);
}
