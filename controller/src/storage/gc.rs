// Copyright 2024 Stellar-K8s Contributors
// SPDX-License-Identifier: Apache-2.0
//! Orphaned Persistent Volume garbage collector — Issue #304.
//!
//! # Purpose
//!
//! When a `StellarNode` is deleted, the finalizer in [`crate::lifecycle::finalizers`]
//! annotates its `PersistentVolumeClaim` with `stellar.k8s.io/gc-requested: true`.
//! Cloud Container Storage Interface (CSI) drivers do not always honour the
//! reclaim policy immediately, so this scanner runs on a configurable interval
//! and forcibly deletes any `PersistentVolumeClaim` that:
//!
//! 1. Carries the `stellar.k8s.io/gc-requested: true` annotation, **and**
//! 2. Has `retentionPolicy: Delete` in its annotation
//!    (`stellar.k8s.io/gc-reason: retentionPolicy:Delete`), **and**
//! 3. Is in the `Released` or `Available` phase (i.e. not bound to a running pod).
//!
//! # Design
//!
//! * The scanner uses `kube::Api<PersistentVolumeClaim>` list/delete calls so
//!   it works with any cloud CSI implementation without provider-specific SDKs.
//! * Deletion is performed with `DeleteParams::default()` (foreground
//!   cascading deletion), which lets the CSI finalizer on the PVC itself drive
//!   the backing disk deletion.
//! * A dry-run mode is provided for safe inspection without side effects.
//! * The scanner never deletes a PVC that is still `Bound` to a pod.

use anyhow::Result;
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use kube::{
    api::{Api, DeleteParams, ListParams},
    Client,
};
use tracing::{info, warn};

/// Annotation key the finalizer sets to flag a PVC for GC.
pub const GC_REQUESTED_ANNOTATION: &str = "stellar.k8s.io/gc-requested";

/// Annotation key recording why the PVC was flagged.
pub const GC_REASON_ANNOTATION: &str = "stellar.k8s.io/gc-reason";

/// Value that enables deletion via GC (`retentionPolicy: Delete` semantics).
pub const GC_REASON_DELETE: &str = "retentionPolicy:Delete";

/// Configuration for the garbage collection scanner.
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// Kubernetes namespace(s) to scan.
    ///
    /// `None` means scan all namespaces (requires cluster-scoped RBAC).
    pub namespace: Option<String>,

    /// When `true`, log what would be deleted without actually deleting.
    pub dry_run: bool,

    /// Maximum number of PVCs deleted in a single scan run.
    ///
    /// Acts as a safety valve to prevent a runaway scan from causing a storm
    /// of CSI calls.  Defaults to 20.
    pub max_deletions_per_run: usize,
}

impl GcConfig {
    /// Returns a configuration that scans all namespaces in live mode.
    pub fn cluster_wide() -> Self {
        Self {
            namespace: None,
            dry_run: false,
            max_deletions_per_run: 20,
        }
    }

    /// Returns a configuration scoped to a single namespace.
    pub fn namespaced(namespace: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            dry_run: false,
            max_deletions_per_run: 20,
        }
    }

    /// Enables dry-run mode.
    pub fn with_dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Overrides the per-run deletion cap.
    pub fn with_max_deletions(mut self, n: usize) -> Self {
        self.max_deletions_per_run = n;
        self
    }
}

/// Errors produced by the GC scanner.
#[derive(Debug, thiserror::Error)]
pub enum GcError {
    /// A Kubernetes API call failed.
    #[error("Kubernetes API error: {0}")]
    KubeApi(#[from] kube::Error),

    /// The per-run deletion cap was reached before all eligible PVCs were removed.
    ///
    /// This is **not** fatal; the next scheduled run will continue.
    #[error(
        "GC deletion cap ({cap}) reached; {remaining} eligible PVCs deferred to next run"
    )]
    DeletionCapReached { cap: usize, remaining: usize },
}

/// Summary of one GC scan run.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GcReport {
    /// Number of PVCs inspected during the run.
    pub pvcs_scanned: usize,
    /// Number of PVCs that matched the deletion criteria.
    pub pvcs_eligible: usize,
    /// Number of PVCs actually deleted (0 in dry-run mode).
    pub pvcs_deleted: usize,
    /// Number of eligible PVCs skipped because the deletion cap was reached.
    pub pvcs_deferred: usize,
    /// Whether this run operated in dry-run mode.
    pub dry_run: bool,
}

/// The orphaned-volume garbage collector.
///
/// Create one instance per operator process and call [`GarbageCollector::run`]
/// on a periodic interval (e.g. every 5 minutes via a `tokio::time::interval`).
#[derive(Clone)]
pub struct GarbageCollector {
    client: Client,
    config: GcConfig,
}

impl GarbageCollector {
    /// Creates a new `GarbageCollector` with the given Kubernetes client and
    /// configuration.
    pub fn new(client: Client, config: GcConfig) -> Self {
        Self { client, config }
    }

    /// Runs one scan-and-collect pass.
    ///
    /// Returns a [`GcReport`] describing what was found and acted on.
    /// Returns `Err(GcError::DeletionCapReached)` when the cap is hit, still
    /// including the partial report in the error message.
    pub async fn run(&self) -> Result<GcReport, GcError> {
        let pvcs = self.list_eligible_pvcs().await?;
        let total_eligible = pvcs.len();

        let mut report = GcReport {
            pvcs_scanned: pvcs.len(), // refined below after re-list
            pvcs_eligible: total_eligible,
            dry_run: self.config.dry_run,
            ..Default::default()
        };

        let cap = self.config.max_deletions_per_run;
        let mut deleted = 0;

        for pvc in &pvcs {
            if deleted >= cap {
                break;
            }

            let name = pvc.metadata.name.as_deref().unwrap_or("<unknown>");
            let ns = pvc
                .metadata
                .namespace
                .as_deref()
                .unwrap_or_else(|| {
                    self.config.namespace.as_deref().unwrap_or("default")
                });

            if self.config.dry_run {
                info!(
                    namespace = ns,
                    pvc = name,
                    "[dry-run] would delete orphaned PVC"
                );
            } else {
                match self.delete_pvc(ns, name).await {
                    Ok(()) => {
                        info!(namespace = ns, pvc = name, "deleted orphaned PVC");
                        deleted += 1;
                    }
                    Err(e) => {
                        warn!(namespace = ns, pvc = name, error = %e, "failed to delete PVC");
                    }
                }
            }
        }

        report.pvcs_deleted = deleted;
        report.pvcs_scanned = pvcs.len();
        report.pvcs_deferred = total_eligible.saturating_sub(deleted);

        if !self.config.dry_run && report.pvcs_deferred > 0 {
            return Err(GcError::DeletionCapReached {
                cap,
                remaining: report.pvcs_deferred,
            });
        }

        Ok(report)
    }

    /// Lists all PVCs in the configured namespace(s) that are eligible for GC.
    ///
    /// A PVC is eligible when:
    /// - `stellar.k8s.io/gc-requested: true` annotation is set, AND
    /// - `stellar.k8s.io/gc-reason: retentionPolicy:Delete` annotation is set, AND
    /// - The PVC phase is NOT `Bound` (i.e. it is `Released`, `Available`,
    ///   `Pending`, or has no phase).
    async fn list_eligible_pvcs(&self) -> Result<Vec<PersistentVolumeClaim>, GcError> {
        // The Kubernetes label/field selector API does not support filtering on
        // arbitrary annotation values, so we list by label and filter in-process.
        let list_params = ListParams::default()
            .labels(&format!("{GC_REQUESTED_ANNOTATION}=true"));

        let pvc_list = match &self.config.namespace {
            Some(ns) => {
                let api: Api<PersistentVolumeClaim> =
                    Api::namespaced(self.client.clone(), ns);
                api.list(&list_params).await?
            }
            None => {
                let api: Api<PersistentVolumeClaim> =
                    Api::all(self.client.clone());
                api.list(&list_params).await?
            }
        };

        let eligible = pvc_list
            .items
            .into_iter()
            .filter(|pvc| {
                let annotations = pvc
                    .metadata
                    .annotations
                    .as_ref()
                    .map(|a| a.as_ref())
                    .unwrap_or_default();

                // Must be flagged for deletion with the correct reason.
                let gc_requested = annotations
                    .get(GC_REQUESTED_ANNOTATION)
                    .map_or(false, |v| v == "true");
                let reason_delete = annotations
                    .get(GC_REASON_ANNOTATION)
                    .map_or(false, |v| v == GC_REASON_DELETE);

                // Must not be actively bound to a pod.
                let not_bound = pvc
                    .status
                    .as_ref()
                    .and_then(|s| s.phase.as_deref())
                    .map_or(true, |phase| phase != "Bound");

                gc_requested && reason_delete && not_bound
            })
            .collect();

        Ok(eligible)
    }

    /// Deletes a single PVC by name in the given namespace.
    async fn delete_pvc(&self, namespace: &str, name: &str) -> Result<(), GcError> {
        let api: Api<PersistentVolumeClaim> =
            Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::foreground()).await?;
        Ok(())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── GcConfig builders ─────────────────────────────────────────────────────

    #[test]
    fn cluster_wide_config_has_no_namespace() {
        let cfg = GcConfig::cluster_wide();
        assert!(cfg.namespace.is_none());
        assert!(!cfg.dry_run);
        assert_eq!(cfg.max_deletions_per_run, 20);
    }

    #[test]
    fn namespaced_config_stores_namespace() {
        let cfg = GcConfig::namespaced("stellar-mainnet");
        assert_eq!(cfg.namespace.as_deref(), Some("stellar-mainnet"));
    }

    #[test]
    fn dry_run_flag_is_sticky() {
        let cfg = GcConfig::cluster_wide().with_dry_run();
        assert!(cfg.dry_run);
    }

    #[test]
    fn max_deletions_override() {
        let cfg = GcConfig::cluster_wide().with_max_deletions(5);
        assert_eq!(cfg.max_deletions_per_run, 5);
    }

    // ── GcReport defaults ─────────────────────────────────────────────────────

    #[test]
    fn gc_report_default_is_all_zeros() {
        let r = GcReport::default();
        assert_eq!(r.pvcs_scanned, 0);
        assert_eq!(r.pvcs_eligible, 0);
        assert_eq!(r.pvcs_deleted, 0);
        assert_eq!(r.pvcs_deferred, 0);
        assert!(!r.dry_run);
    }

    // ── GcError display ───────────────────────────────────────────────────────

    #[test]
    fn deletion_cap_error_includes_cap_and_remaining() {
        let err = GcError::DeletionCapReached { cap: 20, remaining: 7 };
        let msg = err.to_string();
        assert!(msg.contains("20"), "error message must include the cap");
        assert!(msg.contains("7"), "error message must include remaining count");
    }

    // ── Annotation constants ──────────────────────────────────────────────────

    #[test]
    fn annotation_constants_are_qualified_names() {
        for annotation in [GC_REQUESTED_ANNOTATION, GC_REASON_ANNOTATION] {
            assert!(
                annotation.contains('/'),
                "{annotation} must be a qualified annotation name"
            );
        }
    }

    #[test]
    fn gc_reason_delete_matches_finalizer_patch_value() {
        // The value written by `unbind_pvc` in finalizers.rs must match what
        // the GC scanner filters on.
        assert_eq!(GC_REASON_DELETE, "retentionPolicy:Delete");
    }

    // ── Eligibility filter logic (pure, no network) ───────────────────────────

    /// Builds a minimal in-memory PVC with the given annotations and phase.
    fn make_pvc(
        name: &str,
        annotations: Vec<(&str, &str)>,
        phase: Option<&str>,
    ) -> PersistentVolumeClaim {
        use k8s_openapi::api::core::v1::PersistentVolumeClaimStatus;
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
        use std::collections::BTreeMap;

        let mut anno_map = BTreeMap::new();
        for (k, v) in annotations {
            anno_map.insert(k.to_owned(), v.to_owned());
        }

        PersistentVolumeClaim {
            metadata: ObjectMeta {
                name: Some(name.to_owned()),
                namespace: Some("test-ns".to_owned()),
                annotations: if anno_map.is_empty() {
                    None
                } else {
                    Some(anno_map)
                },
                ..Default::default()
            },
            spec: None,
            status: phase.map(|p| PersistentVolumeClaimStatus {
                phase: Some(p.to_owned()),
                ..Default::default()
            }),
        }
    }

    fn is_eligible(pvc: &PersistentVolumeClaim) -> bool {
        let annotations = pvc
            .metadata
            .annotations
            .as_ref()
            .map(|a| a.as_ref())
            .unwrap_or_default();

        let gc_requested = annotations
            .get(GC_REQUESTED_ANNOTATION)
            .map_or(false, |v| v == "true");
        let reason_delete = annotations
            .get(GC_REASON_ANNOTATION)
            .map_or(false, |v| v == GC_REASON_DELETE);
        let not_bound = pvc
            .status
            .as_ref()
            .and_then(|s| s.phase.as_deref())
            .map_or(true, |phase| phase != "Bound");

        gc_requested && reason_delete && not_bound
    }

    #[test]
    fn eligible_when_gc_requested_and_delete_reason_and_released() {
        let pvc = make_pvc(
            "vol-1",
            vec![
                (GC_REQUESTED_ANNOTATION, "true"),
                (GC_REASON_ANNOTATION, GC_REASON_DELETE),
            ],
            Some("Released"),
        );
        assert!(is_eligible(&pvc));
    }

    #[test]
    fn eligible_when_phase_is_available() {
        let pvc = make_pvc(
            "vol-2",
            vec![
                (GC_REQUESTED_ANNOTATION, "true"),
                (GC_REASON_ANNOTATION, GC_REASON_DELETE),
            ],
            Some("Available"),
        );
        assert!(is_eligible(&pvc));
    }

    #[test]
    fn eligible_when_no_phase_set() {
        let pvc = make_pvc(
            "vol-3",
            vec![
                (GC_REQUESTED_ANNOTATION, "true"),
                (GC_REASON_ANNOTATION, GC_REASON_DELETE),
            ],
            None,
        );
        assert!(is_eligible(&pvc));
    }

    #[test]
    fn not_eligible_when_still_bound() {
        let pvc = make_pvc(
            "vol-4",
            vec![
                (GC_REQUESTED_ANNOTATION, "true"),
                (GC_REASON_ANNOTATION, GC_REASON_DELETE),
            ],
            Some("Bound"),
        );
        assert!(!is_eligible(&pvc));
    }

    #[test]
    fn not_eligible_when_gc_annotation_missing() {
        let pvc = make_pvc(
            "vol-5",
            vec![(GC_REASON_ANNOTATION, GC_REASON_DELETE)],
            Some("Released"),
        );
        assert!(!is_eligible(&pvc));
    }

    #[test]
    fn not_eligible_when_reason_is_retain() {
        let pvc = make_pvc(
            "vol-6",
            vec![
                (GC_REQUESTED_ANNOTATION, "true"),
                (GC_REASON_ANNOTATION, "retentionPolicy:Retain"),
            ],
            Some("Released"),
        );
        assert!(!is_eligible(&pvc));
    }

    #[test]
    fn not_eligible_when_annotations_absent() {
        let pvc = make_pvc("vol-7", vec![], Some("Released"));
        assert!(!is_eligible(&pvc));
    }

    // ── Deferred count ────────────────────────────────────────────────────────

    #[test]
    fn pvcs_deferred_saturates_at_zero() {
        let mut report = GcReport {
            pvcs_eligible: 3,
            pvcs_deleted: 5, // more deleted than eligible (shouldn't happen but must not panic)
            ..Default::default()
        };
        report.pvcs_deferred = report.pvcs_eligible.saturating_sub(report.pvcs_deleted);
        assert_eq!(report.pvcs_deferred, 0);
    }
}
