# Zero-Knowledge Proof Verifier (`zk-verifier`)

`zk-verifier` is a Soroban smart contract providing on-chain Zero-Knowledge Proof (ZKP) verification for privacy-preserving asset transfers on the Stellar network. It manages a shielded pool backed by an incremental Merkle tree, tracks spent nullifiers in persistent storage to prevent replay attacks, and verifies both Groth16 and PLONK proofs against registered verifying keys.

## Architecture & Storage

- **Shielded Pool (`pool.rs`)**: Maintains deposited note commitments in an incremental Merkle tree and records historical Merkle roots.
- **Nullifier Registry (`nullifier.rs`)**: Persistently records spent nullifiers so double-spend attempts fail before expensive pairing operations.
- **Verifying Key Registry**: Stores active Groth16 and PLONK verifying keys in contract instance storage.

## Public Functions

All public entry points are exposed by `ZkVerifierContract` in `src/lib.rs`:

| Function | Description |
|---|---|
| `init` | Initialises contract administration, sets up the shielded pool, and optionally registers initial Groth16 and PLONK verifying keys. |
| `deposit` | Inserts a new note commitment into the shielded pool, updates the Merkle tree root, and emits a deposit event. |
| `verify_and_transfer` | Verifies a ZK proof (Groth16 or PLONK), marks the nullifier hash as spent to prevent double-spending, and executes the shielded transfer. |
| `verify_groth16` | Standalone, read-only Groth16 proof verification without modifying contract state or spending nullifiers. |
| `verify_plonk` | Standalone, read-only PLONK proof verification without modifying contract state or spending nullifiers. |
| `is_nullifier_spent` | Queries the nullifier registry to determine whether a given nullifier hash has already been spent. |
| `nullifier_spent_at` | Returns the ledger sequence number at which a nullifier hash was recorded as spent, or `None` if unspent. |
| `get_commitment_count` | Returns the total count of note commitments currently recorded in the shielded pool. |
| `is_known_root` | Checks whether a given 32-byte Merkle root is recorded in the contract's historical root buffer. |
| `set_verifying_key_g16` | Administrative function to update the stored Groth16 verifying key in instance storage. |
| `set_verifying_key_plonk` | Administrative function to update the stored PLONK verifying key in instance storage. |
| `extend_nullifier_ttl` | Permissionless function to batch-extend the persistent storage time-to-live (TTL) for specified nullifier hashes. |

## Build & Test Commands

To run tests and compile the contract from the repository root:

```bash
# Run unit and integration tests
cargo test --manifest-path contracts/zk-verifier/Cargo.toml

# Compile the contract to a release WASM binary
cargo build --manifest-path contracts/zk-verifier/Cargo.toml --target wasm32-unknown-unknown --release
```
