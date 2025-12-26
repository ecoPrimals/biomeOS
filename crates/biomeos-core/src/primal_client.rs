// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Common interface for primal clients
//!
//! This module defines the `PrimalClient` trait that all primal clients must implement.
//! It provides a consistent interface for health checks, availability queries, and
//! generic request execution across all primals in the ecosystem.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Common interface for primal clients
///
/// All primal clients (Songbird, ToadStool, Squirrel, NestGate, BearDog) implement
/// this trait to provide a consistent interface for BiomeOS to interact with them.
///
/// # Design Philosophy
/// - **Discovery over Configuration**: Clients should be discoverable at runtime
/// - **Graceful Degradation**: Missing primals should not crash the system
/// - **Clear Errors**: When a primal is unavailable, errors should be actionable
///
/// # Example
/// ```no_run
/// use biomeos_core::primal_client::PrimalClient;
///
/// async fn check_primal<P: PrimalClient>(client: &P) {
///     if client.is_available().await {
///         println!("{} is available at {}", client.name(), client.endpoint());
///     }
/// }
/// ```
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Get the primal's name (e.g., "songbird", "toadstool")
    fn name(&self) -> &str;

    /// Get the primal's endpoint URL
    fn endpoint(&self) -> &str;

    /// Check if the primal is currently available
    ///
    /// This performs a lightweight check (typically just a health endpoint call)
    /// to determine if the primal is reachable and responding.
    async fn is_available(&self) -> bool;

    /// Perform a comprehensive health check
    ///
    /// Returns detailed health status including any diagnostic information
    /// the primal provides.
    ///
    /// # Errors
    /// Returns an error if the primal is unreachable or returns an error response.
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Execute a generic request to the primal
    ///
    /// This is a low-level interface for making arbitrary requests. Most code
    /// should use the specialized methods on concrete client types instead.
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Request path (e.g., "/api/v1/services")
    /// * `body` - Optional request body as JSON
    ///
    /// # Errors
    /// Returns an error if the request fails or the primal returns an error response.
    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value>;
}

/// Health status information from a primal
///
/// Returned by `PrimalClient::health_check()` to provide detailed information
/// about a primal's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the primal is healthy
    pub healthy: bool,

    /// Human-readable status message
    pub message: String,

    /// Optional detailed diagnostic information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl HealthStatus {
    /// Create a healthy status
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            healthy: true,
            message: message.into(),
            details: None,
        }
    }

    /// Create an unhealthy status
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            message: message.into(),
            details: None,
        }
    }

    /// Add detailed diagnostic information
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_creation() {
        let healthy = HealthStatus::healthy("All systems operational");
        assert!(healthy.healthy);
        assert_eq!(healthy.message, "All systems operational");
        assert!(healthy.details.is_none());

        let unhealthy = HealthStatus::unhealthy("Service unavailable");
        assert!(!unhealthy.healthy);
        assert_eq!(unhealthy.message, "Service unavailable");
    }

    #[test]
    fn test_health_status_with_details() {
        let status = HealthStatus::healthy("OK").with_details(serde_json::json!({"uptime": 3600}));
        assert!(status.details.is_some());
    }
}
