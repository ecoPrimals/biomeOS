// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use super::nucleus_procs::{
    DEFAULT_SOCKET_POLL_INTERVAL, auto_register_with_discovery_provider, cleanup_stale_sockets,
    detect_ecosystem, discover_binaries, health_check_with_backoff, start_primal,
    wait_for_shutdown_signal, wait_for_socket,
};
use super::types::{
    EcosystemState, NucleusMode, resolve_startup_config, socket_path_for_capability,
};

/// Configuration for a NUCLEUS startup.
pub struct NucleusRunConfig {
    /// Deployment pattern (tower, node, nest, core, full).
    pub mode: String,
    /// Unique identifier for this node.
    pub node_id: String,
    /// Optional family identity for BTSP authentication.
    pub family_id: Option<String>,
    /// Optional TCP port override.
    pub tcp_port: Option<u16>,
    /// Whether to skip UDS and bind TCP only (SELinux/Android substrates).
    pub tcp_only: bool,
    /// Optional bind address override (e.g., "0.0.0.0").
    pub bind: Option<String>,
}

/// Format nucleus summary lines (pure, testable).
/// children: (name, pid) pairs.
pub(crate) fn format_nucleus_summary(
    children: &[(String, u32)],
    socket_dir: &std::path::Path,
    family_id: &str,
    node_id: &str,
    mode: NucleusMode,
    mode_label: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(format!("NUCLEUS started ({mode:?} mode, {mode_label})"));
    lines.push(format!("  Family:  {family_id}"));
    lines.push(format!("  Node:    {node_id}"));
    lines.push(format!("  Sockets: {}", socket_dir.display()));
    lines.push(format!("  Logs:    {}/logs/", socket_dir.display()));
    lines.push("  Health:  monitoring active (10s interval)".to_string());
    lines.push(String::new());
    for (name, pid) in children {
        let socket = socket_dir.join(format!("{name}-{family_id}.sock"));
        lines.push(format!("  {} (PID {}) -> {}", name, pid, socket.display()));
    }
    lines.push(String::new());
    let security_socket = socket_path_for_capability(socket_dir, family_id, "security");
    lines.push(format!(
        "Health check: echo '{{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"params\":{{}},\"id\":1}}' | nc -U {} -w 2 -q 1",
        security_socket.display()
    ));
    lines
}

/// Drain child processes and clean up their socket + PID files.
async fn shutdown_children(
    children: Vec<(String, tokio::process::Child)>,
    socket_dir: &Path,
    family_id: &str,
) {
    let started_names: Vec<String> = children.iter().map(|(n, _)| n.clone()).collect();

    for (name, mut child) in children {
        if tokio::time::timeout(
            biomeos_types::constants::timeouts::NUCLEUS_CHILD_REAP_TIMEOUT,
            child.wait(),
        )
        .await
        .is_ok()
        {
            info!("  {} exited", name);
        } else {
            let _ = child.kill().await;
            info!("  {} force-killed", name);
        }
    }

    for name in &started_names {
        let sock = socket_dir.join(format!("{name}-{family_id}.sock"));
        let pid_file = socket_dir.join(format!("{name}-{family_id}.pid"));
        if tokio::fs::remove_file(&sock).await.is_ok() {
            info!("  Cleaned up socket: {}", sock.display());
        }
        let _ = tokio::fs::remove_file(&pid_file).await;
    }
}

/// Run the nucleus startup
#[expect(clippy::too_many_lines, reason = "nucleus startup flow")]
pub async fn run(cfg: NucleusRunConfig) -> Result<()> {
    let tcp_only = cfg.tcp_only || biomeos_types::env_config::is_tcp_only_bind_mode();
    if tcp_only {
        info!(
            "TCP-only mode active — UDS skipped. \
             Required for SELinux/Android substrates."
        );
    }

    let config = resolve_startup_config(&cfg.mode, &cfg.node_id, cfg.family_id.as_deref())?;
    let mode = config.mode;
    let family_id = config.family_id;
    let socket_dir = config.socket_dir;
    let node_id = config.node_id;
    let tcp_port = cfg.tcp_port;
    let bind = cfg.bind;

    info!("Starting NUCLEUS in {:?} mode", mode);
    info!("  Node ID:   {}", node_id);
    info!("  Family ID: {}", family_id);
    info!("  Socket dir: {}", socket_dir.display());
    tokio::fs::create_dir_all(&socket_dir).await?;

    // R9: remove stale sockets left by previous crashes before probing.
    // This prevents consumers from hitting dead sockets (50+ observed by
    // wetSpring in production — see WETSPRING_UPSTREAM_BIOMEOS_STALE_SOCKETS).
    cleanup_stale_sockets(&socket_dir).await;

    // Bootstrap detection: check if ecosystem already exists
    let ecosystem = detect_ecosystem(&socket_dir, &family_id).await;
    let primals_needed = match &ecosystem {
        EcosystemState::Bootstrap => {
            info!("  Mode: BOOTSTRAP (no existing ecosystem detected)");
            mode.primals()
        }
        EcosystemState::Coordinated { active_primals } => {
            info!(
                "  Mode: COORDINATED ({} active primals: {})",
                active_primals.len(),
                active_primals.join(", ")
            );
            // Filter out primals that are already running
            let needed: Vec<&str> = mode
                .primals()
                .into_iter()
                .filter(|p| !active_primals.contains(&p.to_string()))
                .collect();
            if needed.is_empty() {
                info!("  All primals already running -- nothing to start");
                warn!("NUCLEUS already running with all required primals.");
                return Ok(());
            }
            info!("  Need to start: {:?}", needed);
            needed
        }
    };

    // Discover primal binaries
    let binary_map = discover_binaries(&primals_needed)?;

    info!("  Primals: {:?}", primals_needed);
    for (name, path) in &binary_map {
        info!("    {} -> {}", name, path.display());
    }

    // Create lifecycle manager for post-startup health monitoring
    let lifecycle = biomeos_atomic_deploy::lifecycle_manager::LifecycleManager::new(&family_id);

    // Start primals in dependency order
    let mut children: Vec<(String, tokio::process::Child)> = Vec::new();

    for primal in &primals_needed {
        let binary = binary_map
            .get(*primal)
            .ok_or_else(|| anyhow::anyhow!("Binary not found for primal: {primal}"))?;

        let socket_path = socket_dir.join(format!("{primal}-{family_id}.sock"));

        // Primals that expose tarpc on .sock provide a separate .jsonrpc.sock for
        // JSON-RPC health checks. Use the jsonrpc socket when it exists.
        let jsonrpc_socket = socket_dir.join(format!("{primal}-{family_id}.jsonrpc.sock"));
        let health_socket = if jsonrpc_socket.exists() {
            jsonrpc_socket
        } else {
            socket_path.clone()
        };

        info!("Starting {} ...", primal);

        let child = start_primal(
            primal,
            binary,
            &socket_path,
            &family_id,
            &node_id,
            &socket_dir,
        )
        .await
        .with_context(|| format!("Failed to start {primal}"))?;

        let pid = child.id();

        // Write PID file alongside socket so consumers can use kill(pid, 0)
        // for instant liveness checks without connect overhead (R9).
        if let Some(p) = pid {
            let pid_path = socket_dir.join(format!("{primal}-{family_id}.pid"));
            if let Err(e) = tokio::fs::write(&pid_path, p.to_string()).await {
                warn!("Failed to write PID file for {primal}: {e}");
            }
        }

        // Wait for socket to appear (use health_socket for primals with separate JSON-RPC sockets).
        // Non-fatal: on SELinux/Android, the primal may be alive on TCP without a UDS socket.
        let socket_appeared = wait_for_socket(
            &health_socket,
            biomeos_types::constants::timeouts::NUCLEUS_SOCKET_WAIT_TIMEOUT,
            DEFAULT_SOCKET_POLL_INTERVAL,
        )
        .await
        .is_ok();

        if socket_appeared {
            let health_ok = health_check_with_backoff(&health_socket).await;
            if health_ok {
                info!("  {} healthy (PID: {:?})", primal, pid);
            } else {
                warn!("{} health check failed after retries (incubating)", primal);
            }
        } else {
            warn!(
                "{} socket did not appear within timeout — primal may be running \
                 in TCP-only mode (SELinux/Android substrate)",
                primal
            );
        }

        // Register with lifecycle manager for ongoing monitoring and auto-restart
        lifecycle
            .register_primal_binary(
                *primal,
                health_socket.clone(),
                pid,
                binary.clone(),
                &node_id,
            )
            .await?;

        // Primals using .jsonrpc.sock typically expose `health.status` rather
        // than the legacy `health` method. Register the namespaced method.
        if health_socket != socket_path {
            lifecycle.set_health_method(primal, "health.status").await;
        }

        children.push((primal.to_string(), child));
    }

    // Start background health monitoring (checks all registered primals periodically)
    lifecycle.start_monitoring().await?;

    // Auto-register all launched primals with songBird's discovery service.
    // This makes the capability mesh operational without manual ipc.register calls.
    auto_register_with_discovery_provider(&primals_needed, &socket_dir, &family_id).await;

    // In Full mode, start the Neural API server alongside the primals so that
    // graph.deploy, capability.call, and composition health are reachable.
    // Without this, biomeOS appears DOWN to external probes.
    if mode == NucleusMode::Full {
        let graphs_dir = PathBuf::from("graphs");
        let neural_socket = socket_dir.join(format!("neural-api-{family_id}.sock"));
        info!("Starting Neural API server (Full NUCLEUS)...");
        if tcp_only {
            info!(
                "  Transport: TCP-only (port {}). UDS skipped — SELinux/Android substrate.",
                tcp_port.unwrap_or(0)
            );
        } else if let Some(port) = tcp_port {
            info!("  Socket: {}", neural_socket.display());
            info!("  TCP Port: {port} (alongside UDS)");
        } else {
            info!("  Socket: {}", neural_socket.display());
        }
        if let Some(ref addr) = bind {
            info!("  Bind Address: {addr}");
        }
        let neural_family = family_id.clone();
        let neural_tcp_port = tcp_port;
        let neural_tcp_only = tcp_only;
        let neural_bind = bind.clone();
        tokio::spawn(async move {
            if let Err(e) = super::super::neural_api::run(
                graphs_dir,
                neural_family,
                Some(neural_socket),
                neural_tcp_port,
                neural_tcp_only,
                neural_bind,
                false,
            )
            .await
            {
                tracing::error!("Neural API server exited with error: {e}");
            }
        });
        tokio::time::sleep(biomeos_types::constants::timeouts::NUCLEUS_POST_START_DELAY).await;

        // Trigger Neural API primal discovery to populate NeuralRouter.
        // Without this, capability.call has 0 registered providers since
        // NUCLEUS registers only with Songbird (mesh discovery), not NeuralRouter.
        let rescan_socket = socket_dir.join(format!("neural-api-{family_id}.sock"));
        tokio::spawn(async move {
            use biomeos_core::atomic_client::AtomicClient;
            let client = AtomicClient::unix(&rescan_socket)
                .with_timeout(biomeos_types::constants::timeouts::DEFAULT_IPC_TIMEOUT);
            match client.call("topology.rescan", serde_json::json!({})).await {
                Ok(resp) => {
                    let count = resp
                        .get("registered_capabilities")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0);
                    info!("Neural API rescan complete: {count} capabilities registered");
                }
                Err(e) => {
                    warn!("Neural API rescan failed (will retry on first capability.call): {e}");
                }
            }
        });
    }

    // Print summary
    let mode_label = match &ecosystem {
        EcosystemState::Bootstrap => "bootstrap",
        EcosystemState::Coordinated { .. } => "coordinated",
    };
    let children_pids: Vec<(String, u32)> = children
        .iter()
        .map(|(name, child)| (name.clone(), child.id().unwrap_or(0)))
        .collect();
    let summary_lines = format_nucleus_summary(
        &children_pids,
        &socket_dir,
        &family_id,
        &node_id,
        mode,
        mode_label,
    );
    for line in summary_lines {
        info!("{line}");
    }

    // Supervisor loop: stay alive until SIGINT or SIGTERM
    info!("NUCLEUS supervisor active. Send SIGINT or SIGTERM to stop.");
    wait_for_shutdown_signal().await;

    // Coordinated shutdown via lifecycle manager
    info!("Shutting down NUCLEUS...");
    if let Err(e) = lifecycle.shutdown_all().await {
        warn!("Lifecycle shutdown error (continuing cleanup): {e}");
    }

    shutdown_children(children, &socket_dir, &family_id).await;
    info!("NUCLEUS stopped.");

    Ok(())
}
