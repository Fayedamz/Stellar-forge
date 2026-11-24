# Yield Stablecoin

`yield-stablecoin` is a SEP-41–compatible fungible token for Stellar Soroban
that layers two capabilities on top of the standard token interface:
programmable compliance and proportional yield distribution. Every mint,
burn, transfer, and approval passes through a compliance gate that blocks
blacklisted (sanctions-listed) and frozen addresses, and an admin multi-sig
can distribute yield by minting new tokens in proportion to each holder's
share of the total supply. The contract is a standalone Soroban workspace
(it declares its own `[workspace]` in `Cargo.toml`), independent of the root
Stellar-K8s binary-crate workspace.

## Compliance model

Two independent restrictions are enforced by the hooks in `src/hooks.rs`
before any state mutation:

- **Blacklist (sanctions list)** — permanently blocks an address from all
  inbound and outbound flows. Adding an address requires the admin; removing
  one requires the stricter `unblacklist_threshold` multi-sig quorum.
- **Freeze** — temporarily halts all flows for an address and is reversible
  by a single admin via `thaw_address`.

## Yield distribution

`distribute_yield` is authorized by the multi-sig quorum and mints
`total_yield_amount` tokens, crediting each recipient
`recipient_balance * total_yield_amount / total_supply_snapshot` (integer
division; rounding dust stays unminted). Blacklisted or frozen recipients are
skipped without aborting the epoch, so one restricted address cannot block
the whole distribution.

## Public functions

All functions below are defined in `src/lib.rs` on the `YieldStablecoin`
contract.

- `__constructor` — Initializes the contract once: sets the admin, token
  metadata (name, symbol, decimals), the multi-sig signer set, and the
  standard and un-blacklist quorum thresholds.
- `name` — Returns the token name.
- `symbol` — Returns the token symbol.
- `decimals` — Returns the number of decimal places (immutable after
  initialization).
- `total_supply` — Returns the total circulating supply.
- `balance` — Returns the token balance of an address (0 if the address has
  never held tokens).
- `allowance` — Returns the spending allowance an owner granted to a spender.
- `transfer` — Transfers `amount` tokens from `from` to `to`; requires the
  sender's authorization and compliance clearance for both parties.
- `transfer_from` — Transfers `amount` tokens from `from` to `to` using the
  spender's pre-approved allowance.
- `approve` — Approves `spender` to transfer up to `amount` on behalf of
  `owner`.
- `mint` — Mints `amount` new tokens to `to`; admin-only, and the recipient
  must pass the compliance gate.
- `burn` — Burns `amount` tokens from `from`; requires the holder's
  authorization and send-clearance.
- `distribute_yield` — Multi-sig authorized yield distribution: mints
  `total_yield_amount` and credits each recipient a proportional share of it
  based on their balance over the total-supply snapshot, skipping restricted
  recipients.
- `blacklist_address` — Adds an address to the on-chain sanctions list
  (admin-only).
- `unblacklist_address` — Removes an address from the sanctions list;
  requires the stricter `unblacklist_threshold` multi-sig quorum.
- `freeze_address` — Freezes an address, halting all inbound and outbound
  flows (admin-only).
- `thaw_address` — Lifts the freeze on an address (admin-only).
- `update_multisig` — Replaces the multi-sig signer set and both thresholds;
  requires the current quorum.
- `set_admin` — Transfers admin authority to a new address.
- `is_blacklisted` — Returns `true` if the address is on the sanctions list.
- `is_frozen` — Returns `true` if the address is currently frozen.

## Build

The contract uses `soroban-sdk` 27, which builds for the `wasm32v1-none`
target on Rust 1.82+ (not `wasm32-unknown-unknown`, whose default Wasm
features the Soroban host rejects).

```bash
cd contracts/yield-stablecoin
rustup target add wasm32v1-none   # one-time, if the target is missing
cargo build --target wasm32v1-none --release
```

## Test

```bash
cd contracts/yield-stablecoin
cargo test
```

The unit-test suite in `src/test.rs` covers metadata and supply accounting,
mint/burn/transfer happy paths, approve and `transfer_from`, blacklist and
freeze enforcement (including the multi-sig quorum needed to un-blacklist),
proportional yield distribution with dust and restricted-recipient skipping,
multi-sig config updates, and admin transfer.
