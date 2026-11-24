# Debug Examples

This directory contains temporary Kubernetes Pod manifests for inspecting, troubleshooting, and recovering corrupted PersistentVolumeClaims (PVCs) and node databases in Stellar-K8s deployments. Before applying any manifest here, suspend the affected `StellarNode` and ensure its workload pod is terminated to prevent concurrent-write corruption, and delete the debug pod immediately after recovery.

## Files

- `maintenance-pod.yaml` — Multi-container maintenance debug Pod (`stellar-maintenance-debug`) with `stellar-core`, `postgres`, and `ubuntu` tool containers for PVC corruption recovery (see `docs/operations/pvc-troubleshooting.md`).
- `repair-pod.yaml` — Lightweight root-privileged Alpine debug shell Pod (`db-repair`) that mounts a node's data PVC read-write at `/repair-data` for inspecting and repairing corrupted SQLite or PostgreSQL databases (see `docs/operations/storage-repair.md`).
