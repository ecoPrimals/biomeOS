//! Health check node executors
//!
//! **TRUE ecoBin v2.0:** Runtime primal discovery for health checks.
//!
//! Node types handled:
//! - `health.check_atomic` - Check atomic (Tower/Node/Nest) health

use crate::executor::context::ExecutionContext;
use crate::executor::helpers::{parse_config_optional, substitute_env};
use crate::graph::GraphNode;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration, Instant};
use tracing::{debug, warn};

/// Execute: health.check_atomic
///
/// Checks the health of an atomic deployment (Tower, Node, or Nest).
///
/// # Config Parameters
///
/// - `atomic_type` (optional): Type of atomic ("tower", "node", "nest")
/// - `primal` (optional): Specific primal to check
///
/// # Returns
///
/// ```json
/// {
///   "healthy": true,
///   "atomic": "tower",
///   "primal": "beardog",
///   "socket": "/run/user/1000/biomeos/beardog.sock",
///   "response_time_ms": 12
/// }
/// ```
pub async fn check_atomic(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let atomic_type: Option<String> = parse_config_optional(&node.config, "atomic_type")?;
    let atomic_type = atomic_type.as_deref().unwrap_or("unknown");

    let primal_name: Option<String> = parse_config_optional(&node.config, "primal")?;

    // If primal specified, check its health
    if let Some(primal) = primal_name {
        let family_id = context
            .env
            .get("FAMILY_ID")
            .cloned()
            .unwrap_or_else(|| "nat0".to_string());
        
        let socket_path = build_socket_path(&primal, &family_id, &context.env);

        // Check if socket exists (basic health)
        let socket_exists = std::path::Path::new(&socket_path).exists();

        if socket_exists {
            // Try to ping the primal
            match ping_primal(&socket_path).await {
                Ok(response_time_ms) => {
                    debug!("   Primal {} healthy ({} ms)", primal, response_time_ms);
                    return Ok(serde_json::json!({
                        "healthy": true,
                        "atomic": atomic_type,
                        "primal": primal,
                        "socket": socket_path,
                        "response_time_ms": response_time_ms
                    }));
                }
                Err(e) => {
                    warn!("Primal {} health check failed: {}", primal, e);
                    return Ok(serde_json::json!({
                        "healthy": false,
                        "atomic": atomic_type,
                        "primal": primal,
                        "error": e.to_string()
                    }));
                }
            }
        } else {
            warn!("Primal {} socket not found: {}", primal, socket_path);
            return Ok(serde_json::json!({
                "healthy": false,
                "atomic": atomic_type,
                "primal": primal,
                "error": "Socket not found"
            }));
        }
    }

    // No specific primal - return basic healthy status
    Ok(serde_json::json!({
        "healthy": true,
        "atomic": atomic_type,
        "note": "No specific primal to check"
    }))
}

/// Ping a primal via its socket to check health
///
/// Sends a JSON-RPC health.ping request and measures response time.
async fn ping_primal(socket_path: &str) -> Result<u64> {
    let start = Instant::now();

    // Connect with timeout
    let stream = timeout(Duration::from_secs(5), UnixStream::connect(socket_path))
        .await
        .context("Connection timeout")?
        .context("Connection failed")?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send health ping request (JSON-RPC 2.0)
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.ping",
        "params": {},
        "id": 1
    });
    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    // Read response with timeout
    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .context("Response timeout")?
        .context("Read failed")?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    // Validate response is valid JSON-RPC
    let _response: serde_json::Value = serde_json::from_str(response_line.trim())?;

    Ok(elapsed_ms)
}

/// Build socket path for a primal (reused from primal.rs pattern)
fn build_socket_path(
    primal_name: &str,
    family_id: &str,
    env: &std::collections::HashMap<String, String>,
) -> String {
    // Try XDG-compliant path first
    if let Ok(paths) = biomeos_types::SystemPaths::new() {
        return paths
            .primal_socket(&format!("{}-{}", primal_name, family_id))
            .to_string_lossy()
            .to_string();
    }

    // Fallback to XDG_RUNTIME_DIR
    if let Some(runtime_dir) = env
        .get("XDG_RUNTIME_DIR")
        .or_else(|| std::env::var("XDG_RUNTIME_DIR").ok().as_ref())
    {
        return format!("{}/biomeos/{}-{}.sock", runtime_dir, primal_name, family_id);
    }

    // Final fallback
    format!("/tmp/{}-{}.sock", primal_name, family_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    #[ignore] // Requires running primal
    async fn test_check_atomic() {
        let node = GraphNode {
            id: "test_health".to_string(),
            node_type: "health.check_atomic".to_string(),
            config: serde_json::json!({
                "atomic_type": "tower",
                "primal": "beardog"
            }),
            dependencies: vec![],
        };

        let env = HashMap::from([("FAMILY_ID".to_string(), "nat0".to_string())]);
        let context = ExecutionContext::new(env);

        // This will fail unless primal is actually running
        let result = check_atomic(&node, &context).await;
        drop(result); // Just verify it compiles
    }
}
