# Formal Verification and SMT

Symbolic execution and Satisfiability Modulo Theories (SMT) constraint solving provide mathematical certainty that smart contract invariants hold across all possible execution states and transaction inputs, rather than relying solely on sample-based unit testing.

Rusty Chain Kit embeds a native symbolic execution engine (`verify.rs`) and integrates with the Z3 SMT solver to prove protocol safety properties, solvency conditions, and state transition equations.

---

## 1. Symbolic Execution Pipeline

```
               [ Solidity Source Code / Invariant Specification ]
                                       |
                           (Symbolic Branch Splitting)
                                       |
                 +---------------------+---------------------+
                 |                                           |
         [ Path Condition A ]                        [ Path Condition B ]
         balance >= amount                           balance < amount
                 |                                           |
     (State Equation Update)                       (Branch Revert Path)
     balance' = balance - amount                   require(false, "insufficient")
                 |                                           |
     (Safety Invariant Query)                      (SMT Solver Verification)
     assert(balance' + amount == balance)                    |
                 |                                           v
                 v                                     [ SAT Check ]
          [ UNSAT Proof ]                      Counterexample Generated or
     Invariant Formally Proved!                       Revert Confirmed
```

---

## 2. Invariant Formulation in Solidity

Developers specify formal invariants directly inside their contract files using standard Solidity function conventions prefixed with `invariant_` or `prove_`:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Vault {
    mapping(address => uint256) public balances;
    uint256 public totalDeposits;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
        totalDeposits += msg.value;
    }

    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, "insufficient balance");
        balances[msg.sender] -= amount;
        totalDeposits -= amount;
        (bool ok, ) = msg.sender.call{value: amount}("");
        require(ok, "transfer failed");
    }

    // Formal Invariant: Contract solvency condition
    function invariant_solvency() external view {
        assert(address(this).balance >= totalDeposits);
    }
}
```

---

## 3. SMT-LIB2 Constraint Encoding

When `para verify Vault.sol --invariant solvency` is invoked:

1. **Symbolic Memory & Storage**: State variables are modeled as unbounded bitvectors (`(_ BitVec 256)`) and mappings are modeled as uninterpreted SMT arrays (`(Array (_ BitVec 256) (_ BitVec 256))`).
2. **Path Constraints**: Branching conditionals (`require`, `if/else`) introduce path constraints accumulated in a first-order logic formula.
3. **Inversion & SAT Query**: The invariant condition $P(S)$ is negated: $\neg P(S')$.
   - **UNSAT (Unsatisfiable)**: No assignment of inputs can violate the invariant. The property is mathematically proven.
   - **SAT (Satisfiable)**: The SMT solver identifies an assignment of symbolic variables that satisfies $\neg P(S')$, yielding a concrete counterexample transaction sequence and call values.

---

## 4. Constant-Product AMM Proof Example

Consider an automated market maker pool invariant $k = x \cdot y$:

```solidity
contract ConstantProductAMM {
    uint256 public reserve0;
    uint256 public reserve1;

    function swap(uint256 amount0In, uint256 amount1Out) external {
        require(amount0In > 0, "zero input");
        uint256 balance0 = reserve0 + amount0In;
        uint256 balance1 = reserve1 - amount1Out;

        // Invariant: k must not decrease
        require(balance0 * balance1 >= reserve0 * reserve1, "K");

        reserve0 = balance0;
        reserve1 = balance1;
    }

    function invariant_k_non_decreasing() external view {
        assert(reserve0 * reserve1 >= 1000000);
    }
}
```

Running the formal verifier mathematically guarantees that regardless of integer rounding or multi-token swap sequences, pool reserves can never drop below the minimum liquidity threshold.

---

## 5. CLI Execution

Formally prove specific invariants:
```bash
para verify contracts/Vault.sol --invariant solvency
```

Combine static audit scanning with symbolic proof verification:
```bash
para audit contracts/ --prove
```

Output proof diagnostics and counterexamples in JSON format:
```bash
para verify contracts/Vault.sol --invariant solvency --json
```
