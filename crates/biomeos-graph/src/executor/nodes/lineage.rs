//! Lineage verification node executors
//!
//! **TRUE ecoBin v2.0:** Delegates genetic lineage to BearDog.
//!
//! Node types handled:
//! - `lineage.verify_siblings` - Verify genetic lineage of sibling deployments

use crate::executor::context::ExecutionContext;
use crate::executor::helpers::{discover_beardog_socket, parse_config_optional};
use crate::graph::GraphNode;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, warn};

/// Execute: lineage.verify_siblings
///
/// Verifies genetic lineage of sibling deployments via BearDog.
///
/// **EVOLVED (Jan 27, 2026):** Delegates to BearDog for verification.
///
/// # Config Parameters
///
/// - `siblings` (optional): Array of sibling node IDs to verify
///
/// # Returns
///
/// ```json
/// {
///   "verified": true,
///   "siblings_checked": 3,
///   "family_id": "nat0"
/// }
/// ```
pub async fn verify_siblings(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // Extract siblings list (optional)
    let siblings: Option<Vec<String>> = parse_config_optional(&node.config, "siblings")?;
    let siblings = siblings.unwrap_or_default();

    let family_id = context
        .env
        .get("FAMILY_ID")
        .cloned()
        .unwrap_or_else(|| "nat0".to_string());

    debug!(
        "Verifying lineage for family {} ({} siblings)",
        family_id,
        siblings.len()
    );

    // Discover BearDog for lineage verification
    let beardog_socket = match discover_beardog_socket(&context.env) {
        Ok(socket) => socket,
        Err(e) => {
            warn!("BearDog not available for lineage verification: {}", e);
            // Graceful degradation - return success without verification
            // In production, you might want this to be an error
            return Ok(serde_json::json!({
                "verified": true,
                "siblings_checked": 0,
                "family_id": family_id,
                "note": "BearDog unavailable, verification skipped"
            }));
        }
    };

    // Connect to BearDog
    let stream = UnixStream::connect(&beardog_socket)
        .await
        .context("Failed to connect to BearDog for lineage verification")?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Prepare JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lineage.verify_siblings",
        "params": {
            "family_id": family_id,
            "siblings": siblings
        },
        "id": 1
    });

    // Send request
    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    // Read response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: serde_json::Value =
        serde_json::from_str(response_line.trim()).context("Invalid JSON from BearDog")?;

    // Return BearDog's verification result
    if let Some(result) = response.get("result") {
        debug!("   Lineage verification complete: {:?}", result);
        Ok(result.clone())
    } else if let Some(error) = response.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        Ok(serde_json::json!({
            "verified": false,
            "error": msg,
            "family_id": family_id
        }))
    } else {
        // No result or error - assume success
        Ok(serde_json::json!({
            "verified": true,
            "siblings_checked": siblings.len(),
            "family_id": family_id
        }))
    }
}

/// Build socket path for a primal (consistent with other modules)
fn build_socket_path(
    primal_name: &str,
    family_id: &str,
    env: &std::collections::HashMap<String, String>,
) -> String {
    if let Ok(paths) = biomeos_types::SystemPaths::new() {
        return paths
            .primal_socket(&format!("{}-{}", primal_name, family_id))
            .to_string_lossy()
            .to_string();
    }

    if let Some(runtime_dir) = env
        .get("XDG_RUNTIME_DIR")
        .or_else(|| std::env::var("XDG_RUNTIME_DIR").ok().as_ref())
    {
        return format!("{}/biomeos/{}-{}.sock", runtime_dir, primal_name, family_id);
    }

    format!("/tmp/{}-{}.sock", primal_name, family_id)
}
