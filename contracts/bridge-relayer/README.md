# Bridge Relayer Contract

The `bridge-relayer` Soroban smart contract verifies and dispatches cross-chain messages for Stellar-K8s. It validates that a configured threshold of authorized relayers have signed a message's SHA-256 digest with Ed25519 signatures before allowing execution. It also tracks per-domain nonces to enforce replay protection across source and destination chain pairs.

## Public Functions

- `initialize(env, relayers, threshold)` — Initializes the contract with the authorized relayer addresses and the required signature threshold.
- `verify_attestations(env, message, attestations)` — Verifies that at least `threshold` distinct authorized relayers have provided valid Ed25519 signatures over the message digest.
- `execute(env, message, attestations)` — Verifies relayer attestations, consumes the message nonce for replay protection, and returns the `DispatchResult` containing the nonce, target, and payload.
- `current_nonce(env, source_chain, destination_chain)` — Returns the current expected nonce for a given source and destination chain pair.

## Build and Test

Run the following commands from the repository root:

```bash
# Build the contract
cargo build -p bridge-relayer --target wasm32-unknown-unknown --release

# Run contract tests
cargo test -p bridge-relayer
```
