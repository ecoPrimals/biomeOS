// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal Lifecycle Manager
//!
//! Robust lifecycle management for NUCLEUS (Tower, Node, Nest) deployments:
//! - **Germination**: Birth primal with minimal knowledge
//! - **Incubation**: Health monitoring during startup
//! - **Active**: Running and healthy
//! - **Degraded**: Running but unhealthy (will attempt resurrection)
//! - **Apoptosis**: Programmed graceful shutdown
//! - **Resurrection**: Automatic restart from deployment graph
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   PRIMAL LIFECYCLE MANAGER                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  GERMINATION → INCUBATION → ACTIVE ←→ DEGRADED → APOPTOSIS     │
//! │       ↑                        ↓                    ↓          │
//! │       └────── RESURRECTION ←───┴────────────────────┘          │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Health Monitoring**: Configurable interval JSON-RPC pings
//! - **Crash Detection**: Socket timeout or process death
//! - **Auto-Resurrection**: Restart from retained deployment graph
//! - **Dependency Awareness**: Respects primal dependency order
//! - **Graceful Apoptosis**: Coordinated shutdown with cleanup

mod apoptosis;
mod germination;
mod helpers;
mod monitoring;
mod resurrection;
mod types;

pub use types::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::health_check::HealthChecker;
use crate::neural_graph::Graph;
use crate::nucleation::SocketNucleation;

/// Primal Lifecycle Manager
///
/// Manages the lifecycle of all primals in a NUCLEUS deployment:
/// - Monitors health and detects crashes
/// - Resurrects dead primals from deployment graphs
/// - Coordinates graceful shutdown (apoptosis)
pub struct LifecycleManager {
    /// Managed primals (name -> `ManagedPrimal`)
    pub(crate) primals: Arc<RwLock<HashMap<String, ManagedPrimal>>>,

    /// Family ID
    pub(crate) family_id: String,

    /// Socket nucleation for deterministic paths
    pub(crate) nucleation: Arc<RwLock<SocketNucleation>>,

    /// Health checker
    pub(crate) health_checker: HealthChecker,

    /// Deployment graphs (for resurrection)
    pub(crate) deployment_graphs: Arc<RwLock<HashMap<String, Graph>>>,

    /// Global health check interval
    pub(crate) health_check_interval: Duration,

    /// Shutdown flag
    pub(crate) shutdown: Arc<RwLock<bool>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(family_id: impl Into<String>) -> Self {
        let family_id = family_id.into();

        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
            family_id,
            nucleation: Arc::new(RwLock::new(SocketNucleation::default())),
            health_checker: HealthChecker::new_default(),
            deployment_graphs: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval: Duration::from_secs(10),
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        family_id: impl Into<String>,
        health_check_interval: Duration,
        nucleation: Arc<RwLock<SocketNucleation>>,
    ) -> Self {
        let family_id = family_id.into();

        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
            family_id,
            nucleation,
            health_checker: HealthChecker::new_default(),
            deployment_graphs: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
