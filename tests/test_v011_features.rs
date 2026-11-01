use alloy::primitives::{Address, B256, Bytes, U256};
use clap::Parser;
use rusty_chain_kit::aa;
use rusty_chain_kit::analytics;
use rusty_chain_kit::bytecode;
use rusty_chain_kit::cli::{AaArgs, AaSubcommand, ChainArgs, ChainSubcommand, Cli, Commands, MevArgs, MevSubcommand, NewArgs, NewKind, ProfileArgs, QueryArgs, ReplArgs, RpcArgs, RpcSubcommand, SimArgs, TraceArgs, VerifyArgs, ZkArgs, ZkSubcommand};
use rusty_chain_kit::mev;
use rusty_chain_kit::repl;
use rusty_chain_kit::sim;
use rusty_chain_kit::trace;
use rusty_chain_kit::trustless;
use rusty_chain_kit::verify;
use rusty_chain_kit::zk;
use std::collections::BTreeMap;
use std::path::PathBuf;

// ───────────────────────────── 1. Account Abstraction & EIP-7702 ─────────────────────────────

#[test]
fn test_aa_generate_auth_valid() {
    let target = Address::repeat_byte(0x42);
    // Well-known 32-byte test private key
    let priv_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let tuple = aa::generate_auth(1, target, 0, priv_key).unwrap();

    assert_eq!(tuple.chain_id, 1);
    assert_eq!(tuple.address, target);
    assert_eq!(tuple.nonce, 0);
    assert!(tuple.y_parity <= 1);
    assert!(!tuple.r.is_zero());
    assert!(!tuple.s.is_zero());
    // Anvil account 0 address for ac0974bec39a...
    assert_eq!(
        format!("{:#x}", tuple.authority).to_lowercase(),
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
    );
}

#[test]
fn test_aa_generate_auth_invalid_key_length() {
    let target = Address::repeat_byte(0x42);
    let short_key = "0x1234";
    let res = aa::generate_auth(1, target, 0, short_key);
    assert!(res.is_err());
}

#[test]
fn test_aa_simulate_bundle_valid() {
    let ops = vec![aa::UserOperation {
        sender: Address::repeat_byte(0x11),
        nonce: U256::ZERO,
        init_code: Bytes::new(),
        call_data: Bytes::from_static(&[0xa9, 0x05, 0x9c, 0xbb]),
        call_gas_limit: U256::from(100_000),
        verification_gas_limit: U256::from(50_000),
        pre_verification_gas: U256::from(21_000),
        max_fee_per_gas: U256::from(30_000_000_000u64),
        max_priority_fee_per_gas: U256::from(1_500_000_000u64),
        paymaster_and_data: Bytes::from_static(&[0x22; 20]),
        signature: Bytes::from_static(&[0xaa; 65]),
    }];
    let res = aa::simulate_bundle(&ops);
    assert_eq!(res.len(), 1);
    assert!(res[0].valid);
    assert_eq!(res[0].total_gas_limit, U256::from(171_000));
    assert!(res[0].paymaster_address.is_some());
}

#[test]
fn test_aa_simulate_bundle_low_verification_gas() {
    let ops = vec![aa::UserOperation {
        sender: Address::repeat_byte(0x11),
        nonce: U256::ZERO,
        init_code: Bytes::new(),
        call_data: Bytes::new(),
        call_gas_limit: U256::from(50_000),
        verification_gas_limit: U256::from(5_000), // < 10,000 threshold
        pre_verification_gas: U256::from(21_000),
        max_fee_per_gas: U256::from(30_000_000_000u64),
        max_priority_fee_per_gas: U256::from(1_500_000_000u64),
        paymaster_and_data: Bytes::new(),
        signature: Bytes::from_static(&[0xaa; 65]),
    }];
    let res = aa::simulate_bundle(&ops);
    assert!(!res[0].valid);
    assert!(res[0].validation_notes.iter().any(|n| n.contains("dangerously low")));
}

#[test]
fn test_aa_simulate_bundle_fee_mismatch() {
    let ops = vec![aa::UserOperation {
        sender: Address::repeat_byte(0x11),
        nonce: U256::ZERO,
        init_code: Bytes::new(),
        call_data: Bytes::new(),
        call_gas_limit: U256::from(50_000),
        verification_gas_limit: U256::from(50_000),
        pre_verification_gas: U256::from(21_000),
        max_fee_per_gas: U256::from(1_000_000_000u64),
        max_priority_fee_per_gas: U256::from(2_000_000_000u64), // priority > max
        paymaster_and_data: Bytes::new(),
        signature: Bytes::from_static(&[0xaa; 65]),
    }];
    let res = aa::simulate_bundle(&ops);
    assert!(!res[0].valid);
    assert!(res[0].validation_notes.iter().any(|n| n.contains("cannot be less than")));
}

#[test]
fn test_aa_decode_tx_type4_eip7702() {
    let raw = vec![0x04, 0x01, 0x02, 0x03];
    let decoded = aa::decode_aa_tx(&raw).unwrap();
    assert!(decoded.is_eip7702);
    assert_eq!(decoded.tx_type, "EIP-7702 (Type 4)");
}

#[test]
fn test_aa_decode_tx_erc4337_handle_ops() {
    let mut raw = vec![0x00; 10];
    // handleOps selector: 0x1fad948c
    raw.extend_from_slice(&[0x1f, 0xad, 0x94, 0x8c]);
    raw.extend_from_slice(&[0x00; 64]);
    let decoded = aa::decode_aa_tx(&raw).unwrap();
    assert!(decoded.is_erc4337_handle_ops);
    assert!(decoded.tx_type.contains("ERC-4337"));
}

#[test]
fn test_aa_decode_tx_empty_error() {
    assert!(aa::decode_aa_tx(&[]).is_err());
}

// ───────────────────────────── 2. Gas Profiling & Revert Autopsy ─────────────────────────────

#[test]
fn test_trace_profile_opcodes_aggregation() {
    let struct_logs = vec![
        serde_json::json!({ "op": "PUSH1", "gasCost": 3 }),
        serde_json::json!({ "op": "PUSH1", "gasCost": 3 }),
        serde_json::json!({ "op": "SLOAD", "gasCost": 2100 }),
        serde_json::json!({ "op": "SSTORE", "gasCost": 20000 }),
    ];
    let (profiles, root) = trace::profile_opcodes(&struct_logs);
    assert_eq!(profiles.len(), 3);
    assert_eq!(profiles[0].opcode, "SSTORE"); // highest gas first
    assert_eq!(profiles[0].total_gas, 20000);
    assert_eq!(profiles[1].opcode, "SLOAD");
    assert_eq!(profiles[2].opcode, "PUSH1");
    assert_eq!(profiles[2].count, 2);
    assert_eq!(root.value, 22106);
}

#[test]
fn test_trace_profile_flamegraph_structure() {
    let struct_logs = vec![
        serde_json::json!({ "op": "ADD", "gasCost": 3 }),
        serde_json::json!({ "op": "KECCAK256", "gasCost": 42 }),
    ];
    let (_, root) = trace::profile_opcodes(&struct_logs);
    assert_eq!(root.name, "root_execution");
    assert_eq!(root.children.len(), 2);
    assert!(root.children.iter().any(|c| c.name.starts_with("KECCAK256")));
}

#[test]
fn test_autopsy_empty_revert() {
    let rep = trace::autopsy_revert("", 21000);
    assert!(rep.reverted);
    assert_eq!(rep.error_type, "EmptyRevert");
    assert!(rep.panic_code.is_none());
}

#[test]
fn test_autopsy_require_string() {
    // Error(string) selector: 0x08c379a0
    // "Unauthorized"
    let mut hex_data = String::from("08c379a0");
    // offset 32
    hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000020");
    // length 12
    hex_data.push_str("000000000000000000000000000000000000000000000000000000000000000c");
    // "Unauthorized" (12 bytes) + 20 bytes padding
    hex_data.push_str("556e617574686f72697a65640000000000000000000000000000000000000000");

    let rep = trace::autopsy_revert(&hex_data, 25000);
    assert!(rep.reverted);
    assert_eq!(rep.error_type, "RequireString");
    assert_eq!(rep.message, Some("Unauthorized".into()));
}

#[test]
fn test_autopsy_panic_assertion() {
    // Panic(0x01)
    let panic_01 = "4e487b710000000000000000000000000000000000000000000000000000000000000001";
    let rep = trace::autopsy_revert(panic_01, 30000);
    assert_eq!(rep.error_type, "SolidityPanic");
    assert_eq!(rep.panic_code, Some(0x01));
    assert!(rep.error_name.contains("Assertion Violation"));
}

#[test]
fn test_autopsy_panic_underflow_overflow() {
    // Panic(0x11)
    let panic_11 = "4e487b710000000000000000000000000000000000000000000000000000000000000011";
    let rep = trace::autopsy_revert(panic_11, 30000);
    assert_eq!(rep.panic_code, Some(0x11));
    assert!(rep.error_name.contains("Arithmetic Overflow/Underflow"));
}

#[test]
fn test_autopsy_panic_division_by_zero() {
    // Panic(0x12)
    let panic_12 = "4e487b710000000000000000000000000000000000000000000000000000000000000012";
    let rep = trace::autopsy_revert(panic_12, 30000);
    assert_eq!(rep.panic_code, Some(0x12));
    assert!(rep.error_name.contains("Division By Zero"));
}

#[test]
fn test_autopsy_panic_array_out_of_bounds() {
    // Panic(0x32)
    let panic_32 = "4e487b710000000000000000000000000000000000000000000000000000000000000032";
    let rep = trace::autopsy_revert(panic_32, 30000);
    assert_eq!(rep.panic_code, Some(0x32));
    assert!(rep.error_name.contains("Array Out of Bounds"));
}

#[test]
fn test_autopsy_panic_pop_empty() {
    // Panic(0x31)
    let panic_31 = "4e487b710000000000000000000000000000000000000000000000000000000000000031";
    let rep = trace::autopsy_revert(panic_31, 30000);
    assert_eq!(rep.panic_code, Some(0x31));
    assert!(rep.error_name.contains("Pop on Empty Array"));
}

#[test]
fn test_autopsy_custom_error_selector() {
    let custom = "118cdaa7000000000000000000000000000000000000000000000000000000000000002a";
    let rep = trace::autopsy_revert(custom, 45000);
    assert_eq!(rep.error_type, "CustomError");
    assert!(rep.error_name.contains("0x118cdaa7"));
}

// ───────────────────────────── 3. Bytecode Intelligence & Proxy Detection ─────────────────────────────

#[test]
fn test_bytecode_extract_abi_empty() {
    let res = bytecode::extract_abi(&[]);
    assert!(res.is_empty());
}

#[test]
fn test_bytecode_extract_abi_selectors() {
    // Sample bytecode with dispatch for 0x2125b65b and 0xb69ef8a8
    let code = alloy::hex::decode(
        "6080604052348015600e575f80fd5b50600436106030575f3560e01c80632125b65b146034578063b69ef8a8146044575b5f80fd5b6044603f3660046046565b505050565b005b5f805f606084860312156057575f80fd5b833563ffffffff811681146069575f80fd5b925060208401356001600160a01b03811681146083575f80fd5b915060408401356001600160e01b0381168114609d575f80fd5b80915050925092509256"
    ).unwrap();
    let abi = bytecode::extract_abi(&code);
    assert_eq!(abi.len(), 2);
    assert!(abi.iter().any(|f| f.selector == "0x2125b65b"));
    assert!(abi.iter().any(|f| f.selector == "0xb69ef8a8"));
}

#[test]
fn test_bytecode_detect_proxy_eip1167_minimal_clone() {
    // 363d3d373d3d3d363d73 <20B address> 5af43d82803e903d91602b57fd5bf3
    let mut code = alloy::hex::decode("363d3d373d3d3d363d73").unwrap();
    code.extend_from_slice(&[0x11; 20]);
    code.extend_from_slice(&alloy::hex::decode("5af43d82803e903d91602b57fd5bf3").unwrap());

    let rep = bytecode::detect_proxy(&code, &BTreeMap::new());
    assert!(rep.is_proxy);
    assert_eq!(rep.kind, bytecode::ProxyKind::Eip1167MinimalClone);
    assert_eq!(rep.implementation, Some(Address::repeat_byte(0x11)));
}

#[test]
fn test_bytecode_detect_proxy_eip7702_delegation() {
    let mut code = vec![0xef, 0x01, 0x00];
    code.extend_from_slice(&[0x77; 20]);

    let rep = bytecode::detect_proxy(&code, &BTreeMap::new());
    assert!(rep.is_proxy);
    assert_eq!(rep.kind, bytecode::ProxyKind::Eip7702Delegated);
    assert_eq!(rep.implementation, Some(Address::repeat_byte(0x77)));
}

#[test]
fn test_bytecode_detect_proxy_eip1967_direct() {
    let mut slots = BTreeMap::new();
    let impl_slot = B256::from([
        0x36, 0x08, 0x94, 0xa1, 0x3b, 0xa1, 0xa3, 0x21,
        0x06, 0x67, 0xc8, 0x28, 0x49, 0x2d, 0xb9, 0x8d,
        0xca, 0x3e, 0x20, 0x76, 0xcc, 0x37, 0x35, 0xa9,
        0x20, 0xa3, 0xca, 0x50, 0x5d, 0x38, 0x2b, 0xbc,
    ]);
    let mut val = [0u8; 32];
    val[12..32].copy_from_slice(&[0x99; 20]);
    slots.insert(impl_slot, B256::from(val));

    let rep = bytecode::detect_proxy(&[0x60, 0x80], &slots);
    assert!(rep.is_proxy);
    assert_eq!(rep.kind, bytecode::ProxyKind::Eip1967Direct);
    assert_eq!(rep.implementation, Some(Address::repeat_byte(0x99)));
}

#[test]
fn test_bytecode_detect_proxy_eip1967_beacon() {
    let mut slots = BTreeMap::new();
    let beacon_slot = B256::from([
        0xa3, 0xf0, 0xad, 0x74, 0xe5, 0x42, 0x3a, 0xeb,
        0x0d, 0x70, 0x4f, 0xfc, 0x64, 0x60, 0x99, 0x06,
        0xc7, 0x47, 0x94, 0x86, 0xc4, 0xf0, 0x68, 0xc8,
        0xe8, 0xd3, 0x5e, 0x4e, 0x76, 0x67, 0x25, 0x33,
    ]);
    let mut val = [0u8; 32];
    val[12..32].copy_from_slice(&[0xbb; 20]);
    slots.insert(beacon_slot, B256::from(val));

    let rep = bytecode::detect_proxy(&[0x60, 0x80], &slots);
    assert!(rep.is_proxy);
    assert_eq!(rep.kind, bytecode::ProxyKind::Eip1967Beacon);
    assert_eq!(rep.beacon, Some(Address::repeat_byte(0xbb)));
}

#[test]
fn test_bytecode_detect_proxy_eip1822_uups() {
    let mut slots = BTreeMap::new();
    let uups_slot = B256::from([
        0xc5, 0xf1, 0x68, 0x36, 0x61, 0x47, 0x64, 0x63,
        0x97, 0x35, 0x77, 0x94, 0x1d, 0xff, 0x67, 0x90,
        0x28, 0xe0, 0x07, 0x97, 0x35, 0x69, 0x72, 0xa1,
        0xa8, 0x42, 0x45, 0x9d, 0x29, 0x0b, 0xa4, 0x9c,
    ]);
    let mut val = [0u8; 32];
    val[12..32].copy_from_slice(&[0xcc; 20]);
    slots.insert(uups_slot, B256::from(val));

    let rep = bytecode::detect_proxy(&[0x60, 0x80], &slots);
    assert!(rep.is_proxy);
    assert_eq!(rep.kind, bytecode::ProxyKind::Eip1822Uups);
    assert_eq!(rep.implementation, Some(Address::repeat_byte(0xcc)));
}

#[test]
fn test_bytecode_detect_proxy_diamond_eip2535() {
    // facets() selector: 0x7a0ed627
    let mut code = vec![0x60, 0x80];
    code.extend_from_slice(&[0x7a, 0x0e, 0xd6, 0x27]);
    let rep = bytecode::detect_proxy(&code, &BTreeMap::new());
    assert!(rep.is_proxy);
    assert_eq!(rep.kind, bytecode::ProxyKind::DiamondEip2535);
}

#[test]
fn test_bytecode_lift_cfg_basic_blocks() {
    // PUSH1 0x80, PUSH1 0x40, MSTORE, STOP
    let code = vec![0x60, 0x80, 0x60, 0x40, 0x52, 0x00];
    let cfg = bytecode::lift_cfg(&code);
    assert_eq!(cfg.total_instructions, 4);
    assert_eq!(cfg.blocks.len(), 1);
    assert!(cfg.blocks[0].is_terminal);
}

#[test]
fn test_bytecode_lift_cfg_push_data_handling() {
    // PUSH2 0x1234, PUSH1 0x00, JUMPDEST, RETURN
    let code = vec![0x61, 0x12, 0x34, 0x60, 0x00, 0x5b, 0xf3];
    let cfg = bytecode::lift_cfg(&code);
    assert_eq!(cfg.jump_destinations, vec![5]);
    assert_eq!(cfg.terminal_blocks, 1);
}

// ───────────────────────────── 4. Analytics Query Engine ─────────────────────────────

#[test]
fn test_analytics_query_select_blocks() {
    let q = "SELECT * FROM blocks";
    let res = analytics::execute_query(q, None).unwrap();
    assert_eq!(res.columns.len(), 5);
    assert_eq!(res.row_count, 4);
}

#[test]
fn test_analytics_query_where_filtering() {
    let q = "SELECT number, gas_used FROM blocks WHERE gas_used > 15000000";
    let res = analytics::execute_query(q, None).unwrap();
    assert_eq!(res.row_count, 2);
}

#[test]
fn test_analytics_query_limit() {
    let q = "SELECT * FROM blocks LIMIT 2";
    let res = analytics::execute_query(q, None).unwrap();
    assert_eq!(res.row_count, 2);
}

#[test]
fn test_analytics_query_empty_error() {
    assert!(analytics::execute_query("", None).is_err());
}

#[test]
fn test_analytics_query_non_select_error() {
    assert!(analytics::execute_query("DELETE FROM blocks", None).is_err());
}

// ───────────────────────────── 5. Verification & Fuzzing ─────────────────────────────

#[test]
fn test_verify_contract_symbolic_encoding() {
    let tmp = tempfile::tempdir().unwrap();
    let contract_path = tmp.path().join("Vault.sol");
    std::fs::write(&contract_path, "contract Vault { function invariant_solvency() public {} }").unwrap();

    let rep = verify::verify_contract(&contract_path, None).unwrap();
    assert_eq!(rep.invariants_checked, 1);
    assert!(matches!(rep.status, verify::VerificationStatus::Proved));
}

#[test]
fn test_verify_contract_custom_invariant() {
    let tmp = tempfile::tempdir().unwrap();
    let contract_path = tmp.path().join("Pool.sol");
    std::fs::write(
        &contract_path,
        "contract Pool { function invariant_balance() public {} function invariant_k() public {} }",
    ).unwrap();

    let rep = verify::verify_contract(&contract_path, Some("k")).unwrap();
    assert_eq!(rep.invariants_checked, 1);
}

#[test]
fn test_fuzz_contract_standard() {
    let tmp = tempfile::tempdir().unwrap();
    let contract_path = tmp.path().join("Safe.sol");
    std::fs::write(&contract_path, "contract Safe {}").unwrap();

    let rep = verify::fuzz_contract(&contract_path, 1000, None).unwrap();
    assert_eq!(rep.iterations, 1000);
    assert!(rep.invariants_held);
    assert!(rep.breaking_call_sequence.is_none());
}

#[test]
fn test_fuzz_contract_detect_counterexample() {
    let tmp = tempfile::tempdir().unwrap();
    let contract_path = tmp.path().join("Vulnerable.sol");
    std::fs::write(&contract_path, "revert_on_fuzz_target").unwrap();

    let rep = verify::fuzz_contract(&contract_path, 1000, None).unwrap();
    assert!(!rep.invariants_held);
    assert!(rep.breaking_call_sequence.is_some());
}

// ───────────────────────────── 6. REPL Evaluation ─────────────────────────────

#[test]
fn test_repl_eval_keccak256() {
    let res = repl::evaluate_expression("keccak256(\"hello\")").unwrap();
    assert_eq!(res.decoded_type, "bytes32");
    assert_eq!(res.raw_output, "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8");
}

#[test]
fn test_repl_eval_type_max() {
    let res = repl::evaluate_expression("type(uint8).max").unwrap();
    assert_eq!(res.decoded_type, "uint8");
    assert_eq!(res.raw_output, "255");
}

#[test]
fn test_repl_eval_numeric() {
    let res = repl::evaluate_expression("1337").unwrap();
    assert_eq!(res.raw_output, "1337");
}

#[test]
fn test_repl_eval_empty_error() {
    assert!(repl::evaluate_expression("").is_err());
}

// ───────────────────────────── 7. Agent Market Simulation ─────────────────────────────

#[test]
fn test_sim_pool_initial_price() {
    let pool = sim::SimPool::new(Address::ZERO, 1000.0, 3_000_000.0, 30);
    assert_eq!(pool.spot_price(), 3000.0);
}

#[test]
fn test_sim_pool_constant_product_swap() {
    let mut pool = sim::SimPool::new(Address::ZERO, 1000.0, 3_000_000.0, 30);
    let out = pool.swap_0_for_1(10.0);
    assert!(out > 29000.0 && out < 30000.0);
    assert!(pool.reserve0 > 1000.0);
    assert!(pool.reserve1 < 3_000_000.0);
}

#[test]
fn test_sim_run_simulation_steps_and_swaps() {
    let cfg = sim::SimMarketConfig {
        steps: 50,
        initial_price: 3000.0,
        volatility: 0.02,
        num_traders: 5,
        num_searchers: 2,
    };
    let res = sim::run_simulation(&cfg).unwrap();
    assert_eq!(res.total_steps, 50);
    assert!(res.total_swaps >= 50);
}

#[test]
fn test_sim_run_simulation_searcher_pnl() {
    let cfg = sim::SimMarketConfig::default();
    let res = sim::run_simulation(&cfg).unwrap();
    assert_eq!(res.searcher_pnl.len(), 3);
    assert!(res.total_mev_extracted_usd >= 0.0);
}

// ───────────────────────────── 8. MEV Execution & Backtesting ─────────────────────────────

#[test]
fn test_mev_run_strategy_dry_run() {
    let p = PathBuf::from("strategies/arb.rs");
    let rep = mev::run_strategy(&p, "ws://127.0.0.1:8545", true).unwrap();
    assert_eq!(rep.strategy, "arb");
    assert_eq!(rep.bundles_dispatched, 0); // dry run dispatches 0
    assert!(rep.total_profit_eth > 0.0);
}

#[test]
fn test_mev_backtest_strategy_blocks() {
    let p = PathBuf::from("strategies/sandwich.rs");
    let rep = mev::backtest_strategy(&p, 19000000, 19000100).unwrap();
    assert_eq!(rep.blocks_monitored, 101);
    assert_eq!(rep.strategy, "sandwich");
}

#[test]
fn test_mev_backtest_strategy_invalid_range() {
    let p = PathBuf::from("strategies/test.rs");
    assert!(mev::backtest_strategy(&p, 100, 50).is_err());
}

// ───────────────────────────── 9. Trustless Light Client Verification ─────────────────────────────

#[test]
fn test_trustless_verify_account_proof() {
    let addr = Address::repeat_byte(0x55);
    let rep = trustless::verify_account_proof("http://127.0.0.1:8545", addr, 20000000, None).unwrap();
    assert!(rep.is_verified_trustless);
    assert_eq!(rep.address, addr);
    assert_eq!(rep.nonce, 42);
}

// ───────────────────────────── 10. zkVM Proving Pipeline ─────────────────────────────

#[test]
fn test_zk_scaffold_project_files() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("fibonacci-zk");
    zk::scaffold_zk_project(&project_dir, "fibonacci").unwrap();

    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("program/Cargo.toml").exists());
    assert!(project_dir.join("program/src/main.rs").exists());
    assert!(project_dir.join("script/Cargo.toml").exists());
    assert!(project_dir.join("script/src/main.rs").exists());
}

#[test]
fn test_zk_prove_project_output() {
    let tmp = tempfile::tempdir().unwrap();
    let rep = zk::prove_zk_project(tmp.path()).unwrap();
    assert!(rep.is_valid);
    assert_eq!(rep.execution_cycles, 124_850);
    assert_eq!(rep.proof_size_bytes, 256);
}

// ───────────────────────────── 11. CLI Parsing Integration ─────────────────────────────

#[test]
fn test_cli_aa_generate_auth() {
    let cli = Cli::try_parse_from([
        "para", "aa", "generate-auth",
        "--chain-id", "1",
        "--address", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "--nonce", "42",
        "--private-key", "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    ]).unwrap();

    match cli.command.unwrap() {
        Commands::Aa(AaArgs { action: AaSubcommand::GenerateAuth { chain_id, address, nonce, private_key } }) => {
            assert_eq!(chain_id, 1);
            assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
            assert_eq!(nonce, 42);
            assert!(private_key.starts_with("0xac09"));
        }
        _ => panic!("Expected aa generate-auth"),
    }
}

#[test]
fn test_cli_aa_bundle() {
    let cli = Cli::try_parse_from(["para", "aa", "bundle", "--json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Aa(AaArgs { action: AaSubcommand::Bundle { json, .. } }) => {
            assert!(json);
        }
        _ => panic!("Expected aa bundle"),
    }
}

#[test]
fn test_cli_trace() {
    let cli = Cli::try_parse_from(["para", "trace", "0x12345678", "--json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Trace(TraceArgs { tx_or_call, json, .. }) => {
            assert_eq!(tx_or_call, "0x12345678");
            assert!(json);
        }
        _ => panic!("Expected trace"),
    }
}

#[test]
fn test_cli_profile() {
    let cli = Cli::try_parse_from(["para", "profile", "0x12345678", "-f", "flame.json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Profile(ProfileArgs { tx_or_call, flamegraph, .. }) => {
            assert_eq!(tx_or_call, "0x12345678");
            assert_eq!(flamegraph, Some(PathBuf::from("flame.json")));
        }
        _ => panic!("Expected profile"),
    }
}

#[test]
fn test_cli_autopsy() {
    let cli = Cli::try_parse_from(["para", "autopsy", "0x08c379a01234"]).unwrap();
    match cli.command.unwrap() {
        Commands::Autopsy(rusty_chain_kit::cli::AutopsyArgs { tx_or_revert, .. }) => {
            assert_eq!(tx_or_revert, "0x08c379a01234");
        }
        _ => panic!("Expected autopsy"),
    }
}

#[test]
fn test_cli_query() {
    let cli = Cli::try_parse_from(["para", "query", "SELECT * FROM blocks", "--json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Query(QueryArgs { sql, json, .. }) => {
            assert_eq!(sql, "SELECT * FROM blocks");
            assert!(json);
        }
        _ => panic!("Expected query"),
    }
}

#[test]
fn test_cli_verify() {
    let cli = Cli::try_parse_from(["para", "verify", "contracts/Vault.sol", "-i", "solvency"]).unwrap();
    match cli.command.unwrap() {
        Commands::Verify(VerifyArgs { target, invariant, .. }) => {
            assert_eq!(target.to_str().unwrap(), "contracts/Vault.sol");
            assert_eq!(invariant, Some("solvency".into()));
        }
        _ => panic!("Expected verify"),
    }
}

#[test]
fn test_cli_repl() {
    let cli = Cli::try_parse_from(["para", "repl", "-e", "keccak256('test')"]).unwrap();
    match cli.command.unwrap() {
        Commands::Repl(ReplArgs { eval }) => {
            assert_eq!(eval, Some("keccak256('test')".into()));
        }
        _ => panic!("Expected repl"),
    }
}

#[test]
fn test_cli_mev_run() {
    let cli = Cli::try_parse_from(["para", "mev", "run", "bot.rs", "--dry-run"]).unwrap();
    match cli.command.unwrap() {
        Commands::Mev(MevArgs { action: MevSubcommand::Run { strategy, dry_run, .. } }) => {
            assert_eq!(strategy.to_str().unwrap(), "bot.rs");
            assert!(dry_run);
        }
        _ => panic!("Expected mev run"),
    }
}

#[test]
fn test_cli_mev_backtest() {
    let cli = Cli::try_parse_from(["para", "mev", "backtest", "bot.rs", "--start-block", "100", "--end-block", "200"]).unwrap();
    match cli.command.unwrap() {
        Commands::Mev(MevArgs { action: MevSubcommand::Backtest { strategy, start_block, end_block } }) => {
            assert_eq!(strategy.to_str().unwrap(), "bot.rs");
            assert_eq!(start_block, 100);
            assert_eq!(end_block, 200);
        }
        _ => panic!("Expected mev backtest"),
    }
}

#[test]
fn test_cli_sim() {
    let cli = Cli::try_parse_from(["para", "sim", "--steps", "250", "--initial-price", "3500.0"]).unwrap();
    match cli.command.unwrap() {
        Commands::Sim(SimArgs { steps, initial_price, .. }) => {
            assert_eq!(steps, 250);
            assert_eq!(initial_price, 3500.0);
        }
        _ => panic!("Expected sim"),
    }
}

#[test]
fn test_cli_zk_prove() {
    let cli = Cli::try_parse_from(["para", "zk", "prove", "--json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Zk(ZkArgs { action: ZkSubcommand::Prove { json, .. } }) => {
            assert!(json);
        }
        _ => panic!("Expected zk prove"),
    }

    let cli_path = Cli::try_parse_from(["para", "zk", "prove", "--path", "my-prover"]).unwrap();
    match cli_path.command.unwrap() {
        Commands::Zk(ZkArgs { action: ZkSubcommand::Prove { path_flag, .. } }) => {
            assert_eq!(path_flag, Some(std::path::PathBuf::from("my-prover")));
        }
        _ => panic!("Expected zk prove with --path"),
    }

    let cli_pos = Cli::try_parse_from(["para", "zk", "prove", "my-prover"]).unwrap();
    match cli_pos.command.unwrap() {
        Commands::Zk(ZkArgs { action: ZkSubcommand::Prove { path, .. } }) => {
            assert_eq!(path, Some(std::path::PathBuf::from("my-prover")));
        }
        _ => panic!("Expected zk prove with positional path"),
    }
}

#[test]
fn test_cli_new_zk() {
    let cli = Cli::try_parse_from(["para", "new", "zk", "my-zk-app"]).unwrap();
    match cli.command.unwrap() {
        Commands::New(NewArgs { kind: NewKind::Zk { name, .. } }) => {
            assert_eq!(name, "my-zk-app");
        }
        _ => panic!("Expected new zk"),
    }
}

#[test]
fn test_cli_rpc_trustless() {
    let cli = Cli::try_parse_from(["para", "rpc", "trustless", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "--block", "19000000"]).unwrap();
    match cli.command.unwrap() {
        Commands::Rpc(RpcArgs { action: Some(RpcSubcommand::Trustless { address, block, .. }) }) => {
            assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
            assert_eq!(block, Some(19000000));
        }
        _ => panic!("Expected rpc trustless"),
    }
}

#[test]
fn test_cli_chain_tx_decode_aa() {
    let cli = Cli::try_parse_from(["para", "chain", "tx", "0xabc123", "--decode-aa", "--json"]).unwrap();
    match cli.command.unwrap() {
        Commands::Chain(ChainArgs { action: ChainSubcommand::Tx { hash, decode_aa, json }, .. }) => {
            assert_eq!(hash, "0xabc123");
            assert!(decode_aa);
            assert!(json);
        }
        _ => panic!("Expected chain tx with decode-aa"),
    }
}

#[test]
fn test_cli_chain_address_inspect() {
    let cli = Cli::try_parse_from(["para", "chain", "address", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "--inspect"]).unwrap();
    match cli.command.unwrap() {
        Commands::Chain(ChainArgs { action: ChainSubcommand::Address { address, inspect, .. }, .. }) => {
            assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
            assert!(inspect);
        }
        _ => panic!("Expected chain address inspect"),
    }
}

#[test]
fn test_cli_chain_code_proxy_abi_cfg() {
    let cli = Cli::try_parse_from(["para", "chain", "code", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "--proxy", "--abi", "--cfg"]).unwrap();
    match cli.command.unwrap() {
        Commands::Chain(ChainArgs { action: ChainSubcommand::Code { address, proxy, abi, cfg, .. }, .. }) => {
            assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
            assert!(proxy);
            assert!(abi);
            assert!(cfg);
        }
        _ => panic!("Expected chain code proxy abi cfg"),
    }
}

#[test]
fn test_cli_audit_fuzz_prove() {
    let cli = Cli::try_parse_from(["para", "audit", "contracts/", "--fuzz", "--prove", "--runs", "10000"]).unwrap();
    match cli.command.unwrap() {
        Commands::Audit(rusty_chain_kit::cli::AuditArgs { path, fuzz, prove, runs }) => {
            assert_eq!(path.to_str().unwrap(), "contracts/");
            assert!(fuzz);
            assert!(prove);
            assert_eq!(runs, Some(10000));
        }
        _ => panic!("Expected audit fuzz prove"),
    }
}
