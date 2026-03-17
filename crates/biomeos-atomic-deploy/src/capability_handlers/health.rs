// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability-based health check handler
//!
//! Performs health checks on primals via capability-based discovery,
//! using JSON-RPC health.check over Unix sockets.

use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::executor::context::ExecutionContext;
use crate::neural_graph::GraphNode;

use super::discovery::{known_primal_names, resolve_capability_to_primal};

/// Call primal health endpoint via JSON-RPC using AtomicClient
///
/// Uses Universal IPC v3.0 AtomicClient for multi-transport support.
/// This enables Unix sockets, abstract sockets (Android), and TCP fallback.
pub(crate) async fn call_primal_health(socket_path: &str) -> Result<bool> {
    use biomeos_core::atomic_client::AtomicClient;

    // Create AtomicClient from socket path (supports Unix sockets)
    let client = AtomicClient::unix(socket_path);

    // Use health.check method (semantic naming standard)
    let response = client.call("health.check", json!({})).await?;

    Ok(response
        .get("healthy")
        .and_then(|h| h.as_bool())
        .unwrap_or(false))
}

/// Health check for capability-based deployment
pub async fn health_check_capability(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    info!("🏥 Health check for capability-based deployment");

    let operation = node
        .operation
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;

    let params = &operation.params;

    debug!("   Health check params: {:?}", params);

    let family_id = params
        .get("family_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);

    let mut checks_passed = Vec::new();
    let mut checks_failed = Vec::new();

    // Determine which primals to check
    let primals_to_check = if let Some(ref primal) = node.primal {
        if let Some(ref capability) = primal.by_capability {
            // Check specific capability
            resolve_capability_to_primal(capability)
                .map(|p| vec![p])
                .unwrap_or_default()
        } else {
            vec![]
        }
    } else {
        // Check all known primals (from capability domains)
        // See: config/capability_registry.toml for domain definitions
        known_primal_names()
    };

    // Check each primal
    for primal_name in primals_to_check {
        let socket_path = context.get_socket_path(primal_name).await;

        if PathBuf::from(&socket_path).exists() {
            // Try JSON-RPC health check
            match call_primal_health(&socket_path).await {
                Ok(healthy) if healthy => {
                    info!("   ✅ {} @ {} - healthy", primal_name, socket_path);
                    checks_passed.push(primal_name.to_string());
                }
                Ok(_) => {
                    warn!("   ⚠️  {} @ {} - unhealthy", primal_name, socket_path);
                    checks_failed.push(primal_name.to_string());
                }
                Err(e) => {
                    warn!(
                        "   ⚠️  {} @ {} - RPC failed: {}",
                        primal_name, socket_path, e
                    );
                    // Socket exists but RPC failed - could be starting up
                    checks_failed.push(primal_name.to_string());
                }
            }
        } else {
            debug!("   ⏸️  {} - socket not found", primal_name);
            // Not running - not necessarily a failure
        }
    }

    let all_healthy = checks_failed.is_empty() && !checks_passed.is_empty();

    if all_healthy {
        info!(
            "   ✅ All health checks passed ({} checks)",
            checks_passed.len()
        );
    } else if checks_failed.is_empty() {
        info!("   ⏸️  No primals running to check");
    } else {
        warn!("   ⚠️  Some health checks failed: {:?}", checks_failed);
    }

    Ok(json!({
        "healthy": all_healthy,
        "family_id": family_id,
        "checks_passed": checks_passed,
        "checks_failed": checks_failed,
        "total_checks": checks_passed.len() + checks_failed.len()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::context::ExecutionContext;
    use crate::neural_graph::{GraphNode, Operation, PrimalSelector};
    use std::collections::HashMap;

    fn make_health_node(capability: Option<&str>, family_id: Option<&str>) -> GraphNode {
        let mut params = HashMap::new();
        if let Some(fid) = family_id {
            params.insert("family_id".to_string(), serde_json::json!(fid));
        }

        let operation = Some(Operation {
            name: "health_check".to_string(),
            params,
            environment: None,
        });

        let primal = capability.map(|c| PrimalSelector {
            by_capability: Some(c.to_string()),
            by_name: None,
        });

        GraphNode {
            id: "health_node".to_string(),
            primal,
            operation,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_health_check_capability_missing_operation() {
        let node = GraphNode {
            id: "test".to_string(),
            primal: Some(PrimalSelector {
                by_capability: Some("encryption".to_string()),
                by_name: None,
            }),
            ..Default::default()
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = health_check_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when operation is missing");
        assert!(
            err.to_string().contains("operation"),
            "Error should mention operation: {err}"
        );
    }

    #[tokio::test]
    async fn test_health_check_capability_specific_capability() {
        let node = make_health_node(Some("encryption"), Some("test_family"));
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx)
            .await
            .expect("Health check should succeed");

        assert_eq!(result["family_id"], "test_family");
        assert!(
            result.get("healthy").is_some(),
            "Result should have healthy field"
        );
        assert!(result.get("checks_passed").is_some());
        assert!(result.get("checks_failed").is_some());
        assert!(result.get("total_checks").is_some());
        let total = result["total_checks"].as_u64().unwrap_or(0);
        assert!(
            total <= 1,
            "Single capability should check at most 1 primal"
        );
    }

    #[tokio::test]
    async fn test_health_check_capability_unknown_capability() {
        let node = make_health_node(Some("unknown_cap"), None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx)
            .await
            .expect("Health check should succeed with empty primals");

        assert_eq!(result["total_checks"], 0);
        assert!(!result["healthy"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_health_check_capability_all_primals() {
        let node = make_health_node(None, None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx)
            .await
            .expect("Health check should succeed");

        let total = result["total_checks"].as_u64().unwrap_or(0);
        assert!(
            total <= 5,
            "Should check at most 5 known primals, got {total}"
        );
        assert!(result.get("healthy").is_some());
        assert!(result.get("checks_passed").is_some());
        assert!(result.get("checks_failed").is_some());
    }

    #[tokio::test]
    async fn test_health_check_capability_output_json_structure() {
        let node = make_health_node(Some("encryption"), None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx).await.unwrap();

        assert!(result.get("healthy").is_some());
        assert!(result.get("family_id").is_some());
        assert!(result.get("checks_passed").is_some());
        assert!(result.get("checks_failed").is_some());
        assert!(result.get("total_checks").is_some());
        let serialized = serde_json::to_string(&result).expect("Output should serialize");
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_health_check_json_serialization() {
        let output = serde_json::json!({
            "healthy": false,
            "family_id": "test",
            "checks_passed": [],
            "checks_failed": ["beardog"],
            "total_checks": 1
        });
        let s = serde_json::to_string(&output).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(parsed["healthy"], false);
        assert_eq!(parsed["total_checks"], 1);
    }
}
