# Monitoring Examples

This folder contains example Prometheus Operator manifests for collecting and alerting on Stellar-K8s and its workloads.

- [`pod-monitor.yaml`](pod-monitor.yaml) — Defines `PodMonitor` and `ServiceMonitor` resources that scrape metrics from the Stellar operator and other Stellar-K8s pods.
- [`prometheus-rules.yaml`](prometheus-rules.yaml) — Defines Prometheus alert rules and recording rules for Stellar Core, Horizon, Soroban RPC, Kubernetes infrastructure, and performance metrics.
