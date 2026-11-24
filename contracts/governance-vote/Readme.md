# Governance Vote Contract

## Overview

The Governance Vote contract is a Soroban smart contract that provides an on-chain governance system for proposing, voting on, and executing changes to governance parameters. It uses weighted voting, dynamic quorum calculations, and configurable execution timelocks to ensure proposals are properly evaluated before changes are applied. It also supports proposal expiration and integration with an external GitOps manager for forwarding approved infrastructure payloads.

## Features

- **Proposal Management:** Create and track governance proposals for parameter changes.
- **Weighted Voting:** Allow participants to vote using their assigned voting weight.
- **Vote Changes:** Support changing an existing vote while automatically updating vote tallies.
- **Dynamic Quorum:** Calculate the required voting participation based on total participating weight and a configurable quorum percentage.
- **Execution Timelock:** Enforce a waiting period before approved proposals can be executed.
- **Proposal Expiration:** Gracefully expire abandoned proposals after a configurable expiration horizon.
- **GitOps Integration:** Forward approved infrastructure payload hashes to an external GitOps manager.
- **Administrative Controls:** Allow the administrator to configure voting parameters and contract settings.

## Public Functions

### Initialization

| Function | Description |
|---|---|
| `__constructor` | Initializes the contract with an administrator, total participating weight, quorum percentage in basis points, and proposal expiration horizon. |

### Administrative Configuration

| Function | Description |
|---|---|
| `set_admin` | Updates the contract administrator. |
| `set_total_weight` | Updates the total participating weight used to calculate the dynamic quorum threshold. |
| `set_quorum_percent_bps` | Updates the required quorum participation percentage in basis points. |
| `set_expiration_horizon` | Updates the number of ledgers after a voting deadline before an abandoned proposal can expire. |
| `set_gitops_manager` | Configures or removes the external GitOps manager contract address. |
| `gitops_manager` | Returns the currently configured GitOps manager address, if one exists. |

### Proposal Lifecycle and Voting

| Function | Description |
|---|---|
| `propose` | Creates a new governance proposal with a title, parameter, proposed value, voting duration, and execution delay. Returns the proposal ID. |
| `vote` | Allows an authenticated voter to cast or change a weighted vote on an active proposal. |
| `finalize` | Finalizes a proposal after voting closes, determining whether it passed or was rejected based on quorum and voting results. |
| `execute` | Applies an approved proposal's parameter change after its execution timelock has elapsed. |
| `forward_to_gitops` | Forwards an approved proposal's 32-byte payload hash to the configured GitOps manager after the execution timelock has elapsed. |
| `expire` | Expires an abandoned active proposal after its voting deadline and configured expiration horizon have passed. |

### Read-Only Queries

| Function | Description |
|---|---|
| `admin` | Returns the current administrator address. |
| `total_weight` | Returns the currently configured total participating weight. |
| `quorum_percent_bps` | Returns the configured quorum percentage in basis points. |
| `expiration_horizon` | Returns the configured proposal expiration horizon in ledgers. |
| `proposal_count` | Returns the total number of proposals created. |
| `current_quorum_threshold` | Returns the current quorum threshold as an absolute voting weight. |
| `get_proposal` | Returns the proposal record for a given proposal ID, or `None` if it does not exist. |
| `get_vote` | Returns the stored ballot for a specific voter and proposal, or `None` if no ballot exists. |
| `get_parameter` | Returns the currently applied value of a governance parameter, or `None` if it has not been set. |

## Proposal Lifecycle

A governance proposal follows these stages:

1. **Proposal Creation:** An authenticated proposer creates a proposal using `propose`.
2. **Voting:** Participants cast weighted votes using `vote`. Participants can change their votes while voting remains open.
3. **Finalization:** After the voting period closes, anyone can call `finalize` to determine the outcome.
4. **Quorum Evaluation:** The contract checks whether total voting participation meets the required quorum and whether votes in favor exceed votes against.
5. **Execution Waiting Period:** Successful proposals enter the Passed state and must wait for the configured execution delay.
6. **Execution:** The `execute` function applies the approved parameter change and marks the proposal as Executed.

Abandoned active proposals can also be marked as Expired after the configured expiration horizon.

## Proposal States

| State | Description |
|---|---|
| `Active` | The proposal is open for voting. |
| `Passed` | The proposal met the quorum requirement and received more votes in favor than against. |
| `Rejected` | The proposal failed to meet quorum or did not receive a strict majority of votes in favor. |
| `Executed` | The approved proposal's parameter change has been applied. |
| `Expired` | The proposal was abandoned and expired after its configured expiration horizon. |

## Voting and Quorum

The contract uses weighted voting rather than counting each participant as one vote.

The quorum threshold is calculated using:

```text
Quorum Threshold = Total Weight × Quorum Percentage
```

The quorum percentage is configured in basis points, where:

- `10000` basis points = 100%.
- `6000` basis points = 60%.
- `5000` basis points = 50%.

Votes are categorized as:

- `For`: Supports the proposal.
- `Against`: Opposes the proposal.
- `Abstain`: Counts toward participation quorum but does not contribute to votes in favor or against.

A proposal passes only when quorum is met and the total weight voting in favor is strictly greater than the total weight voting against.

## GitOps Integration

The contract supports integration with an external GitOps manager.

Once a proposal has passed and its execution timelock has elapsed, `forward_to_gitops` can forward a 32-byte infrastructure payload hash and the proposal ID to the configured GitOps manager.

The GitOps manager is responsible for aggregating the required Ed25519 signatures and emitting a webhook-compatible event.

The GitOps integration must be configured using `set_gitops_manager` before payloads can be forwarded.

## Building and Testing

Navigate to the contract directory:

```bash
cd contracts/governance-vote
```

### Build

Compile the contract for the WebAssembly target:

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Run Tests

Execute the contract test suite:

```bash
cargo test
```

## Project Structure

```text
contracts/
└── governance-vote/
    ├── src/
    │   ├── lib.rs
    │   └── proposal.rs
    ├── Cargo.toml
    └── README.md
```

## Notes

- The contract is built using the Soroban SDK.
- Administrative configuration changes require administrator authorization.
- Proposal creation and voting require address authorization.
- Voting is weighted and supports vote changes while voting is open.
- Proposal execution is permissionless after the required timelock.
- Expired and rejected proposals cannot be executed.
- The contract emits events for proposal creation, voting, finalization, execution, and expiration.

