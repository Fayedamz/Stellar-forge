# examples/troubleshooting

Operational runbook scripts for recovering a Stellar/Horizon deployment from common failure states.
These scripts are **destructive** and require `kubectl` access to the target cluster.
Read each script's header carefully and confirm you understand the impact before running it.

---

## Files

| File | Description |
|------|-------------|
| `force-scp.sh` | Guarded wrapper for split-brain validator recovery. Automates the destructive half of the split-brain runbook: wipes a node's local ledger DB and buckets (`stellar-core new-db`), replays canonical history from a trusted ledger sequence, and optionally plants the `stellar-core --force-scp` flag (Path B) for one supervised restart. Requires `--node`, `--namespace`, and `--trusted-ledger`; use `--force-scp` only when explicitly following Path B in `docs/operations/split-brain-recovery.md`. |
| `reset-captive-core.sh` | Safely wipes the Captive Core ephemeral state directory inside a Horizon pod and restarts the Kubernetes Deployment so Horizon can rebuild ledger state from network history archives. Targets only the Captive Core state directory; it will not delete the Horizon PostgreSQL database, modify PersistentVolumeClaims, or touch validator seed secrets. Accepts `DRY_RUN=true` to preview all actions without executing them. |

---

## When to use this folder

| Symptom | Script |
|---------|--------|
| Validator node has diverged from the canonical chain (split-brain) | `force-scp.sh` |
| Horizon is stuck replaying history or has stale Captive Core state | `reset-captive-core.sh` |

---

## Prerequisites

- `kubectl` configured and pointing at the correct cluster context.
- `jq` installed (`force-scp.sh` only).
- Read and understand the referenced runbook before running either script.

---

> **Warning:** These scripts perform irreversible operations on live infrastructure.
> Always run with `DRY_RUN=true` (or without `--yes`) on a test environment first.
