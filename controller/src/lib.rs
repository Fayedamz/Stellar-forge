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
//! Stellar-K8s controller crate.
//!
//! This crate provides the deployment pipeline, admission webhook modules,
//! finalizer lifecycle management, and orphaned-volume garbage collection
//! used by the Stellar-K8s operator.
//!
//! # Modules
//!
//! - [`apis`] — Custom resource validation and API schema utilities,
//!   including [`apis::AutoscalerSpec`] and [`apis::AutoscalerStatus`]
//!   added by issue #471.
//! - [`autoscaler`] — Soroban RPC Auto-Scaling Controller (#471).
//!   Drives `spec.replicas` from pending RPC queue-length metrics with
//!   integer-only scaling math, a scale-down stabilisation window, and
//!   HPA co-existence guards.
//! - [`deployment`] — StellarNode deployment pipeline, including the
//!   [`deployment::optimizer`] WASM bytecode optimizer.
//! - [`lifecycle`] — kube-rs finalizer injection and execution logic, including
//!   the hard-timeout fallback that prevents stalled finalizers from blocking
//!   namespace deletion ([`lifecycle::finalizers`]).
//! - [`pdb`] — PodDisruptionBudget auto-tuning controller that keeps
//!   `maxUnavailable` at 0 while validators sync and only relaxes it to 1
//!   when the quorum can survive losing one validator.
//! - [`finalizers`] — Kubernetes finalizer cleanup recovery controller.
//!   Detects `StellarNode` resources stuck in `Terminating` state, verifies
//!   cloud storage detachment via AWS EBS / GCP PD APIs, and safely removes
//!   orphaned finalizers without risking data loss.
//! - [`storage`] — Persistent storage lifecycle management: in-memory tmpfs
//!   volumes ([`storage::tmpfs`]) and the orphaned-PVC garbage collector
//!   ([`storage::gc`]).
//! - [`quorum`] — Quorum-set graph analysis utilities.

pub mod apis;
pub mod autoscaler;
pub mod deployment;
pub mod lifecycle;
pub mod quorum;
pub mod storage;
