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
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

/// Detected ecosystem state at startup
#[derive(Debug)]
enum EcosystemState {
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
                "Unknown nucleus mode: '{}'. Use tower|node|nest|full",
                s
            )),
        }
    }
}

impl NucleusMode {
    /// Get the primals needed for this mode (in startup order)
    fn primals(&self) -> Vec<&'static str> {
        match self {
            NucleusMode::Tower => vec!["beardog", "songbird"],
            NucleusMode::Node => vec!["beardog", "songbird", "toadstool"],
            NucleusMode::Nest => vec!["beardog", "songbird", "nestgate", "squirrel"],
            NucleusMode::Full => vec!["beardog", "songbird", "nestgate", "toadstool", "squirrel"],
        }
    }
}

/// Run the nucleus startup
pub async fn run(mode: String, node_id: String, family_id: Option<String>) -> Result<()> {
    let mode: NucleusMode = mode.parse()?;

    // Resolve family ID
    let family_id = family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);

    info!("Starting NUCLEUS in {:?} mode", mode);
    info!("  Node ID:   {}", node_id);
    info!("  Family ID: {}", family_id);

    // Resolve socket directory
    let socket_dir = resolve_socket_dir()?;
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
            .ok_or_else(|| anyhow::anyhow!("Binary not found for primal: {}", primal))?;

        let socket_path = socket_dir.join(format!("{}-{}.sock", primal, family_id));

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
        .with_context(|| format!("Failed to start {}", primal))?;

        let pid = child.id();

        // Wait for socket to appear
        wait_for_socket(&socket_path, Duration::from_secs(10)).await?;

        // Health check
        if let Err(e) = health_check(&socket_path).await {
            warn!("{} health check failed: {} (continuing)", primal, e);
        } else {
            info!("  {} healthy (PID: {:?})", primal, pid);
        }

        // Register with lifecycle manager for ongoing monitoring
        lifecycle
            .register_primal(
                *primal,
                socket_path.clone(),
                pid,
                None, // No deployment graph node (direct binary launch)
            )
            .await?;

        children.push((primal.to_string(), child));
    }

    // Start background health monitoring (checks all registered primals periodically)
    lifecycle.start_monitoring().await?;

    // Print summary
    let mode_label = match &ecosystem {
        EcosystemState::Bootstrap => "bootstrap",
        EcosystemState::Coordinated { .. } => "coordinated",
    };
    println!();
    println!("NUCLEUS started ({:?} mode, {})", mode, mode_label);
    println!("  Family:  {}", family_id);
    println!("  Node:    {}", node_id);
    println!("  Sockets: {}", socket_dir.display());
    println!("  Health:  monitoring active (10s interval)");
    println!();
    for (name, child) in &children {
        let pid = child.id().unwrap_or(0);
        let socket = socket_dir.join(format!("{}-{}.sock", name, family_id));
        println!("  {} (PID {}) -> {}", name, pid, socket.display());
    }
    println!();
    println!("Health check: echo '{{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"params\":{{}},\"id\":1}}' | nc -U {}/beardog-{}.sock -w 2 -q 1",
             socket_dir.display(), family_id);

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

    let known_primals = ["beardog", "songbird", "nestgate", "toadstool", "squirrel"];
    let mut active = Vec::new();

    for primal in &known_primals {
        let socket_path = socket_dir.join(format!("{}-{}.sock", primal, family_id));
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

    // Search paths in priority order
    let search_paths = vec![
        // Current architecture livespore
        PathBuf::from("livespore-usb")
            .join(std::env::consts::ARCH)
            .join("primals"),
        // Generic livespore
        PathBuf::from("livespore-usb/primals"),
        // Build output
        PathBuf::from("plasmidBin"),
        PathBuf::from("plasmidBin/optimized").join(std::env::consts::ARCH),
        // Cargo build output
        PathBuf::from("target/release"),
    ];

    for primal in primals {
        let mut found = false;
        for search in &search_paths {
            let path = search.join(primal);
            if path.exists() && path.is_file() {
                map.insert(primal.to_string(), path);
                found = true;
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
    // Remove stale socket
    let _ = tokio::fs::remove_file(socket_path).await;

    let mut cmd = Command::new(binary);

    // Primal-specific startup arguments
    match name {
        "beardog" => {
            cmd.arg("server").arg("--socket").arg(socket_path);
        }
        "songbird" => {
            let beardog_socket = socket_dir.join(format!("beardog-{}.sock", family_id));
            cmd.arg("server")
                .arg("--socket")
                .arg(socket_path)
                .env("SONGBIRD_SECURITY_PROVIDER", &beardog_socket)
                .env("BEARDOG_SOCKET", &beardog_socket);
        }
        "nestgate" => {
            cmd.arg("daemon")
                .arg("--socket-only")
                .env("NESTGATE_JWT_SECRET", generate_jwt_secret());
        }
        "toadstool" => {
            cmd.arg("server").arg("--socket").arg(socket_path);
        }
        "squirrel" => {
            let songbird_socket = socket_dir.join(format!("songbird-{}.sock", family_id));
            cmd.arg("server")
                .arg("--socket")
                .arg(socket_path)
                .env("HTTP_REQUEST_PROVIDER_SOCKET", &songbird_socket);

            // Load AI providers if env vars are set
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
            cmd.arg("server").arg("--socket").arg(socket_path);
        }
    }

    // Common environment
    cmd.env("FAMILY_ID", family_id)
        .env("NODE_ID", node_id)
        .env("BEARDOG_NODE_ID", node_id);

    // Spawn as background process
    let child = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn {}", name))?;

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
    let response = client
        .call("health", serde_json::json!({}))
        .await
        .context("Health check RPC failed")?;

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
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
