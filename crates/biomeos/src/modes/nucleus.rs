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
#[path = "nucleus_procs.rs"]
mod nucleus_procs;

use anyhow::{Context, Result};
use biomeos_types::defaults::env_vars::socket_env_key;
use biomeos_types::primal_names::{
    BARRACUDA, BEARDOG, CORALREEF, LOAMSPINE, NESTGATE, PETALTONGUE, RHIZOCRYPT, SKUNKBAT,
    SONGBIRD, SQUIRREL, SWEETGRASS, TOADSTOOL,
};
use nucleus_launch::load_nucleus_profiles;
use std::path::PathBuf;
use std::time::Duration;
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
    /// BearDog + Songbird + skunkBat (security + mesh + defense)
    Tower,
    /// Tower + ToadStool + barraCuda + coralReef (compute pipeline)
    Node,
    /// Tower + NestGate + rhizoCrypt + loamSpine + sweetGrass + Squirrel (storage + provenance)
    Nest,
    /// Core 5: BearDog + Songbird + NestGate + ToadStool + Squirrel (legacy compat)
    Core,
    /// All 13 primals + Neural API
    Full,
}

impl std::str::FromStr for NucleusMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tower" => Ok(NucleusMode::Tower),
            "node" => Ok(NucleusMode::Node),
            "nest" => Ok(NucleusMode::Nest),
            "core" => Ok(NucleusMode::Core),
            "full" | "nucleus" => Ok(NucleusMode::Full),
            _ => Err(anyhow::anyhow!(
                "Unknown nucleus mode: '{s}'. Use tower|node|nest|core|full"
            )),
        }
    }
}

impl NucleusMode {
    /// Get the primals needed for this mode (in startup order).
    ///
    /// Startup ordering: security first (bearDog), then mesh (songbird),
    /// then defense (skunkBat), then compute (toadstool, coralreef, barracuda),
    /// then storage/provenance (nestgate, rhizocrypt, loamspine, sweetgrass),
    /// then AI (squirrel), then UI (petaltongue).
    fn primals(self) -> Vec<&'static str> {
        match self {
            NucleusMode::Tower => vec![BEARDOG, SONGBIRD, SKUNKBAT],
            NucleusMode::Node => vec![BEARDOG, SONGBIRD, SKUNKBAT, TOADSTOOL, CORALREEF, BARRACUDA],
            NucleusMode::Nest => vec![
                BEARDOG, SONGBIRD, SKUNKBAT, NESTGATE, RHIZOCRYPT, LOAMSPINE, SWEETGRASS, SQUIRREL,
            ],
            NucleusMode::Core => vec![BEARDOG, SONGBIRD, NESTGATE, TOADSTOOL, SQUIRREL],
            NucleusMode::Full => vec![
                BEARDOG,
                SONGBIRD,
                SKUNKBAT,
                TOADSTOOL,
                CORALREEF,
                BARRACUDA,
                NESTGATE,
                RHIZOCRYPT,
                LOAMSPINE,
                SWEETGRASS,
                SQUIRREL,
                PETALTONGUE,
            ],
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
    bind: Option<String>,
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
        if let Some(ref addr) = bind {
            info!("  Bind Address: {addr}");
        }
        let neural_family = family_id.clone();
        let neural_tcp_port = tcp_port;
        let neural_tcp_only = tcp_only;
        let neural_bind = bind.clone();
        tokio::spawn(async move {
            if let Err(e) = super::neural_api::run(
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

    // Collect names before consuming children (for socket cleanup below)
    let started_names: Vec<String> = children.iter().map(|(n, _)| n.clone()).collect();

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

    // Remove socket + PID files for primals we launched (prevents stale
    // sockets if child processes didn't clean up on exit).
    for name in &started_names {
        let sock = socket_dir.join(format!("{name}-{family_id}.sock"));
        let pid_file = socket_dir.join(format!("{name}-{family_id}.pid"));
        if tokio::fs::remove_file(&sock).await.is_ok() {
            info!("  Cleaned up socket: {}", sock.display());
        }
        let _ = tokio::fs::remove_file(&pid_file).await;
    }

    info!("NUCLEUS stopped.");

    Ok(())
}

use nucleus_procs::{
    cleanup_stale_sockets, detect_ecosystem, discover_binaries, start_primal, wait_for_socket,
    DEFAULT_SOCKET_POLL_INTERVAL,
};
pub(crate) use nucleus_procs::{discover_binaries_with, resolve_socket_dir_with};
use nucleus_procs::{generate_jwt_secret, health_check};

#[cfg(test)]
use nucleus_procs::discover_search_path;

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
