# GitOps Examples

This folder contains a complete Argo CD and Kustomize layout for running Stellar-K8s with GitOps: a shared base of `StellarNode` resources, one overlay per Stellar network, and the Argo CD Applications that sync them. Testnet and Futurenet sync automatically; Mainnet syncs manually. See [GitOps with Argo CD](../../docs/operations/gitops-argocd.md) for the full workflow.

- [`argocd/`](./argocd/) holds the Argo CD Applications: an app-of-apps root to bootstrap the pipeline and one Application per network overlay. Its [README](./argocd/README.md) covers bootstrap and promotion to Mainnet.
- [`base/`](./base/) is the Kustomize base shared by every environment: the `stellar` namespace plus validator, Horizon and Soroban RPC `StellarNode` resources, with the network, version, storage and resource values left for overlays to set.
- [`overlays/`](./overlays/) has one Kustomize overlay per network (`testnet/`, `futurenet/`, `mainnet/`). Each sets its own namespace, name prefix and network label and patches the base `StellarNode` resources for that network (Futurenet keeps the base Horizon settings).
