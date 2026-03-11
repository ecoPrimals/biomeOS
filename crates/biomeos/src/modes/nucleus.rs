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

        // Toadstool exposes tarpc on .sock and JSON-RPC on .jsonrpc.sock
        // NUCLEUS health checks use JSON-RPC, so use the jsonrpc socket for health monitoring
        let health_socket = if *primal == "toadstool" {
            socket_dir.join(format!("{}-{}.jsonrpc.sock", primal, family_id))
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
        .with_context(|| format!("Failed to start {}", primal))?;

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
        if *primal == "toadstool" {
            lifecycle
                .set_health_method("toadstool", "toadstool.health")
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
                .arg("--family-id")
                .arg(family_id)
                .env("NESTGATE_JWT_SECRET", generate_jwt_secret());
        }
        "toadstool" => {
            cmd.arg("server")
                .arg("--socket")
                .arg(socket_path.as_os_str())
                .env("TOADSTOOL_SOCKET", socket_path.as_os_str())
                .env("TOADSTOOL_FAMILY_ID", family_id);
        }
        "squirrel" => {
            let songbird_socket = socket_dir.join(format!("songbird-{}.sock", family_id));
            cmd.arg("server")
                .arg("--socket")
                .arg(socket_path.as_os_str())
                .env("SQUIRREL_SOCKET", socket_path.as_os_str())
                .env("HTTP_REQUEST_PROVIDER_SOCKET", songbird_socket.as_os_str());

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
            let semantic_method = format!("{}.health", primal_name);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nucleus_mode_from_str_valid() {
        assert!(matches!(
            "tower".parse::<NucleusMode>().expect("tower should parse"),
            NucleusMode::Tower
        ));
        assert!(matches!(
            "Tower"
                .parse::<NucleusMode>()
                .expect("Tower should parse (case insensitive)"),
            NucleusMode::Tower
        ));
        assert!(matches!(
            "node".parse::<NucleusMode>().expect("node should parse"),
            NucleusMode::Node
        ));
        assert!(matches!(
            "nest".parse::<NucleusMode>().expect("nest should parse"),
            NucleusMode::Nest
        ));
        assert!(matches!(
            "full".parse::<NucleusMode>().expect("full should parse"),
            NucleusMode::Full
        ));
        assert!(matches!(
            "nucleus"
                .parse::<NucleusMode>()
                .expect("nucleus should parse"),
            NucleusMode::Full
        ));
    }

    #[test]
    fn test_nucleus_mode_from_str_invalid() {
        let err = "invalid".parse::<NucleusMode>().unwrap_err();
        assert!(err.to_string().contains("Unknown nucleus mode"));
        assert!(err.to_string().contains("invalid"));
        assert!(err.to_string().contains("tower|node|nest|full"));

        let err2 = "".parse::<NucleusMode>().unwrap_err();
        assert!(err2.to_string().contains("Unknown nucleus mode"));
    }

    #[test]
    fn test_nucleus_mode_primals() {
        assert_eq!(
            NucleusMode::Tower.primals(),
            vec!["beardog", "songbird"],
            "Tower mode primals"
        );
        assert_eq!(
            NucleusMode::Node.primals(),
            vec!["beardog", "songbird", "toadstool"],
            "Node mode primals"
        );
        assert_eq!(
            NucleusMode::Nest.primals(),
            vec!["beardog", "songbird", "nestgate", "squirrel"],
            "Nest mode primals"
        );
        assert_eq!(
            NucleusMode::Full.primals(),
            vec!["beardog", "songbird", "nestgate", "toadstool", "squirrel"],
            "Full mode primals"
        );
    }

    #[test]
    fn test_base64_encode_empty() {
        assert_eq!(base64_encode(&[]), "");
    }

    #[test]
    fn test_base64_encode_single_byte() {
        // "M" in base64 is 0x0 in first 6 bits -> 'A', next 6 bits 0 -> 'A', padding
        let result = base64_encode(&[0x4d]);
        assert_eq!(result.len(), 4);
        assert!(result.ends_with("=="));
    }

    #[test]
    fn test_base64_encode_three_bytes() {
        // "Man" -> TWFu in standard base64
        let result = base64_encode(b"Man");
        assert_eq!(result, "TWFu");
    }

    #[test]
    fn test_base64_encode_roundtrip_alphabet() {
        let data = b"Hello, World!";
        let encoded = base64_encode(data);
        assert!(!encoded.is_empty());
        assert!(encoded.len() <= data.len().div_ceil(3) * 4 + 4);
        for c in encoded.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=',
                "Invalid base64 char: {:?}",
                c
            );
        }
    }

    #[test]
    fn test_resolve_socket_dir_env_override() {
        let test_path = "/tmp/biomeos-test-socket-dir";
        std::env::set_var("BIOMEOS_SOCKET_DIR", test_path);
        let result = resolve_socket_dir().expect("resolve_socket_dir should succeed");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
        assert_eq!(result, std::path::PathBuf::from(test_path));
    }

    #[test]
    fn test_discover_binaries_empty_primals() {
        let map = discover_binaries(&[]).expect("empty primals should succeed");
        assert!(map.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn test_discover_binaries_finds_in_path() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let unique_name = "biomeos_test_binary_xyz";
        let binary_path = temp_dir.path().join(unique_name);
        std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();

        let original_path = std::env::var("PATH").ok();
        let dir_str = temp_dir.path().to_string_lossy().into_owned();
        std::env::set_var(
            "PATH",
            format!("{}:{}", dir_str, original_path.as_deref().unwrap_or("")),
        );

        let map = discover_binaries(&[unique_name]).expect("discover should succeed");
        if let Some(original) = original_path {
            std::env::set_var("PATH", original);
        } else {
            std::env::remove_var("PATH");
        }

        assert!(
            map.contains_key(unique_name),
            "{} should be found in PATH, got: {:?}",
            unique_name,
            map
        );
    }
}
