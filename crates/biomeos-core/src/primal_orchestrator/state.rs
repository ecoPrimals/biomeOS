// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal lifecycle state and `ManagedPrimal` trait

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use biomeos_types::identifiers::{Endpoint, PrimalId};

use crate::capabilities::Capability;
use crate::discovery_modern::HealthStatus;

/// Represents a primal's lifecycle state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimalState {
    /// Not yet started
    Pending,
    /// Currently starting up
    Starting,
    /// Healthy and operational
    Running,
    /// Started but degraded
    Degraded,
    /// Failed to start or crashed
    Failed {
        /// Human-readable failure reason
        reason: String,
    },
    /// Intentionally stopped
    Stopped,
}

/// Represents a primal that can be orchestrated
pub trait ManagedPrimal: Send + Sync {
    /// Get the primal's ID
    fn id(&self) -> &PrimalId;

    /// Get capabilities this primal provides
    fn provides(&self) -> &[Capability];

    /// Get capabilities this primal requires
    fn requires(&self) -> &[Capability];

    /// Get the primal's endpoint (if running)
    fn endpoint(&self) -> Pin<Box<dyn Future<Output = Option<Endpoint>> + Send + '_>>;

    /// Start the primal
    fn start(
        &self,
    ) -> Pin<Box<dyn Future<Output = biomeos_types::error::BiomeResult<()>> + Send + '_>>;

    /// Stop the primal
    fn stop(
        &self,
    ) -> Pin<Box<dyn Future<Output = biomeos_types::error::BiomeResult<()>> + Send + '_>>;

    /// Check if the primal is healthy
    fn health_check(
        &self,
    ) -> Pin<Box<dyn Future<Output = biomeos_types::error::BiomeResult<HealthStatus>> + Send + '_>>;

    /// Get the startup timeout
    fn startup_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

/// Internal record for a registered primal
pub struct PrimalRecord {
    pub primal: std::sync::Arc<dyn ManagedPrimal>,
    pub state: PrimalState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_state_debug() {
        assert!(format!("{:?}", PrimalState::Pending).contains("Pending"));
        assert!(format!("{:?}", PrimalState::Starting).contains("Starting"));
        assert!(format!("{:?}", PrimalState::Running).contains("Running"));
        assert!(format!("{:?}", PrimalState::Degraded).contains("Degraded"));
        assert!(format!("{:?}", PrimalState::Stopped).contains("Stopped"));
        let failed = PrimalState::Failed {
            reason: "boom".into(),
        };
        let dbg = format!("{failed:?}");
        assert!(dbg.contains("Failed"));
        assert!(dbg.contains("boom"));
    }

    #[test]
    fn test_primal_state_clone_and_eq() {
        let states = vec![
            PrimalState::Pending,
            PrimalState::Starting,
            PrimalState::Running,
            PrimalState::Degraded,
            PrimalState::Stopped,
            PrimalState::Failed { reason: "x".into() },
        ];
        for s in &states {
            let cloned = s.clone();
            assert_eq!(s, &cloned);
        }
        assert_ne!(PrimalState::Pending, PrimalState::Running);
    }

    #[test]
    fn test_primal_state_failed_different_reasons() {
        let f1 = PrimalState::Failed { reason: "a".into() };
        let f2 = PrimalState::Failed { reason: "b".into() };
        assert_ne!(f1, f2);
    }
}
