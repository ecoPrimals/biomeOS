// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Lifecycle management handler for Neural API
//!
//! Exposes lifecycle management operations via JSON-RPC:
//! - `lifecycle.status` - Get status of all managed primals
//! - `lifecycle.get` - Get detailed info for a specific primal
//! - `lifecycle.composition` - Live composition state for dashboards (active/degraded/dead)
//! - `lifecycle.resurrect` - Force resurrection of a degraded/dead primal
//! - `lifecycle.apoptosis` - Initiate graceful shutdown
//! - `lifecycle.register` - Register a primal for management
//! - `lifecycle.shutdown_all` - Initiate system-wide shutdown
//! - `composition.reload` - Hot-swap a single primal without full restart (JH-3)
//! - `composition.status` - Adaptive daemon surface: active_users, primal_health, resource_pressure
//!
//! Split into submodules by JSON-RPC concern (`status`, `registration`, `transitions`,
//! `dashboard`, `spring`, `mesh`). `composition.status` / `composition.health` live in
//! sibling [`super::composition`].

mod dashboard;
mod helpers;
mod mesh;
mod registration;
mod spring;
mod status;
mod transitions;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lifecycle_manager::LifecycleManager;

/// Lifecycle handler for Neural API
#[derive(Clone)]
pub struct LifecycleHandler {
    pub(crate) manager: Arc<RwLock<LifecycleManager>>,
    /// Monotonic topology version, incremented on each composition change
    /// (register, reload, apoptosis). Used by `composition.reload` contract.
    pub(crate) topology_version: Arc<std::sync::atomic::AtomicU64>,
    /// Shared graph execution status map (for workload counts in `spring_status`).
    executions: Option<
        Arc<RwLock<std::collections::HashMap<String, crate::handlers::graph::ExecutionStatus>>>,
    >,
}

impl LifecycleHandler {
    /// Create a new lifecycle handler
    #[must_use]
    pub fn new(family_id: &str) -> Self {
        Self {
            manager: Arc::new(RwLock::new(LifecycleManager::new(family_id))),
            topology_version: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            executions: None,
        }
    }

    /// Create with an existing manager
    pub fn with_manager(manager: Arc<RwLock<LifecycleManager>>) -> Self {
        Self {
            manager,
            topology_version: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            executions: None,
        }
    }

    /// Wire the shared graph executions map (enables workload counts in `spring_status`).
    pub fn with_executions(
        mut self,
        executions: Arc<
            RwLock<std::collections::HashMap<String, crate::handlers::graph::ExecutionStatus>>,
        >,
    ) -> Self {
        self.executions = Some(executions);
        self
    }

    pub(crate) fn bump_topology(&self) -> u64 {
        self.topology_version
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    /// Start monitoring loop
    pub async fn start_monitoring(&self) -> Result<()> {
        let manager = self.manager.read().await;
        manager.start_monitoring().await
    }
}
