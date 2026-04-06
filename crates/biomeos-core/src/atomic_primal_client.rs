// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Atomic Primal Client - High-level API for common primal operations
//!
//! **Universal IPC v3.0**: Supports multi-transport discovery and communication.
//!
//! This client provides convenience methods for common primal operations
//! like health checks, command execution, and capability queries.
//! Extracted from `atomic_client.rs` to keep files under 1000 lines.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

use crate::atomic_client::AtomicClient;
use crate::socket_discovery::TransportEndpoint;

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Standard output from the command
    pub stdout: String,
    /// Standard error output
    pub stderr: String,
    /// Process exit code (if available)
    pub exit_code: Option<i32>,
}

/// Atomic Primal Client - High-level API for common primal operations
///
/// **Universal IPC v3.0**: Supports multi-transport discovery and communication.
///
/// This client provides convenience methods for common primal operations
/// like health checks, command execution, and capability queries.
#[derive(Debug, Clone)]
pub struct AtomicPrimalClient {
    client: AtomicClient,
    primal_name: String,
}

impl AtomicPrimalClient {
    /// Discover a primal and create a high-level client
    ///
    /// **Universal IPC v3.0**: Uses automatic transport fallback.
    pub async fn discover(primal_name: &str) -> Result<Self> {
        let client = AtomicClient::discover(primal_name).await?;
        Ok(Self {
            client,
            primal_name: primal_name.to_string(),
        })
    }

    /// Create a client with explicit Unix socket path
    pub fn unix(primal_name: impl Into<String>, socket_path: impl AsRef<Path>) -> Self {
        Self {
            client: AtomicClient::unix(socket_path),
            primal_name: primal_name.into(),
        }
    }

    /// Create a client with explicit TCP endpoint
    pub fn tcp(primal_name: impl Into<String>, host: impl AsRef<str>, port: u16) -> Self {
        Self {
            client: AtomicClient::tcp(host, port),
            primal_name: primal_name.into(),
        }
    }

    /// Create a client from a transport endpoint
    pub fn from_endpoint(primal_name: impl Into<String>, endpoint: TransportEndpoint) -> Self {
        Self {
            client: AtomicClient::from_endpoint(endpoint),
            primal_name: primal_name.into(),
        }
    }

    /// Health check (ping)
    pub async fn health_check(&self) -> Result<()> {
        let result = self.client.call("health.ping", Value::Null).await?;

        if result.get("status") == Some(&Value::String("ok".to_string())) {
            Ok(())
        } else {
            anyhow::bail!("Primal health check failed: {result:?}")
        }
    }

    /// Get primal identity and capabilities
    pub async fn get_identity(&self) -> Result<Value> {
        self.client.call("identity.get", Value::Null).await
    }

    /// Execute a command in the primal (if supported)
    pub async fn execute_command(&self, command: &str) -> Result<ExecutionResult> {
        let result = self
            .client
            .call(
                "execute_command",
                serde_json::json!({
                    "command": command,
                    "timeout_seconds": 60
                }),
            )
            .await?;

        Ok(ExecutionResult {
            stdout: result
                .get("stdout")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            stderr: result
                .get("stderr")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            exit_code: result
                .get("exit_code")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
        })
    }

    /// Get the primal name
    #[must_use]
    pub fn primal_name(&self) -> &str {
        &self.primal_name
    }

    /// Get direct access to the atomic client
    #[must_use]
    pub const fn atomic_client(&self) -> &AtomicClient {
        &self.client
    }

    /// Get the transport endpoint
    #[must_use]
    pub fn endpoint(&self) -> &TransportEndpoint {
        self.client.endpoint()
    }

    /// Check if the primal is available
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.client.is_available()
    }
}
