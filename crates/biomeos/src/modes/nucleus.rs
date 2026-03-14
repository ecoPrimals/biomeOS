// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! NUCLEUS Startup Mode
//!
//! Pure Rust replacement for `start_nucleus.sh`.
//! Discovers primals, starts them in dependency order, registers capabilities.
//!
//! ## Bootstrap Detection
//!
//! Before launching, the nucleus checks if an existing ecosystem is already
//! running. This determines the startup strategy:
//!
//! - **Bootstrap Mode**: No existing BearDog socket found. biomeOS acts as the
//!   genesis orchestrator, starts all primals from scratch, and creates the
//!   initial capability registry.
//!
//! - **Coordinated Mode**: An existing BearDog socket is detected and responds
//!   to health checks. biomeOS joins the existing ecosystem, potentially
//!   starting only supplementary primals (e.g., adding Toadstool to an existing
//!   Tower).

use anyhow::{Context, Result};
use biomeos_types::primal_names::{self, BEARDOG, NESTGATE, SONGBIRD, SQUIRREL, TOADSTOOL};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

/// Detected ecosystem state at startup
#[derive(Debug)]
pub(crate) enum EcosystemState {
    /// No ecosystem detected -- we are the genesis orchestrator
    Bootstrap,
    /// Existing ecosystem detected with these active primals
    Coordinated { active_primals: Vec<String> },
}

/// NUCLEUS deployment pattern
#[derive(Debug, Clone, Copy)]
pub enum NucleusMode {
    /// BearDog + Songbird
    Tower,
    /// Tower + Toadstool
    Node,
    /// Tower + NestGate + Squirrel
    Nest,
    /// All primals + Neural API
    Full,
}

impl std::str::FromStr for NucleusMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tower" => Ok(NucleusMode::Tower),
            "node" => Ok(NucleusMode::Node),
            "nest" => Ok(NucleusMode::Nest),
            "full" | "nucleus" => Ok(NucleusMode::Full),
            _ => Err(anyhow::anyhow!(
                "Unknown nucleus mode: '{s}'. Use tower|node|nest|full"
            )),
        }
    }
}

impl NucleusMode {
    /// Get the primals needed for this mode (in startup order)
    fn primals(&self) -> Vec<&'static str> {
        match self {
            NucleusMode::Tower => vec![BEARDOG, SONGBIRD],
            NucleusMode::Node => vec![BEARDOG, SONGBIRD, TOADSTOOL],
            NucleusMode::Nest => vec![BEARDOG, SONGBIRD, NESTGATE, SQUIRREL],
            NucleusMode::Full => vec![BEARDOG, SONGBIRD, NESTGATE, TOADSTOOL, SQUIRREL],
        }
    }
}

/// Resolved startup configuration (pure, testable)
#[derive(Debug, Clone)]
pub(crate) struct StartupConfig {
    pub mode: NucleusMode,
    pub node_id: String,
    pub family_id: String,
    pub socket_dir: PathBuf,
}

/// Resolve startup configuration from mode string and optional overrides.
pub(crate) fn resolve_startup_config(
    mode: &str,
    node_id: &str,
    family_id: Option<&str>,
) -> Result<StartupConfig> {
    let mode: NucleusMode = mode.parse()?;
    let family_id = family_id
        .map(String::from)
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_dir = resolve_socket_dir()?;
    Ok(StartupConfig {
        mode,
        node_id: node_id.to_string(),
        family_id,
        socket_dir,
    })
}

/// Resolve socket path for a capability using taxonomy (not hardcoded primal names).
fn socket_path_for_capability(
    socket_dir: &std::path::Path,
    family_id: &str,
    capability: &str,
) -> PathBuf {
    let primal_name = biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability).unwrap_or(
        match capability {
            "security" | "encryption" => BEARDOG,
            "discovery" | "registry" => SONGBIRD,
            _ => "unknown",
        },
    );
    socket_dir.join(format!("{primal_name}-{family_id}.sock"))
}

/// Build a primal process command (testable, no spawn).
/// Returns std::process::Command for inspection and testing.
/// Socket paths use capability-based resolution via taxonomy.
pub(crate) fn build_primal_command(
    name: &str,
    binary: &std::path::Path,
    socket_dir: &std::path::Path,
    family_id: &str,
    node_id: &str,
) -> std::process::Command {
    let socket_path = socket_dir.join(format!("{name}-{family_id}.sock"));
    let mut cmd = std::process::Command::new(binary);

    match name {
        BEARDOG => {
            cmd.arg("server").arg("--socket").arg(&socket_path);
        }
        SONGBIRD => {
            let security_socket = socket_path_for_capability(socket_dir, family_id, "security");
            cmd.arg("server")
                .arg("--socket")
                .arg(&socket_path)
                .env("SONGBIRD_SECURITY_PROVIDER", &security_socket)
                .env("BIOMEOS_SECURITY_SOCKET", &security_socket)
                .env("BEARDOG_SOCKET", &security_socket); // Legacy; prefer BIOMEOS_SECURITY_SOCKET
        }
        NESTGATE => {
            cmd.arg("daemon")
                .arg("--socket-only")
                .arg("--family-id")
                .arg(family_id)
                .env("NESTGATE_JWT_SECRET", generate_jwt_secret());
        }
        TOADSTOOL => {
            cmd.arg("server")
                .arg("--socket")
                .arg(socket_path.as_os_str())
                .env("TOADSTOOL_SOCKET", socket_path.as_os_str())
                .env("TOADSTOOL_FAMILY_ID", family_id);
        }
        SQUIRREL => {
            let discovery_socket = socket_path_for_capability(socket_dir, family_id, "discovery");
            cmd.arg("server")
                .arg("--socket")
                .arg(socket_path.as_os_str())
                .env("SQUIRREL_SOCKET", socket_path.as_os_str())
                .env("BIOMEOS_DISCOVERY_SOCKET", &discovery_socket)
                .env("HTTP_REQUEST_PROVIDER_SOCKET", discovery_socket.as_os_str());
            if std::env::var("ANTHROPIC_API_KEY").is_ok() || std::env::var("OPENAI_API_KEY").is_ok()
            {
                cmd.env(
                    "AI_HTTP_PROVIDERS",
                    std::env::var("AI_HTTP_PROVIDERS")
                        .unwrap_or_else(|_| "anthropic,openai".to_string()),
                );
            }
        }
        _ => {
            cmd.arg("server").arg("--socket").arg(&socket_path);
        }
    }

    cmd.env("FAMILY_ID", family_id)
        .env("NODE_ID", node_id)
        .env("BEARDOG_NODE_ID", node_id);
    cmd
}

/// Format nucleus summary lines (pure, testable).
/// children: (name, pid) pairs.
pub(crate) fn format_nucleus_summary(
    children: &[(String, u32)],
    socket_dir: &std::path::Path,
    family_id: &str,
    node_id: &str,
    mode: &NucleusMode,
    mode_label: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(format!("NUCLEUS started ({mode:?} mode, {mode_label})"));
    lines.push(format!("  Family:  {family_id}"));
    lines.push(format!("  Node:    {node_id}"));
    lines.push(format!("  Sockets: {}", socket_dir.display()));
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

/// Run the nucleus startup
pub async fn run(mode: String, node_id: String, family_id: Option<String>) -> Result<()> {
    let config = resolve_startup_config(&mode, &node_id, family_id.as_deref())?;
    let mode = config.mode;
    let family_id = config.family_id;
    let socket_dir = config.socket_dir;
    let node_id = config.node_id;

    info!("Starting NUCLEUS in {:?} mode", mode);
    info!("  Node ID:   {}", node_id);
    info!("  Family ID: {}", family_id);
    info!("  Socket dir: {}", socket_dir.display());
    tokio::fs::create_dir_all(&socket_dir).await?;

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
                println!("NUCLEUS already running with all required primals.");
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

        // Toadstool exposes tarpc on .sock and JSON-RPC on .jsonrpc.sock
        // NUCLEUS health checks use JSON-RPC, so use the jsonrpc socket for health monitoring
        let health_socket = if *primal == TOADSTOOL {
            socket_dir.join(format!("{primal}-{family_id}.jsonrpc.sock"))
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

        // Wait for socket to appear (use health_socket for primals with separate JSON-RPC sockets)
        wait_for_socket(&health_socket, Duration::from_secs(10)).await?;

        // Health check via JSON-RPC
        if let Err(e) = health_check(&health_socket).await {
            warn!("{} health check failed: {} (continuing)", primal, e);
        } else {
            info!("  {} healthy (PID: {:?})", primal, pid);
        }

        // Register with lifecycle manager for ongoing monitoring (use health_socket for JSON-RPC pings)
        lifecycle
            .register_primal(
                *primal,
                health_socket.clone(),
                pid,
                None, // No deployment graph node (direct binary launch)
            )
            .await?;

        // Toadstool uses semantic method naming: "toadstool.health" instead of "health"
        if *primal == TOADSTOOL {
            lifecycle
                .set_health_method(TOADSTOOL, "toadstool.health")
                .await;
        }

        children.push((primal.to_string(), child));
    }

    // Start background health monitoring (checks all registered primals periodically)
    lifecycle.start_monitoring().await?;

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
        &mode,
        mode_label,
    );
    for line in summary_lines {
        println!("{line}");
    }

    // Keep running until interrupted
    info!("NUCLEUS running with lifecycle monitoring. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;

    // Coordinated shutdown via lifecycle manager
    info!("Shutting down NUCLEUS...");
    lifecycle.shutdown_all().await?;

    // Clean up child process handles
    for (name, mut child) in children {
        match tokio::time::timeout(Duration::from_secs(2), child.wait()).await {
            Ok(_) => info!("  {} exited", name),
            Err(_) => {
                let _ = child.kill().await;
                info!("  {} force-killed", name);
            }
        }
    }

    info!("NUCLEUS stopped.");

    Ok(())
}

/// Detect whether an existing ecosystem is running
///
/// Scans the socket directory for primal sockets matching the family ID.
/// If any respond to health checks, we're joining an existing ecosystem.
async fn detect_ecosystem(socket_dir: &std::path::Path, family_id: &str) -> EcosystemState {
    if !socket_dir.exists() {
        return EcosystemState::Bootstrap;
    }

    let known_primals = primal_names::CORE_PRIMALS;
    let mut active = Vec::new();

    for primal in known_primals {
        let socket_path = socket_dir.join(format!("{primal}-{family_id}.sock"));
        if socket_path.exists() {
            // Socket file exists -- try a health check
            match health_check(&socket_path).await {
                Ok(()) => {
                    info!("  Detected active {}", primal);
                    active.push(primal.to_string());
                }
                Err(_) => {
                    // Socket exists but primal isn't responding -- stale socket
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

/// Resolve the socket directory
///
/// Uses `BIOMEOS_SOCKET_DIR` env var if set, otherwise delegates to
/// `SystemPaths::new_lazy()` for XDG-compliant runtime directory resolution.
fn resolve_socket_dir() -> Result<PathBuf> {
    // Explicit override takes priority
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // XDG-compliant path via SystemPaths (handles XDG_RUNTIME_DIR, /run/user/$UID, /tmp fallbacks)
    Ok(biomeos_types::paths::SystemPaths::new_lazy()
        .runtime_dir()
        .to_path_buf())
}

/// Discover primal binaries from known locations
fn discover_binaries(primals: &[&str]) -> Result<HashMap<String, PathBuf>> {
    let mut map = HashMap::new();

    // Ecosystem-level plasmidBin (ecoPrimals/plasmidBin/) via env or path traversal
    let ecosystem_plasmid_bin = biomeos_types::env_config::plasmid_bin_dir();

    let mut search_paths = vec![
        // Current architecture livespore
        PathBuf::from("livespore-usb")
            .join(std::env::consts::ARCH)
            .join("primals"),
        // Generic livespore
        PathBuf::from("livespore-usb/primals"),
        // Local plasmidBin
        PathBuf::from("plasmidBin"),
        PathBuf::from("plasmidBin/optimized").join(std::env::consts::ARCH),
    ];

    // Ecosystem root plasmidBin (ecoPrimals/plasmidBin/) — reached from phase2/biomeOS/
    if let Some(ref eco) = ecosystem_plasmid_bin {
        search_paths.push(eco.join("primals"));
        search_paths.push(eco.clone());
    }
    search_paths.push(PathBuf::from("../../plasmidBin/primals"));
    search_paths.push(PathBuf::from("../../plasmidBin"));

    // Cargo build output
    search_paths.push(PathBuf::from("target/release"));

    for primal in primals {
        let mut found = false;
        for search in &search_paths {
            // Try direct match and primal/primal subdir pattern
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
            // Scan PATH directories (pure Rust, no `which` shell-out)
            if let Ok(path_var) = std::env::var("PATH") {
                for dir in path_var.split(':') {
                    let candidate = PathBuf::from(dir).join(primal);
                    if candidate.is_file() {
                        map.insert(primal.to_string(), candidate);
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            warn!("Binary not found for {}", primal);
        }
    }

    Ok(map)
}

/// Start a primal process
async fn start_primal(
    name: &str,
    binary: &std::path::Path,
    socket_path: &std::path::Path,
    family_id: &str,
    node_id: &str,
    socket_dir: &std::path::Path,
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

    let child = tokio_cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn {name}"))?;

    Ok(child)
}

/// Wait for a socket file to appear
async fn wait_for_socket(socket_path: &std::path::Path, timeout: Duration) -> Result<()> {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if socket_path.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(anyhow::anyhow!(
        "Socket {} did not appear within {:?}",
        socket_path.display(),
        timeout
    ))
}

/// Basic health check via JSON-RPC
async fn health_check(socket_path: &std::path::Path) -> Result<()> {
    use biomeos_core::atomic_client::AtomicClient;

    let client = AtomicClient::unix(socket_path).with_timeout(Duration::from_secs(3));

    // Try plain "health" first (BearDog, Songbird, NestGate, Squirrel),
    // then semantic "{primal}.health" (Toadstool follows the naming standard)
    let response = match client.call("health", serde_json::json!({})).await {
        Ok(resp) => resp,
        Err(_) => {
            // Extract primal name from socket path for semantic method naming
            let primal_name = socket_path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.split('-').next())
                .unwrap_or("unknown");
            let semantic_method = format!("{primal_name}.health");
            client
                .call(&semantic_method, serde_json::json!({}))
                .await
                .context("Health check RPC failed")?
        }
    };

    if response.get("status").and_then(|s| s.as_str()) == Some("healthy") {
        Ok(())
    } else {
        // Accept any non-error response as healthy
        Ok(())
    }
}

/// Generate a random JWT secret
fn generate_jwt_secret() -> String {
    use std::io::Read;
    let mut bytes = [0u8; 48];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = f.read_exact(&mut bytes);
    }
    base64_encode(&bytes)
}

/// Simple base64 encoding (no external dependency)
pub(crate) fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = if chunk.len() > 1 {
            u32::from(chunk[1])
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            u32::from(chunk[2])
        } else {
            0
        };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        let idx = |shift: u32| usize::try_from((triple >> shift) & 0x3F).unwrap_or(0);
        result.push(char::from(ALPHABET[idx(18)]));
        result.push(char::from(ALPHABET[idx(12)]));
        if chunk.len() > 1 {
            result.push(char::from(ALPHABET[idx(6)]));
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(char::from(ALPHABET[idx(0)]));
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[path = "nucleus_tests.rs"]
mod tests;
