# HTLC Contract

## Overview

The HTLC (Hash Time-Locked Contract) securely holds tokens until a receiver provides a valid secret preimage before a specified ledger expiry. If the receiver does not claim the funds before expiry, the original sender can refund them. Each HTLC is identified by a unique SHA-256 hashlock, and its status is tracked as `Active`, `Claimed`, or `Refunded`.

## Public Functions

The contract exposes the following public functions:

- `lock`: Transfers tokens from the authorized sender into the contract and creates an HTLC using the receiver's address, token, amount, SHA-256 hashlock, and expiry ledger.
- `claim`: Allows the authorized receiver to claim the locked tokens by providing a preimage whose SHA-256 hash matches the stored hashlock before the expiry ledger.
- `refund`: Allows the authorized original sender to recover the locked tokens once the expiry ledger has been reached.
- `get_htlc`: Retrieves the stored HTLC entry for a given hashlock, returning `None` if no matching entry exists.

## HTLC Lifecycle

An HTLC follows one of three states:

| Status | Description |
|---|---|
| `Active` | Tokens are locked and awaiting a claim or refund. |
| `Claimed` | The receiver successfully revealed the correct preimage and received the tokens. |
| `Refunded` | The sender recovered the tokens after the expiry ledger. |

The receiver can claim only while the current ledger sequence is less than `expiry_ledger`. The sender can refund when the current ledger sequence is equal to or greater than `expiry_ledger`.

## Validation and Security

The contract enforces the following checks:

- The sender must authorize the `lock` operation.
- The locked amount must be greater than zero.
- The expiry ledger must be greater than the current ledger sequence when locking.
- A hashlock cannot be used to create another HTLC while an entry with that hashlock exists.
- Only the designated receiver can claim an HTLC.
- The supplied preimage must match the stored SHA-256 hashlock.
- Only the original sender can refund an HTLC.
- An HTLC cannot be claimed or refunded more than once.

## Build and Test

From the contract directory, run:

```bash
# Build the contract for WebAssembly
cargo build --target wasm32-unknown-unknown --release

# Run the contract tests
cargo test
```

## Project Structure

```text
htlc/
├── src/
│   ├── lib.rs
│   └── crypto.rs
├── Cargo.toml
└── README.md
```
