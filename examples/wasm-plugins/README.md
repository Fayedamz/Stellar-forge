# Wasm Plugins Examples

This folder contains example Rust WebAssembly (`wasm32-unknown-unknown`) admission-validation plugins for the Stellar Kubernetes Operator, demonstrating how to implement custom `StellarNode` CREATE and UPDATE policies against the operator's Wasmtime host ABI.

## Contents

- [`hello-world/`](hello-world/) — Beginner tutorial Wasm validation plugin that requires a `cost-center` label on every `StellarNode`, enforces at least 3 replicas on Mainnet, and emits a warning when the memory limit is below `512Mi`.
- [`registry-enforcer/`](registry-enforcer/) — Enterprise Wasm validation plugin that enforces an approved image-registry prefix allow-list on `spec.version` for every `StellarNode` CREATE and UPDATE request.
