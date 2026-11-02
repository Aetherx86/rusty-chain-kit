# Vectorized SQL Blockchain Analytics

Extracting and querying large-scale blockchain datasets requires high-speed analytical engines capable of filtering millions of records in milliseconds.

Rusty Chain Kit embeds an in-process vectorized SQL OLAP engine (`analytics.rs`) capable of querying extracted Parquet, JSON, and CSV blockchain datasets using standard SQL syntax.

---

## 1. Schema Specifications

The analytics engine provides pre-indexed schemas for core blockchain entities:

### 1. `blocks` Table
| Column Name | Type | Description |
| :--- | :--- | :--- |
| `number` | `uint64` | Block number |
| `hash` | `string` | 32-byte block hash hex string |
| `gas_used` | `uint64` | Total gas consumed by all transactions in block |
| `tx_count` | `uint64` | Number of transactions included |
| `base_fee_gwei` | `float64` | EIP-1559 base fee in Gwei |

### 2. `transactions` Table
| Column Name | Type | Description |
| :--- | :--- | :--- |
| `hash` | `string` | 32-byte transaction hash hex string |
| `block_number` | `uint64` | Block number in which transaction was mined |
| `from` | `string` | Sender account address (`0x...`) |
| `to` | `string` | Recipient or contract address (`0x...`) |
| `value_eth` | `float64` | Value transferred in Ether |
| `gas_price_gwei` | `float64` | Gas price paid in Gwei |
| `gas_used` | `uint64` | Gas consumed by execution |

---

## 2. Query Engine Capabilities

The SQL engine supports standard relational algebra operations:
- **Projection**: Select specific columns or all columns (`SELECT number, gas_used` or `SELECT *`).
- **Filtering**: Predicates on numeric and string attributes (`WHERE gas_used > 15000000`, `WHERE value_eth > 5.0`).
- **Ordering**: Ascending and descending multi-column sort (`ORDER BY gas_used DESC`).
- **Pagination & Limiting**: Record limits (`LIMIT 10`).
- **File Queries**: Query Parquet, CSV, or JSON data files directly (`SELECT * FROM 'data/transfers.parquet'`).

---

## 3. CLI Usage

Execute analytical queries:
```bash
para query "SELECT number, gas_used, tx_count FROM blocks WHERE gas_used > 15000000 ORDER BY gas_used DESC LIMIT 5"
```

Output:
```
+----------+------------+----------+
| number   | gas_used   | tx_count |
+----------+------------+----------+
| 20000004 | 29400000   | 340      |
| 20000002 | 18500000   | 210      |
+----------+------------+----------+
```

Export analytical results as structured JSON:
```bash
para query "SELECT * FROM transactions WHERE value_eth > 10.0" --json
```

Pipe output to external processing pipelines:
```bash
para query "SELECT from, to, value_eth FROM transactions" --json | jq .
```
