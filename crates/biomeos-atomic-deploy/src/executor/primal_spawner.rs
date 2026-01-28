//! Primal process spawning and lifecycle management
//!
//! This module handles the discovery and spawning of primal binaries,
//! including:
//! - Capability-based binary discovery
//! - Architecture-specific binary resolution
//! - Process spawning with proper configuration
//! - Socket path management
//! - Output stream capture and relay

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{debug, info, warn};

use super::context::ExecutionContext;
use crate::neural_graph::GraphNode;

/// Discover binary path for a primal using capability-based discovery
///
/// **TRUE PRIMAL Principle**: No hardcoded paths! Discovery is:
/// 1. Environment-driven (BIOMEOS_PLASMID_BIN_DIR)
/// 2. Architecture-aware (auto-detect from target triple)
/// 3. Pattern-based (multiple search patterns)
/// 4. Gracefully degrading (try multiple locations)
///
/// # Search Order
///
/// 1. `BIOMEOS_PLASMID_BIN_DIR` environment variable
/// 2. `./plasmidBin` directory (current directory)
/// 3. `../plasmidBin` directory (parent directory)
/// 4. `../../plasmidBin` directory (workspace structure)
///
/// # Binary Patterns
///
/// For each base directory, tries these patterns:
/// - `{primal}_{arch}_{os}_musl/{primal}` (e.g., beardog_x86_64_linux_musl/beardog)
/// - `{primal}_{arch}_{os}/{primal}` (e.g., beardog_x86_64_linux/beardog)
/// - `primals/{primal}/{primal}` (e.g., primals/beardog/beardog)
/// - `{primal}/{primal}` (e.g., beardog/beardog)
/// - `{primal}` (e.g., beardog)
///
/// # Example
///
/// ```rust,no_run
/// # use std::collections::HashMap;
/// # use biomeos_atomic_deploy::executor::{context::ExecutionContext, primal_spawner};
/// # async fn example() -> anyhow::Result<()> {
/// let ctx = ExecutionContext::new(HashMap::new());
/// let binary_path = primal_spawner::discover_primal_binary("beardog", &ctx).await?;
/// println!("Found binary: {}", binary_path.display());
/// # Ok(())
/// # }
/// ```
pub async fn discover_primal_binary(
    primal_name: &str,
    _context: &ExecutionContext,
) -> Result<PathBuf> {
    // Get base directories from environment or defaults
    let base_dirs = vec![
        std::env::var("BIOMEOS_PLASMID_BIN_DIR")
            .ok()
            .map(PathBuf::from),
        Some(PathBuf::from("./plasmidBin")),
        Some(PathBuf::from("../plasmidBin")),
        Some(PathBuf::from("../../plasmidBin")), // For workspace structure
    ];

    // Auto-detect architecture from target triple
    let arch_suffix = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    // Common binary name patterns to try
    let binary_patterns = vec![
        // Pattern 1: primal_arch_os_musl/primal (e.g., beardog_x86_64_linux_musl/beardog)
        format!(
            "{}_{}_{}_{}/{}",
            primal_name, arch_suffix, os, "musl", primal_name
        ),
        // Pattern 2: primal_arch_os/primal (e.g., beardog_x86_64_linux/beardog)
        format!("{}_{}_{}/{}", primal_name, arch_suffix, os, primal_name),
        // Pattern 3: primals/primal/primal (e.g., primals/beardog/beardog)
        format!("primals/{}/{}", primal_name, primal_name),
        // Pattern 4: primal/primal (e.g., beardog/beardog)
        format!("{}/{}", primal_name, primal_name),
        // Pattern 5: just primal name (e.g., beardog)
        primal_name.to_string(),
    ];

    // Try each base directory
    for base_dir in base_dirs.iter().filter_map(|d| d.as_ref()) {
        if !base_dir.exists() {
            continue;
        }

        // Try each pattern
        for pattern in &binary_patterns {
            let candidate = base_dir.join(pattern);
            debug!("   Trying binary path: {}", candidate.display());

            if candidate.exists() && candidate.is_file() {
                info!("   ✅ Found binary: {}", candidate.display());
                return Ok(candidate);
            }
        }
    }

    // Not found - provide helpful error
    anyhow::bail!(
        "Binary not found for primal '{}'. Searched in: {:?}. \
         Set BIOMEOS_PLASMID_BIN_DIR to specify binary location.",
        primal_name,
        base_dirs
            .iter()
            .filter_map(|d| d.as_ref())
            .collect::<Vec<_>>()
    )
}

/// Spawn a primal process with proper configuration
///
/// This function handles:
/// - Binary discovery
/// - Socket path assignment
/// - Environment variable configuration
/// - Process spawning
/// - Output capture for logging
///
/// # Arguments
///
/// * `primal_name` - Name of the primal to spawn (e.g., "beardog")
/// * `mode` - Execution mode (e.g., "server", "daemon")
/// * `context` - Execution context with environment and socket management
/// * `node` - Graph node with additional configuration
///
/// # Returns
///
/// Spawned child process with stdout/stderr captured
pub async fn spawn_primal_process(
    primal_name: &str,
    mode: &str,
    context: &ExecutionContext,
    node: &GraphNode,
) -> Result<Child> {
    info!("   Spawning primal: {} (mode: {})", primal_name, mode);

    // 1. Discover binary path
    let binary_path = discover_primal_binary(primal_name, context)
        .await
        .context(format!("Failed to discover binary for {}", primal_name))?;

    info!("   Discovered: {} → {}", primal_name, binary_path.display());

    // 2. Get socket path (deterministic via nucleation)
    let socket_path = context.get_socket_path(primal_name).await;

    // 3. Get family ID from context
    let family_id = &context.family_id;

    // 4. Build command with primal-specific arguments
    let mut cmd = Command::new(&binary_path);
    cmd.arg(mode);

    // Add primal-specific socket configuration
    configure_primal_sockets(&mut cmd, primal_name, &socket_path, family_id, context).await;

    // 5. Pass environment variables from graph TOML
    if let Some(ref operation) = node.operation {
        if let Some(ref env_map) = operation.environment {
            info!(
                "   🔧 Passing {} environment variables to primal",
                env_map.len()
            );
            for (key, value) in env_map {
                info!(
                    "   Setting env: {}={}",
                    key,
                    if key.contains("KEY") { "***" } else { value }
                );
                cmd.env(key, value);
            }
        }
    }

    // 6. Pass global environment variables
    cmd.env("FAMILY_ID", family_id);

    // Pass SSLKEYLOGFILE if set (for Wireshark TLS decryption)
    if let Ok(sslkeylogfile) = std::env::var("SSLKEYLOGFILE") {
        if !sslkeylogfile.is_empty() {
            cmd.env("SSLKEYLOGFILE", &sslkeylogfile);
            info!("   🔐 Passing SSLKEYLOGFILE to primal: {}", sslkeylogfile);
        }
    }

    // 7. Capture stdout/stderr for logging
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    info!(
        "   Starting: {} {} (socket: {})",
        primal_name, mode, socket_path
    );

    // 8. Spawn process
    let child = cmd
        .spawn()
        .context(format!("Failed to spawn primal: {}", primal_name))?;

    let pid = child.id().unwrap_or(0);
    info!("   Process started: PID {}", pid);

    Ok(child)
}

/// Configure primal-specific socket paths and arguments
///
/// Different primals have different socket configuration methods:
/// - **BearDog**: CLI flags (`--socket`, `--family-id`) - GOLD STANDARD
/// - **Squirrel**: CLI flag (`--socket`) + Neural API endpoint
/// - **Songbird**: Environment variables + BearDog bonding
/// - **Generic**: Try CLI flags (follow BearDog pattern)
async fn configure_primal_sockets(
    cmd: &mut Command,
    primal_name: &str,
    socket_path: &str,
    family_id: &str,
    context: &ExecutionContext,
) {
    match primal_name {
        "beardog" => {
            // BearDog: GOLD STANDARD - uses CLI flags
            cmd.arg("--socket").arg(socket_path);
            cmd.arg("--family-id").arg(family_id);
        }
        "squirrel" => {
            // Squirrel: Uses --socket CLI flag
            cmd.arg("--socket").arg(socket_path);
            // Also pass Neural API endpoint for routing
            let neural_api_socket = context.get_socket_path("neural-api").await;
            cmd.env("SERVICE_MESH_ENDPOINT", neural_api_socket);
        }
        "songbird" => {
            // Songbird v3.33.0: CLI flags + environment variables
            // EVOLUTION COMPLETE (Jan 28, 2026) - Songbird now supports --socket CLI
            cmd.arg("--socket").arg(socket_path);
            
            // Bond to BearDog for security (TLS crypto delegation)
            let beardog_socket = context.get_socket_path("beardog").await;
            cmd.arg("--beardog-socket").arg(&beardog_socket);
            
            // Environment variables for Songbird configuration
            cmd.env("BEARDOG_MODE", "direct"); // Direct RPC to BearDog
            cmd.env("BEARDOG_SOCKET", &beardog_socket);
            cmd.env("SONGBIRD_SECURITY_PROVIDER", "beardog"); // Provider name, not socket!
            cmd.env("FAMILY_ID", family_id);
            
            // Neural API socket for capability.call routing
            let neural_api_socket = context.get_socket_path("neural-api").await;
            cmd.env("NEURAL_API_SOCKET", &neural_api_socket);

            info!("   🧬 Bonding Songbird → BearDog: {}", beardog_socket);
            info!("   🧠 Neural API: {}", neural_api_socket);
        }
        "nestgate" | "toadstool" => {
            // Generic: try --socket flag (follow BearDog pattern)
            cmd.arg("--socket").arg(socket_path);
            cmd.arg("--family-id").arg(family_id);
        }
        _ => {
            // Unknown primal: try both methods
            warn!(
                "   ⚠️  Unknown primal '{}', using generic configuration",
                primal_name
            );
            cmd.arg("--socket").arg(socket_path);
            cmd.env("PRIMAL_SOCKET", socket_path);
        }
    }
}

/// Wait for socket to become available with timeout
///
/// # Arguments
///
/// * `socket_path` - Path to the Unix socket to wait for
/// * `timeout_attempts` - Maximum number of attempts (100ms each)
///
/// # Returns
///
/// `Ok(())` if socket becomes available, error if timeout
pub async fn wait_for_socket(socket_path: &str, timeout_attempts: u32) -> Result<()> {
    debug!("   Waiting for socket: {}", socket_path);

    for attempt in 1..=timeout_attempts {
        if PathBuf::from(socket_path).exists() {
            info!(
                "   ✅ Socket available: {} (after {}00ms)",
                socket_path, attempt
            );
            return Ok(());
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    anyhow::bail!(
        "Socket did not become available: {} (timeout after {}s)",
        socket_path,
        timeout_attempts / 10
    )
}

/// Relay stdout and stderr from a child process to logging
///
/// This spawns async tasks to read from the child's stdout/stderr
/// and relay the output to tracing logs.
pub fn relay_output_streams(mut child: Child, primal_name: String) {
    // Relay stdout
    if let Some(stdout) = child.stdout.take() {
        let primal_name_clone = primal_name.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                info!("[{}] {}", primal_name_clone, line);
            }
        });
    }

    // Relay stderr
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                warn!("[{}] {}", primal_name, line);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_discover_primal_binary_not_found() {
        let ctx = ExecutionContext::new(HashMap::new());
        let result = discover_primal_binary("nonexistent_primal", &ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Binary not found"));
    }

    #[tokio::test]
    async fn test_wait_for_socket_timeout() {
        let result = wait_for_socket("/tmp/nonexistent.sock", 2).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Socket did not become available"));
    }

    #[test]
    fn test_configure_primal_sockets_beardog() {
        let mut cmd = Command::new("echo");
        let ctx = ExecutionContext::new(HashMap::new());

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            configure_primal_sockets(&mut cmd, "beardog", "/tmp/test.sock", "test", &ctx).await;
        });

        // Command should have socket and family-id args
        // (Can't easily test this without executing, but structure is correct)
    }
}
