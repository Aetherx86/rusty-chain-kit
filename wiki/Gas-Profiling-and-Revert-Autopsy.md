# Gas Profiling and Revert Autopsy

Optimizing smart contract performance and debugging failed transactions requires deep visibility into opcode execution costs and precise error calldata dissection.

Rusty Chain Kit provides cycle-accurate gas profiling (`trace.rs`) and a comprehensive revert analysis engine (`autopsy.rs`).

---

## 1. Cycle-Accurate Gas Flamegraphs

The profiling engine aggregates gas consumption per opcode and call frame, outputting hierarchical Flamegraphs compatible with [Speedscope](https://www.speedscope.app/) and web visualizers:

```
[ Root Call: Router.execute() - 145,200 gas ]
  |-- [ Vault.deposit() - 65,000 gas ]
  |     |-- SLOAD (Slot 1) - 2,100 gas
  |     |-- SSTORE (Slot 1) - 20,000 gas
  |     +-- LOG3 (DepositEvent) - 1,850 gas
  +-- [ Token.transferFrom() - 48,000 gas ]
        |-- SLOAD (Balance Owner) - 2,100 gas
        |-- SSTORE (Balance Owner) - 5,000 gas
        +-- SSTORE (Balance Recipient) - 20,000 gas
```

### Opcode Attribution Breakdown
- **Storage Opcodes**: `SLOAD`, `SSTORE` (cold vs warm access accounting).
- **Hashing**: `KECCAK256` (base cost plus word-length data expansion).
- **External Calls**: `CALL`, `STATICCALL`, `DELEGATECALL` (gas stipend and memory expansion costs).
- **Event Logging**: `LOG0` through `LOG4` (memory topic encoding and byte payload fees).

---

## 2. Revert Autopsy Engine

When a transaction reverts on-chain, the returned byte array encodes the exact failure reason. Rusty Chain Kit automatically unpacks this payload into plain-language diagnostics:

### 1. Standard Error Strings (`Error(string)`)
- **Selector**: `0x08c379a0`
- **Format**: ABI-encoded string message (e.g., `require(balance >= amount, "insufficient balance")`).
- **Diagnosis**: Extracted string message and failing contract address.

### 2. Custom Solidity Errors
- **Selector**: First 4 bytes of `keccak256("CustomError(uint256,address)")`.
- **Diagnosis**: Identifies custom error selector and unpacks parameter values.

### 3. Compiler Panics (`Panic(uint256)`)
- **Selector**: `0x4e487b71`
- **Specification**: Solidity compiler-generated invariant violations and arithmetic failures:

| Panic Code | Name | Description & Remediation |
| :--- | :--- | :--- |
| `0x01` | Assertion Failure | `assert(false)` failed; an internal contract invariant was violated. |
| `0x11` | Arithmetic Overflow/Underflow | Unchecked integer operation exceeded min or max bounds; verify arithmetic or wrap in `unchecked {}`. |
| `0x12` | Division by Zero | Denominator in `/` or `%` evaluated to `0`; add zero-check before division. |
| `0x21` | Invalid Enum Conversion | Value converted to enum exceeds number of defined enum variants. |
| `0x22` | Storage Byte Array Corruption | Encoded storage byte array header size is invalid. |
| `0x31` | Pop on Empty Array | Called `.pop()` on a dynamic storage array of length 0. |
| `0x32` | Array Out of Bounds | Array access `array[index]` where `index >= array.length`. |
| `0x41` | Out of Memory | Attempted to allocate dynamic memory exceeding available execution bounds. |
| `0x51` | Zero Internal Function Call | Called an uninitialized variable of internal function pointer type. |

---

## 3. CLI Usage

### Profile Gas and Export Flamegraph
```bash
para profile 0x9a8b... --flamegraph profile.json
```

### Trace On-Chain Transaction Execution
```bash
para trace 0x9a8b...
```

### Autopsy Revert Calldata
Pass a raw revert hex string:
```bash
para autopsy 0x4e487b710000000000000000000000000000000000000000000000000000000000000011
```
Output:
```
[Revert Autopsy]
Type: Solidity Compiler Panic
Code: 0x11 (Arithmetic Overflow or Underflow)
Explanation: An integer operation exceeded standard numeric limits (underflow or overflow).
Remediation: Verify input parameters or check for subtraction where subtrahend exceeds minuend.
```

Or pass a failed transaction hash directly:
```bash
para autopsy 0x7c9f...
```
