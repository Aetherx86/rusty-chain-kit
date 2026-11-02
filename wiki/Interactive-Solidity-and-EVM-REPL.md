# Interactive Solidity and EVM REPL

Testing isolated Solidity functions, bitwise operations, or ABI encodings typically requires authoring a temporary contract, configuring a test script, and running a test runner.

Rusty Chain Kit provides an interactive read-eval-print loop (`repl.rs`) accessible via `para repl`, allowing developers to evaluate expressions, hash functions, and opcode sequences instantly.

---

## 1. Capabilities

- **Expression Evaluation**: Direct evaluation of arithmetic, bitwise shifts, and conditional operators.
- **Solidity Type Introspection**: Querying type bounds (e.g., `type(uint256).max`, `type(int128).min`).
- **Cryptographic Primitives**: Computing `keccak256`, `sha256`, and `ripemd160` hashes.
- **ABI Encoding & Decoding**: Evaluating `abi.encode(...)`, `abi.encodePacked(...)`, and `abi.encodeWithSelector(...)`.
- **Gas Estimation**: Outputs estimated execution gas alongside computed return values.

---

## 2. Interactive Session Usage

Launch the interactive REPL:
```bash
para repl
```

Interactive Session Example:
```
Welcome to Para Solidity REPL v0.1.1-beta
Type 'exit' or press Ctrl+D to exit. Type 'help' for instructions.

>> 1 ether / 1 gwei
1000000000 (uint256) [Gas: 3]

>> keccak256("transfer(address,uint256)")
0xa9059cbb2ab09eb219583f4a59a5d0623ade346d962bcd4e46b11da047c9049b (bytes32) [Gas: 36]

>> bytes4(keccak256("transfer(address,uint256)"))
0xa9059cbb (bytes4) [Gas: 39]

>> type(uint128).max
340282366920938463463374607431768211455 (uint128) [Gas: 3]

>> uint256(1) << 255
57896044618658097711785492504343953926634992332820282019728792003956564819968 (uint256) [Gas: 3]
```

---

## 3. Piped Non-Interactive Evaluation

Evaluate a single expression directly from the command line:
```bash
para repl --eval "keccak256('hello world')"
```

Use in shell scripts and automated deployment workflows:
```bash
SELECTOR=$(para repl --eval "bytes4(keccak256('deposit()'))" | awk '{print $1}')
echo "Function selector: $SELECTOR"
```
