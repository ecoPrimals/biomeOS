// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Interface discovery - probe and learn how to talk to primals

use super::types::*;
use anyhow::Result;
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tracing::debug;

/// Discover how to interact with a primal
pub async fn discover_primal_interface(binary: &Path) -> Result<PrimalAdapter> {
    let name = binary
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .trim_end_matches("-bin")
        .to_string();

    let mut adapter = PrimalAdapter::new(name, binary.to_path_buf());

    // Try to discover version first (helps with caching)
    adapter.version = try_get_version(binary).await;

    // Probe interface patterns
    let patterns = probe_interface_patterns(binary).await;

    // Use first successful pattern
    if let Some(pattern) = patterns.first() {
        adapter.interface = pattern.clone();
        adapter.capabilities = discover_capabilities(&adapter).await?;
    }

    Ok(adapter)
}

/// Probe all known interface patterns
pub async fn probe_interface_patterns(binary: &Path) -> Vec<PrimalInterface> {
    let mut discovered = Vec::new();

    // Pattern 1: Direct execution (like Squirrel)
    if let Ok(interface) = try_direct_execution(binary).await {
        discovered.push(interface);
    }

    // Pattern 2: Subcommand patterns
    for cmd in &["serve", "start", "service", "run"] {
        if let Ok(interface) = try_subcommand(binary, cmd).await {
            discovered.push(interface);
        }
    }

    // If nothing worked, mark as unknown
    if discovered.is_empty() {
        discovered.push(PrimalInterface::Unknown {
            attempted_patterns: vec![
                InterfacePattern::Direct,
                InterfacePattern::SubcommandServe,
                InterfacePattern::SubcommandService,
                InterfacePattern::SubcommandStart,
                InterfacePattern::SubcommandRun,
            ],
        });
    }

    discovered
}

/// Try direct execution
async fn try_direct_execution(binary: &Path) -> Result<PrimalInterface> {
    // Try --version to see if binary responds
    let output = tokio::time::timeout(
        Duration::from_secs(2),
        tokio::process::Command::new(binary)
            .arg("--version")
            .output(),
    )
    .await;

    match output {
        Ok(Ok(output)) if output.status.success() => {
            Ok(PrimalInterface::Direct { args: Vec::new() })
        }
        _ => Err(anyhow::anyhow!("Not a direct execution interface")),
    }
}

/// Try subcommand pattern
async fn try_subcommand(binary: &Path, cmd: &str) -> Result<PrimalInterface> {
    // Try "<cmd> --help" to see if subcommand exists
    let output = tokio::time::timeout(
        Duration::from_secs(2),
        tokio::process::Command::new(binary)
            .arg(cmd)
            .arg("--help")
            .output(),
    )
    .await;

    match output {
        Ok(Ok(output)) if output.status.success() => {
            // Try to discover stop command
            let stop_cmd = discover_stop_command(binary).await;

            Ok(PrimalInterface::Subcommand {
                start_cmd: cmd.to_string(),
                stop_cmd,
            })
        }
        _ => Err(anyhow::anyhow!("Subcommand {cmd} not found")),
    }
}

/// Try to get version
async fn try_get_version(binary: &Path) -> Option<String> {
    let output = tokio::time::timeout(
        Duration::from_secs(2),
        tokio::process::Command::new(binary)
            .arg("--version")
            .output(),
    )
    .await;

    if let Ok(Ok(output)) = output {
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string());
        }
    }

    None
}

/// Discover what a primal can do
async fn discover_capabilities(adapter: &PrimalAdapter) -> Result<PrimalCapabilities> {
    let mut capabilities = PrimalCapabilities::default();

    // Can start if we have a known interface
    capabilities.lifecycle.can_start = adapter.interface.is_known();

    // Assume SIGTERM support for process-based interfaces
    capabilities.lifecycle.graceful_shutdown = matches!(
        adapter.interface,
        PrimalInterface::Direct { .. } | PrimalInterface::Subcommand { .. }
    );

    // Check for version command
    capabilities.has_version_cmd = adapter.version.is_some();

    // Try to detect port configuration method
    capabilities.port_config = detect_port_config(&adapter.binary).await;

    // Set default health check pattern
    capabilities.health_check = Some(HealthCheckConfig {
        url_pattern: "http://localhost:PORT/health".to_string(),
        expected_status: 200,
        timeout: Duration::from_secs(2),
    });

    Ok(capabilities)
}

/// Detect how to configure the port
async fn detect_port_config(binary: &Path) -> PortConfigMethod {
    // Check --help for port flags
    let output = tokio::time::timeout(
        Duration::from_secs(2),
        tokio::process::Command::new(binary).arg("--help").output(),
    )
    .await;

    if let Ok(Ok(output)) = output {
        if let Ok(help_text) = String::from_utf8(output.stdout) {
            // Look for port-related flags
            if help_text.contains("--port") {
                return PortConfigMethod::CliFlag("--port".to_string());
            }
            if help_text.contains("-p, --port") {
                return PortConfigMethod::CliFlag("--port".to_string());
            }
            // Check for env var mentions
            if help_text.contains("PORT") || help_text.contains("port") {
                return PortConfigMethod::Multiple(vec![
                    PortConfigMethod::EnvVar("PORT".to_string()),
                    PortConfigMethod::CliFlag("--port".to_string()),
                ]);
            }
        }
    }

    // Default fallback: try env var PORT
    PortConfigMethod::EnvVar("PORT".to_string())
}

/// Discover stop command for a primal
///
/// Tries common stop subcommands to see if the primal supports graceful shutdown.
/// Returns None if no stop command is found (fallback to SIGTERM).
pub(crate) async fn discover_stop_command(binary: &Path) -> Option<String> {
    const STOP_COMMANDS: &[&str] = &["stop", "shutdown", "halt", "quit"];

    for stop_cmd in STOP_COMMANDS {
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            Command::new(binary).arg(stop_cmd).arg("--help").output(),
        )
        .await;

        if let Ok(Ok(output)) = result {
            if output.status.success() {
                debug!(
                    "Discovered stop command '{}' for {}",
                    stop_cmd,
                    binary.display()
                );
                return Some(stop_cmd.to_string());
            }
        }
    }

    // No stop command found - will use SIGTERM
    debug!(
        "No stop command found for {}, will use SIGTERM",
        binary.display()
    );
    None
}
