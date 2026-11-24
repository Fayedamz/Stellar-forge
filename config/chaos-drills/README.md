# Chaos Drills

This folder contains Kubernetes manifests for running scheduled chaos-engineering drills against the Stellar network in a dedicated namespace, with the permissions and shared storage needed by the drill jobs.

- `cronjobs.yaml`: Defines monthly node-failure, network-partition, and storage-failure CronJobs along with the persistent volume claim for their results.
- `namespace.yaml`: Creates the `stellar-chaos` namespace for chaos-engineering resources.
- `rbac.yaml`: Defines the `chaos-driller` ServiceAccount, ClusterRole, and ClusterRoleBinding used to run the drills.
