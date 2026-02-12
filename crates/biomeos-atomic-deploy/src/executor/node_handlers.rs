//! Node execution handlers for Neural API graphs
//!
//! This module contains handlers for each node type in the graph execution system.
//! Each handler is a pure function that executes a specific operation.
//!
//! **Deep Debt Principle**: Node handlers are isolated, testable, and follow
//! capability-based discovery patterns - no hardcoded primal knowledge.

use anyhow::{Context as AnyhowContext, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info, warn};

use super::context::ExecutionContext;
use crate::neural_graph::GraphNode;

/// Execute a filesystem existence check
pub async fn filesystem_check_exists(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<Value> {
    let path = node
        .config
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("filesystem.check_exists requires 'path' config"))?;

    // Substitute environment variables
    let expanded_path = substitute_env(path, context.env());

    let exists = tokio::fs::metadata(&expanded_path).await.is_ok();

    info!("📁 Filesystem check: {} exists={}", expanded_path, exists);

    Ok(json!({
        "path": expanded_path,
        "exists": exists
    }))
}

/// Execute seed derivation via capability-based discovery
pub async fn crypto_derive_seed(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let source = node
        .config
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("family");

    let family_id = &context.family_id;

    // Capability-based: Find security provider via environment or discovery
    let security_socket = discover_capability_provider(context, "security").await;

    match security_socket {
        Some(socket_path) => {
            // Use BearDog for real crypto derivation
            info!(
                "🔐 Using security provider at {} for seed derivation",
                socket_path
            );

            let request = json!({
                "jsonrpc": "2.0",
                "method": "crypto.derive_seed",
                "params": {
                    "family_id": family_id,
                    "source": source
                },
                "id": 1
            });

            let response = call_primal_rpc(&socket_path, &request).await?;

            if let Some(result) = response.get("result") {
                Ok(result.clone())
            } else if let Some(err) = response.get("error") {
                anyhow::bail!("Crypto derive failed: {}", err);
            } else {
                anyhow::bail!("Invalid response from security provider");
            }
        }
        None => {
            // Fallback: Generate deterministic seed from family_id
            warn!("⚠️  No security provider found, using deterministic fallback");
            let seed = format!("seed-{}-{}", family_id, source);
            Ok(json!({
                "seed": seed,
                "derived_from": source,
                "method": "deterministic_fallback"
            }))
        }
    }
}

/// Execute primal launch via capability-based discovery
pub async fn primal_launch(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("primal.launch requires 'primal_name' config"))?;

    let mode = node
        .config
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("serve");

    let socket_path = context.get_socket_path(primal_name).await;

    info!("🚀 Launching primal: {} at {}", primal_name, socket_path);

    // Spawn the process using the spawner (handles discovery internally)
    let child =
        super::primal_spawner::spawn_primal_process(primal_name, mode, context, node).await?;

    // Wait for socket to be available (300 attempts = 30 seconds at 100ms each)
    super::primal_spawner::wait_for_socket(&socket_path, 300).await?;

    // Get the binary path for the response
    let binary_path = super::primal_spawner::discover_primal_binary(primal_name, context).await?;

    info!(
        "✅ Primal {} launched successfully (pid: {})",
        primal_name,
        child.id().unwrap_or(0)
    );

    Ok(json!({
        "primal": primal_name,
        "socket": socket_path,
        "binary": binary_path.display().to_string(),
        "status": "running"
    }))
}

/// Execute health check on a primal
pub async fn health_check(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("health_check requires 'primal_name' config"))?;

    let socket_path = context.get_socket_path(primal_name).await;
    let timeout_secs = node
        .config
        .get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(10);

    info!("🏥 Health check: {} at {}", primal_name, socket_path);

    let request = json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });

    // Call with timeout
    let response = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        call_primal_rpc(&socket_path, &request),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Health check timeout after {}s", timeout_secs))??;

    let healthy = response
        .get("result")
        .and_then(|r| r.get("healthy"))
        .and_then(|h| h.as_bool())
        .unwrap_or(false);

    if healthy {
        info!("✅ {} is healthy", primal_name);
    } else {
        warn!("⚠️  {} health check failed", primal_name);
    }

    Ok(json!({
        "primal": primal_name,
        "healthy": healthy,
        "response": response
    }))
}

/// Execute lineage verification via capability-based discovery
pub async fn lineage_verify(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("lineage.verify requires 'primal_name' config"))?;

    info!("🧬 Verifying lineage for: {}", primal_name);

    // Find security provider for verification
    let security_socket = discover_capability_provider(context, "security").await;

    match security_socket {
        Some(socket_path) => {
            let request = json!({
                "jsonrpc": "2.0",
                "method": "lineage.verify",
                "params": {
                    "primal_name": primal_name,
                    "family_id": context.family_id
                },
                "id": 1
            });

            let response = call_primal_rpc(&socket_path, &request).await?;

            if let Some(result) = response.get("result") {
                info!("✅ Lineage verified for {}", primal_name);
                Ok(result.clone())
            } else {
                warn!("⚠️  Lineage verification response missing result");
                Ok(json!({
                    "verified": false,
                    "error": "No result in response"
                }))
            }
        }
        None => {
            warn!("⚠️  No security provider for lineage verification, assuming valid");
            Ok(json!({
                "verified": true,
                "method": "assumed_valid_no_provider"
            }))
        }
    }
}

/// Log info message
pub async fn log_info(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let message = node
        .config
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");

    let expanded = substitute_env(message, context.env());
    info!("📝 {}", expanded);

    Ok(json!({
        "level": "info",
        "message": expanded
    }))
}

/// Log warning message
pub async fn log_warn(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let message = node
        .config
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");

    let expanded = substitute_env(message, context.env());
    warn!("⚠️  {}", expanded);

    Ok(json!({
        "level": "warn",
        "message": expanded
    }))
}

/// Log error message
pub async fn log_error(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let message = node
        .config
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");

    let expanded = substitute_env(message, context.env());
    error!("❌ {}", expanded);

    Ok(json!({
        "level": "error",
        "message": expanded
    }))
}

/// Generate deployment report
pub async fn deployment_report(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let title = node
        .config
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Deployment Report");

    let statuses = context.all_statuses().await;

    let completed = statuses
        .values()
        .filter(|s| matches!(s, super::context::NodeStatus::Completed(_)))
        .count();

    let failed = statuses
        .values()
        .filter(|s| matches!(s, super::context::NodeStatus::Failed(_)))
        .count();

    info!("📊 {}: {} completed, {} failed", title, completed, failed);

    Ok(json!({
        "title": title,
        "completed": completed,
        "failed": failed,
        "total": statuses.len(),
        "success": failed == 0
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Substitute environment variables in a string
///
/// Supports both ${VAR} and $VAR syntax.
pub fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
    let mut result = s.to_string();

    for (key, value) in env {
        result = result.replace(&format!("${{{}}}", key), value);
        result = result.replace(&format!("${}", key), value);
    }

    result
}

/// Discover a capability provider via environment-based discovery
///
/// **Deep Debt Principle**: No hardcoded primal names - only capabilities.
/// Discovery order:
/// 1. Environment variable ({CAPABILITY}_SOCKET or {CAPABILITY}_ENDPOINT)
/// 2. (REMOVED) Well-known primal names - evolved to runtime-only discovery
///
/// Note: This function no longer falls back to hardcoded primal names.
/// Users must explicitly configure capability providers via environment variables
/// or ensure the Neural API capability registry is accessible for runtime discovery.
async fn discover_capability_provider(
    context: &ExecutionContext,
    capability: &str,
) -> Option<String> {
    let cap_upper = capability.to_uppercase();

    // 1. Check environment for explicit socket
    if let Some(socket) = context.env().get(&format!("{}_SOCKET", cap_upper)) {
        if tokio::fs::metadata(socket).await.is_ok() {
            debug!(
                "Found {} provider via {}_SOCKET: {}",
                capability, cap_upper, socket
            );
            return Some(socket.clone());
        }
    }

    // 2. Check for endpoint environment variable
    if let Some(endpoint) = context.env().get(&format!("{}_ENDPOINT", cap_upper)) {
        debug!(
            "Found {} provider via {}_ENDPOINT: {}",
            capability, cap_upper, endpoint
        );
        return Some(endpoint.clone());
    }

    // 3. No hardcoded fallback - require explicit configuration
    // EVOLVED: Removed hardcoded capability→primal mappings
    // TRUE PRIMAL principle: No compile-time knowledge of specific primal names
    debug!(
        "No provider found for capability '{}'. \
         Resolution options: \
         (1) Start a primal providing '{}' capability, \
         (2) Set {}_ENDPOINT environment variable, or \
         (3) Ensure Neural API capability registry is accessible.",
        capability,
        capability,
        capability.to_uppercase()
    );

    None
}

/// Call a primal via JSON-RPC over Unix socket
///
/// **Deep Debt Principle**: Pure JSON-RPC, no HTTP dependencies.
async fn call_primal_rpc(socket_path: &str, request: &Value) -> Result<Value> {
    let stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Failed to connect to {}", socket_path))?;

    let (read_half, mut write_half) = stream.into_split();

    // Send request
    let request_json = serde_json::to_string(request)?;
    write_half.write_all(request_json.as_bytes()).await?;
    write_half.write_all(b"\n").await?;
    write_half.flush().await?;

    // Read response
    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: Value = serde_json::from_str(&response_line)
        .with_context(|| format!("Invalid JSON response from {}", socket_path))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::super::context::NodeStatus;
    use super::*;
    use crate::neural_graph::GraphNode;

    // ========================================================================
    // substitute_env tests
    // ========================================================================

    #[test]
    fn test_substitute_env() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("FAMILY_ID".to_string(), "1894e909e454".to_string());

        assert_eq!(substitute_env("${FOO}", &env), "bar");
        assert_eq!(substitute_env("$FOO", &env), "bar");
        assert_eq!(
            substitute_env("prefix-${FAMILY_ID}-suffix", &env),
            "prefix-1894e909e454-suffix"
        );
    }

    #[test]
    fn test_substitute_env_missing() {
        let env = HashMap::new();
        assert_eq!(substitute_env("${MISSING}", &env), "${MISSING}");
    }

    #[test]
    fn test_substitute_env_multiple_vars() {
        let mut env = HashMap::new();
        env.insert("A".to_string(), "alpha".to_string());
        env.insert("B".to_string(), "beta".to_string());
        env.insert("C".to_string(), "gamma".to_string());

        assert_eq!(substitute_env("${A}/${B}/${C}", &env), "alpha/beta/gamma");
    }

    #[test]
    fn test_substitute_env_xdg_runtime_dir() {
        let mut env = HashMap::new();
        env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string());
        env.insert("FAMILY_ID".to_string(), "cf7e8729".to_string());

        assert_eq!(
            substitute_env("${XDG_RUNTIME_DIR}/biomeos/beardog-${FAMILY_ID}.sock", &env),
            "/run/user/1000/biomeos/beardog-cf7e8729.sock"
        );
    }

    #[test]
    fn test_substitute_env_empty_value() {
        let mut env = HashMap::new();
        env.insert("EMPTY".to_string(), "".to_string());

        assert_eq!(
            substitute_env("prefix-${EMPTY}-suffix", &env),
            "prefix--suffix"
        );
    }

    #[test]
    fn test_substitute_env_no_vars_in_string() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        assert_eq!(
            substitute_env("no variables here", &env),
            "no variables here"
        );
    }

    #[test]
    fn test_substitute_env_dollar_sign_syntax() {
        let mut env = HashMap::new();
        env.insert("PORT".to_string(), "8080".to_string());

        // $PORT syntax (without braces)
        assert_eq!(substitute_env("localhost:$PORT", &env), "localhost:8080");
    }

    #[test]
    fn test_substitute_env_repeated_var() {
        let mut env = HashMap::new();
        env.insert("HOST".to_string(), "gate2".to_string());

        assert_eq!(substitute_env("${HOST}:${HOST}", &env), "gate2:gate2");
    }

    // ========================================================================
    // Helper: create test GraphNode with config
    // ========================================================================

    fn test_node_with_config(id: &str, config: HashMap<String, serde_json::Value>) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            depends_on: vec![],
            primal: None,
            output: None,
            operation: None,
            constraints: None,
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config,
            outputs: vec![],
        }
    }

    fn test_context() -> ExecutionContext {
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test-family".to_string());
        ExecutionContext::new(env)
    }

    fn test_context_with_env(env: HashMap<String, String>) -> ExecutionContext {
        ExecutionContext::new(env)
    }

    // ========================================================================
    // log_info / log_warn / log_error tests
    // ========================================================================

    #[tokio::test]
    async fn test_log_info() {
        let node = test_node_with_config("log1", {
            let mut c = HashMap::new();
            c.insert("message".to_string(), json!("Hello from biomeOS"));
            c
        });
        let ctx = test_context();

        let result = log_info(&node, &ctx).await.unwrap();
        assert_eq!(result["level"], "info");
        assert_eq!(result["message"], "Hello from biomeOS");
    }

    #[tokio::test]
    async fn test_log_info_with_env_substitution() {
        let node = test_node_with_config("log2", {
            let mut c = HashMap::new();
            c.insert("message".to_string(), json!("Family: ${FAMILY_ID}"));
            c
        });
        let ctx = test_context();

        let result = log_info(&node, &ctx).await.unwrap();
        assert_eq!(result["message"], "Family: test-family");
    }

    #[tokio::test]
    async fn test_log_info_no_message() {
        let node = test_node_with_config("log3", HashMap::new());
        let ctx = test_context();

        let result = log_info(&node, &ctx).await.unwrap();
        assert_eq!(result["message"], "(no message)");
    }

    #[tokio::test]
    async fn test_log_warn() {
        let node = test_node_with_config("warn1", {
            let mut c = HashMap::new();
            c.insert("message".to_string(), json!("Something concerning"));
            c
        });
        let ctx = test_context();

        let result = log_warn(&node, &ctx).await.unwrap();
        assert_eq!(result["level"], "warn");
        assert_eq!(result["message"], "Something concerning");
    }

    #[tokio::test]
    async fn test_log_error() {
        let node = test_node_with_config("err1", {
            let mut c = HashMap::new();
            c.insert("message".to_string(), json!("Critical failure"));
            c
        });
        let ctx = test_context();

        let result = log_error(&node, &ctx).await.unwrap();
        assert_eq!(result["level"], "error");
        assert_eq!(result["message"], "Critical failure");
    }

    // ========================================================================
    // deployment_report tests
    // ========================================================================

    #[tokio::test]
    async fn test_deployment_report_empty() {
        let node = test_node_with_config("report1", {
            let mut c = HashMap::new();
            c.insert("title".to_string(), json!("Test Report"));
            c
        });
        let ctx = test_context();

        let result = deployment_report(&node, &ctx).await.unwrap();
        assert_eq!(result["title"], "Test Report");
        assert_eq!(result["completed"], 0);
        assert_eq!(result["failed"], 0);
        assert_eq!(result["total"], 0);
        assert_eq!(result["success"], true);
    }

    #[tokio::test]
    async fn test_deployment_report_with_completed_nodes() {
        let node = test_node_with_config("report2", {
            let mut c = HashMap::new();
            c.insert("title".to_string(), json!("NUCLEUS Deployment"));
            c
        });
        let ctx = test_context();

        // Simulate completed nodes
        ctx.set_status("beardog", NodeStatus::Completed(json!({"status": "ok"})))
            .await;
        ctx.set_status("songbird", NodeStatus::Completed(json!({"status": "ok"})))
            .await;

        let result = deployment_report(&node, &ctx).await.unwrap();
        assert_eq!(result["completed"], 2);
        assert_eq!(result["failed"], 0);
        assert_eq!(result["total"], 2);
        assert_eq!(result["success"], true);
    }

    #[tokio::test]
    async fn test_deployment_report_with_failures() {
        let node = test_node_with_config("report3", HashMap::new());
        let ctx = test_context();

        ctx.set_status("beardog", NodeStatus::Completed(json!({"status": "ok"})))
            .await;
        ctx.set_status("songbird", NodeStatus::Failed("Socket timeout".to_string()))
            .await;

        let result = deployment_report(&node, &ctx).await.unwrap();
        assert_eq!(result["title"], "Deployment Report"); // default title
        assert_eq!(result["completed"], 1);
        assert_eq!(result["failed"], 1);
        assert_eq!(result["total"], 2);
        assert_eq!(result["success"], false);
    }

    #[tokio::test]
    async fn test_deployment_report_mixed_statuses() {
        let node = test_node_with_config("report4", HashMap::new());
        let ctx = test_context();

        ctx.set_status("beardog", NodeStatus::Completed(json!({})))
            .await;
        ctx.set_status("songbird", NodeStatus::Running).await;
        ctx.set_status("toadstool", NodeStatus::Failed("OOM".to_string()))
            .await;
        ctx.set_status("nestgate", NodeStatus::Pending).await;
        ctx.set_status("squirrel", NodeStatus::Skipped).await;

        let result = deployment_report(&node, &ctx).await.unwrap();
        assert_eq!(result["completed"], 1);
        assert_eq!(result["failed"], 1);
        assert_eq!(result["total"], 5);
        assert_eq!(result["success"], false);
    }

    // ========================================================================
    // filesystem_check_exists tests
    // ========================================================================

    #[tokio::test]
    async fn test_filesystem_check_exists_present() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test_seed.bin");
        std::fs::write(&test_file, b"seed data").unwrap();

        let node = test_node_with_config("fs1", {
            let mut c = HashMap::new();
            c.insert(
                "path".to_string(),
                json!(test_file.to_string_lossy().to_string()),
            );
            c
        });
        let ctx = test_context();

        let result = filesystem_check_exists(&node, &ctx).await.unwrap();
        assert_eq!(result["exists"], true);
        assert_eq!(result["path"], test_file.to_string_lossy().to_string());
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_missing() {
        let node = test_node_with_config("fs2", {
            let mut c = HashMap::new();
            c.insert("path".to_string(), json!("/nonexistent/path/seed.bin"));
            c
        });
        let ctx = test_context();

        let result = filesystem_check_exists(&node, &ctx).await.unwrap();
        assert_eq!(result["exists"], false);
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_with_env_var() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("family.seed");
        std::fs::write(&test_file, b"seed").unwrap();

        let node = test_node_with_config("fs3", {
            let mut c = HashMap::new();
            c.insert("path".to_string(), json!("${SEED_DIR}/family.seed"));
            c
        });
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        env.insert(
            "SEED_DIR".to_string(),
            temp_dir.path().to_string_lossy().to_string(),
        );
        let ctx = test_context_with_env(env);

        let result = filesystem_check_exists(&node, &ctx).await.unwrap();
        assert_eq!(result["exists"], true);
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_missing_path_config() {
        let node = test_node_with_config("fs4", HashMap::new());
        let ctx = test_context();

        let result = filesystem_check_exists(&node, &ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path"));
    }

    // ========================================================================
    // discover_capability_provider tests
    // ========================================================================

    #[tokio::test]
    async fn test_discover_capability_provider_via_socket_env() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sock_path = temp_dir.path().join("beardog.sock");
        // Create a regular file (not a real socket, but metadata check uses exists)
        std::fs::write(&sock_path, b"").unwrap();

        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        env.insert(
            "SECURITY_SOCKET".to_string(),
            sock_path.to_string_lossy().to_string(),
        );
        let ctx = test_context_with_env(env);

        let result = discover_capability_provider(&ctx, "security").await;
        assert_eq!(result, Some(sock_path.to_string_lossy().to_string()));
    }

    #[tokio::test]
    async fn test_discover_capability_provider_via_endpoint_env() {
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        env.insert(
            "SECURITY_ENDPOINT".to_string(),
            "http://localhost:8080".to_string(),
        );
        let ctx = test_context_with_env(env);

        let result = discover_capability_provider(&ctx, "security").await;
        assert_eq!(result, Some("http://localhost:8080".to_string()));
    }

    #[tokio::test]
    async fn test_discover_capability_provider_none_found() {
        let ctx = test_context();

        let result = discover_capability_provider(&ctx, "unknown_capability").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_discover_capability_provider_socket_not_exists() {
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        env.insert(
            "SECURITY_SOCKET".to_string(),
            "/nonexistent/beardog.sock".to_string(),
        );
        let ctx = test_context_with_env(env);

        // Socket file doesn't exist, should fall through to endpoint check
        let result = discover_capability_provider(&ctx, "security").await;
        assert!(result.is_none());
    }

    // ========================================================================
    // crypto_derive_seed tests (fallback path — no security provider)
    // ========================================================================

    #[tokio::test]
    async fn test_crypto_derive_seed_fallback() {
        let node = test_node_with_config("derive1", {
            let mut c = HashMap::new();
            c.insert("source".to_string(), json!("tower"));
            c
        });
        let ctx = test_context();

        // No security socket configured → should use deterministic fallback
        let result = crypto_derive_seed(&node, &ctx).await.unwrap();
        assert_eq!(result["method"], "deterministic_fallback");
        assert_eq!(result["derived_from"], "tower");
        assert!(result["seed"].as_str().unwrap().contains("test-family"));
    }

    #[tokio::test]
    async fn test_crypto_derive_seed_default_source() {
        let node = test_node_with_config("derive2", HashMap::new());
        let ctx = test_context();

        let result = crypto_derive_seed(&node, &ctx).await.unwrap();
        assert_eq!(result["derived_from"], "family"); // default
    }

    // ========================================================================
    // lineage_verify tests (fallback path — no security provider)
    // ========================================================================

    #[tokio::test]
    async fn test_lineage_verify_no_provider() {
        let node = test_node_with_config("verify1", {
            let mut c = HashMap::new();
            c.insert("primal_name".to_string(), json!("beardog"));
            c
        });
        let ctx = test_context();

        let result = lineage_verify(&node, &ctx).await.unwrap();
        assert_eq!(result["verified"], true);
        assert_eq!(result["method"], "assumed_valid_no_provider");
    }

    #[tokio::test]
    async fn test_lineage_verify_missing_primal_name() {
        let node = test_node_with_config("verify2", HashMap::new());
        let ctx = test_context();

        let result = lineage_verify(&node, &ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("primal_name"));
    }

    // ========================================================================
    // health_check tests (error paths)
    // ========================================================================

    #[tokio::test]
    async fn test_health_check_missing_primal_name() {
        let node = test_node_with_config("hc1", HashMap::new());
        let ctx = test_context();

        let result = health_check(&node, &ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("primal_name"));
    }
}
