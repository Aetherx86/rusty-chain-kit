# Storage Layout and Upgrade Inspector

The EVM utilizes a key-value storage model where persistent state variables reside in 32-byte (256-bit) slots indexed from slot `0` to $2^{256} - 1$. Improper ordering of state variables causes severe gas inefficiencies, and improper modifications between proxy upgrades can corrupt entire protocol balance sheets.

Rusty Chain Kit provides a dedicated storage layout and proxy collision analyzer (`storage.rs`) accessible via `para storage`.

---

## 1. Storage Slot Packing Rules

Contiguous state variables whose cumulative sizes fit within 32 bytes are packed into a single storage slot to minimize expensive `SSTORE` (20,000 / 5,000 gas) and `SLOAD` (2,100 / 100 gas) operations:

```
Slot Packing Example:
+-----------------------------------------------------------------------+
| Slot 0: [ owner: address (20B) ][ isPaused: bool (1B) ][ wasted: 11B ]|
+-----------------------------------------------------------------------+
| Slot 1: [ totalSupply: uint256 (32B)                                 ]|
+-----------------------------------------------------------------------+
| Slot 2: [ balances: mapping(address => uint256) (32B slot reservation)]|
+-----------------------------------------------------------------------+
```

### Type Size Mapping Reference

| Solidity Type | Size (Bytes) | Packing Behavior |
| :--- | :--- | :--- |
| `bool` | 1 | Packs with adjacent sub-32-byte types |
| `uint8` / `int8` | 1 | Packs with adjacent sub-32-byte types |
| `uint16` / `int16` | 2 | Packs with adjacent sub-32-byte types |
| `uint32` / `int32` | 4 | Packs with adjacent sub-32-byte types |
| `uint64` / `int64` | 8 | Packs with adjacent sub-32-byte types |
| `uint128` / `int128` | 16 | Two fit perfectly in one slot |
| `address` | 20 | Leaves 12 bytes available in current slot |
| `bytes32` / `uint256` | 32 | Always occupies an entire slot |
| `mapping(...)` | 32 | Base slot reserved; elements stored at `keccak256(key . slot)` |
| `array[]` | 32 | Length stored at base slot; elements at `keccak256(slot) + index` |

---

## 2. Upgrade Collision Hazards

In upgradeable proxy architectures (ERC-1967 Transparent, UUPS ERC-1822, Beacon, Diamond), state storage is preserved across upgrades while execution logic switches from Implementation V1 to Implementation V2.

If Implementation V2 modifies existing variable offsets, storage collision occurs:

```
[ Implementation V1 ]                                  [ Implementation V2 (Faulty) ]
+------------------------------------+                 +------------------------------------+
| Slot 0: [ owner (20B) ][ flag(1B) ]|                 | Slot 0: [ new_admin (20B) ]        | <-- COLLISION: SlotShift!
+------------------------------------+                 +------------------------------------+
| Slot 1: [ total_supply (32B) ]     |                 | Slot 1: [ owner (20B) ][ flag(1B) ]|
+------------------------------------+                 +------------------------------------+
| Slot 2: [ balances (mapping 32B) ] |                 | Slot 2: [ total_supply (uint128) ] | <-- MUTATION: TypeMutation!
+------------------------------------+                 +------------------------------------+
```

### Collision Severity Categories

1. **`SlotShift` (Critical)**: A new state variable was inserted before or between existing variables, shifting all subsequent variables into different slots. When upgraded, existing storage values will be interpreted as the wrong variables.
2. **`TypeMutation` (Critical)**: A variable's bitwidth or signedness changed between versions (e.g., `uint256` mutated to `uint128`), corrupting arithmetic decoding.
3. **`VariableRemoved` (High)**: A variable was removed in V2 without a placeholder gap, shifting all subsequent variables forward.
4. **`ProxyCollision` (Critical)**: An implementation variable overlaps with standard proxy storage slots.

---

## 3. Proxy Pattern Resolver

Rusty Chain Kit automatically scans runtime bytecode and on-chain contracts to identify proxy architectures:

- **EIP-1967 Transparent & Beacon Slots**:
  - Implementation Slot: `0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc`
  - Admin Slot: `0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103`
  - Beacon Slot: `0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50`
- **EIP-1167 Minimal Proxy Clones**: Identifies standard 45-byte runtime clone bytecode (`363d3d373d3d3d363d73...5af43d82803e903d91602b57fd5bf3`).
- **EIP-1822 UUPS**: Universal Upgradeable Proxy Standard implementation slot (`0xc5f1683af45b311782f7d9e63297920ab4b673b22ab188e73c0249e45e59e7e9`).
- **EIP-2535 Diamond Multi-Facet**: Identifies diamond cut selectors and facet routing mappings.
- **EIP-7702 Code Delegation**: Detects account delegation prefix `0xef0100` followed by the 20-byte designated implementation address.

---

## 4. CLI Usage

Inspect storage packing of a single contract:
```bash
para storage contracts/VaultV1.sol
```

Compare V1 and V2 implementations for upgrade safety:
```bash
para storage contracts/VaultV1.sol contracts/VaultV2.sol
```

Inspect on-chain proxy contracts directly:
```bash
para chain code 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --proxy
```
