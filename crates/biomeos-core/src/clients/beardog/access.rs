// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Access control and audit logging
//!
//! This module provides security policy enforcement and auditing:
//! - Access control validation
//! - Audit log retrieval
//! - Policy evaluation
//!
//! # Security Model
//!
//! BearDog uses a capability-based access control model where:
//! - Every request includes: subject, resource, action
//! - Policies are evaluated in real-time
//! - All decisions are logged for audit

use super::client::BearDogClient;
use super::types::{AccessRequest, AccessDecision, AuditEntry};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use tracing::{debug, info};

impl BearDogClient {
    /// Validate an access control request
    ///
    /// Uses BearDog's JSON-RPC API: `access.validate`
    ///
    /// # Arguments
    /// * `request` - Access request with subject, resource, action
    ///
    /// # Returns
    /// Access decision (allow/deny) with reasoning
    ///
    /// # Errors
    /// Returns an error if policy evaluation fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::{BearDogClient, AccessRequest};
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// 
    /// let request = AccessRequest {
    ///     subject: "user:alice".to_string(),
    ///     resource: "file:sensitive-data.txt".to_string(),
    ///     action: "read".to_string(),
    ///     context: json!({"ip": "192.168.1.100"}),
    /// };
    /// 
    /// let decision = beardog.validate_access(&request).await?;
    /// if decision.decision == "allow" {
    ///     println!("Access granted!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_access(&self, request: &AccessRequest) -> Result<AccessDecision> {
        info!(
            "🔒 Validating access: {} -> {} ({})",
            request.subject, request.resource, request.action
        );

        let response = self
            .transport
            .call(
                "access.validate",
                Some(json!({
                    "subject": request.subject,
                    "resource": request.resource,
                    "action": request.action,
                    "context": request.context,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call access.validate")?;

        let decision = response["decision"]
            .as_str()
            .context("Missing decision in response")?
            .to_string();

        debug!("✅ Access decision: {}", decision);

        Ok(AccessDecision {
            decision,
            reason: response["reason"]
                .as_str()
                .unwrap_or("No reason provided")
                .to_string(),
            confidence: response["confidence"].as_f64().unwrap_or(1.0),
            metadata: response["metadata"].clone(),
        })
    }

    /// Retrieve audit log entries
    ///
    /// Uses BearDog's JSON-RPC API: `audit.get_log`
    ///
    /// # Arguments
    /// * `filters` - Optional filters for the audit log:
    ///   - `subject`: Filter by subject
    ///   - `resource`: Filter by resource
    ///   - `action`: Filter by action
    ///   - `start_time`: Filter by start time
    ///   - `end_time`: Filter by end time
    ///
    /// # Returns
    /// List of audit entries
    ///
    /// # Errors
    /// Returns an error if audit log retrieval fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// 
    /// // Get all audit entries
    /// let entries = beardog.get_audit_log(None).await?;
    /// 
    /// // Get filtered entries
    /// let filters = json!({"subject": "user:alice", "action": "read"});
    /// let entries = beardog.get_audit_log(Some(&filters)).await?;
    /// 
    /// for entry in entries {
    ///     println!("{}: {} -> {}", entry.timestamp, entry.subject, entry.resource);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_audit_log(&self, filters: Option<&Value>) -> Result<Vec<AuditEntry>> {
        info!("📋 Retrieving audit log");

        let mut params = json!({
            "family_id": self.family_id
        });

        if let Some(filters) = filters {
            params["filters"] = filters.clone();
        }

        let response = self
            .transport
            .call("audit.get_log", Some(params))
            .await
            .context("Failed to call audit.get_log")?;

        let entries: Vec<AuditEntry> = serde_json::from_value(
            response["entries"].clone()
        )
        .context("Failed to parse audit entries")?;

        debug!("✅ Retrieved {} audit entries", entries.len());

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_validate_access() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let request = AccessRequest {
            subject: "test-user".to_string(),
            resource: "test-resource".to_string(),
            action: "read".to_string(),
            context: json!({}),
        };
        let decision = client.validate_access(&request).await.unwrap();
        assert!(!decision.decision.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_get_audit_log() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let entries = client.get_audit_log(None).await.unwrap();
        // Just verify we can retrieve entries
        assert!(entries.len() >= 0);
    }
}
