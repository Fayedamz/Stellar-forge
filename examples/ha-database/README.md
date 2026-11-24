# HA Database Example

This folder contains a highly available PostgreSQL setup for Horizon. Patroni runs a three-member PostgreSQL cluster with Kubernetes-based leader tracking, PgBouncer pools Horizon connections to the current primary, and Horizon is configured to use the pooler through the Kubernetes operator.

## Contents

- `Dockerfile` - Builds the PostgreSQL 17 and Patroni 4.1.5 image used by the database cluster.
- `horizon.yaml` - Defines the three-replica Horizon API tier and its connection to the PgBouncer service and database credentials.
- `patroni-config.yaml` - Defines the Patroni RBAC, Kubernetes DCS configuration, PostgreSQL settings, primary and replica services, StatefulSet, and PodDisruptionBudget.
- `pgbouncer.yaml` - Defines the PgBouncer transaction-pooling configuration, two-replica deployment, service, and PodDisruptionBudget in front of Patroni.
- `validation/` - Contains kind and Kubernetes manifests plus a script for testing planned switchover, primary pod loss, and primary-node partition behavior.
