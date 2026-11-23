# Upgrade Proxy Contract

A Soroban smart contract that provides a secure, transparent governance framework for managing upgradeable on-chain applications on the Stellar network. The contract enforces a mandatory 7-day review window before any upgrade can be executed, allowing the community to inspect incoming WASM binaries and abort malicious or flawed upgrades.

## Overview

The upgrade proxy wraps Soroban's `update_current_contract_wasm` call behind a strict governance layer with time-locked approval. Only an authorized admin can propose and execute upgrades, while both the admin and a separate DAO council can abort pending upgrades during the review period.

## Public Functions

### Initialization

- **`initialize(admin: Address, dao_council: Address)`** — Initialize the proxy governance state with admin and DAO council addresses (call exactly once at deployment)

### Upgrade Lifecycle

- **`propose_upgrade(new_wasm: Bytes)`** — Upload new WASM bytecode and start the mandatory 7-day review window (admin only)
- **`abort_upgrade(caller: Address)`** — Cancel a pending upgrade before it takes effect (admin or DAO council only)
- **`execute_upgrade()`** — Apply a pending upgrade once the 7-day timelock has elapsed (admin only)

### Query Functions

- **`pending_upgrade()`** — Return the current pending upgrade details, or None if no upgrade is queued
- **`admin()`** — Return the current admin address
- **`dao_council()`** — Return the current DAO council address
- **`timelock_remaining()`** — Return the number of seconds remaining until the pending upgrade can be executed

## Building

Build the contract for deployment:

```bash
cd contracts/upgrade-proxy
cargo build --target wasm32-unknown-unknown --release -p upgrade-proxy
```

The compiled WASM binary will be at:
```
target/wasm32-unknown-unknown/release/upgrade_proxy.wasm
```

## Testing

Run the test suite:

```bash
cd contracts/upgrade-proxy
cargo test -p upgrade-proxy
```

## Security Model

- **7-day timelock**: All upgrades must wait 604,800 seconds (7 days) before execution
- **Dual authorization**: Admin proposes/executes upgrades; both admin and DAO council can abort
- **Single pending upgrade**: Only one upgrade can be queued at a time
- **Timestamp-based**: Uses Unix timestamps for reliable, human-readable guarantees independent of ledger velocity
- **Storage persistence**: Governance state survives WASM upgrades through namespaced persistent storage keys

## License

Copyright 2024 Stellar-K8s Contributors. Licensed under the Apache License, Version 2.0.
