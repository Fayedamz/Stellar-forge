\# Wrapped Token



\## Overview



The `wrapped-token` contract implements a SEP-41-compatible wrapped asset token on Stellar Soroban. It supports token transfers, allowances, minting, burning, and bridge relayer administration.



\## Public Functions



| Function | Description |

|---|---|

| `admin` | Returns the contract administrator. |

| `allowance` | Returns the approved allowance for an owner and spender. |

| `approve` | Sets a spender's token allowance and expiration ledger. |

| `balance` | Returns an account's token balance. |

| `burn` | Burns tokens from an account through the authorized relayer. |

| `burn\_from` | Burns tokens using an account's allowance through the authorized relayer. |

| `decimals` | Returns the token's decimal precision. |

| `initialize` | Initializes the token with its admin, relayer, metadata, and initial supply. |

| `mint` | Mints tokens to an account through the authorized relayer. |

| `name` | Returns the token name. |

| `relayer` | Returns the configured bridge relayer. |

| `paused` | Returns whether the contract is paused. |

| `set\_relayer` | Updates the bridge relayer. |

| `set\_paused` | Updates the contract's pause state. |

| `symbol` | Returns the token symbol. |

| `total\_supply` | Returns the total token supply. |

| `transfer` | Transfers tokens from one account to another. |

| `transfer\_from` | Transfers tokens using an approved allowance. |



\## Build



```bash

stellar contract build --manifest-path contracts/wrapped-token/Cargo.toml

```



\## Test



```bash

cargo test --manifest-path contracts/wrapped-token/Cargo.toml

```



\## License



Apache-2.0

