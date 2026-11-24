# GitOps Manager Contract

## Overview

The GitOps Manager contract coordinates multi-signature approval for infrastructure deployment proposals. An admin registers a proposal containing a payload hash, repository, Git reference, and approval threshold; authorized signers then approve it using Ed25519 signatures. Once the required number of approvals is reached, the proposal can be executed, emitting an event that GitHub Actions, GitLab CI, or another GitOps pipeline can consume.

## Public Functions

The contract exposes the following public functions:

- `initialize`: Initializes the contract with an admin address and sets the global nonce to zero.
- `admin`: Returns the current admin address.
- `nonce`: Returns the current global nonce used for replay protection.
- `register_proposal`: Allows the admin to register a proposal with a payload hash, repository, Git reference, and required approval threshold.
- `approve`: Records and verifies an authorized signer's Ed25519 signature for a registered proposal and increments its approval count.
- `execute`: Executes a proposal after its approval count meets the required threshold and emits a GitOps trigger event.
- `get_proposal`: Retrieves a proposal using its payload hash. Returns an error if the proposal does not exist.
- `has_approved`: Checks whether a specific signer has already approved a proposal.

## Proposal Workflow

1. **Initialize:** The admin initializes the contract.
2. **Register:** The admin registers a proposal containing the infrastructure payload hash, repository, Git reference, and approval threshold.
3. **Approve:** Authorized signers submit their Ed25519 signatures for the proposal.
4. **Execute:** Once the approval threshold is met, the proposal can be executed.
5. **Trigger:** The contract emits a GitOps event containing the payload hash, repository, Git reference, nonce, and approval information.

## Security and Validation

The contract includes the following protections:

- Only the admin can register proposals.
- Approval thresholds must be between 1 and 32.
- Duplicate proposals using the same payload hash are rejected.
- Each signer can approve a proposal only once.
- Signatures are verified using Ed25519.
- Signed messages include a domain separator, payload hash, nonce, and signer identity to prevent replay.
- Proposals cannot be approved or executed again after execution.
- Raw infrastructure manifests are not stored on-chain; only their 32-byte payload hashes are stored.

## Build and Test

Run these commands from the contract directory:

```bash
# Build the contract for WebAssembly
cargo build --target wasm32-unknown-unknown --release

# Run contract tests
cargo test
```

## Project Structure

```text
gitops-manager/
├── src/
│   ├── lib.rs
│   └── events.rs
├── Cargo.toml
└── README.md
```
