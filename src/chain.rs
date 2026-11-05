//! Chain queries through alloy (the local `../alloy` checkout). Uses `AnyNetwork` so L2s with
//! custom tx types (OP deposits, etc.) decode fine. Results are serializable for the GUI.

use alloy::network::AnyNetwork;
use alloy::primitives::{Address, U256, keccak256, utils::format_ether};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use std::time::Instant;

pub type Client = DynProvider<AnyNetwork>;

pub async fn connect(url: &str) -> Result<Client> {
    let p = ProviderBuilder::new_with_network::<AnyNetwork>()
        .connect(url)
        .await
        .with_context(|| format!("connecting to {url}"))?;
    Ok(p.erased())
}

async fn raw(
    p: &Client,
    method: &'static str,
    params: impl serde::Serialize + Send + Sync + Clone + std::fmt::Debug + std::marker::Unpin + 'static,
) -> Result<Value> {
    p.raw_request::<_, Value>(method.into(), params)
        .await
        .with_context(|| method.to_string())
}

pub fn hex_u64(v: &Value) -> Option<u64> {
    v.as_str()
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
}

pub fn hex_u256(v: &Value) -> Option<U256> {
    v.as_str()
        .and_then(|s| U256::from_str_radix(s.trim_start_matches("0x"), 16).ok())
}

fn gwei(wei: u128) -> String {
    format!("{:.4}", wei as f64 / 1e9)
}

// ───────────────────────────── chain info ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct ChainInfo {
    pub url: String,
    pub chain_id: u64,
    pub chain_name: Option<String>,
    pub block_number: u64,
    pub gas_price_gwei: String,
    pub base_fee_gwei: Option<String>,
    pub client_version: Option<String>,
    pub syncing: bool,
    pub latency_ms: u128,
}

pub fn chain_name(id: u64) -> Option<&'static str> {
    Some(match id {
        1 => "Ethereum Mainnet",
        10 => "OP Mainnet",
        56 => "BNB Chain",
        100 => "Gnosis",
        130 => "Unichain",
        137 => "Polygon",
        324 => "zkSync Era",
        8453 => "Base",
        42161 => "Arbitrum One",
        43114 => "Avalanche C",
        59144 => "Linea",
        81457 => "Blast",
        534352 => "Scroll",
        11155111 => "Sepolia",
        17000 => "Holesky",
        560048 => "Hoodi",
        31337 => "Anvil / Hardhat (local)",
        1337 => "Dev (reth --dev / geth)",
        _ => return None,
    })
}

pub async fn info(url: &str) -> Result<ChainInfo> {
    let p = connect(url).await?;
    let t = Instant::now();
    let chain_id = p.get_chain_id().await?;
    let latency_ms = t.elapsed().as_millis();
    let (block_number, gas_price, client_version, syncing, latest) = tokio::join!(
        p.get_block_number(),
        p.get_gas_price(),
        p.get_client_version(),
        raw(&p, "eth_syncing", ()),
        raw(&p, "eth_getBlockByNumber", ("latest", false)),
    );
    let base_fee = latest
        .ok()
        .and_then(|b| hex_u256(&b["baseFeePerGas"]))
        .map(|w| gwei(w.to::<u128>()));
    Ok(ChainInfo {
        url: url.to_string(),
        chain_id,
        chain_name: chain_name(chain_id).map(str::to_string),
        block_number: block_number?,
        gas_price_gwei: gas_price.map(gwei).unwrap_or_else(|_| "?".into()),
        base_fee_gwei: base_fee,
        client_version: client_version.ok(),
        syncing: syncing.map(|v| v != Value::Bool(false)).unwrap_or(false),
        latency_ms,
    })
}

// ───────────────────────────── blocks ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct BlockSummary {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
    pub time: String,
    pub tx_count: usize,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub base_fee_gwei: Option<String>,
    pub miner: String,
    pub transactions: Vec<String>,
}

pub async fn block(url: &str, id: &str) -> Result<(BlockSummary, Value)> {
    let p = connect(url).await?;
    let id = id.trim();
    let v = if id.len() == 66 && id.starts_with("0x") {
        raw(&p, "eth_getBlockByHash", (id.to_string(), false)).await?
    } else {
        let tag = match id {
            "" | "latest" | "pending" | "safe" | "finalized" | "earliest" => {
                if id.is_empty() {
                    "latest".to_string()
                } else {
                    id.to_string()
                }
            }
            n => {
                let n: u64 = n.parse().with_context(|| format!("bad block id `{n}`"))?;
                format!("0x{n:x}")
            }
        };
        raw(&p, "eth_getBlockByNumber", (tag, false)).await?
    };
    if v.is_null() {
        bail!("block {id} not found");
    }
    let ts = hex_u64(&v["timestamp"]).unwrap_or(0);
    let txs: Vec<String> = v["transactions"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let s = BlockSummary {
        number: hex_u64(&v["number"]).unwrap_or(0),
        hash: v["hash"].as_str().unwrap_or_default().to_string(),
        timestamp: ts,
        time: chrono::DateTime::from_timestamp(ts as i64, 0)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default(),
        tx_count: txs.len(),
        gas_used: hex_u64(&v["gasUsed"]).unwrap_or(0),
        gas_limit: hex_u64(&v["gasLimit"]).unwrap_or(0),
        base_fee_gwei: hex_u256(&v["baseFeePerGas"]).map(|w| gwei(w.to::<u128>())),
        miner: v["miner"].as_str().unwrap_or_default().to_string(),
        transactions: txs,
    };
    Ok((s, v))
}

// ───────────────────────────── transactions ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct TxSummary {
    pub hash: String,
    pub status: String,
    pub block: Option<u64>,
    pub from: String,
    pub to: Option<String>,
    pub contract_created: Option<String>,
    pub value_eth: String,
    pub nonce: u64,
    pub tx_type: u64,
    pub gas_used: Option<u64>,
    pub gas_limit: u64,
    pub effective_gas_price_gwei: Option<String>,
    pub fee_eth: Option<String>,
    pub selector: Option<String>,
    pub input_bytes: usize,
    pub logs: usize,
}

pub async fn tx(url: &str, hash: &str) -> Result<(TxSummary, Value, Value)> {
    let p = connect(url).await?;
    let (t, r) = tokio::join!(
        raw(&p, "eth_getTransactionByHash", (hash.to_string(),)),
        raw(&p, "eth_getTransactionReceipt", (hash.to_string(),)),
    );
    let t = t?;
    if t.is_null() {
        bail!("transaction {hash} not found");
    }
    let r = r.unwrap_or(Value::Null);
    let input = t["input"].as_str().unwrap_or("0x");
    let gas_used = hex_u64(&r["gasUsed"]);
    let egp = hex_u256(&r["effectiveGasPrice"]).or_else(|| hex_u256(&t["gasPrice"]));
    let s = TxSummary {
        hash: hash.to_string(),
        status: match r["status"].as_str() {
            Some("0x1") => "success".into(),
            Some("0x0") => "reverted".into(),
            _ if r.is_null() => "pending".into(),
            _ => "unknown".into(),
        },
        block: hex_u64(&t["blockNumber"]),
        from: t["from"].as_str().unwrap_or_default().into(),
        to: t["to"].as_str().map(String::from),
        contract_created: r["contractAddress"].as_str().map(String::from),
        value_eth: hex_u256(&t["value"]).map(format_ether).unwrap_or_default(),
        nonce: hex_u64(&t["nonce"]).unwrap_or(0),
        tx_type: hex_u64(&t["type"]).unwrap_or(0),
        gas_used,
        gas_limit: hex_u64(&t["gas"]).unwrap_or(0),
        effective_gas_price_gwei: egp.map(|w| gwei(w.to::<u128>())),
        fee_eth: match (gas_used, egp) {
            (Some(g), Some(p)) => Some(format_ether(U256::from(g) * p)),
            _ => None,
        },
        selector: (input.len() >= 10).then(|| input[..10].to_string()),
        input_bytes: input.len().saturating_sub(2) / 2,
        logs: r["logs"].as_array().map(|a| a.len()).unwrap_or(0),
    };
    Ok((s, t, r))
}

// ───────────────────────────── addresses ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance_wei: String,
    pub balance_eth: String,
    pub nonce: u64,
    pub is_contract: bool,
    pub code_size: usize,
    pub code_hash: Option<String>,
}

pub async fn address(url: &str, addr: &str) -> Result<AddressInfo> {
    let a: Address = addr
        .trim()
        .parse()
        .with_context(|| format!("bad address `{addr}`"))?;
    let p = connect(url).await?;
    let (bal, nonce, code) = tokio::join!(
        p.get_balance(a),
        p.get_transaction_count(a),
        p.get_code_at(a)
    );
    let bal = bal?;
    let code = code?;
    Ok(AddressInfo {
        address: a.to_checksum(None),
        balance_wei: bal.to_string(),
        balance_eth: format_ether(bal),
        nonce: nonce?,
        is_contract: !code.is_empty(),
        code_size: code.len(),
        code_hash: (!code.is_empty()).then(|| keccak256(&code).to_string()),
    })
}

pub async fn code(url: &str, addr: &str) -> Result<String> {
    let a: Address = addr
        .trim()
        .parse()
        .with_context(|| format!("bad address `{addr}`"))?;
    let p = connect(url).await?;
    Ok(p.get_code_at(a).await?.to_string())
}

/// Poll for new blocks and call `f` for each (works over plain HTTP).
pub async fn watch_blocks(url: &str, mut f: impl FnMut(BlockSummary)) -> Result<()> {
    let p = connect(url).await?;
    let mut last = p.get_block_number().await?;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let Ok(head) = p.get_block_number().await else {
            continue;
        };
        while last < head {
            last += 1;
            if let Ok((b, _)) = block(url, &last.to_string()).await {
                f(b);
            }
        }
    }
}
