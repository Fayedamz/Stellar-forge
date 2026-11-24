// Copyright 2024 Stellar-K8s Contributors
// SPDX-License-Identifier: Apache-2.0
//! Storage management modules.
//!
//! This module groups components responsible for persistent storage lifecycle,
//! including the in-memory tmpfs volume manager and the orphaned-volume garbage
//! collection scanner.

pub mod gc;
pub mod tmpfs;

pub use gc::{GarbageCollector, GcConfig, GcError, GcReport};
