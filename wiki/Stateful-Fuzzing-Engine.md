# Stateful Fuzzing Engine

Static analysis identifies known syntax patterns but misses complex runtime edge cases. Standard property-based testing evaluates single functions in isolation, but smart contract exploits typically involve multi-transaction state transitions across multiple blocks.

Rusty Chain Kit embeds a high-throughput stateful snapshot fuzzer (`verify.rs`) capable of executing thousands of multi-call transaction sequences per second.

---

## 1. Stateful Fuzzing Architecture

```
[ Target Contract State S_0 ]
              |
      (Snapshot State S_0)
              |
      +-------+-------+
      | (Call Seq 1)  | (Call Seq 2)
      v               v
 [ Step 1: deposit ] [ Step 1: mint ]
 [ Step 2: swap    ] [ Step 2: burn ]
 [ Step 3: withdraw] [ Step 3: transfer ]
      |               |
      +-------+-------+
              |
              v
 [ Invariant Verification: S_n ]
 assert(vault.solvency())
              |
      +-------+-------+
      | (Passed)      | (Failed)
      v               v
 (Next Sequence) [ Counterexample Found ]
                      |
              (Call Graph Shrinking)
                      |
                      v
             [ Minimal 2-Step PoC ]
```

---

## 2. Mutational Call Sequence Generation

The fuzzer tests smart contracts through randomized, multi-actor call graphs:

1. **Transaction Interleaving**: Generates sequences where multiple caller addresses (`Alice`, `Bob`, `Attacker`) interact with the contract in non-linear sequences (e.g., `deposit(Alice) -> pause() -> transfer(Bob) -> withdraw(Alice)`).
2. **Boundary Value Injection**: Generates edge-case arguments including:
   - Zero values: `0`, `address(0)`.
   - Numeric extrema: `type(uint8).max`, `type(uint256).max`, `2^128 - 1`.
   - Off-by-one offsets: `block.timestamp + 1`, `msg.value - 1`.
   - Empty and oversized dynamic byte arrays.
3. **State Rollback via Snapshots**: After evaluating each sequence, the execution environment instantly reverts to the clean initial snapshot, preventing state contamination between independent test iterations.

---

## 3. Automated Call Sequence Shrinking

When an invariant violation is discovered, the raw sequence may contain 50+ irrelevant transactions. The shrinking engine automatically trims the call graph:
- Removes non-essential calls one by one.
- Simplifies numeric parameters toward zero.
- Produces a minimal, reproducible sequence that triggers the exact failure.

---

## 4. CLI Usage

Run the stateful fuzzer across 5,000 transaction sequences:
```bash
para audit contracts/ --fuzz
```

Increase fuzzing iterations to 50,000 runs for deep protocol audits:
```bash
para audit contracts/ --fuzz --runs 50000
```

Combine stateful fuzzing with formal SMT proofs:
```bash
para audit contracts/ --fuzz --prove
```
