# Cross-Compiler Benchmarking

Solidity compiler optimizations can dramatically alter runtime bytecode size, deployment gas, and execution efficiency. In particular, compiling through the Yul intermediate representation (`via-ir`) produces significant runtime gas reductions for complex contracts, but can increase compilation times and occasionally bloat bytecode size.

Rusty Chain Kit provides an automated differential compiler benchmarker (`bench.rs`) accessible via `para bench`.

---

## 1. Benchmarking Matrix

The benchmarker executes differential compilation runs comparing baseline compiler options against aggressive optimization pipelines:

```
[ Target Contract: Token.sol ]
              |
      +-------+-------+
      |               |
      v               v
 [ Baseline ]   [ Optimized (via-ir) ]
 Runs: 200      Runs: 1,000,000 + IR
      |               |
      +-------+-------+
              |
              v
 [ Differential Report ]
 - Bytecode Size Reduction: -14.2% (Under 24KB EIP-170 limit)
 - Deployment Gas Savings:   -88,250 gas
 - Transfer Execution Gas:  -420 gas / call
 - Compile Time Overhead:   +850 ms
```

---

## 2. Metrics Evaluated

### 1. Bytecode Footprint (EIP-170 Limit Verification)
EIP-170 imposes a hard limit of 24,576 bytes (`0x6000`) on runtime bytecode. Contracts exceeding this limit fail to deploy on mainnet. The benchmarker flags contracts approaching or exceeding this ceiling and measures exact byte reductions under different optimizer settings.

### 2. Deployment Gas
Measures the gas consumed during contract deployment (`CREATE` / `CREATE2`). Gas reductions directly reduce protocol deployment costs during high-gas network conditions.

### 3. Execution Gas Profiling
Simulates execution calls against the contract's primary entrypoints (e.g., `transfer`, `mint`, `swap`), computing the exact gas delta per invocation.

### 4. Compilation Latency Overhead
Measures the wall-clock compilation time between standard bytecode generation and `via-ir` optimization to help engineering teams determine whether to enable `via-ir` during local development or reserve it for release builds.

---

## 3. Optimization Profiles

| Profile Name | Optimizer Runs | Pipeline Mode | Target Goal |
| :--- | :--- | :--- | :--- |
| `standard` | 200 | Legacy AST | Fast compilation for local test iteration |
| `aggressive` | 1,000,000 | `via-ir` Enabled | Maximum runtime execution gas savings for production |
| `size` | 1 | Stack-to-memory optimizer | Minimum bytecode footprint to satisfy EIP-170 |

---

## 4. CLI Usage

Run a differential benchmark on a contract:
```bash
para bench contracts/Token.sol
```

Benchmark with a specific target profile:
```bash
para bench contracts/Token.sol --profile aggressive
```

Export complete benchmark metrics to JSON:
```bash
para bench contracts/Token.sol --json
```

Example JSON Output:
```json
{
  "target": "contracts/Token.sol",
  "baseline": {
    "profile": "standard",
    "bytecode_size_bytes": 18450,
    "deployment_gas": 1450200,
    "runtime_avg_gas": 42150,
    "compile_time_ms": 140
  },
  "optimized": {
    "profile": "aggressive",
    "bytecode_size_bytes": 15820,
    "deployment_gas": 1361950,
    "runtime_avg_gas": 41730,
    "compile_time_ms": 980
  },
  "size_delta_percent": -14.25,
  "deployment_gas_saved": 88250,
  "compile_time_overhead_ms": 840
}
```
