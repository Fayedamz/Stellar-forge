# Kubernetes manifests

This directory contains the Stellar-K8s Operator's Operator Lifecycle Manager (OLM) ClusterServiceVersion base and OPA Gatekeeper policies for `StellarNode` resources. The `kustomization.yaml` file includes only the OLM base; the Gatekeeper policy manifests are separate and are not included by that Kustomize resource list.

## Top-level entries

- `bases/` — OLM ClusterServiceVersion base used to describe the Stellar-K8s Operator for bundle generation.
- `gatekeeper/` — Gatekeeper constraint templates, constraint instances, and configuration for Kubernetes admission policies.
- `kustomization.yaml` — Kustomize resource list referencing `bases/stellar-operator.clusterserviceversion.yaml`.

## `bases/`

- `stellar-operator.clusterserviceversion.yaml` — Defines the operator's OLM ClusterServiceVersion metadata, version, and supported installation modes.

## `gatekeeper/`

- `approved-registries-constraint.yaml` — Sets allowed registry prefixes for the `StellarNode` image version field.
- `approved-registries-template.yaml` — Defines Rego validation of `StellarNode.spec.version` against the allowed registry prefixes.
- `default-security-policy-library.yaml` — Stores sample Gatekeeper policies and `StellarNode` CEL rules in a ConfigMap.
- `gatekeeper-config.yaml` — Configures Gatekeeper to exclude the `kube-system` and `gatekeeper-system` namespaces from policy matching.
- `required-labels-constraint.yaml` — Lists the metadata labels required on `StellarNode` resources.
- `required-labels-template.yaml` — Defines a Gatekeeper Rego check for missing required `StellarNode` metadata labels.
- `resource-limits-constraint.yaml` — Sets maximum CPU and memory limits for `StellarNode` resources.
- `resource-limits-template.yaml` — Defines Gatekeeper Rego checks for required resource limits and configured CPU/memory maxima.
