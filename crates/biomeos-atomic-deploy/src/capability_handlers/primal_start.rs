// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability-based primal start handler
//!
//! Starts primals via capability-based discovery, configures sockets,
//! and relays output to tracing.

use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};

use crate::executor::context::ExecutionContext;
use crate::neural_graph::GraphNode;

use super::discovery::{discover_primal_binary, resolve_capability_to_primal};
use crate::executor::primal_spawner::configure_primal_sockets;

/// Start a primal via capability-based discovery
///
/// This is the capability-based alternative to direct primal name starting.
/// It resolves capabilities to primals at runtime.
pub async fn primal_start_capability(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    info!("🚀 Starting primal via capability-based discovery");

    // Extract capability and operation parameters
    let capability = node
        .primal
        .as_ref()
        .and_then(|p| p.by_capability.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Missing primal.by_capability"))?;

    let operation = node
        .operation
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;

    let params = &operation.params;
    let mode = params
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("server");

    let family_id = params
        .get("family_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);

    debug!("   Capability: {}", capability);
    debug!("   Mode: {}", mode);
    debug!("   Family ID: {}", family_id);

    // 1. Capability → Primal Name Discovery
    let primal_name = match resolve_capability_to_primal(capability) {
        Some(name) => name,
        None => {
            warn!("Unknown capability '{}', skipping", capability);
            return Ok(json!({
                "started": false,
                "capability": capability,
                "error": format!("Unknown capability: {}", capability)
            }));
        }
    };

    // 2. Discover binary path
    let binary_full_path = match discover_primal_binary(primal_name, context).await {
        Ok(path) => path,
        Err(e) => {
            warn!("   Binary discovery failed for {}: {}", primal_name, e);
            return Ok(json!({
                "started": false,
                "capability": capability,
                "primal": primal_name,
                "error": format!("Binary not found: {}", e)
            }));
        }
    };

    info!(
        "   Discovered: {} → {}",
        primal_name,
        binary_full_path.display()
    );

    // 3. Build socket path using nucleation
    let socket_path = context.get_socket_path(primal_name).await;

    // 4. Build command with primal-specific arguments (data-driven via primal_launch_profiles.toml)
    let mut cmd = tokio::process::Command::new(&binary_full_path);
    cmd.arg(mode);

    // TCP-only cascade: when parent biomeOS runs --tcp-only, child primals
    // bind TCP instead of UDS (Android/Windows/cross-gate deployment).
    let tcp_port = if context.tcp_only {
        let port = context.next_tcp_port();
        cmd.arg("--port").arg(port.to_string());
        cmd.env("PRIMAL_TRANSPORT", "tcp");
        cmd.env("PRIMAL_TCP_PORT", port.to_string());
        info!(
            "   📡 TCP-only cascade: {} will bind TCP :{port}",
            primal_name
        );
        context.register_tcp_port(primal_name, port).await;
        Some(port)
    } else {
        None
    };

    configure_primal_sockets(&mut cmd, primal_name, &socket_path, &family_id, context).await;

    cmd.env("FAMILY_ID", &family_id);

    // Pass SSLKEYLOGFILE if set (for TLS debugging)
    if let Ok(sslkeylogfile) = std::env::var("SSLKEYLOGFILE") {
        if !sslkeylogfile.is_empty() {
            cmd.env("SSLKEYLOGFILE", &sslkeylogfile);
            info!("   🔐 Passing SSLKEYLOGFILE to primal: {}", sslkeylogfile);
        }
    }

    // Capture stdout/stderr for visibility
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    info!(
        "   Starting: {} {} (socket: {})",
        primal_name, mode, socket_path
    );

    // Pass environment variables from graph TOML (with ${VAR} substitution)
    if let Some(ref operation) = node.operation {
        if let Some(ref env_map) = operation.environment {
            info!(
                "   🔧 Passing {} environment variables to primal",
                env_map.len()
            );
            for (key, value) in env_map {
                let expanded =
                    crate::executor::substitute_env(value, &std::collections::HashMap::new());
                let expanded =
                    crate::executor::primal_spawner::substitute_from_process_env(&expanded);
                info!(
                    "   Setting env: {}={}",
                    key,
                    if key.contains("KEY") || key.contains("SEED") {
                        "***"
                    } else {
                        &expanded
                    }
                );
                cmd.env(key, &expanded);
            }
        }
    }

    // 5. Start process
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            error!("   Failed to spawn process: {}", e);
            return Ok(json!({
                "started": false,
                "capability": capability,
                "primal": primal_name,
                "error": format!("Failed to spawn: {}", e)
            }));
        }
    };

    let pid = child.id().unwrap_or(0);
    info!("   Process started: PID {}", pid);

    // 6. Relay stdout/stderr to logs
    spawn_output_relays(&mut child, primal_name);

    // 7. Wait for socket (TCP port in tcp_only mode, UDS path otherwise)
    let socket_confirmed = if let Some(port) = tcp_port {
        crate::executor::primal_spawner::wait_for_tcp_port(port, 300)
            .await
            .is_ok()
    } else {
        wait_for_socket_with_timeout(&socket_path, 30).await
    };

    if socket_confirmed {
        let endpoint_label = tcp_port
            .map(|p| format!("tcp://127.0.0.1:{p}"))
            .unwrap_or_else(|| socket_path.clone());
        info!("   ✅ Endpoint available: {}", endpoint_label);

        if !node.capabilities.is_empty() {
            info!(
                "   📝 Registering {} capabilities...",
                node.capabilities.len()
            );
            for cap in &node.capabilities {
                info!("      - {} → {} @ {}", cap, primal_name, endpoint_label);
            }
        }

        Ok(json!({
            "started": true,
            "capability": capability,
            "primal": primal_name,
            "mode": mode,
            "family_id": family_id,
            "pid": pid,
            "socket": socket_path,
            "tcp_port": tcp_port,
            "socket_confirmed": true
        }))
    } else {
        warn!("   ⚠️  Endpoint not available after timeout: {}", socket_path);

        Ok(json!({
            "started": true,
            "capability": capability,
            "primal": primal_name,
            "mode": mode,
            "family_id": family_id,
            "pid": pid,
            "socket": socket_path,
            "socket_confirmed": false,
            "warning": "Socket not detected within 3 seconds"
        }))
    }
}

/// Spawn tasks to relay child stdout/stderr to tracing
fn spawn_output_relays(child: &mut tokio::process::Child, primal_name: &str) {
    let stdout_name = primal_name.to_string();
    let stderr_name = primal_name.to_string();

    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                info!("[{}] {}", stdout_name, line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                warn!("[{}] {}", stderr_name, line);
            }
        });
    }
}

/// Wait for socket with timeout
pub async fn wait_for_socket_with_timeout(socket_path: &str, attempts: u32) -> bool {
    for attempt in 1..=attempts {
        if PathBuf::from(socket_path).exists() {
            debug!("   Socket available after {}00ms", attempt);
            return true;
        }
        sleep(Duration::from_millis(100)).await;
    }
    false
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::executor::context::ExecutionContext;
    use crate::neural_graph::{GraphNode, Operation, PrimalSelector};
    use std::collections::HashMap;
    fn create_beardog_stub(temp: &tempfile::TempDir) {
        let bin_path = temp.path().join("beardog");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::copy("/bin/true", &bin_path).expect("copy true");
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&bin_path, "").expect("write stub");
        }
    }

    fn make_node(
        capability: Option<&str>,
        mode: &str,
        family_id: Option<&str>,
        env_vars: Option<HashMap<String, String>>,
    ) -> GraphNode {
        let mut params = HashMap::new();
        params.insert("mode".to_string(), serde_json::json!(mode));
        if let Some(fid) = family_id {
            params.insert("family_id".to_string(), serde_json::json!(fid));
        }

        let operation = Some(Operation {
            name: "start".to_string(),
            params,
            environment: env_vars,
        });

        let primal = capability.map(|c| PrimalSelector {
            by_capability: Some(c.to_string()),
            by_name: None,
        });

        GraphNode {
            id: "test_node".to_string(),
            primal,
            operation,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_primal_start_capability_missing_by_capability() {
        let node = GraphNode {
            id: "test".to_string(),
            operation: Some(Operation {
                name: "start".to_string(),
                params: HashMap::new(),
                environment: None,
            }),
            ..Default::default()
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = primal_start_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when primal.by_capability is missing");
        assert!(
            err.to_string().contains("by_capability"),
            "Error should mention by_capability: {err}"
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_primal_without_by_capability() {
        let node = GraphNode {
            id: "test".to_string(),
            primal: Some(PrimalSelector {
                by_capability: None,
                by_name: Some("beardog".to_string()),
            }),
            operation: Some(Operation {
                name: "start".to_string(),
                params: HashMap::new(),
                environment: None,
            }),
            ..Default::default()
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = primal_start_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when by_capability is None");
        assert!(
            err.to_string().contains("by_capability"),
            "Error should mention by_capability: {err}"
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_missing_operation() {
        let node = GraphNode {
            id: "test".to_string(),
            primal: Some(PrimalSelector {
                by_capability: Some("encryption".to_string()),
                by_name: None,
            }),
            ..Default::default()
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = primal_start_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when operation is missing");
        assert!(
            err.to_string().contains("operation"),
            "Error should mention operation: {err}"
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_unknown_capability() {
        let node = make_node(Some("nonexistent_capability"), "server", None, None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = primal_start_capability(&node, &ctx)
            .await
            .expect("Unknown capability returns Ok with started: false");

        assert_eq!(
            result["started"], false,
            "Unknown capability should not start"
        );
        assert_eq!(result["capability"], "nonexistent_capability");
        assert!(
            result["error"]
                .as_str()
                .unwrap()
                .contains("Unknown capability"),
            "Error field should describe unknown capability"
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_binary_not_found() {
        let temp = tempfile::tempdir().expect("temp dir");

        let node = make_node(Some("encryption"), "server", None, None);
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx)
            .await
            .expect("Binary not found returns Ok with started: false");

        assert_eq!(result["started"], false);
        assert_eq!(result["capability"], "encryption");
        assert_eq!(result["primal"], "beardog");
        assert!(
            result["error"]
                .as_str()
                .unwrap()
                .contains("Binary not found"),
            "Error should indicate binary discovery failure"
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_mode_default() {
        let temp = tempfile::tempdir().expect("temp dir");

        create_beardog_stub(&temp);

        let node = make_node(Some("encryption"), "server", None, None);
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        assert_eq!(result["mode"], "server", "Default mode should be server");
    }

    #[tokio::test]
    async fn test_primal_start_capability_mode_from_params() {
        let temp = tempfile::tempdir().expect("temp dir");

        create_beardog_stub(&temp);

        let node = make_node(Some("encryption"), "client", None, None);
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        assert_eq!(result["mode"], "client");
    }

    #[tokio::test]
    async fn test_primal_start_capability_family_id_from_params() {
        let temp = tempfile::tempdir().expect("temp dir");

        create_beardog_stub(&temp);

        let node = make_node(
            Some("encryption"),
            "server",
            Some("custom_family_123"),
            None,
        );
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        assert_eq!(result["family_id"], "custom_family_123");
    }

    #[tokio::test]
    async fn test_primal_start_capability_output_json_structure() {
        let node = make_node(Some("unknown_cap"), "server", None, None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        assert!(result.get("started").is_some());
        assert!(result.get("capability").is_some());
        assert!(result.get("error").is_some());
        let serialized = serde_json::to_string(&result).expect("Output should serialize to JSON");
        assert!(!serialized.is_empty());
    }

    // -------------------------------------------------------------------------
    // wait_for_socket_with_timeout - Edge cases
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_wait_for_socket_exists_immediately() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        let path = temp.path().to_string_lossy().to_string();

        let found = wait_for_socket_with_timeout(&path, 5).await;

        assert!(found, "Should find socket that exists immediately");
    }

    #[tokio::test]
    async fn test_wait_for_socket_not_found() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp
            .path()
            .join("nonexistent.sock")
            .to_string_lossy()
            .to_string();

        let found = wait_for_socket_with_timeout(&path, 2).await;

        assert!(
            !found,
            "Should not find nonexistent socket within 2 attempts"
        );
    }

    #[tokio::test]
    async fn test_wait_for_socket_zero_attempts() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        let path = temp.path().to_string_lossy().to_string();

        let found = wait_for_socket_with_timeout(&path, 0).await;

        assert!(
            !found,
            "Zero attempts should not check (1..=0 is empty range)"
        );
    }

    #[test]
    fn test_json_output_serialization_roundtrip() {
        let output = serde_json::json!({
            "started": false,
            "capability": "encryption",
            "error": "Binary not found: test"
        });
        let s = serde_json::to_string(&output).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(parsed["started"], false);
        assert_eq!(parsed["capability"], "encryption");
    }
}
