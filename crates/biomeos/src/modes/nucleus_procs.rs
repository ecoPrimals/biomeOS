// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! NUCLEUS startup utility functions.
//!
//! Process management, binary discovery, socket cleanup, and health checking.
//! Extracted from `nucleus.rs` for file size and separation of concerns.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

use super::{EcosystemState, build_primal_command};

/// Remove stale `.sock` files from the socket directory (R9).
///
/// Scans ALL `.sock` files in the directory. For each, attempts a non-blocking
/// Unix stream connect. If the connect fails (no listener), the socket file is
/// removed. This prevents consumers from discovering dead sockets left behind
/// by crashes or unclean shutdowns.
///
/// Called once on startup before `detect_ecosystem` to sanitize the directory.
/// See `CAPABILITY_BASED_DISCOVERY_STANDARD.md` §4 (crash recovery) and
/// `WETSPRING_UPSTREAM_BIOMEOS_STALE_SOCKETS_MAY18_2026.md`.
pub(super) async fn cleanup_stale_sockets(socket_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(socket_dir) else {
        return;
    };

    let mut removed = 0u32;
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        match ext {
            Some("sock") if std::os::unix::net::UnixStream::connect(&path).is_err() => {
                if std::fs::remove_file(&path).is_ok() {
                    removed += 1;
                    info!("  Removed stale socket: {}", path.display());
                }
                let pid_path = path.with_extension("pid");
                let _ = std::fs::remove_file(&pid_path);
            }
            Some("pid") if !path.with_extension("sock").exists() => {
                let _ = std::fs::remove_file(&path);
            }
            _ => {}
        }
    }
    if removed > 0 {
        info!(
            "  Cleaned up {removed} stale socket(s) from {}",
            socket_dir.display()
        );
    }
}

/// Detect whether an existing ecosystem is running.
///
/// Scans the socket directory for any `*-{family_id}.sock` files and health-
/// checks them. Does NOT iterate a hardcoded primal list — any primal that
/// follows the `{name}-{family_id}.sock` convention is discovered.
pub(super) async fn detect_ecosystem(socket_dir: &Path, family_id: &str) -> EcosystemState {
    if !socket_dir.exists() {
        return EcosystemState::Bootstrap;
    }

    let suffix = format!("-{family_id}.sock");
    let mut active = Vec::new();

    let Ok(entries) = std::fs::read_dir(socket_dir) else {
        return EcosystemState::Bootstrap;
    };

    for entry in entries.flatten() {
        let filename = entry.file_name();
        let name = filename.to_string_lossy();
        if let Some(primal) = name.strip_suffix(&suffix) {
            if primal.contains('.') {
                continue;
            }
            let socket_path = entry.path();
            match health_check(&socket_path).await {
                Ok(()) => {
                    info!("  Detected active {}", primal);
                    active.push(primal.to_string());
                }
                Err(_) => {
                    info!("  Stale socket for {} (will replace)", primal);
                }
            }
        }
    }

    if active.is_empty() {
        EcosystemState::Bootstrap
    } else {
        EcosystemState::Coordinated {
            active_primals: active,
        }
    }
}

/// Resolve the socket directory with an explicit override.
///
/// Uses the provided `socket_dir` override if set, otherwise delegates to
/// `SystemPaths::new_lazy()` for XDG-compliant runtime directory resolution.
pub(crate) fn resolve_socket_dir_with(socket_dir: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = socket_dir {
        return Ok(PathBuf::from(dir));
    }
    Ok(biomeos_types::paths::SystemPaths::new_lazy()
        .runtime_dir()
        .to_path_buf())
}

/// Discover primal binaries from known locations.
pub(super) fn discover_binaries(primals: &[&str]) -> Result<HashMap<String, PathBuf>> {
    let plasmid_bin_dir = biomeos_types::env_config::plasmid_bin_dir();
    let path_owned: Vec<PathBuf> = std::env::var(biomeos_types::env_config::vars::SYS_PATH)
        .ok()
        .map(|s| s.split(':').map(PathBuf::from).collect())
        .unwrap_or_default();
    let path_dirs: Vec<&Path> = path_owned.iter().map(std::path::PathBuf::as_path).collect();
    discover_binaries_with(primals, plasmid_bin_dir.as_deref(), &path_dirs, None)
}

pub(super) fn discover_search_path(rel: PathBuf, cwd: Option<&Path>) -> PathBuf {
    match cwd {
        Some(c) => c.join(rel),
        None => rel,
    }
}

/// `cwd`, when set, resolves relative search roots under that directory instead of the process cwd.
///
/// Search priority: ECOPRIMALS_PLASMID_BIN (depot) first when set, then local
/// plasmidBin dirs, then livespore-usb fallbacks, then $PATH.
pub(crate) fn discover_binaries_with(
    primals: &[&str],
    plasmid_bin_dir: Option<&Path>,
    path_dirs: &[&Path],
    cwd: Option<&Path>,
) -> Result<HashMap<String, PathBuf>> {
    let mut map = HashMap::new();

    let mut search_paths = Vec::new();

    // Depot (ECOPRIMALS_PLASMID_BIN) is highest priority when set
    if let Some(eco) = plasmid_bin_dir {
        search_paths.push(eco.join("primals"));
        search_paths.push(eco.to_path_buf());
    }

    // Local plasmidBin directories
    search_paths.push(discover_search_path(PathBuf::from("plasmidBin"), cwd));
    search_paths.push(discover_search_path(
        PathBuf::from("plasmidBin/optimized").join(std::env::consts::ARCH),
        cwd,
    ));
    search_paths.push(discover_search_path(
        PathBuf::from("../../plasmidBin/primals"),
        cwd,
    ));
    search_paths.push(discover_search_path(PathBuf::from("../../plasmidBin"), cwd));

    // livespore-usb fallback (lower priority than depot)
    search_paths.push(discover_search_path(
        PathBuf::from("livespore-usb")
            .join(std::env::consts::ARCH)
            .join("primals"),
        cwd,
    ));
    search_paths.push(discover_search_path(PathBuf::from("livespore-usb/primals"), cwd));

    for primal in primals {
        let mut found = false;
        for search in &search_paths {
            for candidate in &[search.join(primal), search.join(primal).join(primal)] {
                if candidate.exists() && candidate.is_file() {
                    map.insert(primal.to_string(), candidate.clone());
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if !found {
            for dir in path_dirs {
                let candidate = dir.join(primal);
                if candidate.is_file() {
                    map.insert(primal.to_string(), candidate);
                    found = true;
                    break;
                }
            }
        }
        if !found {
            warn!("Binary not found for {}", primal);
        }
    }

    Ok(map)
}

/// Start a primal process.
pub(super) async fn start_primal(
    name: &str,
    binary: &Path,
    socket_path: &Path,
    family_id: &str,
    node_id: &str,
    socket_dir: &Path,
) -> Result<tokio::process::Child> {
    let _ = tokio::fs::remove_file(socket_path).await;

    let std_cmd = build_primal_command(name, binary, socket_dir, family_id, node_id);
    let mut tokio_cmd = Command::new(std_cmd.get_program());
    tokio_cmd.args(std_cmd.get_args());
    for (k, v) in std_cmd.get_envs() {
        if let Some(v) = v {
            tokio_cmd.env(k, v);
        }
    }

    let log_dir = socket_dir.join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let stdout_path = log_dir.join(format!("{name}.stdout.log"));
    let stderr_path = log_dir.join(format!("{name}.stderr.log"));

    let child = if let (Ok(out), Ok(err)) = (
        std::fs::File::create(&stdout_path),
        std::fs::File::create(&stderr_path),
    ) {
        tokio_cmd.stdout(out).stderr(err).spawn()
    } else {
        warn!("Could not create log files for {name}, output discarded");
        tokio_cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
    }
    .with_context(|| format!("Failed to spawn {name}"))?;

    Ok(child)
}

/// Default poll interval when waiting for socket (100ms).
pub(super) const DEFAULT_SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Wait for a socket file to appear.
pub(super) async fn wait_for_socket(
    socket_path: &Path,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<()> {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if socket_path.exists() {
            return Ok(());
        }
        tokio::time::sleep(poll_interval).await;
    }
    Err(anyhow::anyhow!(
        "Socket {} did not appear within {:?}",
        socket_path.display(),
        timeout
    ))
}

/// Health check with exponential backoff: 100ms, 500ms, 1s, 2s.
/// Returns true if any attempt succeeds within the retry window.
pub(super) async fn health_check_with_backoff(socket_path: &Path) -> bool {
    const BACKOFF: [Duration; 4] = [
        Duration::from_millis(100),
        Duration::from_millis(500),
        Duration::from_secs(1),
        Duration::from_secs(2),
    ];
    for (i, delay) in BACKOFF.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(*delay).await;
        }
        if health_check(socket_path).await.is_ok() {
            return true;
        }
    }
    false
}

/// Basic health check via JSON-RPC.
pub(super) async fn health_check(socket_path: &Path) -> Result<()> {
    use biomeos_core::atomic_client::AtomicClient;

    let client = AtomicClient::unix(socket_path)
        .with_timeout(biomeos_types::constants::timeouts::DEFAULT_IPC_TIMEOUT);

    let response = if let Ok(resp) = client.call("health", serde_json::json!({})).await {
        resp
    } else {
        client
            .call("health.status", serde_json::json!({}))
            .await
            .context("Health check RPC failed")?
    };

    let _ = response.get("status").and_then(|s| s.as_str());
    Ok(())
}

/// Wait for SIGINT (Ctrl+C) or SIGTERM. Returns when either signal is received.
/// Handles both signals so NUCLEUS shuts down cleanly whether stopped interactively
/// or by a process manager (systemd, supervisord, etc.).
pub(super) async fn wait_for_shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => info!("Received SIGINT"),
            _ = sigterm.recv() => info!("Received SIGTERM"),
        }
    }

    #[cfg(not(unix))]
    {
        let _ = ctrl_c.await;
        info!("Received SIGINT");
    }
}

/// Generate a random JWT secret using the `rand` crate (no /dev/urandom read).
pub(super) fn generate_jwt_secret() -> String {
    use base64::Engine;
    use rand::RngCore;

    let mut bytes = [0u8; 48];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Auto-register all launched primals with the discovery provider.
///
/// Resolves the discovery provider via `BIOMEOS_NETWORK_PROVIDER` env or
/// `CapabilityTaxonomy::resolve_to_primal("discovery")`. For each primal,
/// probes `capabilities.list` on its socket and registers each capability
/// via `discovery.register_capability`. Falls back to taxonomy hints when
/// the probe returns empty.
///
/// Best-effort: logs warnings on failure but never aborts NUCLEUS startup.
pub(super) async fn auto_register_with_songbird(
    primals: &[&str],
    socket_dir: &Path,
    family_id: &str,
) {
    use biomeos_core::atomic_client::AtomicClient;
    use biomeos_core::socket_discovery::cap_probe::probe_unix_socket_capabilities_list;
    use biomeos_types::capability_taxonomy::helpers::capabilities_for_primal;

    let discovery_provider = std::env::var(biomeos_types::env_config::vars::NETWORK_PROVIDER)
        .ok()
        .or_else(|| {
            biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal("discovery")
                .map(String::from)
        });

    let Some(ref discovery_name) = discovery_provider else {
        warn!("No discovery provider resolved — skipping auto-registration");
        return;
    };

    let discovery_socket = socket_dir.join(format!("{discovery_name}-{family_id}.sock"));
    if !discovery_socket.exists() {
        warn!(
            "Discovery provider socket not found at {} — skipping auto-registration",
            discovery_socket.display()
        );
        return;
    }

    let discovery_client = AtomicClient::unix(&discovery_socket)
        .with_timeout(biomeos_types::constants::timeouts::NUCLEUS_REGISTRATION_TIMEOUT);

    let mut registered = 0usize;
    let mut failed = 0usize;

    for &primal in primals {
        if primal == discovery_name.as_str() {
            continue;
        }

        let primal_socket = socket_dir.join(format!("{primal}-{family_id}.sock"));
        let jsonrpc_socket = socket_dir.join(format!("{primal}-{family_id}.jsonrpc.sock"));
        let probe_socket = if jsonrpc_socket.exists() {
            &jsonrpc_socket
        } else {
            &primal_socket
        };

        let capabilities = if probe_socket.exists() {
            let probed = probe_unix_socket_capabilities_list(probe_socket).await;
            if probed.is_empty() {
                capabilities_for_primal(primal)
            } else {
                probed
            }
        } else {
            capabilities_for_primal(primal)
        };

        for capability in &capabilities {
            let params = serde_json::json!({
                "capability": capability,
                "endpoint": {
                    "type": "unix_socket",
                    "path": primal_socket.to_string_lossy(),
                },
                "metadata": {
                    "primal": primal,
                    "source": "nucleus-auto-register",
                    "family_id": family_id,
                }
            });

            match discovery_client
                .call("discovery.register_capability", params)
                .await
            {
                Ok(_) => {
                    registered += 1;
                }
                Err(e) => {
                    failed += 1;
                    warn!(
                        "Failed to register {primal}/{capability} with {discovery_name}: {e}"
                    );
                }
            }
        }
    }

    if registered > 0 {
        info!(
            "📡 Auto-registered {registered} capabilities with {discovery_name} ({failed} failed)"
        );
    } else if failed > 0 {
        warn!("{discovery_name} auto-registration: 0 succeeded, {failed} failed");
    }
}
