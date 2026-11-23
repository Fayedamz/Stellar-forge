# Architecture

This folder contains a deployment example for separating Horizon ledger ingestion from its horizontally scalable API tier, plus local validation resources for that topology.

- [`captive-core-scaling.yaml`](./captive-core-scaling.yaml) deploys one isolated StatefulSet-based Horizon ingestion pod with a single Captive Core and a separate, horizontally scalable Horizon API Deployment.
- [`validation/`](./validation/) provides a kind-cluster overlay, standalone Stellar network dependencies, and a verification script for testing the decoupled Captive Core and API topology.
