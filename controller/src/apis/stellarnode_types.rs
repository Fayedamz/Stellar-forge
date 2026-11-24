// Copyright 2024 Stellar-K8s Contributors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! CRD types for `StellarNode` — `api/v1alpha1/stellarnode_types.rs`
//!
//! This module defines the Kubernetes custom resource types for `StellarNode`,
//! including the [`AutoscalerSpec`] and [`AutoscalerStatus`] fields added
//! by issue #471.
//!
//! # Schema additions (issue #471)
//!
//! ```yaml
//! spec:
//!   autoscaler:                        # optional; omit to disable
//!     enabled: true
//!     minReplicas: 2
//!     maxReplicas: 20
//!     targetPendingPerReplica: 50
//!     scaleDownStabilisationSecs: 300
//!     metricsPort: 9090
//!     metricsPath: /metrics
//!
//! status:
//!   autoscaler:
//!     currentReplicas: 4
//!     desiredReplicas: 7
//!     lastScaleTime: "2024-11-19T10:00:00Z"
//!     scalingActive: true
//!     hpaClamped: false
//!     totalPendingRequests: 350
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AutoscalerSpec
// ---------------------------------------------------------------------------

/// Autoscaler configuration embedded in `StellarNodeSpec`.
///
/// When this field is present and `enabled` is `true` the operator manages
/// `spec.replicas` on the Soroban RPC `Deployment` through the queue-length
/// controller defined in [`crate::autoscaler`].
///
/// All integer fields use `u32` to match the scaling algorithm, which
/// intentionally avoids floating-point arithmetic.
///
/// # Example
///
/// ```yaml
/// spec:
///   autoscaler:
///     enabled: true
///     minReplicas: 2
///     maxReplicas: 20
///     targetPendingPerReplica: 50
///     scaleDownStabilisationSecs: 300
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoscalerSpec {
    /// Whether the autoscaler reconcile loop is active.
    ///
    /// Set to `false` to pause autoscaling without removing the spec block.
    /// Defaults to `true` when the field is present.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Minimum number of Soroban RPC replicas.
    ///
    /// The autoscaler never scales below this floor even when queue depth is
    /// zero.  Must be ≥ 1.  Defaults to `2`.
    #[serde(default = "default_min_replicas")]
    pub min_replicas: u32,

    /// Maximum number of Soroban RPC replicas.
    ///
    /// The autoscaler never scales above this ceiling regardless of queue
    /// depth.  Must be ≥ `min_replicas`.  Defaults to `20`.
    #[serde(default = "default_max_replicas")]
    pub max_replicas: u32,

    /// Number of pending RPC requests one replica is expected to absorb
    /// at saturation.
    ///
    /// Used as the divisor in the scaling formula:
    /// `desired = ceil(total_pending / target_pending_per_replica)`.
    /// Must be ≥ 1.  Defaults to `50`.
    #[serde(default = "default_target_pending_per_replica")]
    pub target_pending_per_replica: u32,

    /// Seconds the queue depth must stay below the scale-down threshold
    /// before the controller actually reduces replicas.
    ///
    /// Prevents rapid pod thrashing after a traffic burst subsides.
    /// Defaults to `300` (5 minutes).
    #[serde(default = "default_scale_down_stabilisation_secs")]
    pub scale_down_stabilisation_secs: u32,

    /// Port on which Soroban RPC pods expose Prometheus metrics.
    ///
    /// Defaults to `9090`.
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    /// HTTP path of the Prometheus metrics endpoint on each pod.
    ///
    /// Defaults to `"/metrics"`.
    #[serde(default = "default_metrics_path")]
    pub metrics_path: String,
}

impl Default for AutoscalerSpec {
    fn default() -> Self {
        Self {
            enabled: true,
            min_replicas: default_min_replicas(),
            max_replicas: default_max_replicas(),
            target_pending_per_replica: default_target_pending_per_replica(),
            scale_down_stabilisation_secs: default_scale_down_stabilisation_secs(),
            metrics_port: default_metrics_port(),
            metrics_path: default_metrics_path(),
        }
    }
}

impl AutoscalerSpec {
    /// Validate the spec, returning a human-readable error on the first
    /// constraint violation found.
    ///
    /// Called by the admission webhook before persisting the resource.
    pub fn validate(&self) -> Result<(), String> {
        if self.min_replicas < 1 {
            return Err("autoscaler.minReplicas must be ≥ 1".to_string());
        }
        if self.max_replicas < self.min_replicas {
            return Err(format!(
                "autoscaler.maxReplicas ({}) must be ≥ minReplicas ({})",
                self.max_replicas, self.min_replicas
            ));
        }
        if self.target_pending_per_replica < 1 {
            return Err("autoscaler.targetPendingPerReplica must be ≥ 1".to_string());
        }
        if self.metrics_path.is_empty() {
            return Err("autoscaler.metricsPath must not be empty".to_string());
        }
        if !self.metrics_path.starts_with('/') {
            return Err(format!(
                "autoscaler.metricsPath '{}' must start with '/'",
                self.metrics_path
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// AutoscalerStatus
// ---------------------------------------------------------------------------

/// Autoscaler runtime state written back to `StellarNodeStatus` by the
/// reconcile loop so operators can observe scaling behaviour without
/// tailing controller logs.
///
/// All fields are optional so the status block serialises gracefully when
/// the autoscaler has not yet run (e.g. immediately after creation).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoscalerStatus {
    /// Replica count at the start of the most recent reconcile pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_replicas: Option<u32>,

    /// Replica count the algorithm computed in the most recent pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desired_replicas: Option<u32>,

    /// RFC 3339 timestamp of the last scale patch, if any.
    ///
    /// `None` when no patch has been issued since the resource was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scale_time: Option<String>,

    /// `true` when the most recent pass issued a scale patch.
    #[serde(default)]
    pub scaling_active: bool,

    /// `true` when the desired count was clamped to fit inside a co-existing
    /// HPA's `[minReplicas, maxReplicas]` range.
    #[serde(default)]
    pub hpa_clamped: bool,

    /// Sum of `soroban_rpc_pending_requests` across all pods at the most
    /// recent scrape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pending_requests: Option<u64>,

    /// Number of pods for which the metric scrape failed in the most recent
    /// reconcile pass.
    #[serde(default)]
    pub scrape_failures: usize,
}

// ---------------------------------------------------------------------------
// Serde default helpers
// ---------------------------------------------------------------------------

fn default_true() -> bool {
    true
}

fn default_min_replicas() -> u32 {
    2
}

fn default_max_replicas() -> u32 {
    20
}

fn default_target_pending_per_replica() -> u32 {
    50
}

fn default_scale_down_stabilisation_secs() -> u32 {
    300
}

fn default_metrics_port() -> u16 {
    9090
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── AutoscalerSpec defaults ─────────────────────────────────────────────

    #[test]
    fn default_spec_is_valid() {
        let spec = AutoscalerSpec::default();
        assert!(spec.validate().is_ok(), "default spec must pass validation");
        assert!(spec.enabled);
        assert_eq!(spec.min_replicas, 2);
        assert_eq!(spec.max_replicas, 20);
        assert_eq!(spec.target_pending_per_replica, 50);
        assert_eq!(spec.scale_down_stabilisation_secs, 300);
        assert_eq!(spec.metrics_port, 9090);
        assert_eq!(spec.metrics_path, "/metrics");
    }

    // ── AutoscalerSpec::validate ────────────────────────────────────────────

    #[test]
    fn validate_rejects_zero_min_replicas() {
        let spec = AutoscalerSpec { min_replicas: 0, ..AutoscalerSpec::default() };
        let err = spec.validate().unwrap_err();
        assert!(err.contains("minReplicas"), "error should mention minReplicas: {err}");
    }

    #[test]
    fn validate_rejects_max_less_than_min() {
        let spec = AutoscalerSpec {
            min_replicas: 5,
            max_replicas: 3,
            ..AutoscalerSpec::default()
        };
        let err = spec.validate().unwrap_err();
        assert!(
            err.contains("maxReplicas"),
            "error should mention maxReplicas: {err}"
        );
    }

    #[test]
    fn validate_rejects_zero_target() {
        let spec = AutoscalerSpec {
            target_pending_per_replica: 0,
            ..AutoscalerSpec::default()
        };
        let err = spec.validate().unwrap_err();
        assert!(err.contains("targetPendingPerReplica"), "{err}");
    }

    #[test]
    fn validate_rejects_empty_metrics_path() {
        let spec = AutoscalerSpec {
            metrics_path: String::new(),
            ..AutoscalerSpec::default()
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn validate_rejects_path_without_leading_slash() {
        let spec = AutoscalerSpec {
            metrics_path: "metrics".to_string(),
            ..AutoscalerSpec::default()
        };
        let err = spec.validate().unwrap_err();
        assert!(err.contains("must start with '/'"), "{err}");
    }

    #[test]
    fn validate_accepts_min_equal_to_max() {
        // min == max is degenerate but valid: fixed replica count
        let spec = AutoscalerSpec {
            min_replicas: 5,
            max_replicas: 5,
            ..AutoscalerSpec::default()
        };
        assert!(spec.validate().is_ok());
    }

    // ── Serde round-trips ───────────────────────────────────────────────────

    #[test]
    fn spec_roundtrip_full() {
        let spec = AutoscalerSpec {
            enabled: true,
            min_replicas: 3,
            max_replicas: 15,
            target_pending_per_replica: 100,
            scale_down_stabilisation_secs: 180,
            metrics_port: 8080,
            metrics_path: "/custom/metrics".to_string(),
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let back: AutoscalerSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(spec, back);
    }

    #[test]
    fn spec_deserialize_empty_object_uses_defaults() {
        let back: AutoscalerSpec = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(back, AutoscalerSpec::default());
    }

    #[test]
    fn spec_deserialize_partial_uses_defaults_for_missing() {
        let json = r#"{"minReplicas": 4}"#;
        let spec: AutoscalerSpec = serde_json::from_str(json).expect("deserialize");
        assert_eq!(spec.min_replicas, 4);
        assert_eq!(spec.max_replicas, 20); // default
        assert_eq!(spec.target_pending_per_replica, 50); // default
    }

    #[test]
    fn spec_camel_case_field_names_in_json() {
        let spec = AutoscalerSpec::default();
        let json = serde_json::to_value(&spec).expect("to_value");
        // Kubernetes CRD convention: camelCase field names
        assert!(json.get("minReplicas").is_some(), "expected camelCase minReplicas");
        assert!(json.get("maxReplicas").is_some());
        assert!(json.get("targetPendingPerReplica").is_some());
        assert!(json.get("scaleDownStabilisationSecs").is_some());
        assert!(json.get("metricsPort").is_some());
        assert!(json.get("metricsPath").is_some());
        // No snake_case keys
        assert!(json.get("min_replicas").is_none());
        assert!(json.get("max_replicas").is_none());
    }

    // ── AutoscalerStatus ────────────────────────────────────────────────────

    #[test]
    fn status_default_is_all_none() {
        let s = AutoscalerStatus::default();
        assert_eq!(s.current_replicas, None);
        assert_eq!(s.desired_replicas, None);
        assert_eq!(s.last_scale_time, None);
        assert!(!s.scaling_active);
        assert!(!s.hpa_clamped);
        assert_eq!(s.total_pending_requests, None);
        assert_eq!(s.scrape_failures, 0);
    }

    #[test]
    fn status_roundtrip() {
        let status = AutoscalerStatus {
            current_replicas: Some(4),
            desired_replicas: Some(7),
            last_scale_time: Some("2024-11-19T10:00:00Z".to_string()),
            scaling_active: true,
            hpa_clamped: false,
            total_pending_requests: Some(350),
            scrape_failures: 1,
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let back: AutoscalerStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(status, back);
    }

    #[test]
    fn status_skips_none_fields_in_json() {
        let status = AutoscalerStatus::default();
        let json = serde_json::to_value(&status).expect("to_value");
        // None fields should be absent (skip_serializing_if = "Option::is_none")
        assert!(json.get("currentReplicas").is_none());
        assert!(json.get("desiredReplicas").is_none());
        assert!(json.get("lastScaleTime").is_none());
        assert!(json.get("totalPendingRequests").is_none());
        // Bool fields with #[serde(default)] are always present
        assert!(json.get("scalingActive").is_some());
        assert!(json.get("hpaClamped").is_some());
    }

    #[test]
    fn status_camel_case_field_names_in_json() {
        let status = AutoscalerStatus {
            current_replicas: Some(2),
            desired_replicas: Some(4),
            ..AutoscalerStatus::default()
        };
        let json = serde_json::to_value(&status).expect("to_value");
        assert!(json.get("currentReplicas").is_some());
        assert!(json.get("desiredReplicas").is_some());
        assert!(json.get("scalingActive").is_some());
        assert!(json.get("hpaClamped").is_some());
        assert!(json.get("scrapeFailures").is_some());
        // No snake_case
        assert!(json.get("current_replicas").is_none());
        assert!(json.get("scaling_active").is_none());
    }
}
