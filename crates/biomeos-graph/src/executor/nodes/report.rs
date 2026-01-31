//! Deployment reporting node executors
//!
//! **TRUE ecoBin v2.0:** Simple, focused reporting.
//!
//! Node types handled:
//! - `report.deployment_success` - Generate deployment success report

use crate::executor::context::ExecutionContext;
use crate::executor::helpers::parse_config_optional;
use crate::graph::GraphNode;
use anyhow::Result;
use tracing::info;

/// Execute: report.deployment_success
///
/// Generates a deployment success report with metadata.
///
/// # Config Parameters
///
/// - `atomics_deployed` (optional): Array of atomic names deployed
/// - `message` (optional): Custom success message
///
/// # Returns
///
/// ```json
/// {
///   "success": true,
///   "atomics_deployed": ["tower", "node", "nest"],
///   "timestamp": "2026-01-30T18:00:00Z",
///   "message": "NUCLEUS deployed successfully"
/// }
/// ```
pub async fn deployment_success(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // Extract atomics deployed (optional)
    let atomics: Option<Vec<String>> = parse_config_optional(&node.config, "atomics_deployed")?;
    let atomics = atomics.unwrap_or_default();

    // Extract custom message (optional)
    let message: Option<String> = parse_config_optional(&node.config, "message")?;

    // Get family ID
    let family_id = context
        .env
        .get("FAMILY_ID")
        .cloned()
        .unwrap_or_else(|| "nat0".to_string());

    info!("📊 Deployment report: {} atomics deployed", atomics.len());

    // Generate report
    let mut report = serde_json::json!({
        "success": true,
        "atomics_deployed": atomics,
        "family_id": family_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // Add custom message if provided
    if let Some(msg) = message {
        report["message"] = serde_json::Value::String(msg);
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_deployment_success() {
        let node = GraphNode {
            id: "test_report".to_string(),
            node_type: "report.deployment_success".to_string(),
            config: serde_json::json!({
                "atomics_deployed": ["tower", "node", "nest"],
                "message": "Test deployment complete"
            }),
            dependencies: vec![],
        };

        let env = HashMap::from([("FAMILY_ID".to_string(), "nat0".to_string())]);
        let context = ExecutionContext::new(env);

        let result = deployment_success(&node, &context).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report["success"], true);
        assert_eq!(report["atomics_deployed"].as_array().unwrap().len(), 3);
        assert_eq!(report["family_id"], "nat0");
        assert!(report["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_deployment_success_minimal() {
        let node = GraphNode {
            id: "test_report".to_string(),
            node_type: "report.deployment_success".to_string(),
            config: serde_json::json!({}),
            dependencies: vec![],
        };

        let context = ExecutionContext::new(HashMap::new());
        let result = deployment_success(&node, &context).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report["success"], true);
    }
}
