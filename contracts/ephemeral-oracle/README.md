# Ephemeral Oracle

`ephemeral-oracle` is a high-frequency price feed that stores signed off-chain
prices exclusively in Soroban Temporary Storage, so every entry is
automatically evicted by the network once its TTL expires and the contract
accumulates no long-term state. A single authorized `updater` address submits
prices, while an `admin` address controls configuration and can rotate the
updater or revoke a bad price. Dependent contracts should read prices with
`get_price_checked`, which rejects prices that are about to expire.

## How it works

Prices are keyed by raw asset symbol bytes (for example `b"XLM/USD"`) and are
written only to Temporary Storage — never to Persistent Storage. Each entry
lives for the configured `price_ttl` (default 50 ledgers, roughly 5 minutes)
and carries the price in the smallest unit (× 10^7 for USD pairs), the ledger
at which it was written, and the updater's Ed25519 signature. The `admin` and
`updater` addresses and the `OracleConfig` are kept in Instance Storage.

## Public functions

- `initialize` — Set the admin and updater addresses and the initial configuration; callable exactly once, with optional `price_ttl` and `max_batch_size` overrides.
- `set_updater` — Rotate the updater address; admin only.
- `update_config` — Change `price_ttl` and/or `max_batch_size` at runtime; admin only.
- `update_price` — Submit a signed price for one asset; updater authorization required.
- `batch_update` — Submit signed prices for many assets in one transaction, up to `max_batch_size` (default 100, hard cap 500); updater authorization required.
- `get_price` — Return the latest price for an asset, with its write ledger and remaining TTL; errors if the entry has been evicted.
- `get_price_checked` — Like `get_price`, but errors with `PriceFeedExpired` when the remaining TTL is at most 1 ledger; recommended for DeFi consumers.
- `bump_ttl` — Extend a price entry's TTL by a number of ledgers; callable by anyone, for example keeper bots.
- `revoke_price` — Immediately remove a price entry, for emergency recall of bad data; admin only.
- `get_config` — Return the current `OracleConfig` (`price_ttl`, `max_batch_size`).
- `get_updater` — Return the registered updater address.
- `get_admin` — Return the admin address.

## Checks

Run the contract tests with `cargo test --manifest-path contracts/ephemeral-oracle/Cargo.toml`.
Build the Wasm artifact with
`cargo build --manifest-path contracts/ephemeral-oracle/Cargo.toml --target wasm32-unknown-unknown --release`.
