//! Primal lifecycle node executors
//!
//! **TRUE ecoBin v2.0:** Runtime discovery, no hardcoded paths.
//!
//! Node types handled:
//! - `primal.launch` - Launch a primal process
//!
//! ## Deep Debt Principles
//!
//! - **Self-knowledge only:** Primals don't know about each other's locations
//! - **Runtime discovery:** Binary paths resolved at runtime
//! - **Platform-agnostic:** Works on any platform
//! - **Capability-based:** Uses environment and standard locations

use crate::executor::context::{ExecutionContext, RollbackAction};
use crate::executor::helpers::{parse_config, substitute_env};
use crate::graph::GraphNode;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};

/// Execute: primal.launch
///
/// Launches a primal process in server mode.
///
/// **EVOLVED (Jan 27, 2026):** Complete implementation via process spawning.
///
/// # Config Parameters
///
/// - `primal` (required): Primal name (e.g., "beardog", "songbird")
/// - `binary_path` (optional): Explicit binary path (supports env vars)
///
/// # Returns
///
/// ```json
/// {
///   "launched": true,
///   "primal": "beardog",
///   "pid": 12345,
///   "socket": "/run/user/1000/biomeos/beardog.sock"
/// }
/// ```
pub async fn launch(node: &GraphNode, context: &ExecutionContext) -> Result<serde_json::Value> {
    // Extract primal name
    let primal_name: String = parse_config(&node.config, "primal")?;

    // Resolve binary path
    let binary_path = if let Ok(explicit_path) = parse_config::<String>(&node.config, "binary_path")
    {
        // User provided explicit path - substitute env vars
        substitute_env(&explicit_path, &context.env)
    } else {
        // Discover binary using standard locations
        resolve_primal_binary(&primal_name, &context.env)?
    };

    // Build socket path
    let family_id = context
        .env
        .get("FAMILY_ID")
        .cloned()
        .unwrap_or_else(|| "nat0".to_string());
    let socket_path = build_socket_path(&primal_name, &family_id, &context.env);

    info!("🚀 Launching primal: {} -> {}", primal_name, binary_path);
    debug!("   Socket path: {}", socket_path);

    // Spawn the primal process
    let mut cmd = Command::new(&binary_path);
    cmd.arg("server") // UniBin mode: "primal server"
        .arg("--socket")
        .arg(&socket_path)
        .stdout(Stdio::null()) // Don't capture stdout (let it log)
        .stderr(Stdio::null()); // Don't capture stderr

    // Pass family seed if available
    if let Some(seed_path) = context.env.get("BIOMEOS_FAMILY_SEED") {
        cmd.env("BIOMEOS_FAMILY_SEED", seed_path);
    }

    // Pass XDG_RUNTIME_DIR if available
    if let Some(runtime_dir) = context.env.get("XDG_RUNTIME_DIR") {
        cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    }

    // Launch!
    let child = cmd
        .spawn()
        .context(format!("Failed to spawn primal: {}", primal_name))?;

    let pid = child.id().unwrap_or(0);

    // Record rollback action for cleanup on failure
    context
        .record_rollback(
            &node.id,
            RollbackAction::StopProcess {
                primal: primal_name.clone(),
                pid,
                socket: socket_path.clone(),
            },
        )
        .await;

    info!("   ✅ Primal {} launched (PID: {})", primal_name, pid);

    Ok(serde_json::json!({
        "launched": true,
        "primal": primal_name,
        "pid": pid,
        "socket": socket_path
    }))
}

/// Resolve primal binary path
///
/// **TRUE ecoBin v2.0:** Runtime discovery, no hardcoded paths.
///
/// Priority order:
/// 1. Explicit environment variable (e.g., BEARDOG_BINARY)
/// 2. SPORE_ROOT/primals/{primal}
/// 3. plasmidBin/{primal}
/// 4. primals/{primal} (current directory)
fn resolve_primal_binary(primal_name: &str, env: &HashMap<String, String>) -> Result<String> {
    // Priority 1: Explicit environment variable
    let env_key = format!("{}_BINARY", primal_name.to_uppercase());
    if let Some(path) = env.get(&env_key) {
        debug!("Primal {} binary from env {}: {}", primal_name, env_key, path);
        return Ok(path.clone());
    }
    if let Ok(path) = std::env::var(&env_key) {
        debug!(
            "Primal {} binary from system env {}: {}",
            primal_name, env_key, path
        );
        return Ok(path);
    }

    // Priority 2: SPORE_ROOT/primals/{primal}
    if let Some(spore_root) = env
        .get("SPORE_ROOT")
        .or_else(|| std::env::var("SPORE_ROOT").ok().as_ref())
    {
        let path = format!("{}/primals/{}", spore_root, primal_name);
        if std::path::Path::new(&path).exists() {
            debug!("Primal {} binary from SPORE_ROOT: {}", primal_name, path);
            return Ok(path);
        }
    }

    // Priority 3: PRIMAL_DIR/{primal}
    if let Some(primal_dir) = env
        .get("PRIMAL_DIR")
        .or_else(|| std::env::var("PRIMAL_DIR").ok().as_ref())
    {
        let path = format!("{}/{}", primal_dir, primal_name);
        if std::path::Path::new(&path).exists() {
            debug!("Primal {} binary from PRIMAL_DIR: {}", primal_name, path);
            return Ok(path);
        }
    }

    // Priority 4: plasmidBin/stable/x86_64/primals/{primal}
    let plasmid_path = format!("plasmidBin/stable/x86_64/primals/{}", primal_name);
    if std::path::Path::new(&plasmid_path).exists() {
        debug!("Primal {} binary from plasmidBin: {}", primal_name, plasmid_path);
        return Ok(plasmid_path);
    }

    // Priority 5: Current directory primals/
    let local_path = format!("primals/{}", primal_name);
    if std::path::Path::new(&local_path).exists() {
        debug!("Primal {} binary from local primals/: {}", primal_name, local_path);
        return Ok(local_path);
    }

    anyhow::bail!(
        "Primal binary not found: {}. Set {}_BINARY, SPORE_ROOT, or PRIMAL_DIR.",
        primal_name,
        primal_name.to_uppercase()
    )
}

/// Build socket path for a primal
///
/// Uses XDG-compliant paths when possible.
fn build_socket_path(primal_name: &str, family_id: &str, env: &HashMap<String, String>) -> String {
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

    // Final fallback to /tmp (least preferred but works)
    format!("/tmp/{}-{}.sock", primal_name, family_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_primal_binary_from_env() {
        let env = HashMap::from([("BEARDOG_BINARY".to_string(), "/usr/bin/beardog".to_string())]);

        let result = resolve_primal_binary("beardog", &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/usr/bin/beardog");
    }

    #[test]
    fn test_resolve_primal_binary_missing() {
        let env = HashMap::new();

        let result = resolve_primal_binary("nonexistent_primal", &env);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_build_socket_path() {
        let env = HashMap::from([("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string())]);

        let socket = build_socket_path("beardog", "nat0", &env);
        assert_eq!(socket, "/run/user/1000/biomeos/beardog-nat0.sock");
    }

    #[test]
    fn test_build_socket_path_fallback() {
        let env = HashMap::new();

        let socket = build_socket_path("beardog", "nat0", &env);
        assert_eq!(socket, "/tmp/beardog-nat0.sock");
    }
}
