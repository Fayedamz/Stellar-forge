# Fee Vault

Fee Vault provides an allowance that limits how much an operator can spend during a time window. Callers can check the remaining allowance and consume an amount; consumption requires the operator's authorization.

## Public functions

These functions are defined on the `FeeVault` contract in `src/lib.rs`.

- `initialize` — Initialization entry point; currently has no implementation.
- `set_allowance` — Creates and returns an allowance for an authorized operator, with a spending limit and time window.
- `check_allowance` — Returns whether the allowance has at least the requested amount available.
- `consume` — Requires the operator's authorization and attempts to charge an amount against the allowance, returning whether it succeeded.

## Build

Build the contract's WebAssembly artifact from the repository root:

```bash
rustup target add wasm32-unknown-unknown
cargo build --manifest-path contracts/fee-vault/Cargo.toml --target wasm32-unknown-unknown --release
```

## Test

Run the contract tests from the repository root:

```bash
cargo test --manifest-path contracts/fee-vault/Cargo.toml
```
