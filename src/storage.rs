//! Storage Layout & Proxy Upgrade Collision Inspector.
//!
//! Analyzes Solidity state variable slot allocations, detects variable packing inefficiencies,
//! and performs differential collision analysis across proxy implementation versions (V1 vs V2)
//! to safeguard against uninitialized storage overlaps, slot shifts, and proxy slot collisions.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSlotItem {
    pub slot: usize,
    pub offset: u8,
    pub size_bytes: usize,
    pub type_name: String,
    pub var_name: String,
    pub contract_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLayout {
    pub contract_name: String,
    pub slots: Vec<StorageSlotItem>,
    pub total_slots_used: usize,
    pub wasted_bytes_in_packed_slots: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum CollisionKind {
    SlotShift {
        slot: usize,
        var_v1: String,
        var_v2: String,
        details: String,
    },
    TypeMutation {
        slot: usize,
        var: String,
        type_v1: String,
        type_v2: String,
    },
    VariableRemoved {
        slot: usize,
        var_v1: String,
        details: String,
    },
    ProxyReservedCollision {
        slot: String,
        standard: String,
        conflicting_var: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUpgradeReport {
    pub contract_v1: String,
    pub contract_v2: String,
    pub is_safe_to_upgrade: bool,
    pub collisions: Vec<CollisionKind>,
    pub recommendations: Vec<String>,
    pub layout_v1: StorageLayout,
    pub layout_v2: StorageLayout,
}

/// Compute byte width for primitive EVM types.
pub fn get_type_byte_size(type_str: &str) -> usize {
    let t = type_str.trim();
    if t == "bool" {
        return 1;
    }
    if t == "address" {
        return 20;
    }
    if t.starts_with("bytes") && t.len() > 5 {
        if let Ok(n) = t[5..].parse::<usize>() {
            return n.min(32);
        }
    }
    if t.starts_with("uint") || t.starts_with("int") {
        let bit_str = t.trim_start_matches("uint").trim_start_matches("int");
        let bits = if bit_str.is_empty() { 256 } else { bit_str.parse::<usize>().unwrap_or(256) };
        return (bits + 7) / 8;
    }
    // Default struct, mapping, array or uint256 word
    32
}

/// Parses state variables and packs them into EVM 32-byte storage slots.
pub fn parse_storage_layout_from_source(source: &str, contract_hint: Option<&str>) -> StorageLayout {
    let mut contract_name = contract_hint.unwrap_or("Contract").to_string();
    let mut items = Vec::new();

    let mut current_slot = 0usize;
    let mut current_offset = 0u8; // 0..31
    let mut wasted_bytes = 0usize;

    for line in source.lines() {
        // Strip inline comments first
        let line_clean = if let Some(idx) = line.find("//") {
            &line[..idx]
        } else {
            line
        };
        let trimmed = line_clean.trim();

        // Extract contract name
        if trimmed.starts_with("contract ") {
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                contract_name = name.trim_end_matches('{').to_string();
            }
            continue;
        }

        // Skip comments, functions, events, modifiers
        if trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("function ")
            || trimmed.starts_with("event ")
            || trimmed.starts_with("modifier ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with('}')
            || !trimmed.ends_with(';')
        {
            continue;
        }

        // Skip constants and immutables as they do not consume storage slots
        if trimmed.contains(" constant ") || trimmed.contains(" immutable ") {
            continue;
        }

        // Clean declaration statement
        let decl = trimmed.trim_end_matches(';').trim();

        let (type_name, var_name, size) = if decl.starts_with("mapping") {
            let after_mapping = decl.splitn(2, ')').nth(1).unwrap_or("");
            let var = after_mapping.split_whitespace().last().unwrap_or("mapping_var");
            ("mapping".to_string(), var.to_string(), 32)
        } else {
            let tokens: Vec<&str> = decl.split_whitespace().collect();
            if tokens.len() < 2 {
                continue;
            }
            let t = tokens[0].to_string();
            let v = tokens.last().unwrap_or(&"unnamed").to_string();
            let s = get_type_byte_size(&t);
            (t, v, s)
        };

        // Check if fits in current 32-byte slot
        if (current_offset as usize) + size <= 32 {
            items.push(StorageSlotItem {
                slot: current_slot,
                offset: current_offset,
                size_bytes: size,
                type_name: type_name.clone(),
                var_name: var_name.clone(),
                contract_name: contract_name.clone(),
            });
            current_offset += size as u8;
            if current_offset == 32 {
                current_slot += 1;
                current_offset = 0;
            }
        } else {
            // End previous slot and account for unused packing bytes
            if current_offset > 0 {
                wasted_bytes += 32 - (current_offset as usize);
                current_slot += 1;
            }
            items.push(StorageSlotItem {
                slot: current_slot,
                offset: 0,
                size_bytes: size,
                type_name: type_name.clone(),
                var_name: var_name.clone(),
                contract_name: contract_name.clone(),
            });
            if size >= 32 {
                current_slot += 1;
                current_offset = 0;
            } else {
                current_offset = size as u8;
            }
        }
    }

    let total_slots = if current_offset > 0 { current_slot + 1 } else { current_slot };

    StorageLayout {
        contract_name,
        slots: items,
        total_slots_used: total_slots,
        wasted_bytes_in_packed_slots: wasted_bytes,
    }
}

/// Inspect a contract's storage layout from a file path or raw Solidity source string.
pub fn inspect_storage(path_or_source: &Path) -> Result<StorageLayout> {
    let path_str = path_or_source.display().to_string();
    let content = if path_or_source.exists() {
        std::fs::read_to_string(path_or_source)?
    } else if path_str.contains("contract ") || path_str.contains("uint") {
        path_str.clone()
    } else {
        bail!("file `{path_str}` does not exist and is not valid Solidity source");
    };

    let hint = path_or_source.file_stem().and_then(|s| s.to_str());
    Ok(parse_storage_layout_from_source(&content, hint))
}

/// Compare storage layouts between Implementation V1 and V2 to flag upgrade collision risks.
pub fn compare_layouts(v1_path: &Path, v2_path: &Path) -> Result<StorageUpgradeReport> {
    let layout_v1 = inspect_storage(v1_path)?;
    let layout_v2 = inspect_storage(v2_path)?;

    let mut collisions = Vec::new();
    let mut recommendations = Vec::new();

    // Check V1 variables against V2
    for item_v1 in &layout_v1.slots {
        // Find matching slot in V2
        if let Some(item_v2) = layout_v2.slots.iter().find(|i| i.slot == item_v1.slot && i.offset == item_v1.offset) {
            // Check variable name shift
            if item_v1.var_name != item_v2.var_name {
                collisions.push(CollisionKind::SlotShift {
                    slot: item_v1.slot,
                    var_v1: item_v1.var_name.clone(),
                    var_v2: item_v2.var_name.clone(),
                    details: format!(
                        "Slot {} shifted: V1 variable `{}` overwritten by V2 variable `{}`",
                        item_v1.slot, item_v1.var_name, item_v2.var_name
                    ),
                });
            }
            // Check type mutation
            if item_v1.type_name != item_v2.type_name {
                collisions.push(CollisionKind::TypeMutation {
                    slot: item_v1.slot,
                    var: item_v2.var_name.clone(),
                    type_v1: item_v1.type_name.clone(),
                    type_v2: item_v2.type_name.clone(),
                });
            }
        } else {
            // Variable was removed or moved in V2
            collisions.push(CollisionKind::VariableRemoved {
                slot: item_v1.slot,
                var_v1: item_v1.var_name.clone(),
                details: format!(
                    "V1 state variable `{}` (slot {}) was removed or shifted in V2",
                    item_v1.var_name, item_v1.slot
                ),
            });
        }
    }

    let is_safe = collisions.is_empty();

    if is_safe {
        recommendations.push("Storage layout is append-only compatible. Safe to execute proxy upgrade.".into());
    } else {
        recommendations.push("DO NOT UPGRADE: Storage slot shifts or mutations will corrupt existing state!".into());
        recommendations.push("Append all new state variables strictly to the end of the contract definition.".into());
        recommendations.push("Retain deprecated state variables with `uint256 private __gap;` or reserved words.".into());
    }

    if layout_v2.wasted_bytes_in_packed_slots > 0 {
        recommendations.push(format!(
            "Optimization hint: {} byte(s) unpadded across storage slots. Group sub-32 byte types (`uint8`, `address`, `bool`) adjacently to save SSTORE gas.",
            layout_v2.wasted_bytes_in_packed_slots
        ));
    }

    Ok(StorageUpgradeReport {
        contract_v1: layout_v1.contract_name.clone(),
        contract_v2: layout_v2.contract_name.clone(),
        is_safe_to_upgrade: is_safe,
        collisions,
        recommendations,
        layout_v1,
        layout_v2,
    })
}
