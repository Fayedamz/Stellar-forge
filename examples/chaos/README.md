# Chaos

This folder contains Chaos Mesh experiments for testing Stellar-K8s resilience and operator recovery under network and pod failures.

- [`network-partition.yaml`](./network-partition.yaml) isolates one validator pod from its quorum peers bidirectionally for 120 seconds using a Chaos Mesh `NetworkChaos`.
- [`pod-kill-horizon.yaml`](./pod-kill-horizon.yaml) repeatedly kills syncing Horizon pods for 60 seconds using a Chaos Mesh `PodChaos` to verify automatic operator recovery.
