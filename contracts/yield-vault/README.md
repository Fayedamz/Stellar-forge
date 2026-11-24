# Yield Vault Contract

An ERC-4626 style auto-compounding yield vault for Stellar Soroban. Users deposit an underlying token (e.g., USDC) to receive proportional vault share tokens representing a fractional claim on the pooled assets, which grow as yield is harvested from an external yield protocol and compounded back into the vault. A configurable basis-point protocol fee is deducted from gross yield only (minted as fee shares to the fee recipient, never taken from principal), and a runtime monotonicity guard reverts any harvest that would decrease the share price.

## Public Functions

All contract entry points defined in `src/lib.rs`:

- `initialize` — Initializes the vault with an admin, underlying asset, external yield protocol, fee recipient, and protocol fee in basis points (`<= 10_000`).
- `deposit` — Transfers underlying asset tokens from the caller into the vault and mints proportional share tokens (1:1 on the initial deposit).
- `withdraw` — Burns the caller's share tokens and returns the corresponding amount of underlying assets.
- `harvest` — Claims pending rewards from the external yield protocol, mints protocol fee shares to the fee recipient, compounds net yield into total assets, and enforces share-price monotonicity, returning `(gross_yield, fee_assets, net_yield)`.
- `set_fee_recipient` — Updates the address that receives protocol fee shares (admin only).
- `set_fee_bps` — Updates the protocol fee in basis points, up to `10_000` (admin only).
- `set_paused` — Pauses or unpauses deposits and withdrawals while keeping `harvest` operational (admin only).
- `total_assets` — Returns the total amount of underlying assets currently managed by the vault.
- `total_shares` — Returns the total number of vault share tokens currently outstanding.
- `share_balance` — Returns the share token balance held by a given address.
- `share_price` — Returns the current share price scaled by `SHARE_PRECISION` (`1e18`), defaulting to `1e18` when no shares exist.
- `preview_withdraw` — Calculates how many underlying asset units a given number of shares would currently redeem for.
- `preview_deposit` — Calculates how many share tokens a given underlying asset deposit would currently mint.
- `last_share_price` — Returns the last recorded `SHARE_PRECISION`-scaled share price used by the monotonicity guard.
- `get_config` — Returns the vault configuration tuple `(asset, yield_protocol, fee_recipient, fee_bps)`.

## Modules

| File | Purpose |
|------|---------|
| `src/lib.rs` | `YieldVault` contract entry points, storage keys (`DataKey`), error codes (`VaultError`), and share accounting |
| `src/harvest.rs` | Harvest execution and fee/monotonicity helpers: `execute_harvest`, `compute_fee`, `compute_net_yield`, `assert_share_price_monotonic` |
| `src/test.rs` | Unit tests covering deposits, withdrawals, fee deduction, pause controls, and multi-cycle harvest compounding |

## Building

```bash
cd contracts/yield-vault
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cd contracts/yield-vault
cargo test
```
