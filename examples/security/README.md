# Security Examples

This folder contains deployment examples and validation scripts for hardening Stellar-K8s with Kubernetes RBAC, Pod Security Admission, network isolation, and external secret management.

- [`audit-rbac.sh`](./audit-rbac.sh) validates the strict RBAC manifest and Pod Security Admission behavior against a live Kubernetes cluster.
- [`externalsecret-aws.yaml`](./externalsecret-aws.yaml) configures External Secrets Operator with AWS Secrets Manager and IRSA for a validator seed Secret.
- [`externalsecret-vault.yaml`](./externalsecret-vault.yaml) configures External Secrets Operator with HashiCorp Vault Kubernetes authentication for a validator seed Secret.
- [`network-policy.yaml`](./network-policy.yaml) defines default-deny and service-specific ingress and egress policies for validator, Horizon, Soroban RPC, operator, and monitoring workloads.
- [`operator-rbac.yaml`](./operator-rbac.yaml) provides a namespaced least-privilege RBAC profile for an operator watching `stellar-mainnet`, with a separate read-only namespace ClusterRole.
- [`strict-rbac.yaml`](./strict-rbac.yaml) provides the recommended strict operator and workload RBAC profile, namespace Pod Security Admission labels, and separate operator and workload ServiceAccounts.
