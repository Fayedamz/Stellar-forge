# Custom Resource Definitions

This folder contains Kubernetes `CustomResourceDefinition` manifests for the Stellar operator's custom resources, covering node lifecycle management and the supporting configuration for traffic shaping, autoscaling, benchmarking, disaster recovery, federation, GitOps, observability, security, and upgrades.

- `stellar-traffic-shaping-crd.yaml`: Defines the namespaced `TrafficShapingPolicy` resource for configuring CNI-level traffic classes, bandwidth limits, priority rules, dynamic scaling, and applied-policy status.
- `stellaraiops-crd.yaml`: Defines the `StellarAIOps` resource for configuring anomaly detection, root-cause analysis, automated remediation, capacity planning, and AI operations status.
- `stellarautoscaler-crd.yaml`: Defines the `StellarAutoscaler` resource for scaling a target node with custom metrics, policy strategies, predictive scaling, cost controls, and scaling status.
- `stellarbenchmark-crd.yaml`: Defines the `StellarBenchmark` resource for running timed load tests against Horizon or Soroban RPC endpoints and storing their results.
- `stellarbenchmarkreport-crd.yaml`: Defines the `BenchmarkReport` resource for holding aggregated performance metrics, configuration snapshots, and per-pod results from a benchmark run.
- `stellardr-crd.yaml`: Defines the backup, restore, disaster recovery drill, and failover resources used to schedule backups, restore to a ledger or timestamp, exercise failover, and manage active-passive clusters.
- `stellarfederation-crd.yaml`: Defines the `StellarFederation` resource for multi-cluster membership, replication settings, traffic routing strategies, health checks, and regional status.
- `stellargitopsconfig-crd.yaml`: Defines the `StellarGitOpsConfig` resource for ArgoCD or FluxCD repository sync, health-based rollback, and encrypted secret injection.
- `stellarnode-crd.yaml`: Defines the core `StellarNode` resource for Validator, Horizon, and SorobanRpc deployments, including networking, storage, scaling, snapshots, upgrades, validator secrets, and node status.
- `stellarobservability-crd.yaml`: Defines the `StellarObservability` resource for configuring tracing, logging, alerting, and anomaly detection for a target Stellar node.
- `stellarsecuritypolicy-crd.yaml`: Defines the `StellarSecurityPolicy` resource for pod security standards, network policies, secret management, compliance frameworks, and security monitoring.
- `stellarupgrade-crd.yaml`: Defines the `StellarUpgrade` resource for canary and progressive rollouts with health validation, traffic ramping, and rollback policies.
