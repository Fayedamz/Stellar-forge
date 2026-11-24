# Options AMM

Options AMM is a Soroban contract designed to use a pool of collateral to
underwrite call and put options. Liquidity providers deposit collateral for
pool shares, while traders can request a premium quote, open a position, and
settle it after expiry.

## Contract entry points

| Function | What it does |
| --- | --- |
| `initialize` | Sets the pool's collateral token and initial implied volatility once. |
| `deposit` | Adds collateral to the pool and returns the number of liquidity shares credited. |
| `withdraw` | Redeems liquidity shares for collateral subject to the pool's collateral checks. |
| `quote` | Calculates the premium for a call or put using the supplied spot price and the pool's implied volatility. |
| `buy_option` | Opens and records a call or put position if the pool has enough collateral. |
| `settle` | Calculates the payoff for an expired position, records settlement, and returns the payout amount. |
| `get_state` | Returns the current pool state, including collateral, total shares, and implied volatility. |
| `get_position` | Returns the stored option position for the supplied ID. |

## Build and test

From the repository root, use these commands with Rust and Cargo installed:

```sh
rustup target add wasm32v1-none
cargo build --manifest-path contracts/options-amm/Cargo.toml --release --target wasm32v1-none
cargo test --manifest-path contracts/options-amm/Cargo.toml
```

The current manifest spells the SDK dependency `sorban-sdk` rather than
`soroban-sdk`, and the source contains unfinished Rust expressions. These
commands describe the intended build and test workflow once the crate is fixed.
