// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tower Orchestration - Testable library logic for the tower binary.
//!
//! All business logic for PID file management, socket directory resolution,
//! primal conversion, and command dispatch lives here so it can be tested
//! without spawning the binary. The `tower` binary is a thin CLI wrapper
//! that delegates to these functions.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use biomeos_types::defaults::DEFAULT_FAMILY_ID;
use tracing::{error, info, warn};

use crate::{
    Capability, LogSessionTracker, PrimalBuilder, PrimalHealthMonitor, PrimalMetadata,
    PrimalOrchestrator, RetryPolicy, TowerConfig, TowerPrimalConfig, create_discovery_orchestrator,
    create_security_provider, discover_primals, start_in_waves,
};

/// Resolve the tower PID file path from the environment.
///
/// Precedence:
/// 1. `$XDG_RUNTIME_DIR/biomeos/tower.pid`
/// 2. `/tmp/biomeos-{family_id}/tower.pid` (tier 4 fallback)
pub fn pid_file_path(env: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    use biomeos_types::constants::runtime_paths;

    if let Some(runtime) = env("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime)
            .join(runtime_paths::BIOMEOS_SUBDIR)
            .join("tower.pid");
    }

    let family_id = env("BIOMEOS_FAMILY_ID")
        .or_else(|| env("FAMILY_ID"))
        .unwrap_or_else(|| DEFAULT_FAMILY_ID.to_string());

    runtime_paths::fallback_runtime_dir(&family_id).join("tower.pid")
}

/// Resolve the socket directory from the environment.
///
/// Precedence:
/// 1. `$BIOMEOS_SOCKET_DIR`
/// 2. `$XDG_RUNTIME_DIR/biomeos/sockets`
/// 3. `/tmp/biomeos-{family_id}/sockets` (tier 4 fallback)
pub fn socket_dir_path(env: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    use biomeos_types::constants::runtime_paths;

    if let Some(dir) = env("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir);
    }

    if let Some(runtime) = env("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime)
            .join(runtime_paths::BIOMEOS_SUBDIR)
            .join(runtime_paths::SOCKET_SUBDIR);
    }

    let family_id = env("BIOMEOS_FAMILY_ID")
        .or_else(|| env("FAMILY_ID"))
        .unwrap_or_else(|| DEFAULT_FAMILY_ID.to_string());

    runtime_paths::fallback_runtime_dir(&family_id).join(runtime_paths::SOCKET_SUBDIR)
}

/// Write a PID file for the running tower process.
pub fn write_pid_file(pid_file: &Path) -> Result<()> {
    if let Some(parent) = pid_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let pid = std::process::id();
    std::fs::write(pid_file, pid.to_string())?;

    info!("PID file written: {} (PID: {})", pid_file.display(), pid);
    Ok(())
}

/// Remove the PID file on shutdown.
pub fn cleanup_pid_file(pid_file: &Path) {
    if pid_file.exists() {
        if let Err(e) = std::fs::remove_file(pid_file) {
            warn!("Failed to remove PID file: {}", e);
        }
    }
}

/// Read the PID from a tower PID file.
pub fn read_pid(pid_file: &Path) -> Result<i32> {
    let content = std::fs::read_to_string(pid_file)
        .with_context(|| format!("Failed to read PID file: {}", pid_file.display()))?;
    content
        .trim()
        .parse::<i32>()
        .with_context(|| format!("Invalid PID in file: {content}"))
}

/// Convert `PrimalMetadata` to a `ManagedPrimal` via `PrimalBuilder`.
pub fn metadata_to_primal(metadata: &PrimalMetadata) -> Result<Arc<dyn crate::ManagedPrimal>> {
    let provides: Vec<Capability> = metadata
        .provides
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    let requires: Vec<Capability> = metadata
        .requires
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    let primal = PrimalBuilder::new()
        .binary_path(metadata.binary.display().to_string())
        .provides(provides)
        .requires(requires)
        .build()?;

    Ok(primal)
}

/// Convert a `TowerPrimalConfig` to a `ManagedPrimal`.
///
/// If `auto_discover` is enabled and no capabilities are specified,
/// queries the binary for its metadata.
pub async fn config_to_primal(config: &TowerPrimalConfig) -> Result<Arc<dyn crate::ManagedPrimal>> {
    let (provides_str, requires_str) =
        if config.auto_discover && config.provides.is_empty() && config.requires.is_empty() {
            let id = config
                .id
                .clone()
                .or_else(|| {
                    config
                        .binary
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(std::string::ToString::to_string)
                })
                .unwrap_or_else(|| "unknown".to_string());

            info!("Auto-discovering capabilities for {}", id);
            match crate::query_primal_metadata(&config.binary).await {
                Ok(metadata) => (metadata.provides, metadata.requires),
                Err(e) => {
                    warn!("Could not auto-discover capabilities: {}", e);
                    (config.provides.clone(), config.requires.clone())
                }
            }
        } else {
            (config.provides.clone(), config.requires.clone())
        };

    let provides: Vec<Capability> = provides_str
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    let requires: Vec<Capability> = requires_str
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    let mut builder = PrimalBuilder::new()
        .binary_path(config.binary.display().to_string())
        .provides(provides)
        .requires(requires);

    for (key, value) in &config.env {
        builder = builder.env_var(key.clone(), value.clone());
    }

    if let Some(protocol) = &config.protocol {
        builder = builder.env_var("IPC_PROTOCOL", protocol.as_str());
    }

    if config.http_port > 0 {
        builder = builder.http_port(config.http_port);
    }

    let primal = builder.build()?;
    Ok(primal)
}

/// Collect primals from config and optional scan directory.
pub async fn collect_primals(
    tower_config: &TowerConfig,
    scan_dir: Option<&Path>,
) -> Result<Vec<Arc<dyn crate::ManagedPrimal>>> {
    let mut all_primals: Vec<Arc<dyn crate::ManagedPrimal>> = Vec::new();

    if let Some(scan_dir) = scan_dir {
        info!("Auto-discovering primals from: {}", scan_dir.display());
        let discovered = discover_primals(scan_dir).await?;
        info!("Discovered {} primals", discovered.len());

        for metadata in discovered {
            let primal = metadata_to_primal(&metadata)?;
            all_primals.push(primal);
        }
    }

    for primal_config in &tower_config.primals {
        info!(
            "Loading primal from config: {}",
            primal_config.binary.display()
        );
        let primal = config_to_primal(primal_config).await?;
        all_primals.push(primal);
    }

    Ok(all_primals)
}

/// Run the tower's `run` command: load config, discover, start, monitor.
pub async fn run_tower(
    config_path: &Path,
    scan_dir: Option<PathBuf>,
    concurrent: bool,
    env: &(dyn Fn(&str) -> Option<String> + Send + Sync),
) -> Result<()> {
    info!("Starting tower with modern config-driven orchestration");

    let tower_config = if config_path.exists() {
        info!("Loading configuration from: {}", config_path.display());
        TowerConfig::from_file(config_path).context("Failed to load tower config")?
    } else {
        warn!("Config file not found, using defaults");
        TowerConfig::default_config()
    };

    let health_monitor = Arc::new(PrimalHealthMonitor::builder().build());
    let retry_policy = RetryPolicy::exponential(
        tower_config.health.recovery_attempts as usize,
        Duration::from_millis(100),
    );
    let orchestrator = Arc::new(PrimalOrchestrator::new(
        health_monitor.clone(),
        retry_policy,
    ));

    let all_primals = collect_primals(&tower_config, scan_dir.as_deref()).await?;

    if all_primals.is_empty() {
        error!("No primals configured or discovered!");
        return Ok(());
    }

    info!(
        "Registering {} primals with orchestrator",
        all_primals.len()
    );
    for primal in &all_primals {
        orchestrator.register(primal.clone()).await;
    }

    if concurrent && tower_config.tower.concurrent_startup {
        info!("Starting primals with concurrent wave-based orchestration");
        start_in_waves(&orchestrator, all_primals.clone()).await?;
    } else {
        info!("Starting primals sequentially");
        orchestrator.start_all().await?;
    }

    info!(
        "Tower started successfully with {} primals",
        all_primals.len()
    );

    let pid_file = pid_file_path(env);
    if let Err(e) = write_pid_file(&pid_file) {
        warn!("Failed to write PID file: {}", e);
    }

    tokio::spawn(async move {
        if let Err(e) = health_monitor.start_monitoring() {
            error!("Health monitoring failed: {}", e);
        }
    });

    let node_id = env("NODE_ID").unwrap_or_else(|| "unknown-node".to_string());
    let log_tracker = Arc::new(LogSessionTracker::new(node_id));

    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal, stopping tower...");

    cleanup_pid_file(&pid_file);

    if let Err(e) = log_tracker.archive_all_sessions("graceful_shutdown").await {
        warn!("Failed to archive log sessions: {}", e);
    }

    orchestrator.stop_all().await?;
    info!("Tower stopped gracefully.");

    Ok(())
}

/// Run the tower `start` command (legacy env-based startup).
pub async fn start_tower_legacy(
    security_binary: Option<String>,
    security_port: u16,
    discovery_binary: Option<String>,
    additional: Option<String>,
    env: &(dyn Fn(&str) -> Option<String> + Send + Sync),
) -> Result<()> {
    info!("Starting tower with capability-based orchestration...");

    let health_monitor = Arc::new(PrimalHealthMonitor::builder().build());
    let retry_policy = RetryPolicy::exponential(3, Duration::from_millis(100));
    let orchestrator = PrimalOrchestrator::new(health_monitor.clone(), retry_policy);

    if let Some(security_bin) = security_binary {
        info!("Registering security provider: {}", security_bin);
        let security = create_security_provider(security_bin, security_port)?;
        orchestrator.register(security).await;
    }

    if let Some(discovery_bin) = discovery_binary {
        info!("Registering discovery orchestrator: {}", discovery_bin);
        let discovery = create_discovery_orchestrator(discovery_bin)?;
        orchestrator.register(discovery).await;
    }

    if let Some(additional_bins) = additional {
        for bin_path in additional_bins.split(',') {
            let bin_path = bin_path.trim();
            if !bin_path.is_empty() {
                info!("Registering additional primal: {}", bin_path);
                let primal = PrimalBuilder::new()
                    .binary_path(bin_path.to_string())
                    .provides(Capability::from_env("PRIMAL_PROVIDES"))
                    .requires(Capability::from_env("PRIMAL_REQUIRES"))
                    .build()?;
                orchestrator.register(primal).await;
            }
        }
    }

    info!("Starting all primals with capability-based resolution...");
    orchestrator.start_all().await?;
    info!("Tower started successfully with zero-hardcoded configuration");

    let pid_file = pid_file_path(env);
    if let Err(e) = write_pid_file(&pid_file) {
        warn!("Failed to write PID file: {}", e);
    }

    tokio::spawn(async move {
        if let Err(e) = health_monitor.start_monitoring() {
            error!("Health monitoring failed: {}", e);
        }
    });

    let node_id = env("NODE_ID").unwrap_or_else(|| "unknown-node".to_string());
    let log_tracker = Arc::new(LogSessionTracker::new(node_id));

    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal, stopping tower...");

    cleanup_pid_file(&pid_file);

    if let Err(e) = log_tracker.archive_all_sessions("graceful_shutdown").await {
        warn!("Failed to archive log sessions: {}", e);
    }

    orchestrator.stop_all().await?;
    info!("Tower stopped gracefully.");

    Ok(())
}

/// Stop a running tower by reading its PID file and sending SIGTERM.
pub fn stop_tower(pid_file: &Path) -> Result<()> {
    if !pid_file.exists() {
        anyhow::bail!(
            "No running tower found (PID file not found: {})",
            pid_file.display()
        );
    }

    let pid = read_pid(pid_file)?;
    if pid <= 0 {
        anyhow::bail!("Invalid PID in file: {pid}");
    }

    info!("Sending SIGTERM to tower process (PID: {})", pid);

    #[cfg(unix)]
    {
        use std::process::Command;
        let status = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .context("Failed to send signal")?;

        if status.success() {
            info!("Sent stop signal to tower (PID: {})", pid);
        } else {
            warn!("Process {} may have already stopped", pid);
        }
        let _ = std::fs::remove_file(pid_file);
    }

    #[cfg(not(unix))]
    {
        anyhow::bail!("Stop command only supported on Unix systems");
    }

    Ok(())
}

/// Report status of a running tower.
pub fn tower_status(
    pid_file: &Path,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<TowerStatusReport> {
    if !pid_file.exists() {
        return Ok(TowerStatusReport::NotRunning);
    }

    let pid = read_pid(pid_file)?;
    if pid <= 0 {
        return Ok(TowerStatusReport::InvalidPid);
    }

    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "pid,command"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let socket_dir = socket_dir_path(env);
                let sockets = list_active_sockets(&socket_dir);
                let family = env("BIOMEOS_FAMILY_ID").or_else(|| env("FAMILY_ID"));

                Ok(TowerStatusReport::Running {
                    pid,
                    socket_dir,
                    sockets,
                    family_id: family,
                })
            }
            _ => {
                let _ = std::fs::remove_file(pid_file);
                Ok(TowerStatusReport::Stale { pid })
            }
        }
    }

    #[cfg(not(unix))]
    {
        Ok(TowerStatusReport::Running {
            pid,
            socket_dir: socket_dir_path(env),
            sockets: vec![],
            family_id: None,
        })
    }
}

/// Status report for a tower instance.
#[derive(Debug)]
pub enum TowerStatusReport {
    /// Tower is not running (no PID file).
    NotRunning,
    /// PID file exists but contains invalid data.
    InvalidPid,
    /// Tower process is running.
    Running {
        /// Process ID.
        pid: i32,
        /// Socket directory path.
        socket_dir: PathBuf,
        /// Active socket file names.
        sockets: Vec<String>,
        /// Family ID if set.
        family_id: Option<String>,
    },
    /// PID file exists but process is gone (stale).
    Stale {
        /// Stale PID that was cleaned up.
        pid: i32,
    },
}

/// List active `.sock` files in a directory.
fn list_active_sockets(socket_dir: &Path) -> Vec<String> {
    if !socket_dir.exists() {
        return Vec::new();
    }

    std::fs::read_dir(socket_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|x| x == "sock"))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Format the list of known capabilities for display.
#[must_use]
pub fn format_capabilities() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Security", "Crypto, signing, encryption, key management"),
        ("Discovery", "Service discovery, orchestration"),
        ("Compute", "Execution, processing, containers"),
        ("AI", "ML inference, neural networks"),
        ("Storage", "Content-addressed, distributed storage"),
        ("Observability", "Metrics, logging, tracing"),
        ("Federation", "Multi-org coordination"),
        ("Network", "NAT traversal, routing, mesh"),
    ]
}

/// Standard environment lookup that delegates to `std::env::var`.
#[must_use]
pub fn std_env_lookup(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

#[cfg(test)]
#[path = "tower_orchestration_tests.rs"]
mod tests;
