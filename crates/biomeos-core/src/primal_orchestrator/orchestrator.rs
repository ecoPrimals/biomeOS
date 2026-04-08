// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal Orchestrator - Async, concurrent primal lifecycle management
//!
//! Handles complex startup choreography with dependency resolution.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use biomeos_types::identifiers::PrimalId;

use crate::retry::RetryPolicy;

use super::health_monitor::PrimalHealthMonitor;
use super::state::PrimalRecord;

/// Orchestrates primal lifecycle with dependency resolution
pub struct PrimalOrchestrator {
    pub(super) primals: Arc<RwLock<HashMap<PrimalId, PrimalRecord>>>,
    pub(super) health_monitor: Arc<PrimalHealthMonitor>,
    pub(super) retry_policy: RetryPolicy,
}

impl PrimalOrchestrator {
    /// Create a new orchestrator
    #[must_use]
    pub fn new(health_monitor: Arc<PrimalHealthMonitor>, retry_policy: RetryPolicy) -> Self {
        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
            health_monitor,
            retry_policy,
        }
    }
}
