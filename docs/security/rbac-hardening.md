# Stellar-K8s RBAC and Security Hardening Manual

This manual defines the production least-privilege RBAC profile, `restricted` Kubernetes Pod Security Standards (PSS) enforcement, and external key management service (AWS KMS and HashiCorp Vault) integrations for `stellar-operator` (`stellar-k8s run`) and managed Stellar workload pods.

- **Companion strict RBAC & PSS manifest:** [`examples/security/strict-rbac.yaml`](../../examples/security/strict-rbac.yaml)
- **Companion automated audit script:** [`examples/security/audit-rbac.sh`](../../examples/security/audit-rbac.sh)
- **Companion ESO examples:** [`examples/security/externalsecret-aws.yaml`](../../examples/security/externalsecret-aws.yaml), [`examples/security/externalsecret-vault.yaml`](../../examples/security/externalsecret-vault.yaml)

---

## 1. Reference Topology and Security Invariants

The hardened baseline separates control-plane, workload, and secret-synchronization identities across dedicated namespaces:

| Identity (`namespace/serviceaccount`) | Purpose | Token Automount | Kubernetes API Access |
|---|---|---|---|
| `stellar-system/stellar-operator` | Runs `stellar-k8s run --namespace=stellar-system --watch-namespace=stellar` | `false` on SA (Deployment mounts projected token) | Scoped `Role` in `stellar` & `stellar-system`; minimal read-only `ClusterRole`s |
| `stellar/stellar-node` | Runs Validator, Horizon, and Soroban RPC pods | `false` (projected token only if Vault/IRSA auth needs it) | Empty `Role` (`rules: []`) — zero API access |
| `stellar/external-secrets` | External Secrets Operator (ESO) `SecretStore` auth subject | Explicit projected SA token | Reads AWS SM / Vault; ESO controller writes `Secret`s |

### Mandatory Security Invariants

1. **Zero RBAC wildcards:** No `*` in `apiGroups`, `resources`, or `verbs`.
2. **Operator is read-only to Secrets:** `stellar-operator` holds `get, list, watch` on `secrets` in `stellar` and zero Secret write verbs (`create`, `update`, `patch`, `delete`).
3. **No privilege-escalation or shell access:** `pods/exec`, `pods/attach`, `pods/portforward`, `serviceaccounts/token`, `roles`, `rolebindings`, `clusterroles`, `clusterrolebindings`, and `namespaces` `create` are all denied.
4. **Separate workload and operator identities:** Vault Kubernetes auth roles and AWS IRSA policies bind to `stellar/stellar-node` or `stellar/external-secrets` — never to `stellar-system/stellar-operator`.
5. **Single binding baseline:** Apply `examples/security/strict-rbac.yaml` instead of layering it over the broader Helm-generated `ClusterRole`; Kubernetes RBAC is strictly additive.

---

## 2. Stock-Binary Scope Limitation (`--watch-namespace`)

Passing `--watch-namespace=stellar` scopes the primary `StellarNode` controller and its owned resource watches (`Deployment`, `StatefulSet`, `Service`, `PersistentVolumeClaim`, `PodDisruptionBudget`, `Secret`) in `src/controller/reconciler.rs` (`run_controller()`).

However, `run_operator()` (`src/commands/operator.rs`) and `run_controller()` (`src/controller/reconciler.rs`) unconditionally initialize startup checks and background loops that construct cluster-scoped `Api::all(...)` clients:

| Component | Source Path | Cluster-Scoped API Call |
|---|---|---|
| Startup preflight | `src/preflight.rs` (`check_crd_installed`, `check_namespace_exists`) | `Api::<StellarNode>::all(...).list(limit=1)`; `Api::<Namespace>::all(...).get(ns)` |
| Peer discovery | `src/controller/peer_discovery.rs` (`PeerDiscoveryManager::run`) | `Api::<StellarNode>::all(...).list(...)`; `Service` `get`; writes `ConfigMap/stellar-peers` in `stellar-system` |
| Benchmark controller | `src/controller/benchmark/reconciler.rs` (`run_benchmark_controller`) | `Api::<StellarBenchmark>::all` watch; `Api::<Pod>::all` owned-pod watch |
| Auto-snapshot worker | `src/controller/snapshot_worker.rs` (`run_snapshot_worker`) | `Api::<StellarNode>::all(...).list(...)` every 60 s |
| Finalizer cleanup | `src/controller/finalizers/cleanup.rs` (`run_finalizer_cleanup_controller`) | `Api::<StellarNode>::all(...).list(...)` every 30 s |
| Quorum optimizer | `src/controller/quorum/optimizer.rs` (`QuorumOptimizer::run`) | `Api::<StellarNode>::all(...).list(...)` every 300 s |
| Node drain orchestrator | `src/controller/maintenance/node_drain.rs` (`NodeDrainOrchestrator::run`) | `Api::<Node>::all` watcher; `Api::<Pod>::all(...).list(...)` on cordoned node |
| Hardware metrics helper | `src/infra.rs` (`resolve_stellar_node_infra`) | `Api::<Node>::all(...).get(node_name)` during reconcile when `metrics` feature is active |

Because of these stock background paths, `examples/security/strict-rbac.yaml` creates a dedicated read-only `ClusterRole` (`stellar-operator-observer`) so the stock binary boots and reconciles without `403 Forbidden` log storms, while strictly withholding cluster-wide writes (no benchmark pod creation, no report writes, no `nodes` patching, no `pods/eviction`, and no cluster-wide `secrets` access).

> **Note on optional `AuditWorker`:** If you enable `audit.enabled: true` in `OperatorConfig`, `src/controller/audit_worker.rs` opens a cluster-wide `StellarNode` `watcher` (`list` + `watch`). Keep `audit.enabled: false` in the baseline or review adding `watch` on `stellarnodes` to `stellar-operator-observer`.

---

## 3. Exact `Role` and `ClusterRole` Rules with Justifications

Every rule below maps 1-to-1 to [`examples/security/strict-rbac.yaml`](../../examples/security/strict-rbac.yaml).

### 3.1 Managed Workload Namespace `Role` (`stellar/stellar-operator`)

| `apiGroups` | `resources` | `verbs` | Code-Path Justification |
|---|---|---|---|
| `stellar.org` | `stellarnodes`, `stellarbenchmarks`, `benchmarkreports` | `get, list, watch, update, patch` | Primary `StellarNode` watch/metadata patches (`reconciler.rs`, `audit.rs`, `quorum/analyzer.rs`, `maintenance/compactor.rs`, `finalizers/cleanup.rs`); `create`/`delete` are denied. |
| `stellar.org` | `stellarnodes/status`, `stellarbenchmarks/status`, `benchmarkreports/status` | `get, update, patch` | Publish phase, `conditions`, `readyReplicas`, `ledgerSequence`, and `vaultObservedSecretVersion` (`reconciler.rs`, `kms_secret.rs`, `forensic_snapshot.rs`). |
| `stellar.org` | `stellarnodes/finalizers`, `stellarbenchmarks/finalizers` | `update` | Attach and remove `stellar.org/finalizer` during node lifecycle (`reconciler.rs`, `finalizers/cleanup.rs`). |
| `apps` | `deployments`, `statefulsets` | `get, list, watch, create, update, patch, delete` | Reconcile Horizon/Soroban `Deployment`s, Validator `StatefulSet`s, canary rollouts, and Vault secret-rotation restarts (`resources.rs`, `blue_green_core.rs`, `kms_secret.rs`). |
| `""` | `services`, `configmaps`, `persistentvolumeclaims` | `get, list, watch, create, update, patch, delete` | Manage peer/client `Service`s, node `stellar-core.cfg`/`alerts.yaml` `ConfigMap`s, and data `PVC`s (`resources.rs`, `disk_scaler.rs`). |
| `""` | `pods` | `get, list, watch, update, patch, delete` | Health probes (`health.rs`), stale-node pod restart (`remediation.rs`), Vault annotation checks (`kms_secret.rs`), and peer reload (`peer_discovery.rs`). Direct `create` is withheld; `Deployment`/`StatefulSet` controllers own pod creation. |
| `""` | `pods/log` | `get` | Read pod logs for diagnostics without `pods/exec`. |
| `""` | `pods/ephemeralcontainers` | `get, update, patch` | On-demand break-glass capture in `src/controller/forensic_snapshot.rs`. Remove this rule if break-glass ephemeral containers are prohibited by policy. |
| `""` | `secrets` | `get, list, watch` | Read-only watch (`reconciler.rs`) and `resourceVersion` rotation checks (`secret_watcher.rs`). Zero write verbs in baseline. |
| `networking.k8s.io` | `networkpolicies`, `ingresses` | `get, list, watch, create, update, patch, delete` | Per-node isolation `NetworkPolicy` and optional `Ingress` (`resources.rs`). |
| `policy` | `poddisruptionbudgets` | `get, list, watch, create, update, patch, delete` | Quorum availability protection for multi-replica workloads (`resources.rs`). |
| `autoscaling` | `horizontalpodautoscalers` | `get, list, watch, create, update, patch, delete` | Manage `HPA`s when `spec.autoscaling` is configured (`resources.rs`, `read_pool.rs`). |
| `autoscaling.k8s.io` | `verticalpodautoscalers` | `get, patch, delete` | `reconciler.rs` calls `vpa::delete_vpa()` when `vpaConfig` is absent and `vpa::ensure_vpa()` (`patch`) when enabled. |
| `external-secrets.io` | `externalsecrets` | `get, patch` | Server-side apply `ExternalSecret` CRs and read sync `status.conditions` for `seedSecretSource.externalRef` (`kms_secret.rs`); ESO writes the target `Secret` using its own SA. |
| `postgresql.cnpg.io` | `clusters`, `poolers` | `get, list, watch, create, update, patch, delete` | Optional CloudNativePG database provisioning (`resources.rs`). Remove if CNPG is not installed. |
| `""`, `events.k8s.io` | `events` | `create, patch` | `kube::runtime::events::Recorder` writes `events.k8s.io/v1` events (`reconciler.rs`, `snapshot_worker.rs`, `quorum/optimizer.rs`, `maintenance/compactor.rs`, `maintenance/node_drain.rs`); `finalizers/cleanup.rs` writes core `v1` events. |

### 3.2 Operator Namespace `Role` (`stellar-system/stellar-operator-self`)

| `apiGroups` | `resources` | `verbs` | Code-Path Justification |
|---|---|---|---|
| `apps` | `deployments` | `get, list` | `preflight.rs` (`check_rbac_permissions`) verifies deployment list access in `--namespace`. |
| `""` | `configmaps` | `get, list, watch, patch` | `preflight.rs` check, `feature_flags.rs` (`watch_feature_flags` on `stellar-operator-config`), and `peer_discovery.rs` (`stellar-peers` SSA patch). |
| `coordination.k8s.io` | `leases` | `get, list, create, update, patch` | `preflight.rs` (`check_leader_election_lease`) and leader election on `Lease/stellar-operator-leader` (`commands/operator.rs`). |

### 3.3 Read-Only Runtime Observer `ClusterRole` (`stellar-operator-observer`)

| `apiGroups` | `resources` | `verbs` | Code-Path Justification |
|---|---|---|---|
| `stellar.org` | `stellarnodes` | `list` | `preflight.rs`, `peer_discovery.rs`, `snapshot_worker.rs`, `finalizers/cleanup.rs`, and `quorum/optimizer.rs`. |
| `stellar.org` | `stellarbenchmarks` | `list, watch` | Unconditionally started `run_benchmark_controller()` in `benchmark/reconciler.rs`. |
| `""` | `pods` | `list, watch` | Owned-Pod watch inside `run_benchmark_controller()` (`benchmark/reconciler.rs`) and `NodeDrainOrchestrator::handle_cordoned_node()` (`maintenance/node_drain.rs`). |
| `""` | `services` | `get` | `PeerDiscoveryManager::extract_peer_info()` (`peer_discovery.rs`). |
| `""` | `nodes` | `get, list, watch` | `NodeDrainOrchestrator::run()` cluster Node watch (`maintenance/node_drain.rs`) and `resolve_stellar_node_infra()` Node label lookup (`infra.rs`). Write verbs (`patch`) and `pods/eviction` are denied. |

### 3.4 Pinned Named-Resource `ClusterRole` (`stellar-operator-named-resources`)

| `apiGroups` | `resources` | `resourceNames` | `verbs` | Code-Path Justification |
|---|---|---|---|---|
| `""` | `namespaces` | `["stellar-system", "stellar"]` | `get` | `preflight.rs` reads `stellar-system`; `network_isolation.rs` (`check_namespace_label`) and `pss.rs` read `stellar`. Arbitrary cluster namespaces remain unreadable. |
| `""` | `namespaces` | `["stellar"]` | `patch` | `pss.rs` (`ensure_namespace_pss_labels`) reconciles PSS labels on `stellar`. Namespace `create`/`delete` and patching `kube-system` or `stellar-system` remain denied. |
| `storage.k8s.io` | `storageclasses` | `["local-path", "local-storage"]` | `get` | `resources.rs` (`ensure_pvc`) probes only these two names when `storage.mode == Local` and `storageClass` is unset. Remove if every `StellarNode` specifies `spec.storage.storageClass`. |

### 3.5 Workload Pod `Role` (`stellar/stellar-node-workload`)

`stellar/stellar-node` is bound to `Role/stellar-node-workload` with `rules: []`. Validator, Horizon, and Soroban RPC containers require zero Kubernetes API calls during normal operation.

### 3.6 Explicitly Denied High-Risk Permissions

| Denied Permission | Security Risk Mitigated |
|---|---|
| Any `*` wildcard in `apiGroups`, `resources`, or `verbs` | Prevents unintended privilege expansion when new CRDs or subresources are added. |
| `secrets` `create, update, patch, delete` (baseline) | Prevents operator compromise from overwriting or injecting validator seed keys. |
| Cluster-wide `secrets` `get, list, watch` | Prevents cross-namespace credential harvesting from `kube-system` or sibling tenants. |
| `pods/exec`, `pods/attach`, `pods/portforward` | Prevents arbitrary interactive command execution inside running validator pods. |
| `serviceaccounts/token` `create` | Prevents minting long-lived or arbitrary-audience tokens for other identities. |
| `roles`, `rolebindings`, `clusterroles`, `clusterrolebindings` writes | Prevents RBAC privilege escalation. |
| `namespaces` `create` or unrestricted `namespaces` `patch` | Prevents bypassing namespace isolation or weakening PSS labels on other namespaces. |
| `nodes` `patch` and `pods/eviction` `create` (baseline) | Prevents an operator compromise from cordoning cluster nodes or evicting non-Stellar pods. |

### 3.7 Opt-In Extension Permissions (Outside Strict Baseline)

Do not add these rules unless the corresponding feature is explicitly enabled and reviewed:

1. **`--enable-mtls` or `spec.certManager`:** Requires `secrets` `create, update, patch` (and `cert-manager.io/certificates` `get, list, watch, create, update, patch, delete` for cert-manager) in `stellar` because `src/controller/reconciler.rs` invokes `mtls::ensure_ca` / `mtls::ensure_node_cert`. Scope `secrets` writes with `resourceNames` where possible (`stellar-operator-ca`, `<node>-client-cert`), and note `mtls::ensure_ca` also touches `stellar-system` if `--enable-mtls` is passed to `run_operator()`.
   - *Note for strict non-mTLS clusters:* Keep `--enable-mtls=false` (default) and omit `spec.certManager`. If `mtls::ensure_ca` fails with `403` because `secrets` is read-only in the baseline, verify whether your release requires pre-created `stellar-operator-ca` and `<node>-client-cert` Secrets or a scoped `resourceNames` Secret-write exception.
2. **Scheduled / Pre-Upgrade CSI `VolumeSnapshot`s (`spec.snapshotSchedule`, `spec.strategy` upgrade snapshots):** Add `apiGroups: ["snapshot.storage.k8s.io"]`, `resources: ["volumesnapshots"]`, `verbs: ["get", "list", "watch", "create", "delete"]` in `stellar`.
3. **OCI Snapshots (`spec.ociSnapshot.enabled: true`) or Horizon Blue/Green DB Migration Jobs:** Add `apiGroups: ["batch"]`, `resources: ["jobs"]`, `verbs: ["get", "list", "watch", "create", "delete"]` in `stellar`.
4. **Spot / Preemptible Node Drain (`NODE_NAME` set):** Add `apiGroups: [""]`, `resources: ["nodes"]`, `verbs: ["patch"]` (if cordoning) and `apiGroups: [""]`, `resources: ["pods/eviction"]`, `verbs: ["create"]` in `stellar`.
5. **Service Mesh (`spec.serviceMesh`):** Add `networking.istio.io` (`virtualservices`, `destinationrules`) and `security.istio.io` (`peerauthentications`, `requestauthentications`) with `get, list, watch, create, update, patch, delete` in `stellar`.
6. **REST API Kubernetes Token Authentication:** Add `authentication.k8s.io/tokenreviews` (`create`) and `authorization.k8s.io/subjectaccessreviews` (`create`).

---

## 4. Enforcing `restricted` Pod Security Standards (PSS)

Kubernetes [Pod Security Admission](https://kubernetes.io/docs/concepts/security/pod-security-admission/) validates Pods at admission time using namespace-level labels.

### 4.1 Namespace Policy Configuration

Workload namespaces (`stellar`) **must** enforce `restricted`. The operator namespace (`stellar-system`) uses `audit`/`warn` at `restricted` because the default Helm chart `podSecurityContext` (`charts/stellar-operator/values.yaml`) sets `runAsNonRoot: true`, `runAsUser/Group/fsGroup: 65532`, `allowPrivilegeEscalation: false`, `readOnlyRootFilesystem: true`, and `capabilities.drop: [ALL]`, but does not set `seccompProfile.type: RuntimeDefault` by default.

Apply labels declaratively (already included in `examples/security/strict-rbac.yaml`) or imperatively:

```bash
# Workload namespace: enforce, audit, and warn at restricted
kubectl label namespace stellar \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/enforce-version=latest \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/audit-version=latest \
  pod-security.kubernetes.io/warn=restricted \
  pod-security.kubernetes.io/warn-version=latest \
  stellar.org/node-namespace=true \
  stellar.org/network=mainnet \
  --overwrite

# Operator namespace: audit and warn at restricted
kubectl label namespace stellar-system \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/audit-version=latest \
  pod-security.kubernetes.io/warn=restricted \
  pod-security.kubernetes.io/warn-version=latest \
  --overwrite
```

> To enforce `restricted` on `stellar-system` as well, pass `--set podSecurityContext.seccompProfile.type=RuntimeDefault --set securityContext.seccompProfile.type=RuntimeDefault` in Helm before labelling `pod-security.kubernetes.io/enforce=restricted`.

### 4.2 Pod and Container Security Contexts Enforced by the Operator

`src/controller/pss.rs` and `src/controller/resources.rs` construct every managed workload Pod with:

```yaml
# Pod-level securityContext (src/controller/pss.rs::restricted_pod_security_context)
securityContext:
  runAsNonRoot: true
  runAsUser: 10000
  runAsGroup: 10000
  fsGroup: 10000
  seccompProfile:
    type: RuntimeDefault

# Container-level securityContext (src/controller/pss.rs::restricted_container_security_context)
securityContext:
  allowPrivilegeEscalation: false
  privileged: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  capabilities:
    drop: ["ALL"]
  seccompProfile:
    type: RuntimeDefault
```

The admission webhook (`validate_pss_compliance` in `src/controller/pss.rs`, served by `src/webhook/server.rs`) rejects any `StellarNode` whose `spec.securityContext` or `spec.sidecars[*].securityContext` attempts to set `privileged: true`, `allowPrivilegeEscalation: true`, `runAsNonRoot: false`, `runAsUser: 0`, `seccompProfile.type: Unconfined`, or forbidden capabilities (`NET_ADMIN`, `SYS_ADMIN`, `SYS_PTRACE`, `NET_RAW`, `SYS_MODULE`).

### 4.3 Incompatible Features Under `restricted` PSS

Disable these optional features in strict `restricted` production namespaces:

| Feature | Why Rejected by `restricted` PSS | Compliant Alternative |
|---|---|---|
| `spec.ebpfConfig.enabled: true` | Injects `ebpf-exporter` with `privileged: true` and `hostPath` volumes (`resources.rs`). | Run node-level telemetry in a separate privileged monitoring DaemonSet namespace. |
| `spec.storage.snapshotRef.backupUrl` | `snapshot-restore` init container sets `runAsNonRoot: false` (`resources.rs`). | Use `spec.storage.snapshotRef.volumeSnapshotName` (CSI `VolumeSnapshot`). |
| `spec.forensicSnapshot` annotation trigger | Ephemeral container requests `NET_RAW` / `SYS_PTRACE` (`forensic_snapshot.rs`). | Treat as break-glass only; remove `pods/ephemeralcontainers` in `strict-rbac.yaml` to block it at RBAC level. |

### 4.4 Safe Migration and Admission Verification Commands

When hardening an existing namespace:

```bash
# Step 1: Enable audit and warn without blocking running deployments
kubectl label namespace stellar \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/warn=restricted \
  --overwrite

# Step 2: Test existing Pod/StatefulSet templates via server dry-run
kubectl get statefulset,deploy -n stellar -o json \
  | kubectl apply --dry-run=server -f -

# Step 3: Switch enforce to restricted once warnings are zero
kubectl label namespace stellar \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/enforce-version=latest \
  --overwrite
```

Verify enforcement directly against the API server:

```bash
# Positive test: restricted-compliant pod MUST succeed dry-run admission
kubectl apply --dry-run=server -n stellar -f - <<'YAML'
apiVersion: v1
kind: Pod
metadata:
  name: pss-restricted-verify
spec:
  restartPolicy: Never
  securityContext:
    runAsNonRoot: true
    runAsUser: 10000
    runAsGroup: 10000
    fsGroup: 10000
    seccompProfile:
      type: RuntimeDefault
  containers:
    - name: verify
      image: registry.k8s.io/pause:3.10
      securityContext:
        allowPrivilegeEscalation: false
        readOnlyRootFilesystem: true
        capabilities:
          drop: ["ALL"]
        seccompProfile:
          type: RuntimeDefault
YAML

# Negative test: privileged pod MUST be rejected by PodSecurity Admission
kubectl apply --dry-run=server -n stellar -f - <<'YAML'
apiVersion: v1
kind: Pod
metadata:
  name: pss-privileged-reject
spec:
  containers:
    - name: verify
      image: registry.k8s.io/pause:3.10
      securityContext:
        privileged: true
YAML
# Expected: Error from server (Forbidden): pods "pss-privileged-reject" is forbidden:
# violates PodSecurity "restricted:latest"
```

---

## 5. External Key Management Integration for Validator Seed Keys

Never store raw Stellar validator seeds (`S...`) in Git, Helm values, `StellarNode` manifests, or unencrypted etcd. Choose one of the zero-secret-in-Git workflows below.

### 5.1 AWS KMS & AWS Secrets Manager

Stellar-K8s supports two AWS-backed architectures for validator seed keys:

1. **Pattern A — Direct AWS KMS Envelope Encryption (`keySource: kMS` + `kmsConfig`):** Decrypts ciphertext directly into pod RAM (`emptyDir` with `medium: Memory` at `/keys/validator-seed`) via the `kms-fetcher` init container (`src/controller/resources.rs`), bypassing Kubernetes `Secret` objects completely. Works alongside `SecretPolicy` (`src/crd/secret_policy.rs`) for KMS envelope management and rotation.
2. **Pattern B — AWS Secrets Manager Encrypted with a Customer-Managed AWS KMS Key (ESO `externalRef` or CSI `csiRef`):** Stores the seed in Secrets Manager encrypted with your AWS KMS CMK, using IAM Roles for Service Accounts (IRSA) so neither `stellar-operator` nor Git ever holds credentials.

#### Pattern A: Direct AWS KMS Envelope (`keySource: kMS`)

Note that in `config/crd/stellarnode-crd.yaml`, `KeySource` serializes as `"secret"` or `"kMS"` (`camelCase`).

**1. AWS KMS Customer-Managed Key (CMK) Policy (`kms-key-policy.json`):**

Separate key administration from runtime decryption so the workload pod can only decrypt when the `EncryptionContext` matches its namespace and node:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "EnableKmsAdminControl",
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::111122223333:role/platform-kms-admin"
      },
      "Action": [
        "kms:Create*",
        "kms:Describe*",
        "kms:Enable*",
        "kms:List*",
        "kms:Put*",
        "kms:Update*",
        "kms:Revoke*",
        "kms:Disable*",
        "kms:Get*",
        "kms:Delete*",
        "kms:ScheduleKeyDeletion",
        "kms:CancelKeyDeletion"
      ],
      "Resource": "*"
    },
    {
      "Sid": "AllowValidatorWorkloadDecryptOnly",
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::111122223333:role/stellar-validator-kms-read"
      },
      "Action": [
        "kms:Decrypt",
        "kms:DescribeKey"
      ],
      "Resource": "*",
      "Condition": {
        "StringEquals": {
          "kms:EncryptionContext:stellar:namespace": "stellar",
          "kms:EncryptionContext:stellar:node": "validator-mainnet-0"
        }
      }
    }
  ]
}
```

**2. IAM Policy & EKS IRSA Trust Policy for `stellar/stellar-node`:**

`iam-policy-kms-decrypt.json`:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "DecryptValidatorSeedCmkOnly",
      "Effect": "Allow",
      "Action": [
        "kms:Decrypt",
        "kms:DescribeKey"
      ],
      "Resource": "arn:aws:kms:us-east-1:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab",
      "Condition": {
        "StringEquals": {
          "kms:EncryptionContext:stellar:namespace": "stellar"
        }
      }
    }
  ]
}
```

`irsa-trust-stellar-node.json`:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::111122223333:oidc-provider/oidc.eks.us-east-1.amazonaws.com/id/EXAMPLED539D4633E53DE1B716D3041E"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "oidc.eks.us-east-1.amazonaws.com/id/EXAMPLED539D4633E53DE1B716D3041E:sub": "system:serviceaccount:stellar:stellar-node",
          "oidc.eks.us-east-1.amazonaws.com/id/EXAMPLED539D4633E53DE1B716D3041E:aud": "sts.amazonaws.com"
        }
      }
    }
  ]
}
```

Create the role, annotate the workload `ServiceAccount`, and deploy `StellarNode` + `SecretPolicy`:

```bash
aws kms create-key \
  --description "Stellar mainnet validator-0 seed envelope CMK" \
  --policy file://kms-key-policy.json \
  --region us-east-1

aws iam create-role \
  --role-name stellar-validator-kms-read \
  --assume-role-policy-document file://irsa-trust-stellar-node.json

aws iam put-role-policy \
  --role-name stellar-validator-kms-read \
  --policy-name stellar-kms-decrypt \
  --policy-document file://iam-policy-kms-decrypt.json

kubectl annotate serviceaccount stellar-node -n stellar \
  eks.amazonaws.com/role-arn="arn:aws:iam::111122223333:role/stellar-validator-kms-read" \
  eks.amazonaws.com/sts-regional-endpoints="true" \
  --overwrite
```

```yaml
# validator-kms.yaml
apiVersion: stellar.org/v1alpha1
kind: StellarNode
metadata:
  name: validator-mainnet-0
  namespace: stellar
spec:
  nodeType: Validator
  network: mainnet
  version: "v21.0.0"
  replicas: 1
  validatorConfig:
    keySource: kMS
    kmsConfig:
      provider: aws
      keyId: "arn:aws:kms:us-east-1:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab"
      region: us-east-1
      fetcherImage: "ghcr.io/stellar/kms-fetcher:v1.0.0"
    enableHistoryArchive: true
  storage:
    storageClass: gp3-encrypted
    size: 500Gi
---
apiVersion: stellar.org/v1alpha1
kind: SecretPolicy
metadata:
  name: validator-mainnet-0-seed-policy
  namespace: stellar
spec:
  secretName: validator-mainnet-0-seed
  provider: aws
  aws:
    keyId: "arn:aws:kms:us-east-1:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab"
    region: us-east-1
    roleArn: "arn:aws:iam::111122223333:role/stellar-validator-kms-read"
  rotation:
    interval: "720h"
    zeroDowntime: true
    versionRetention: 5
  audit:
    enabled: true
    anomalyDetection: true
  encryptInTransit: true
```

#### Pattern B: AWS Secrets Manager with CMK Encryption (ESO `externalRef` or CSI `csiRef`)

When using External Secrets Operator (`examples/security/externalsecret-aws.yaml`), bind IRSA to `system:serviceaccount:stellar:external-secrets` (never `stellar-operator`) and restrict `kms:Decrypt` via `kms:ViaService` (`iam-policy-eso-aws-sm-kms.json`):

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "ReadValidatorSecretsManagerEntries",
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue",
        "secretsmanager:DescribeSecret"
      ],
      "Resource": "arn:aws:secretsmanager:us-east-1:111122223333:secret:stellar/prod/validator-*"
    },
    {
      "Sid": "DecryptViaSecretsManagerOnly",
      "Effect": "Allow",
      "Action": "kms:Decrypt",
      "Resource": "arn:aws:kms:us-east-1:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab",
      "Condition": {
        "StringEquals": {
          "kms:ViaService": "secretsmanager.us-east-1.amazonaws.com"
        }
      }
    }
  ]
}
```

Deploy the secret encrypted by your CMK and wire `seedSecretSource.externalRef`:

```bash
aws secretsmanager create-secret \
  --name stellar/prod/validator-0 \
  --kms-key-id "arn:aws:kms:us-east-1:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab" \
  --secret-string '{"seed":"SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"}' \
  --region us-east-1

kubectl apply -f examples/security/externalsecret-aws.yaml
kubectl get externalsecret validator-0-seed -n stellar
```

```yaml
# StellarNode using ESO (operator reconciles ExternalSecret; ESO writes Secret)
spec:
  validatorConfig:
    seedSecretSource:
      externalRef:
        name: validator-0-seed
        secretStoreRef:
          name: aws-secrets-manager
          kind: SecretStore
        remoteKey: stellar/prod/validator-0
        remoteProperty: seed
        refreshInterval: 1h
```

To avoid storing any `Secret` in etcd at all, use the **Secrets Store CSI Driver** (`seedSecretSource.csiRef`) with `SecretProviderClass` (`aws` provider) mounted to `/mnt/secrets/validator/seed` (`STELLAR_SEED_FILE`):

```yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: stellar-validator-0-aws-spc
  namespace: stellar
spec:
  provider: aws
  parameters:
    objects: |
      - objectName: "stellar/prod/validator-0"
        objectType: "secretsmanager"
        jmesPath:
          - path: seed
            objectAlias: seed
---
# Reference from StellarNode (requires IRSA bound to stellar/stellar-node)
# spec:
#   validatorConfig:
#     seedSecretSource:
#       csiRef:
#         secretProviderClassName: stellar-validator-0-aws-spc
#         mountPath: /mnt/secrets/validator
#         seedFileName: seed
```

Verify AWS KMS and Secrets Manager access in CloudTrail:

```bash
aws cloudtrail lookup-events \
  --lookup-attributes AttributeKey=EventName,AttributeValue=Decrypt \
  --region us-east-1 --max-results 10 \
  | jq '.Events[] | {time: .EventTime, user: .Username, source: .EventSource}'
```

---

### 5.2 HashiCorp Vault Integration

Stellar-K8s supports two HashiCorp Vault workflows where `stellar-operator` never reads or writes the plaintext seed:

1. **Pattern A — Vault Agent Injector (`seedSecretSource.vaultRef`):** `src/controller/kms_secret.rs` (`vault_agent_annotations`) emits `vault.hashicorp.com/*` annotations on the validator pod template; the Vault Agent sidecar authenticates as `stellar/stellar-node` and renders `/vault/secrets/stellar-seed` (`STELLAR_SEED_FILE`) into an in-memory tmpfs mount. Setting `restartOnSecretRotation: true` enables `reconcile_vault_secret_rotation()` to roll the `StatefulSet` when Vault updates the secret-version annotation.
2. **Pattern B — External Secrets Operator (`seedSecretSource.externalRef`):** ESO authenticates to Vault as `stellar/external-secrets` using [`examples/security/externalsecret-vault.yaml`](../../examples/security/externalsecret-vault.yaml) and syncs the Kubernetes `Secret`.

#### Pattern A: Vault Agent Injector (`seedSecretSource.vaultRef`)

**1. Least-Privilege Vault KV-v2 Policy (`stellar-validator.hcl`):**

Grant `read` only to a single validator's KV-v2 data path and explicitly `deny` metadata enumeration:

```hcl
# stellar-validator.hcl
path "secret/data/stellar/validators/validator-0" {
  capabilities = ["read"]
}

path "secret/metadata/stellar/validators/*" {
  capabilities = ["deny"]
}
```

**2. Enable Kubernetes Auth, Write Policy, and Bind Workload `ServiceAccount`:**

```bash
vault audit enable file file_path=/var/log/vault/audit.log
vault auth enable kubernetes

# Configure Kubernetes auth using the cluster's API server CA and TokenReview
vault write auth/kubernetes/config \
  kubernetes_host="https://kubernetes.default.svc:443"

vault policy write stellar-validator stellar-validator.hcl

# Bind strictly to stellar/stellar-node (the workload SA, NEVER stellar-operator)
vault write auth/kubernetes/role/stellar-validator \
  bound_service_account_names=stellar-node \
  bound_service_account_namespaces=stellar \
  policies=stellar-validator \
  ttl=1h \
  max_ttl=4h

# Store the seed out-of-band (never via shell command-line history in production)
vault kv put secret/stellar/validators/validator-0 seed=@/secure/ceremony/validator-0.seed
```

> **JWT Audience Warning:** Do not blindly copy `audience` fields across Vault clusters. Match `audience` in `auth/kubernetes/role/stellar-validator` (or `SecretStore`) to the actual audience configured on your Kubernetes API server's projected ServiceAccount tokens.

**3. `StellarNode` Manifest Using `vaultRef`:**

```yaml
apiVersion: stellar.org/v1alpha1
kind: StellarNode
metadata:
  name: validator-0
  namespace: stellar
spec:
  nodeType: Validator
  network: mainnet
  version: "v21.0.0"
  replicas: 1
  validatorConfig:
    seedSecretSource:
      vaultRef:
        role: stellar-validator
        secretPath: secret/data/stellar/validators/validator-0
        secretKey: seed
        secretFileName: stellar-seed
        restartOnSecretRotation: true
    enableHistoryArchive: true
  storage:
    storageClass: gp3-encrypted
    size: 500Gi
```

**4. Verify Vault Agent Injection and Audit Logs:**

```bash
# Confirm STELLAR_SEED_FILE points to the tmpfs Vault render path
kubectl get pod validator-0-0 -n stellar \
  -o jsonpath='{.spec.containers[?(@.name=="stellar-node")].env[?(@.name=="STELLAR_SEED_FILE")].value}'
# Expected: /vault/secrets/stellar-seed

# Confirm Vault audit log records only stellar/stellar-node reading the seed path
grep '"path":"secret/data/stellar/validators/validator-0"' /var/log/vault/audit.log \
  | jq '{time: .time, sa: .auth.metadata.service_account_name, ns: .auth.metadata.service_account_namespace}'
```

#### Pattern B: Vault via External Secrets Operator (`seedSecretSource.externalRef`)

Bind a separate Vault Kubernetes auth role to `stellar/external-secrets` (matching [`examples/security/externalsecret-vault.yaml`](../../examples/security/externalsecret-vault.yaml)):

```bash
vault write auth/kubernetes/role/stellar-mainnet-validator \
  bound_service_account_names=external-secrets \
  bound_service_account_namespaces=stellar \
  policies=stellar-validator \
  ttl=1h

kubectl apply -f examples/security/externalsecret-vault.yaml
kubectl get externalsecret validator-0-seed -n stellar
# Expected: READY=True, STATUS=SecretSynced
```

Zero-downtime rotation and RBAC verification:

```bash
# Rotate seed in Vault and trigger immediate ESO resync
vault kv put secret/stellar/validators/validator-0 seed=@/secure/ceremony/validator-0-new.seed
kubectl annotate externalsecret validator-0-seed -n stellar force-sync="$(date +%s)" --overwrite

# Verify operator remains read-only to Kubernetes Secrets
kubectl auth can-i get secrets -n stellar \
  --as=system:serviceaccount:stellar-system:stellar-operator     # Expected: yes
kubectl auth can-i patch secrets -n stellar \
  --as=system:serviceaccount:stellar-system:stellar-operator     # Expected: no
```

---

## 6. Applying and Validating on an Active Operator Cluster

### 6.1 Apply Strict RBAC Profile (Replacing Broad Bindings)

Remove any existing Helm-generated cluster-wide `ClusterRoleBinding` before applying the strict profile:

```bash
# 1. Inspect and remove any pre-existing broad operator bindings
kubectl get clusterrolebinding | grep stellar-operator
# kubectl delete clusterrolebinding <legacy-broad-binding>

# 2. Apply strict least-privilege RBAC, PSS labels, and workload identities
kubectl apply -f examples/security/strict-rbac.yaml
```

Deploy or restart `stellar-operator` using `serviceAccountName: stellar-operator` in namespace `stellar-system` with `--namespace=stellar-system --watch-namespace=stellar` and `--enable-mtls=false` (or Helm values `operator.watchNamespace=stellar`, `rbac.create=false`, `serviceAccount.create=false`, `serviceAccount.name=stellar-operator`, `mtls.enabled=false`).

### 6.2 Automated RBAC & PSS Audit Script

Run the companion validation script against the cluster:

```bash
bash examples/security/audit-rbac.sh
```

Override environment variables if your deployment uses custom namespace or ServiceAccount names:

```bash
OPERATOR_NAMESPACE=my-operator \
MANAGED_NAMESPACE=my-stellar \
SERVICE_ACCOUNT=my-operator \
DEPLOYMENT_NAME=stellar-operator \
bash examples/security/audit-rbac.sh
```

### 6.3 End-to-End Reconciliation Verification Gate

Verify on a live test cluster (with matching `stellar.org/network` label on namespace `stellar`, e.g. `testnet` for testnet samples) that all reconciliation loops and background tasks run without `403 Forbidden` or permission errors:

```bash
# 1. Align namespace network label if testing with a testnet sample CR
kubectl label namespace stellar stellar.org/network=testnet --overwrite

# 2. Deploy a compliant StellarNode in the enforced namespace
kubectl apply -n stellar -f config/samples/minimal-validator.yaml

# 3. Watch reconciliation reach Ready / Running and verify child resources
kubectl get stellarnodes -n stellar -w
kubectl get statefulset,deploy,svc,cm,pvc,networkpolicy,pdb -n stellar
kubectl get events -n stellar --sort-by=.lastTimestamp

# 4. Scan operator logs across >=10 minutes (covers controller + 30s/60s/300s background loops)
kubectl logs -n stellar-system deploy/stellar-operator --since=10m \
  | grep -Eqi 'forbidden|permission denied|cannot (get|list|watch|create|update|patch|delete)' \
  && echo "RBAC FAILURE" || echo "PASS: zero RBAC errors"

# 5. Run full automated audit gate
bash examples/security/audit-rbac.sh
```

---

## 7. Security Audit Rejection Criteria (PR Review Gate)

A security audit of RBAC or hardening changes **must reject** any profile that introduces:

- Any RBAC wildcard (`*`) in `apiGroups`, `resources`, or `verbs`.
- Secret mutation verbs (`create`, `update`, `patch`, `delete`) in the baseline operator `Role` or `ClusterRole`.
- Cross-namespace `secrets` read permissions.
- `pods/exec`, `pods/attach`, `pods/portforward`, or `serviceaccounts/token` permissions.
- RBAC role or binding mutation (`roles`, `rolebindings`, `clusterroles`, `clusterrolebindings`).
- Cluster-scoped `namespaces` `create`/`delete` or unrestricted `namespaces` `patch` (missing `resourceNames`).
- Cluster-wide `nodes` `patch`, `pods/eviction` `create`, or cluster-wide workload/benchmark writes.
- Weakening `pod-security.kubernetes.io/enforce: restricted` on workload namespaces without an approved break-glass exception.

This permission map is grounded directly in `charts/stellar-operator/templates/rbac.yaml`, `src/commands/operator.rs`, `src/preflight.rs`, `src/controller/reconciler.rs`, `src/controller/resources.rs`, `src/controller/pss.rs`, `src/controller/network_isolation.rs`, `src/controller/kms_secret.rs`, `src/controller/peer_discovery.rs`, `src/controller/benchmark/reconciler.rs`, `src/controller/snapshot_worker.rs`, `src/controller/finalizers/cleanup.rs`, `src/controller/quorum/optimizer.rs`, `src/controller/maintenance/node_drain.rs`, and `src/infra.rs`. Re-audit this manual whenever those files change.
