# Trustless Light Client Verification

Centralized RPC providers (e.g., Infura, Alchemy, public nodes) are single points of failure. An untrusted or compromised RPC endpoint can return fabricated balances, censored events, or outdated state data.

Rusty Chain Kit provides native light-client state root verification (`trustless.rs`), allowing developers to mathematically verify on-chain state without placing blind trust in centralized RPC providers.

---

## 1. Verification Architecture

```
[ Untrusted Remote RPC Endpoint ]
               |
     (Returns Claimed State & Merkle Proofs)
               |
               v
[ Consensus Sync Committee Header Verification ]
               |
     (Validate Consensus Signatures & State Root)
               |
               v
[ Merkle-Patricia Trie Proof Verification ]
  |-- Verify Account Proof: `eth_getProof(address)`
  |     - Account Balance
  |     - Nonce
  |     - Storage Root Hash
  |     - Code Hash
  +-- Verify Storage Proof: `eth_getProof(address, [slot])`
        - Cryptographic proof of slot value against storage root
               |
               v
  [ Trustless On-Chain Verification Complete ]
```

---

## 2. Cryptographic Proof Components

1. **Consensus Header Validation**: Checks the latest finalized or optimistic Ethereum beacon chain block header signed by the active sync committee.
2. **Account State Verification**: Re-computes the Merkle-Patricia Trie path from the consensus state root to the target account's address hash (`keccak256(address)`). Verifies that the returned account node (nonce, balance, storage root, code hash) matches the trie root.
3. **Storage Slot Proof Verification**: Computes the storage trie path for specific storage keys (`keccak256(slot)`), mathematically proving that the value stored at that slot on-chain has not been tampered with.

---

## 3. CLI Usage

Verify an account's state trustlessly:
```bash
para rpc trustless 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
```

Verify state at a specific historical block number:
```bash
para rpc trustless 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --block 20000000
```

Output:
```
[Trustless State Verification]
Subject Address:  0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
Block Number:     20,000,000
State Root:       0x7a8b...3f12
Account Proof:    VALID (Merkle-Patricia Trie path verified)
Verified Balance: 421.58 ETH
Verified Nonce:   1,248
Code Hash:        0xc5d2...ad80 (EOA)
Status:           Trustless Verification Succeeded
```
