# Stellar-K8s operator examples

This folder contains small Rust examples showing how to implement Stellar-K8s SDK operators and hooks for Kubernetes reconciliation.

- `log_shipper.rs` — Defines a sidecar injector that adds a Fluent Bit log-shipper container to reconciled workloads.
- `metrics_exporter.rs` — Defines a reconcile hook that logs a pre-reconcile metrics export event.
- `registry_validator.rs` — Tests registry admission checks by validating a sample StellarRegistry resource and image admission policy.
