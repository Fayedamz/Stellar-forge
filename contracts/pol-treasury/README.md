# POL Treasury

POL Treasury is a Soroban contract that lets a DAO acquire reserve assets in
exchange for its native tokens. The bonding discount falls as reserves grow,
and each bond must meet the configured minimum backing ratio. Native token
payouts vest over time and can be claimed after the configured cliff.

## Contract entry points

| Function | What it does |
| --- | --- |
| `initialize` | Sets the admin, token addresses, bonding parameters, minimum backing ratio, and vesting terms once. |
| `bond` | Accepts reserve tokens, adds the native token payout to the bonder's vesting schedule, and returns the payout and discount in basis points. |
| `claim` | Transfers the beneficiary's currently claimable native tokens and returns the amount transferred. |
| `bond_state` | Returns the current bonding curve state, including the reserve balance and discount parameters. |
| `vesting_schedule` | Returns the beneficiary's vesting schedule and fails if none exists. |
| `reserve_state` | Returns the tracked reserve valuation, supply valuation, and backing ratio in basis points. |
| `get_treasury_state` | Returns a combined snapshot of the token addresses, bonding state, reserve metrics, and locked vesting total. |

## Build and test

Run these commands from the repository root with Rust and Cargo installed:

```sh
rustup target add wasm32v1-none
cargo build --manifest-path contracts/pol-treasury/Cargo.toml --release --target wasm32v1-none
cargo test --manifest-path contracts/pol-treasury/Cargo.toml
```
