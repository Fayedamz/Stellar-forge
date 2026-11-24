# Blue-Green Deployment Manifests

This directory contains Kubernetes manifests for zero-downtime Stellar Core validator upgrades using a blue/green deployment strategy, where two independent `StatefulSet` deployments (`blue` and `green`) alternate as the active slot behind a health-gated `Service` selector switch with automated rollback on failure.

| File | Purpose |
|------|---------|
| `blue-green-deployment.yaml` | Defines the `stellar-validator-blue` and `stellar-validator-green` `StatefulSet`s, the active `stellar-validator` `Service`, blue/green headless `Service`s, and the `stellar-validator-active` `PodDisruptionBudget` |
| `health-check-traffic-switch.yaml` | Defines the `stellar-bg-health-config` `ConfigMap` (health-gate thresholds and rollback timings) and the `stellar-blue-green-alerts` `PrometheusRule` (active-color metric and rollout/rollback alerts) |

## Usage

```bash
# Apply the blue-green base manifests (resources target the stellar-system namespace)
kubectl apply -f config/blue-green/
```

See the [Blue-Green Upgrade Runbook](../../docs/operations/blue-green-upgrade.md) for deployment phases, upgrade triggers, rollout monitoring, and manual rollback procedures.
