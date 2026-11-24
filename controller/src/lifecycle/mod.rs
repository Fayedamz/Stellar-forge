// Copyright 2024 Stellar-K8s Contributors
// SPDX-License-Identifier: Apache-2.0
//! Lifecycle management modules.
//!
//! This module groups components responsible for the complete lifecycle of a
//! [`StellarNode`] resource, including finalizer injection, graceful shutdown
//! sequencing, and the hard-timeout fallback that prevents a stalled finalizer
//! from permanently blocking namespace deletion.

pub mod finalizers;

pub use finalizers::{
    ensure_finalizer, run_finalizer, FinalizerConfig, FinalizerError, STELLAR_NODE_FINALIZER,
};
