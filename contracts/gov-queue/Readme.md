# Gov Queue Contract

## Overview

The Gov Queue contract is a Soroban smart contract that manages on-chain governance proposals through a structured voting and execution process. It allows users to submit proposals, cast votes, and execute approved proposals after a configurable ledger-based delay. It also includes an emergency guardian mechanism that allows authorized guardians to cancel proposals through a multi-signature threshold.

## Features

- **Proposal Management:** Create and manage governance proposals containing a target contract, function name, calldata, and description.
- **Voting System:** Allow authenticated addresses to cast one yes or no vote per proposal.
- **Vote Quorum:** Automatically queue proposals when the configured minimum number of yes votes is reached.
- **Execution Delay:** Enforce a configurable waiting period before queued proposals can be executed.
- **Guardian Protection:** Allow designated guardians to collectively cancel proposals using a configurable signature threshold.
- **Execution Tracking:** Track proposal statuses, including Created, Queued, Executed, Cancelled, and Failed.
- **Administrative Controls:** Allow the contract administrator to update execution delays, guardian addresses, and voting quorum.

## Public Functions

### Initialization

| Function | Description |
|---|---|
| `initialize` | Initializes the contract with an administrator, guardian addresses, guardian threshold, execution delay, and voting quorum. Can only be called once. |

### Configuration

| Function | Description |
|---|---|
| `set_execution_delay` | Updates the minimum ledger delay required before a queued proposal can be executed. |
| `set_guardians` | Replaces the guardian set and updates the minimum signature threshold required for guardian cancellations. |
| `set_vote_quorum` | Updates the minimum number of yes votes required to queue a proposal. |

### Proposal Lifecycle

| Function | Description |
|---|---|
| `submit_proposal` | Creates a new governance proposal and returns its unique proposal ID. |
| `cast_vote` | Allows an authenticated address to cast a yes or no vote on a proposal. Each address can vote only once per proposal. |
| `queue_proposal` | Manually queues a proposal that has reached the required voting quorum but has not yet been queued. |
| `execute_proposal` | Executes a queued proposal after its required ledger-based execution delay has elapsed. |
| `cancel_proposal` | Allows a proposer to cancel their own proposal while it is still in the Created state, or guardians to collectively cancel eligible proposals through multi-signature approval. |

### Read-Only Queries

| Function | Description |
|---|---|
| `get_proposal` | Returns the complete proposal record for a given proposal ID. |
| `get_proposal_status` | Returns the current status of a specified proposal. |
| `get_execution_delay` | Returns the currently configured execution delay in ledgers. |
| `get_vote_quorum` | Returns the currently configured voting quorum. |
| `get_guardians` | Returns the current list of guardian addresses. |
| `proposal_count` | Returns the highest proposal ID issued by the contract, or zero if no proposals exist. |
| `has_voted` | Checks whether a particular address has already voted on a specified proposal. |

## Proposal Lifecycle

A typical governance proposal follows this process:

1. **Creation:** A proposer submits a proposal using `submit_proposal`.
2. **Voting:** Eligible addresses cast yes or no votes using `cast_vote`.
3. **Queuing:** Once the required yes-vote quorum is reached, the proposal automatically transitions to the Queued state. `queue_proposal` provides a manual alternative.
4. **Waiting Period:** The proposal must satisfy the configured execution delay.
5. **Execution:** Anyone can call `execute_proposal` once the proposal becomes eligible for execution.
6. **Final Status:** The proposal is marked as Executed if execution succeeds or Failed if execution fails.

Proposals can also be cancelled before execution through proposer cancellation or guardian multi-signature approval.

## Proposal Statuses

| Status | Description |
|---|---|
| `Created` | The proposal has been submitted and is open for voting. |
| `Queued` | The proposal has reached the required yes-vote quorum and is waiting for its execution delay. |
| `Executed` | The proposal has been successfully executed. |
| `Cancelled` | The proposal has been cancelled by its proposer or the guardian mechanism. |
| `Failed` | The proposal execution attempt failed. |

## Building and Testing

Navigate to the contract directory:

```bash
cd contracts/gov-queue
```

### Build

Compile the contract for the WebAssembly target:

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Run Tests

Run the contract's test suite:

```bash
cargo test
```

## Project Structure

```text
contracts/
└── gov-queue/
    ├── src/
    │   ├── lib.rs
    │   ├── execution.rs
    │   ├── types.rs
    │   └── test.rs
    ├── Cargo.toml
    └── README.md
```

## Notes

- The contract is built using the Soroban SDK.
- The administrator controls governance configuration.
- Proposal voting requires voter authorization.
- Each address can vote only once on a proposal.
- Proposal execution is permissionless once the required delay has elapsed.
- Guardian cancellations require the configured number of unique guardian signatures.

