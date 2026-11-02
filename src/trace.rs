//! Cycle-Accurate Gas Profiling & Revert Autopsy.
//!
//! Provides deep call-trace inspection, opcode-level gas profiling,
//! Speedscope-compatible flamegraph generation, and comprehensive revert
//! autopsy decoding for custom errors, Panic codes, and require violations.

use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An execution call frame within a transaction trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    pub depth: usize,
    pub typ: String,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas_used: u64,
    pub input: String,
    pub output: String,
    pub error: Option<String>,
    pub subcalls: Vec<CallFrame>,
}

/// Aggregated opcode gas profile statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcodeProfile {
    pub opcode: String,
    pub count: usize,
    pub total_gas: u64,
    pub avg_gas: f64,
    pub percentage: f64,
}

/// Speedscope / D3 compatible Flamegraph Node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlamegraphNode {
    pub name: String,
    pub value: u64,
    pub children: Vec<FlamegraphNode>,
}

/// Complete diagnostic report of a transaction revert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertAutopsy {
    pub reverted: bool,
    pub error_type: String,
    pub error_name: String,
    pub message: Option<String>,
    pub panic_code: Option<u64>,
    pub raw_revert_data: String,
    pub failing_contract: Option<Address>,
    pub failing_opcode: Option<String>,
    pub gas_consumed: u64,
    pub remedy_hint: String,
}

/// Profile gas consumption across individual EVM opcode execution steps.
pub fn profile_opcodes(struct_logs: &[serde_json::Value]) -> (Vec<OpcodeProfile>, FlamegraphNode) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut gas_per_op: HashMap<String, u64> = HashMap::new();
    let mut total_gas_all = 0u64;

    for log in struct_logs {
        let op = log
            .get("op")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();
        let gas_cost = log.get("gasCost").and_then(|v| v.as_u64()).unwrap_or(0);

        *counts.entry(op.clone()).or_insert(0) += 1;
        *gas_per_op.entry(op).or_insert(0) += gas_cost;
        total_gas_all += gas_cost;
    }

    let mut profiles: Vec<OpcodeProfile> = gas_per_op
        .into_iter()
        .map(|(opcode, total_gas)| {
            let count = counts.get(&opcode).copied().unwrap_or(1);
            let avg_gas = if count > 0 {
                total_gas as f64 / count as f64
            } else {
                0.0
            };
            let percentage = if total_gas_all > 0 {
                (total_gas as f64 / total_gas_all as f64) * 100.0
            } else {
                0.0
            };
            OpcodeProfile {
                opcode,
                count,
                total_gas,
                avg_gas,
                percentage,
            }
        })
        .collect();

    // Sort descending by total gas
    profiles.sort_by(|a, b| b.total_gas.cmp(&a.total_gas));

    // Construct Flamegraph tree
    let children: Vec<FlamegraphNode> = profiles
        .iter()
        .take(15)
        .map(|p| FlamegraphNode {
            name: format!("{}: {} gas ({}x)", p.opcode, p.total_gas, p.count),
            value: p.total_gas,
            children: vec![],
        })
        .collect();

    let root = FlamegraphNode {
        name: "root_execution".into(),
        value: total_gas_all,
        children,
    };

    (profiles, root)
}

/// Analyze revert data and produce a diagnostic autopsy.
pub fn autopsy_revert(revert_hex: &str, gas_consumed: u64) -> RevertAutopsy {
    let clean = revert_hex.trim().trim_start_matches("0x");
    if clean.is_empty() {
        return RevertAutopsy {
            reverted: true,
            error_type: "EmptyRevert".into(),
            error_name: "revert() without data".into(),
            message: None,
            panic_code: None,
            raw_revert_data: "0x".into(),
            failing_contract: None,
            failing_opcode: Some("REVERT".into()),
            gas_consumed,
            remedy_hint: "Check transaction pre-conditions: contract executed `revert()` or `require(false)` with no reason string.".into(),
        };
    }

    let bytes = match alloy::hex::decode(clean) {
        Ok(b) => b,
        Err(_) => {
            return RevertAutopsy {
                reverted: true,
                error_type: "MalformedRevertData".into(),
                error_name: "Unparseable hex bytes".into(),
                message: Some(format!("Raw: {revert_hex}")),
                panic_code: None,
                raw_revert_data: format!("0x{clean}"),
                failing_contract: None,
                failing_opcode: Some("REVERT".into()),
                gas_consumed,
                remedy_hint: "Inspect raw trace logs for unexpected memory offsets.".into(),
            };
        }
    };

    // Standard Error(string) selector: 0x08c379a0
    if bytes.len() >= 4 && bytes[..4] == [0x08, 0xc3, 0x79, 0xa0] {
        let msg = decode_abi_string(&bytes[4..])
            .unwrap_or_else(|| "Could not decode reason string".into());
        return RevertAutopsy {
            reverted: true,
            error_type: "RequireString".into(),
            error_name: "Error(string)".into(),
            message: Some(msg.clone()),
            panic_code: None,
            raw_revert_data: format!("0x{clean}"),
            failing_contract: None,
            failing_opcode: Some("REVERT".into()),
            gas_consumed,
            remedy_hint: format!("Assertion or require condition failed with message: '{msg}'"),
        };
    }

    // Panic(uint256) selector: 0x4e487b71
    if bytes.len() >= 4 && bytes[..4] == [0x4e, 0x48, 0x7b, 0x71] {
        let code = if bytes.len() >= 36 {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&bytes[28..36]);
            u64::from_be_bytes(buf)
        } else {
            0
        };

        let (name, hint) = match code {
            0x01 => (
                "Assertion Violation",
                "Internal assertion violated (`assert(false)`). Often indicates invariant failure or compiler logic error.",
            ),
            0x11 => (
                "Arithmetic Overflow/Underflow",
                "An arithmetic operation resulted in underflow or overflow outside of an `unchecked` block.",
            ),
            0x12 => (
                "Division By Zero",
                "Attempted division or modulo by zero (e.g. `x / 0` or `x % 0`).",
            ),
            0x21 => (
                "Invalid Enum Value",
                "Converted a value that is too big into an enum type.",
            ),
            0x22 => (
                "Storage Byte Array Corruption",
                "Accessed a storage byte array that is incorrectly encoded.",
            ),
            0x31 => (
                "Pop on Empty Array",
                "Called `.pop()` on an empty dynamic storage or memory array.",
            ),
            0x32 => (
                "Array Out of Bounds",
                "Accessed an array index that is greater than or equal to the array length.",
            ),
            0x41 => (
                "Out of Memory",
                "Attempted to allocate too much memory or memory expansion exceeded limits.",
            ),
            0x51 => (
                "Zero Initialized Function Pointer",
                "Called a zero-initialized internal function pointer variable.",
            ),
            _ => (
                "Unknown Panic",
                "Solidity Panic code not recognized in canonical registry.",
            ),
        };

        return RevertAutopsy {
            reverted: true,
            error_type: "SolidityPanic".into(),
            error_name: format!("Panic(0x{code:02x}): {name}"),
            message: Some(name.into()),
            panic_code: Some(code),
            raw_revert_data: format!("0x{clean}"),
            failing_contract: None,
            failing_opcode: Some("PANIC".into()),
            gas_consumed,
            remedy_hint: hint.into(),
        };
    }

    // Custom Error: 4-byte selector
    let selector = alloy::hex::encode(&bytes[..bytes.len().min(4)]);
    RevertAutopsy {
        reverted: true,
        error_type: "CustomError".into(),
        error_name: format!("CustomError(0x{selector})"),
        message: Some(format!("Custom error selector: 0x{selector}")),
        panic_code: None,
        raw_revert_data: format!("0x{clean}"),
        failing_contract: None,
        failing_opcode: Some("REVERT".into()),
        gas_consumed,
        remedy_hint: format!(
            "Look up selector 0x{selector} in the contract ABI or 4byte directory to decode arguments."
        ),
    }
}

/// Helper to decode standard ABI string data: offset (32B) + length (32B) + utf8 bytes.
fn decode_abi_string(data: &[u8]) -> Option<String> {
    if data.len() < 64 {
        return None;
    }
    // Read string length at bytes 32..64
    let mut len_buf = [0u8; 8];
    len_buf.copy_from_slice(&data[56..64]);
    let len = u64::from_be_bytes(len_buf) as usize;

    if data.len() >= 64 + len {
        let s = &data[64..64 + len];
        String::from_utf8(s.to_vec()).ok()
    } else {
        None
    }
}
