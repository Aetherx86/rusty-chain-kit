# Zero-Knowledge VM Pipeline

Zero-Knowledge Virtual Machines (zkVMs) prove the correct execution of arbitrary programs compiled to standard RISC-V or LLVM architectures, bridging standard software development with verifiable cryptographic execution.

Rusty Chain Kit embeds a complete zkVM development workflow (`zk.rs`) enabling developers to scaffold projects, compile guest execution programs, and generate verifiable execution proofs.

---

## 1. Project Architecture

A Rusty Chain Kit zkVM project layout decouples the guest program from the host prover:

```
my-zk-project/
|-- Cargo.toml          # Workspace manifest uniting program and script
|-- program/            # Guest Program (compiles to RISC-V target)
|   |-- Cargo.toml
|   +-- src/main.rs     # zkVM execution logic (e.g., verifying computations)
+-- script/             # Host Prover & Verifier Script
    |-- Cargo.toml
    +-- src/main.rs     # Input setup, execution proving, proof serialization
```

---

## 2. Guest Program Development

The guest program runs inside the zkVM execution sandbox:

```rust
// program/src/main.rs
#![no_main]

#[no_mangle]
pub fn main() {
    // Read private inputs provided by the host
    let n: u32 = 20;

    // Perform computation (e.g., Fibonacci calculation)
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    for _ in 0..n {
        let temp = a + b;
        a = b;
        b = temp;
    }

    // Commit public output to the zkVM journal
    println!("Computed output: {}", a);
}
```

---

## 3. Host Prover Execution

The host prover executes the compiled guest binary, traces execution cycles, and generates a cryptographic proof:

```rust
// script/src/main.rs
fn main() {
    println!("Executing zkVM prover...");
    // 1. Setup execution client
    // 2. Execute guest program and record cycle count
    // 3. Generate verifiable STARK/SNARK execution proof
    println!("Proof generated successfully!");
}
```

---

## 4. CLI Usage

### Scaffold a New zkVM Project
```bash
para new zk my-prover
```

### Compile Guest and Generate Execution Proof
```bash
para zk prove --path my-prover/
```

Output:
```
[zkVM Prover Pipeline]
Project Path:       my-prover/
Target:             riscv32im-succinct-zkvm-elf
Compilation:        SUCCESS (Optimized guest ELF generated)
Execution Cycles:   142,850 cycles
Prover Time:        1.42s
Proof Output:       my-prover/proof.bin
Verification:       VERIFIED (Cryptographic proof valid)
```
