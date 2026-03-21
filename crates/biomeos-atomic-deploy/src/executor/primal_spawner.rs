// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
use std::collections::HashMap;
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
/// 1. `ECOPRIMALS_PLASMID_BIN` environment variable (ecosystem root)
/// 2. `BIOMEOS_PLASMID_BIN_DIR` environment variable (biomeOS-local)
/// 3. `./plasmidBin` directory (current directory)
/// 4. `../plasmidBin` directory (parent directory)
/// 5. `../../plasmidBin` directory (reaches ecosystem root from phase2/biomeOS/)
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
    let base_dirs: &[Option<PathBuf>] = &[
        std::env::var("ECOPRIMALS_PLASMID_BIN")
            .ok()
            .map(PathBuf::from),
        std::env::var("BIOMEOS_PLASMID_BIN_DIR")
            .ok()
            .map(PathBuf::from),
        Some(PathBuf::from("./plasmidBin")),
        Some(PathBuf::from("../plasmidBin")),
        Some(PathBuf::from("../../plasmidBin")),
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
        // Pattern 4: primals/primal (flat layout in ecoPrimals/plasmidBin/primals/)
        format!("primals/{}", primal_name),
        // Pattern 5: primal/primal (e.g., beardog/beardog)
        format!("{}/{}", primal_name, primal_name),
        // Pattern 6: just primal name (e.g., beardog)
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
         Set ECOPRIMALS_PLASMID_BIN or BIOMEOS_PLASMID_BIN_DIR to specify binary location.",
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
        .context(format!("Failed to discover binary for {primal_name}"))?;

    info!("   Discovered: {} → {}", primal_name, binary_path.display());

    // 2. Get socket path (deterministic via nucleation)
    let socket_path = context.get_socket_path(primal_name).await;

    // 3. Get family ID from context (as_ref for &str)
    let family_id = context.family_id.as_ref();

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

    // AI_DEFAULT_MODEL: Squirrel reads at startup for default model override (Bypass 4 evolution)
    if primal_name.eq_ignore_ascii_case("squirrel") {
        if let Ok(model) = std::env::var("AI_DEFAULT_MODEL") {
            cmd.env("AI_DEFAULT_MODEL", &model);
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
        .context(format!("Failed to spawn primal: {primal_name}"))?;

    let pid = child.id().unwrap_or(0);
    info!("   Process started: PID {}", pid);

    Ok(child)
}

/// Data-driven primal launch profile (loaded from config/primal_launch_profiles.toml)
#[derive(Debug, Clone, serde::Deserialize)]
struct LaunchProfile {
    socket_flag: Option<String>,
    pass_family_id: Option<bool>,
    env_socket: Option<String>,
    #[serde(default)]
    extra_env: HashMap<String, String>,
    #[serde(default)]
    env_sockets: HashMap<String, String>,
    #[serde(default)]
    cli_sockets: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
struct LaunchProfilesConfig {
    default: LaunchProfile,
    #[serde(default)]
    profiles: HashMap<String, LaunchProfile>,
}

static LAUNCH_PROFILES_TOML: &str = include_str!("../../../../config/primal_launch_profiles.toml");

fn load_launch_profiles() -> LaunchProfilesConfig {
    toml::from_str(LAUNCH_PROFILES_TOML).unwrap_or_else(|e| {
        warn!("Failed to parse primal launch profiles: {}", e);
        LaunchProfilesConfig {
            default: LaunchProfile {
                socket_flag: Some("--socket".to_string()),
                pass_family_id: Some(true),
                env_socket: Some("PRIMAL_SOCKET".to_string()),
                extra_env: HashMap::new(),
                env_sockets: HashMap::new(),
                cli_sockets: HashMap::new(),
            },
            profiles: HashMap::new(),
        }
    })
}

/// Configure primal-specific socket paths and arguments
///
/// Uses data-driven launch profiles from `config/primal_launch_profiles.toml`.
/// Primals not listed in the config inherit the `[default]` profile.
/// New primals can be onboarded by adding a TOML entry — no code changes needed.
///
/// Pub(crate) for reuse by capability_handlers::primal_start (capability-based, no hardcoded names).
pub(crate) async fn configure_primal_sockets(
    cmd: &mut Command,
    primal_name: &str,
    socket_path: &str,
    family_id: &str,
    context: &ExecutionContext,
) {
    let config = load_launch_profiles();
    let profile = config.profiles.get(primal_name);
    let defaults = &config.default;

    let socket_flag = profile
        .and_then(|p| p.socket_flag.as_deref())
        .or(defaults.socket_flag.as_deref())
        .unwrap_or("--socket");

    let pass_family_id = profile
        .and_then(|p| p.pass_family_id)
        .or(defaults.pass_family_id)
        .unwrap_or(true);

    let env_socket = profile
        .and_then(|p| p.env_socket.as_deref())
        .or(defaults.env_socket.as_deref());

    // Primary socket CLI flag
    cmd.arg(socket_flag).arg(socket_path);

    if pass_family_id {
        cmd.arg("--family-id").arg(family_id);
    }

    // Env var fallback for socket path (only for unknown primals without a profile)
    if profile.is_none() {
        if let Some(env_name) = env_socket {
            cmd.env(env_name, socket_path);
        }
        warn!("   No launch profile for '{}', using defaults", primal_name);
    }

    // Static extra env vars from the profile
    if let Some(p) = profile {
        for (key, value) in &p.extra_env {
            cmd.env(key, value);
        }

        // Env vars whose values are resolved socket paths of other primals
        for (env_name, socket_ref) in &p.env_sockets {
            if socket_ref == "$family_id" {
                cmd.env(env_name, family_id);
            } else {
                let resolved = context.get_socket_path(socket_ref).await;
                cmd.env(env_name, &resolved);
            }
        }

        // Extra CLI flags whose values are resolved socket paths
        for (flag, socket_ref) in &p.cli_sockets {
            let resolved = context.get_socket_path(socket_ref).await;
            cmd.arg(flag).arg(&resolved);
            info!("   Bonding {} → {}: {}", primal_name, socket_ref, resolved);
        }
    }
}

/// Default poll interval for socket readiness (100ms).
pub const DEFAULT_SOCKET_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// Wait for socket to become available with timeout
///
/// # Arguments
///
/// * `socket_path` - Path to the Unix socket to wait for
/// * `timeout_attempts` - Maximum number of attempts (100ms each by default)
///
/// # Returns
///
/// `Ok(())` if socket becomes available, error if timeout
pub async fn wait_for_socket(socket_path: &str, timeout_attempts: u32) -> Result<()> {
    wait_for_socket_with_poll_interval(socket_path, timeout_attempts, DEFAULT_SOCKET_POLL_INTERVAL)
        .await
}

/// Wait for socket with configurable poll interval (for tests: use `Duration::ZERO`).
pub async fn wait_for_socket_with_poll_interval(
    socket_path: &str,
    timeout_attempts: u32,
    poll_interval: std::time::Duration,
) -> Result<()> {
    debug!("   Waiting for socket: {}", socket_path);

    for attempt in 1..=timeout_attempts {
        if PathBuf::from(socket_path).exists() {
            info!(
                "   ✅ Socket available: {} (after {} attempts)",
                socket_path, attempt
            );
            return Ok(());
        }

        tokio::time::sleep(poll_interval).await;
    }

    anyhow::bail!(
        "Socket did not become available: {} (timeout after {} attempts)",
        socket_path,
        timeout_attempts
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
    use crate::neural_graph::GraphNode;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_discover_primal_binary_not_found() {
        let ctx = ExecutionContext::new(HashMap::new());
        let result = discover_primal_binary("nonexistent_primal", &ctx).await;
        let err = result.expect_err("Should fail for nonexistent primal");
        assert!(
            err.to_string().contains("Binary not found"),
            "Error should mention binary: {err}"
        );
    }

    #[tokio::test]
    async fn test_discover_primal_binary_with_env_dir_empty() {
        use biomeos_test_utils::{remove_test_env, set_test_env};
        let temp_dir = TempDir::new().expect("Create temp dir");
        set_test_env("BIOMEOS_PLASMID_BIN_DIR", temp_dir.path());
        let ctx = ExecutionContext::new(HashMap::new());

        let result = discover_primal_binary("nonexistent_primal", &ctx).await;
        remove_test_env("BIOMEOS_PLASMID_BIN_DIR");
        let err = result.expect_err("Should fail when dir has no binary");
        assert!(
            err.to_string().contains("Binary not found"),
            "Error should mention binary: {err}"
        );
    }

    #[tokio::test]
    async fn test_wait_for_socket_timeout() {
        let result = wait_for_socket_with_poll_interval(
            "/tmp/nonexistent-socket-xyz-12345.sock",
            2,
            std::time::Duration::ZERO,
        )
        .await;
        let err = result.expect_err("Should timeout on nonexistent socket");
        assert!(
            err.to_string().contains("Socket did not become available"),
            "Error should mention socket timeout: {err}"
        );
    }

    #[tokio::test]
    async fn test_wait_for_socket_success() {
        let temp_dir = TempDir::new().expect("Create temp dir");
        let socket_path = temp_dir.path().join("test.sock");
        let _listener =
            std::os::unix::net::UnixListener::bind(&socket_path).expect("Bind test socket");

        let result = wait_for_socket(socket_path.to_str().unwrap(), 10).await;
        result.expect("Should succeed when socket exists");
    }

    #[tokio::test]
    async fn test_configure_primal_sockets_beardog() {
        let mut cmd = Command::new("echo");
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        let ctx = ExecutionContext::new(env);

        configure_primal_sockets(&mut cmd, "beardog", "/tmp/beardog.sock", "test", &ctx).await;

        // Spawn and verify args were added (echo will print them)
        let output = cmd.output().await.expect("spawn echo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("/tmp/beardog.sock") || stdout.contains("--socket"),
            "Beardog should get --socket arg: {stdout}"
        );
    }

    #[tokio::test]
    async fn test_configure_primal_sockets_squirrel() {
        let mut cmd = Command::new("echo");
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        let ctx = ExecutionContext::new(env);

        configure_primal_sockets(&mut cmd, "squirrel", "/tmp/squirrel.sock", "test", &ctx).await;

        let output = cmd.output().await.expect("spawn echo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("/tmp/squirrel.sock"),
            "Squirrel should get socket: {stdout}"
        );
    }

    #[tokio::test]
    async fn test_configure_primal_sockets_songbird() {
        let mut cmd = Command::new("echo");
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        let ctx = ExecutionContext::new(env);

        configure_primal_sockets(&mut cmd, "songbird", "/tmp/songbird.sock", "test", &ctx).await;

        let output = cmd.output().await.expect("spawn echo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("/tmp/songbird.sock"),
            "Songbird should get socket: {stdout}"
        );
    }

    #[tokio::test]
    async fn test_configure_primal_sockets_nestgate() {
        let mut cmd = Command::new("echo");
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        let ctx = ExecutionContext::new(env);

        configure_primal_sockets(&mut cmd, "nestgate", "/tmp/nestgate.sock", "test", &ctx).await;

        let output = cmd.output().await.expect("spawn echo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("/tmp/nestgate.sock"),
            "Nestgate should get socket: {stdout}"
        );
    }

    #[tokio::test]
    async fn test_configure_primal_sockets_unknown_primal() {
        let mut cmd = Command::new("echo");
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());
        let ctx = ExecutionContext::new(env);

        configure_primal_sockets(
            &mut cmd,
            "unknown_primal",
            "/tmp/unknown.sock",
            "test",
            &ctx,
        )
        .await;

        let output = cmd.output().await.expect("spawn echo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("/tmp/unknown.sock"),
            "Unknown primal should get generic socket config: {stdout}"
        );
    }

    fn make_minimal_node(id: &str) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_spawn_primal_process_binary_not_found() {
        let ctx = ExecutionContext::new(HashMap::new());
        let node = make_minimal_node("test-node");

        let result = spawn_primal_process("nonexistent_primal_xyz", "server", &ctx, &node).await;

        let err = result.expect_err("Should fail when binary not found");
        assert!(
            err.to_string().contains("Failed to discover binary")
                || err.to_string().contains("Binary not found"),
            "Error should mention binary discovery: {err}"
        );
    }

    #[tokio::test]
    async fn test_spawn_primal_process_with_env_vars_in_node() {
        let ctx = ExecutionContext::new(HashMap::new());
        let mut node = make_minimal_node("test-node");
        node.operation = Some(crate::neural_graph::Operation {
            name: "test".to_string(),
            params: HashMap::new(),
            environment: Some({
                let mut m = HashMap::new();
                m.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());
                m
            }),
        });

        // Will fail at binary discovery, but we're testing that node with env is accepted
        let result = spawn_primal_process("nonexistent_primal", "server", &ctx, &node).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_patterns_contain_primal_name() {
        let primal_name = "beardog";
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        let pattern_musl = format!("{}_{}_{}_{}/{}", primal_name, arch, os, "musl", primal_name);
        assert!(pattern_musl.contains(primal_name));
        assert!(pattern_musl.contains("musl"));

        let pattern_simple = format!("{primal_name}/{primal_name}");
        assert_eq!(pattern_simple, "beardog/beardog");
    }

    #[tokio::test]
    async fn test_relay_output_streams_no_panic() {
        let child = Command::new("true")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn true");

        relay_output_streams(child, "test-primal".to_string());
        // If we get here without panic, the function accepted the child
    }
}
