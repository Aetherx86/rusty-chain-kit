# Account Abstraction and EIP-7702

EIP-7702 introduces native code delegation for Externally Owned Accounts (EOAs), while ERC-4337 establishes decentralized transaction bundling via smart contract accounts. Rusty Chain Kit provides native tooling (`aa.rs`) to generate authorizations, validate UserOperation bundles, and decode Type 4 transaction envelopes.

---

## 1. EIP-7702: Code Delegation for EOAs

EIP-7702 allows an EOA to temporarily or permanently designate a smart contract implementation address by signing an authorization tuple. While designated, the account executes with full smart contract capabilities (session keys, batch transactions, sponsored gas) while retaining its original EOA identity and private key.

### Authorization Tuple Structure
```rust
pub struct Authorization {
    pub chain_id: u64,
    pub address: Address, // Delegation contract implementation
    pub nonce: u64,
    pub y_parity: u8,
    pub r: U256,
    pub s: U256,
}
```

### Delegation Bytecode Signature
When an account delegates code under EIP-7702, on-chain state queries (`eth_getCode`) return a 23-byte magic prefix:
```
0xef0100 + <20-byte designated contract address>
```
Rusty Chain Kit automatically identifies this prefix when inspecting addresses via `para chain address <addr> --inspect` or `para chain code <addr> --proxy`.

---

## 2. ERC-4337: UserOperation Bundle Simulation

In ERC-4337, users submit `UserOperation` structs to alternate mempools instead of standard EVM transactions:

```rust
pub struct UserOperation {
    pub sender: Address,
    pub nonce: U256,
    pub init_code: Bytes,
    pub call_data: Bytes,
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub paymaster_and_data: Bytes,
    pub signature: Bytes,
}
```

### Bundle Validation Checks
Rusty Chain Kit validates bundles against standard EntryPoint rules before submission:
1. **Verification Gas Threshold**: Verifies that `verificationGasLimit` is sufficient to cover contract validation logic without exceeding bundler safety caps.
2. **Paymaster Sponsorship**: If `paymasterAndData` is specified, checks that the paymaster has sufficient deposited collateral in the EntryPoint contract to sponsor transaction execution.
3. **Fee Consistency**: Validates that `maxFeePerGas` satisfies current block base fee requirements.

---

## 3. CLI Usage

### Generate and Sign an EIP-7702 Authorization Tuple
```bash
para aa generate-auth \
  --chain-id 1 \
  --address 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 \
  --nonce 0 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### Simulate an ERC-4337 UserOperation Bundle
```bash
para aa bundle --json
```

### Decode Type 4 Transactions and EntryPoint Calldata On-Chain
```bash
para chain tx 0x3b89... --decode-aa
```
This automatically decodes:
- Transaction envelope type (e.g., `Type 4 (EIP-7702)`).
- Extracted authorization tuples (target delegation address, chain ID, signer nonce, signature components).
- Unpacked individual `UserOperation` call sequences from EntryPoint `handleOps` calldata.
