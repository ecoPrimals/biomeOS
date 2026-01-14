//! BearDog Access Control Client
//!
//! Provides access control validation and audit logging using BearDog's JSON-RPC API.
//!
//! Uses the real BearDog `access.*` methods discovered from v0.9.0+

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::clients::transport::PrimalTransport;

/// Access control client for BearDog
///
/// Provides real access control validation via JSON-RPC
pub struct AccessClient {
    transport: PrimalTransport,
}

impl AccessClient {
    /// Create a new access client with the given transport
    pub fn new(transport: PrimalTransport) -> Self {
        Self { transport }
    }

    /// Validate access request using BearDog's `access.validate` method
    ///
    /// # Arguments
    /// * `request` - Access request to validate
    ///
    /// # Returns
    /// * Access decision (Allow or Deny)
    pub async fn validate(&self, request: &AccessRequest) -> Result<AccessDecision> {
        let response = self
            .transport
            .call_method(
                "access.validate",
                Some(serde_json::json!({
                    "subject": request.subject,
                    "resource": request.resource,
                    "action": request.action,
                    "context": request.context,
                })),
            )
            .await
            .context("Failed to call access.validate")?;

        let decision = response["decision"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing decision in response"))?;

        match decision {
            "allow" | "Allow" => Ok(AccessDecision::Allow),
            "deny" | "Deny" => {
                let reason = response["reason"]
                    .as_str()
                    .unwrap_or("Access denied")
                    .to_string();
                Ok(AccessDecision::Deny { reason })
            }
            _ => Err(anyhow::anyhow!("Unknown decision: {}", decision)),
        }
    }

    /// Create an audit log entry using BearDog's `access.audit` method
    ///
    /// # Arguments
    /// * `request` - Access request that was performed
    /// * `decision` - Decision that was made
    /// * `metadata` - Additional metadata to log
    ///
    /// # Returns
    /// * Audit entry ID
    pub async fn audit(
        &self,
        request: &AccessRequest,
        decision: &AccessDecision,
        metadata: Option<serde_json::Value>,
    ) -> Result<String> {
        let decision_str = match decision {
            AccessDecision::Allow => "allow",
            AccessDecision::Deny { .. } => "deny",
        };

        let response = self
            .transport
            .call_method(
                "access.audit",
                Some(serde_json::json!({
                    "subject": request.subject,
                    "resource": request.resource,
                    "action": request.action,
                    "decision": decision_str,
                    "context": request.context,
                    "metadata": metadata.unwrap_or(serde_json::json!({})),
                })),
            )
            .await
            .context("Failed to call access.audit")?;

        Ok(response["audit_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string())
    }

    /// Query audit logs using BearDog's `access.query_audit` method
    ///
    /// # Arguments
    /// * `subject` - Optional subject filter
    /// * `resource` - Optional resource filter
    /// * `limit` - Maximum number of entries to return
    ///
    /// # Returns
    /// * Vector of audit entries
    pub async fn query_audit(
        &self,
        subject: Option<&str>,
        resource: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<AuditEntry>> {
        let mut params = serde_json::json!({});

        if let Some(s) = subject {
            params["subject"] = serde_json::Value::String(s.to_string());
        }
        if let Some(r) = resource {
            params["resource"] = serde_json::Value::String(r.to_string());
        }
        if let Some(l) = limit {
            params["limit"] = serde_json::Value::Number(l.into());
        }

        let response = self
            .transport
            .call_method("access.query_audit", Some(params))
            .await
            .context("Failed to call access.query_audit")?;

        let entries_array = response["entries"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected 'entries' array in response"))?;

        let mut entries = Vec::new();
        for entry in entries_array {
            entries.push(AuditEntry {
                audit_id: entry["audit_id"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                subject: entry["subject"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                resource: entry["resource"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                action: entry["action"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                decision: entry["decision"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                timestamp: entry["timestamp"].as_u64().unwrap_or(0),
            });
        }

        Ok(entries)
    }
}

/// Access request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    /// Subject requesting access (user, service, etc.)
    pub subject: String,
    /// Resource being accessed
    pub resource: String,
    /// Action being performed (read, write, execute, etc.)
    pub action: String,
    /// Additional context for access decision
    #[serde(default)]
    pub context: serde_json::Value,
}

/// Access decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessDecision {
    /// Access granted
    Allow,
    /// Access denied with reason
    Deny { reason: String },
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique audit entry ID
    pub audit_id: String,
    /// Subject that performed the action
    pub subject: String,
    /// Resource that was accessed
    pub resource: String,
    /// Action that was performed
    pub action: String,
    /// Decision that was made
    pub decision: String,
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
}
