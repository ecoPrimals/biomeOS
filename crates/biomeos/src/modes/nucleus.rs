// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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

#[path = "nucleus_launch.rs"]
mod nucleus_launch;

use anyhow::{Context, Result};
use biomeos_types::defaults::env_vars::socket_env_key;
use biomeos_types::primal_names::{BEARDOG, NESTGATE, SONGBIRD, SQUIRREL, TOADSTOOL};
use nucleus_launch::load_nucleus_profiles;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn primals(self) -> Vec<&'static str> {
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
    resolve_startup_config_with(
        mode,
        node_id,
        family_id,
        std::env::var("BIOMEOS_SOCKET_DIR").ok().as_deref(),
    )
}

pub(crate) fn resolve_startup_config_with(
    mode: &str,
    node_id: &str,
    family_id: Option<&str>,
    socket_dir_override: Option<&str>,
) -> Result<StartupConfig> {
    let mode: NucleusMode = mode.parse()?;
    let family_id =
        family_id.map_or_else(biomeos_core::family_discovery::get_family_id, String::from);
    let socket_dir = resolve_socket_dir_with(socket_dir_override)?;
    Ok(StartupConfig {
        mode,
        node_id: node_id.to_string(),
        family_id,
        socket_dir,
    })
}

/// Resolve socket path for a capability using taxonomy-based discovery.
///
/// Delegates to `CapabilityTaxonomy::resolve_to_primal` without hardcoded
/// fallbacks — if the taxonomy can't resolve the capability, we return an
/// `unknown-{family_id}.sock` path that simply won't exist on disk,
/// triggering the appropriate "socket not found" error at connect time.
fn socket_path_for_capability(
    socket_dir: &std::path::Path,
    family_id: &str,
    capability: &str,
) -> PathBuf {
    let primal_name =
        biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability).unwrap_or("unknown");
    socket_dir.join(format!("{primal_name}-{family_id}.sock"))
}

/// Configuration for building a primal process command.
#[derive(Debug, Clone)]
pub(crate) struct PrimalCommandConfig<'a> {
    pub name: &'a str,
    pub binary: &'a std::path::Path,
    pub socket_dir: &'a std::path::Path,
    pub family_id: &'a str,
    pub node_id: &'a str,
    pub anthropic_api_key: Option<&'a str>,
    pub openai_api_key: Option<&'a str>,
    pub ai_http_providers: Option<&'a str>,
    /// When set, used instead of reading `AI_DEFAULT_MODEL` from the environment.
    pub ai_default_model: Option<&'a str>,
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
    let has_ai =
        std::env::var("ANTHROPIC_API_KEY").is_ok() || std::env::var("OPENAI_API_KEY").is_ok();
    let ai_providers = has_ai.then(|| {
        std::env::var("AI_HTTP_PROVIDERS").unwrap_or_else(|_| "anthropic,openai".to_string())
    });
    let anthropic = std::env::var("ANTHROPIC_API_KEY").ok();
    let openai = std::env::var("OPENAI_API_KEY").ok();
    let config = PrimalCommandConfig {
        name,
        binary,
        socket_dir,
        family_id,
        node_id,
        anthropic_api_key: anthropic.as_deref(),
        openai_api_key: openai.as_deref(),
        ai_http_providers: ai_providers.as_deref(),
        ai_default_model: None,
    };
    build_primal_command_with(&config)
}

pub(crate) fn build_primal_command_with(config: &PrimalCommandConfig<'_>) -> std::process::Command {
    let socket_path = config
        .socket_dir
        .join(format!("{}-{}.sock", config.name, config.family_id));
    let mut cmd = std::process::Command::new(config.binary);

    let self_socket_key = socket_env_key(config.name);
    cmd.env(&self_socket_key, socket_path.as_os_str());

    let profiles = load_nucleus_profiles();
    let profile = profiles.profiles.get(config.name);
    let defaults = &profiles.default;

    let subcommand = profile
        .and_then(|p| p.subcommand.as_deref())
        .or(defaults.subcommand.as_deref())
        .unwrap_or("server");
    cmd.arg(subcommand);

    let pass_socket = profile
        .and_then(|p| p.pass_socket_flag)
        .or(defaults.pass_socket_flag)
        .unwrap_or(true);
    if pass_socket {
        cmd.arg("--socket").arg(socket_path.as_os_str());
    }

    let pass_family_id = profile
        .and_then(|p| p.pass_family_id_flag)
        .or(defaults.pass_family_id_flag)
        .unwrap_or(false);
    if pass_family_id {
        cmd.arg("--family-id").arg(config.family_id);
    }

    // Capability-resolved socket env vars (e.g. SONGBIRD_SECURITY_PROVIDER → security socket)
    let cap_sockets = profile.map_or(&defaults.capability_sockets, |p| &p.capability_sockets);
    for (env_key, capability) in cap_sockets {
        let resolved = socket_path_for_capability(config.socket_dir, config.family_id, capability);
        cmd.env(env_key, &resolved);
    }

    // Static env vars from profile (with $family_id substitution)
    let env_vars = profile.map_or(&defaults.env_vars, |p| &p.env_vars);
    for (key, value) in env_vars {
        let resolved = value.replace("$family_id", config.family_id);
        cmd.env(key, &resolved);
    }

    // JWT secret generation
    let gen_jwt = profile
        .and_then(|p| p.generate_jwt_secret)
        .or(defaults.generate_jwt_secret)
        .unwrap_or(false);
    if gen_jwt {
        cmd.env("NESTGATE_JWT_SECRET", generate_jwt_secret());
    }

    // AI model passthrough
    let pass_ai_model = profile
        .and_then(|p| p.pass_ai_model)
        .or(defaults.pass_ai_model)
        .unwrap_or(false);
    if pass_ai_model {
        if let Some(model) = config.ai_default_model {
            cmd.env("AI_DEFAULT_MODEL", model);
        } else if let Ok(model) = std::env::var("AI_DEFAULT_MODEL") {
            cmd.env("AI_DEFAULT_MODEL", model);
        }
    }

    // AI HTTP providers passthrough
    let pass_ai_providers = profile
        .and_then(|p| p.pass_ai_providers)
        .or(defaults.pass_ai_providers)
        .unwrap_or(false);
    if pass_ai_providers && (config.anthropic_api_key.is_some() || config.openai_api_key.is_some())
    {
        cmd.env(
            "AI_HTTP_PROVIDERS",
            config.ai_http_providers.unwrap_or("anthropic,openai"),
        );
    }

    cmd.env("FAMILY_ID", config.family_id)
        .env("NODE_ID", config.node_id);
    cmd
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

/// Run the nucleus startup
#[expect(clippy::too_many_lines, reason = "nucleus startup flow")]
pub async fn run(
    mode: String,
    node_id: String,
    family_id: Option<String>,
    tcp_port: Option<u16>,
    tcp_only: bool,
) -> Result<()> {
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

        // Wait for socket to appear (use health_socket for primals with separate JSON-RPC sockets)
        wait_for_socket(
            &health_socket,
            Duration::from_secs(10),
            DEFAULT_SOCKET_POLL_INTERVAL,
        )
        .await?;

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

        // Primals using .jsonrpc.sock typically expose `health.status` rather
        // than the legacy `health` method. Register the namespaced method.
        if health_socket != socket_path {
            lifecycle.set_health_method(primal, "health.status").await;
        }

        children.push((primal.to_string(), child));
    }

    // Start background health monitoring (checks all registered primals periodically)
    lifecycle.start_monitoring().await?;

    // In Full mode, start the Neural API server alongside the primals so that
    // graph.deploy, capability.call, and composition health are reachable.
    // Without this, biomeOS appears DOWN to external probes.
    if mode == NucleusMode::Full {
        let graphs_dir = PathBuf::from("graphs");
        let neural_socket = socket_dir.join(format!("neural-api-{family_id}.sock"));
        info!("Starting Neural API server (Full NUCLEUS)...");
        if tcp_only {
            info!("  Transport: TCP-only (port {})", tcp_port.unwrap_or(0));
        } else if let Some(port) = tcp_port {
            info!("  Socket: {}", neural_socket.display());
            info!("  TCP Port: {port} (alongside UDS)");
        } else {
            info!("  Socket: {}", neural_socket.display());
        }
        let neural_family = family_id.clone();
        let neural_tcp_port = tcp_port;
        let neural_tcp_only = tcp_only;
        tokio::spawn(async move {
            if let Err(e) = super::neural_api::run(
                graphs_dir,
                neural_family,
                Some(neural_socket),
                neural_tcp_port,
                neural_tcp_only,
                false,
            )
            .await
            {
                tracing::error!("Neural API server exited with error: {e}");
            }
        });
        tokio::time::sleep(Duration::from_millis(500)).await;
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

    // Keep running until interrupted
    info!("NUCLEUS running with lifecycle monitoring. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;

    // Coordinated shutdown via lifecycle manager
    info!("Shutting down NUCLEUS...");
    lifecycle.shutdown_all().await?;

    // Clean up child process handles
    for (name, mut child) in children {
        if tokio::time::timeout(Duration::from_secs(2), child.wait())
            .await
            .is_ok()
        {
            info!("  {} exited", name);
        } else {
            let _ = child.kill().await;
            info!("  {} force-killed", name);
        }
    }

    info!("NUCLEUS stopped.");

    Ok(())
}

/// Detect whether an existing ecosystem is running.
///
/// Scans the socket directory for any `*-{family_id}.sock` files and health-
/// checks them. Does NOT iterate a hardcoded primal list — any primal that
/// follows the `{name}-{family_id}.sock` convention is discovered.
async fn detect_ecosystem(socket_dir: &std::path::Path, family_id: &str) -> EcosystemState {
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
            // Skip auxiliary sockets (e.g. `.jsonrpc.sock`)
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

/// Discover primal binaries from known locations
fn discover_binaries(primals: &[&str]) -> Result<HashMap<String, PathBuf>> {
    let plasmid_bin_dir = biomeos_types::env_config::plasmid_bin_dir();
    let path_owned: Vec<PathBuf> = std::env::var("PATH")
        .ok()
        .map(|s| s.split(':').map(PathBuf::from).collect())
        .unwrap_or_default();
    let path_dirs: Vec<&Path> = path_owned.iter().map(std::path::PathBuf::as_path).collect();
    discover_binaries_with(primals, plasmid_bin_dir.as_deref(), &path_dirs, None)
}

fn discover_search_path(rel: PathBuf, cwd: Option<&Path>) -> PathBuf {
    match cwd {
        Some(c) => c.join(rel),
        None => rel,
    }
}

/// `cwd`, when set, resolves relative search roots under that directory instead of the process cwd.
pub(crate) fn discover_binaries_with(
    primals: &[&str],
    plasmid_bin_dir: Option<&Path>,
    path_dirs: &[&Path],
    cwd: Option<&Path>,
) -> Result<HashMap<String, PathBuf>> {
    let mut map = HashMap::new();

    let mut search_paths = vec![
        discover_search_path(
            PathBuf::from("livespore-usb")
                .join(std::env::consts::ARCH)
                .join("primals"),
            cwd,
        ),
        discover_search_path(PathBuf::from("livespore-usb/primals"), cwd),
        discover_search_path(PathBuf::from("plasmidBin"), cwd),
        discover_search_path(
            PathBuf::from("plasmidBin/optimized").join(std::env::consts::ARCH),
            cwd,
        ),
    ];

    if let Some(eco) = plasmid_bin_dir {
        search_paths.push(eco.join("primals"));
        search_paths.push(eco.to_path_buf());
    }
    search_paths.push(discover_search_path(
        PathBuf::from("../../plasmidBin/primals"),
        cwd,
    ));
    search_paths.push(discover_search_path(PathBuf::from("../../plasmidBin"), cwd));
    search_paths.push(discover_search_path(PathBuf::from("target/release"), cwd));

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
const DEFAULT_SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Wait for a socket file to appear
async fn wait_for_socket(
    socket_path: &std::path::Path,
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

/// Basic health check via JSON-RPC
async fn health_check(socket_path: &std::path::Path) -> Result<()> {
    use biomeos_core::atomic_client::AtomicClient;

    let client = AtomicClient::unix(socket_path).with_timeout(Duration::from_secs(3));

    // Try plain "health" first (common legacy surface), then capability `health.status`
    // (registry maps `health.status` → `health.check`; avoids primal-prefixed method names)
    let response = if let Ok(resp) = client.call("health", serde_json::json!({})).await {
        resp
    } else {
        client
            .call("health.status", serde_json::json!({}))
            .await
            .context("Health check RPC failed")?
    };

    // Accept any non-error response as healthy
    let _ = response.get("status").and_then(|s| s.as_str());
    Ok(())
}

/// Generate a random JWT secret using the `rand` crate (no /dev/urandom read).
fn generate_jwt_secret() -> String {
    use base64::Engine;
    use rand::RngCore;

    let mut bytes = [0u8; 48];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[cfg(test)]
#[path = "nucleus_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "nucleus_tests2.rs"]
mod tests2;

#[cfg(test)]
#[path = "nucleus_tests3.rs"]
mod tests3;

#[cfg(test)]
#[path = "nucleus_tests4.rs"]
mod tests4;
