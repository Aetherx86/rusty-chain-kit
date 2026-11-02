use alloy::primitives::U256;
use rusty_chain_kit::chain::{
    AddressInfo, BlockSummary, ChainInfo, TxSummary, chain_name, hex_u64, hex_u256,
};
use serde_json::json;

#[test]
fn test_chain_name_known_networks() {
    assert_eq!(chain_name(1), Some("Ethereum Mainnet"));
    assert_eq!(chain_name(11155111), Some("Sepolia"));
    assert_eq!(chain_name(8453), Some("Base"));
    assert_eq!(chain_name(42161), Some("Arbitrum One"));
    assert_eq!(chain_name(10), Some("OP Mainnet"));
    assert_eq!(chain_name(31337), Some("Anvil / Hardhat (local)"));
}

#[test]
fn test_chain_name_unknown_network() {
    assert_eq!(chain_name(99999999), None);
    assert_eq!(chain_name(0), None);
}

#[test]
fn test_hex_u64_valid() {
    assert_eq!(hex_u64(&json!("0x1")), Some(1));
    assert_eq!(hex_u64(&json!("0x10")), Some(16));
    assert_eq!(hex_u64(&json!("0xff")), Some(255));
    assert_eq!(hex_u64(&json!("0x1000")), Some(4096));
}

#[test]
fn test_hex_u64_zero() {
    assert_eq!(hex_u64(&json!("0x0")), Some(0));
    assert_eq!(hex_u64(&json!("0x0000")), Some(0));
}

#[test]
fn test_hex_u64_uppercase() {
    assert_eq!(hex_u64(&json!("0xABCD")), Some(0xABCD));
    assert_eq!(hex_u64(&json!("0xFF")), Some(255));
}

#[test]
fn test_hex_u64_invalid_strings() {
    assert_eq!(hex_u64(&json!("0xZZZZ")), None);
    assert_eq!(hex_u64(&json!("not_hex")), None);
    assert_eq!(hex_u64(&json!("")), None);
}

#[test]
fn test_hex_u64_non_string_json() {
    assert_eq!(hex_u64(&json!(null)), None);
    assert_eq!(hex_u64(&json!(12345)), None);
    assert_eq!(hex_u64(&json!([])), None);
    assert_eq!(hex_u64(&json!({})), None);
}

#[test]
fn test_hex_u256_valid() {
    let val = hex_u256(&json!("0x1")).unwrap();
    assert_eq!(val, U256::from(1));

    let val_big = hex_u256(&json!("0xde0b6b3a7640000")).unwrap();
    assert_eq!(val_big, U256::from(1000000000000000000u64));
}

#[test]
fn test_hex_u256_zero() {
    assert_eq!(hex_u256(&json!("0x0")), Some(U256::ZERO));
}

#[test]
fn test_hex_u256_invalid() {
    assert_eq!(hex_u256(&json!("0xInvalidG")), None);
    assert_eq!(hex_u256(&json!(null)), None);
    assert_eq!(hex_u256(&json!(42)), None);
}

#[test]
fn test_chain_info_serialization() {
    let info = ChainInfo {
        url: "http://127.0.0.1:8545".into(),
        chain_id: 1,
        chain_name: Some("Ethereum Mainnet".into()),
        block_number: 19500000,
        gas_price_gwei: "15.5".into(),
        base_fee_gwei: Some("14.2".into()),
        client_version: Some("geth/v1.13.0".into()),
        syncing: false,
        latency_ms: 22,
    };
    let json_val = serde_json::to_value(&info).unwrap();
    assert_eq!(json_val["chain_id"], 1);
    assert_eq!(json_val["chain_name"], "Ethereum Mainnet");
    assert_eq!(json_val["block_number"], 19500000);
    assert_eq!(json_val["syncing"], false);
}

#[test]
fn test_block_summary_serialization() {
    let b = BlockSummary {
        number: 19000000,
        hash: "0xBlockSha".into(),
        timestamp: 1704067200,
        time: "2024-01-01T00:00:00Z".into(),
        tx_count: 142,
        gas_used: 12000000,
        gas_limit: 30000000,
        base_fee_gwei: Some("12.0".into()),
        miner: "0xMiner".into(),
        transactions: vec!["0xTx1".into(), "0xTx2".into()],
    };
    let json_val = serde_json::to_value(&b).unwrap();
    assert_eq!(json_val["number"], 19000000);
    assert_eq!(json_val["tx_count"], 142);
    assert_eq!(json_val["miner"], "0xMiner");
}

#[test]
fn test_tx_summary_serialization() {
    let tx = TxSummary {
        hash: "0xTxHash".into(),
        status: "Success".into(),
        block: Some(19000000),
        from: "0xFrom".into(),
        to: Some("0xTo".into()),
        contract_created: None,
        value_eth: "1.5".into(),
        nonce: 42,
        tx_type: 2,
        gas_used: Some(21000),
        gas_limit: 50000,
        effective_gas_price_gwei: Some("20.0".into()),
        fee_eth: Some("0.00042".into()),
        selector: Some("0xa9059cbb".into()),
        input_bytes: 68,
        logs: 1,
    };
    let json_val = serde_json::to_value(&tx).unwrap();
    assert_eq!(json_val["hash"], "0xTxHash");
    assert_eq!(json_val["status"], "Success");
    assert_eq!(json_val["selector"], "0xa9059cbb");
    assert_eq!(json_val["tx_type"], 2);
}

#[test]
fn test_address_info_serialization() {
    let addr = AddressInfo {
        address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".into(),
        balance_eth: "500.5".into(),
        balance_wei: "500500000000000000000".into(),
        nonce: 10,
        is_contract: false,
        code_size: 0,
        code_hash: None,
    };
    let json_val = serde_json::to_value(&addr).unwrap();
    assert_eq!(
        json_val["address"],
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    );
    assert_eq!(json_val["is_contract"], false);
    assert_eq!(json_val["nonce"], 10);
}

#[test]
fn test_address_info_contract() {
    let contract = AddressInfo {
        address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
        balance_eth: "0.0".into(),
        balance_wei: "0".into(),
        nonce: 1,
        is_contract: true,
        code_size: 1984,
        code_hash: Some("0xContractCodeHash".into()),
    };
    assert!(contract.is_contract);
    assert_eq!(contract.code_size, 1984);
}
