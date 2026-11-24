# Disaster Recovery examples

This folder contains operational scripts for recovering state and validator
credentials during disaster recovery scenarios for Stellar-K8s.

## Contents

| File | Purpose |
|---|---|
| [`etcd-snapshot.sh`](etcd-snapshot.sh) | Saves, inspects, and restores verified etcd snapshots for the Kubernetes control plane, with explicit confirmation for destructive restores |
| [`restore-script.sh`](restore-script.sh) | Restores a `.tar.gz` database or consensus snapshot into a dsync data directory while backing up existing data |
| [`vault-recovery.sh`](vault-recovery.sh) | Fetches validator seed keys from Vault and applies them as a Kubernetes Secret |
