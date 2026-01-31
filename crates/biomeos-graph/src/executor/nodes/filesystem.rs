//! Filesystem node executors
//!
//! **TRUE ecoBin v2.0:** Runtime path discovery, no hardcoding.
//!
//! Node types handled:
//! - `filesystem.check_exists` - Verify file/directory existence

use crate::executor::context::ExecutionContext;
use crate::executor::helpers::{parse_config, substitute_env};
use crate::graph::GraphNode;
use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;

/// Execute: filesystem.check_exists
///
/// Verifies that a file or directory exists at the specified path.
/// Optionally validates file size if specified.
///
/// # Config Parameters
///
/// - `path` (required): Path to check
/// - `expected_size` (optional): Expected file size in bytes
///
/// # Returns
///
/// ```json
/// {
///   "exists": true,
///   "path": "/actual/path"
/// }
/// ```
pub async fn check_exists(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // Extract path from config
    let path: String = parse_config(&node.config, "path")?;

    // Substitute environment variables (e.g., ${HOME}/config)
    let path = substitute_env(&path, &context.env);
    let path = PathBuf::from(path);

    debug!("Checking path exists: {}", path.display());

    // Check existence
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Optionally validate file size
    if let Some(expected_size) = node.config.get("expected_size").and_then(|v| v.as_u64()) {
        let metadata = std::fs::metadata(&path)?;
        let actual_size = metadata.len();

        if actual_size != expected_size {
            anyhow::bail!(
                "File size mismatch for {}: expected {}, got {}",
                path.display(),
                expected_size,
                actual_size
            );
        }

        debug!("   Size validated: {} bytes", actual_size);
    }

    Ok(serde_json::json!({
        "exists": true,
        "path": path.to_string_lossy()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_check_exists_valid_path() {
        let temp_file = std::env::temp_dir().join("test_check_exists.txt");
        std::fs::write(&temp_file, "test content").unwrap();

        let node = GraphNode {
            id: "test_node".to_string(),
            node_type: "filesystem.check_exists".to_string(),
            config: serde_json::json!({
                "path": temp_file.to_str().unwrap()
            }),
            dependencies: vec![],
        };

        let context = ExecutionContext::new(HashMap::new());
        let result = check_exists(&node, &context).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["exists"], true);

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_check_exists_missing_path() {
        let node = GraphNode {
            id: "test_node".to_string(),
            node_type: "filesystem.check_exists".to_string(),
            config: serde_json::json!({
                "path": "/nonexistent/path/to/nowhere"
            }),
            dependencies: vec![],
        };

        let context = ExecutionContext::new(HashMap::new());
        let result = check_exists(&node, &context).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not exist"));
    }

    #[tokio::test]
    async fn test_check_exists_with_env_substitution() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_env_sub.txt");
        std::fs::write(&temp_file, "content").unwrap();

        let env = HashMap::from([(
            "TEMP_DIR".to_string(),
            temp_dir.to_string_lossy().to_string(),
        )]);

        let node = GraphNode {
            id: "test_node".to_string(),
            node_type: "filesystem.check_exists".to_string(),
            config: serde_json::json!({
                "path": "${TEMP_DIR}/test_env_sub.txt"
            }),
            dependencies: vec![],
        };

        let context = ExecutionContext::new(env);
        let result = check_exists(&node, &context).await;

        assert!(result.is_ok());

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }
}
