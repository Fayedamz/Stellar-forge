# Example Wasm Admission Plugins

This directory contains example Rust WebAssembly (Wasm) admission webhook plugins for validating `StellarNode` Kubernetes resources in Stellar-forge. Each subdirectory is a standalone Rust crate that compiles to a `.wasm` module for use with the operator's Wasm admission webhook (see [docs/wasm-webhook.md](../../docs/wasm-webhook.md)).

## Contents

- [`example-validator/`](./example-validator/) — Example `wasm32-wasi` validation plugin crate demonstrating `StellarNode` resource validation for the Wasm webhook.
- [`image-registry-validator/`](./image-registry-validator/) — Example `wasm32-unknown-unknown` admission plugin that requires `StellarNode` images to come from approved registries (`docker.io/stellar/*`, `ghcr.io/stellar/*`, `gcr.io/stellar-project/*`).
- [`registry-policy/`](./registry-policy/) — Example `wasm32-unknown-unknown` admission plugin that enforces `StellarNode` `spec.version` values only reference approved Docker registries (`docker.io/stellar/`, `ghcr.io/stellar/`, `gcr.io/stellar-project/`, `registry.k8s.io/stellar/`).
