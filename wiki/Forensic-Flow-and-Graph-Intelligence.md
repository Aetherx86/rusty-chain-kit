# Forensic Flow and Graph Intelligence

On-chain investigations often require tracing funds across multiple intermediary hops to identify the true origin, destination, and risk classification of transacted capital.

Rusty Chain Kit embeds a high-performance forensic graph engine (`forensics.rs`) providing Chainalysis-style entity clustering, multi-hop fund flow tracing, sanctions compliance screening, and risk scoring accessible via the CLI (`para flow`) and the interactive Web GUI canvas.

---

## 1. Multi-Hop Transaction Graph Architecture

```
                   [ Subject Account (0xd8dA6...6045) ]
                                    |
        +---------------------------+---------------------------+
        | (Hop 1: 15 ETH)           | (Hop 1: 50 ETH)           | (Hop 1: 25 USDC)
        v                           v                           v
[ DEX Swap Router ]         [ Bridge Contract ]        [ Aggregator Router ]
        |                                                       |
        | (Hop 2: 10 WETH)                                      | (Hop 2: 24.8 USDT)
        v                                                       v
[ Mixer / Privacy Pool ]                                 [ Exchange Deposit ]
  (Risk: Flagged High)                                     (Risk: Verified Clean)
```

---

## 2. Entity Categorization & Screening

The forensic engine automatically classifies visited addresses into semantic entity categories based on contract bytecode signatures, on-chain event logs, and verified protocol registries:

| Entity Category | Semantic Description | Risk Baseline |
| :--- | :--- | :--- |
| `Subject` | Target account being investigated | Baseline |
| `Dex` | Automated Market Makers and swap routers (Uniswap, Curve) | Clean |
| `Bridge` | Cross-chain bridges and lockbox contracts (Arbitrum, Optimism) | Clean |
| `Exchange` | Centralized exchange hot/deposit wallets (Coinbase, Binance, Kraken)| Clean |
| `Lending` | Decentralized money markets (Aave, Compound, MakerDAO) | Clean |
| `Mixer` | Non-custodial privacy pools and mixing protocols (Tornado Cash, Railgun) | Severe / High |

### Sanctions Screening & Risk Score Engine
Every address encountered in the traversal graph is screened against active global sanctions registries and known illicit cluster lists:

- **Clean**: No interaction with flagged or high-risk counterparties within the traversal depth.
- **Low**: Indirect exposure (3+ hops removed) to non-sanctioned high-volume mixers or unverified contracts.
- **Medium**: Moderate indirect exposure (2 hops) to mixer exit liquidity.
- **High**: Direct incoming or outgoing transaction with unverified mixing protocols.
- **Severe**: Direct counterparty interaction with OFAC-sanctioned contracts (e.g., Tornado Cash router or privacy pool addresses).

---

## 3. Graph Data Structure & Export

The graph engine models transactions as directed edges between vertices:
- **Vertices (Nodes)**: Address, entity category, risk classification, total incoming value, and total outgoing value.
- **Edges (Transfers)**: Transaction hash, source address, destination address, transferred value (ETH / ERC-20), token symbol, block timestamp, and hop level.

### Export Formats
- **Terminal Graph**: Visual tree diagram with entity tags and risk indicators rendered in color.
- **JSON Graph**: Complete node-link adjacency list compatible with D3.js, Cytoscape, and Gephi.
- **CSV Edge List**: Standard table of transfers (`from,to,value,token,tx_hash,hop`) for spreadsheet auditing.

---

## 4. CLI Usage

Trace 3-hop fund flows from a subject account:
```bash
para flow 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --hops 3
```

Export graph data to JSON:
```bash
para flow 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --json
```

Export transfer edges to CSV:
```bash
para flow 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --csv transfers.csv
```

---

## 5. Web GUI Interactive Graph Canvas

When running `para gui`, navigate to the **Forensics** tab to view the live directed graph:
- Drag and zoom interactive nodes.
- Filter by entity type or minimum transaction value.
- Click any node to open its full address dossier with risk breakdown and counterparty transfer history.
