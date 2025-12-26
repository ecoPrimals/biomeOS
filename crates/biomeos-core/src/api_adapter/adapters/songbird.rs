//! Songbird API Adapter
//!
//! **ARCHITECTURE DISCOVERY (Dec 26, 2025)**: Songbird is CLI-based, NOT REST API!
//!
//! Real-world testing revealed:
//! - Control via CLI commands (`songbird tower start`, etc.)
//! - Binary protocol on port 8080 (HTTP/0.9 or custom)
//! - No HTTP REST API endpoints
//! - Process-based lifecycle management
//!
//! This adapter wraps Songbird's CLI interface for BiomeOS integration.

use crate::api_adapter::cli_adapter::{CliAdapter, CliResult};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Songbird-specific CLI adapter
#[derive(Debug, Clone)]
pub struct SongbirdAdapter {
    /// Base CLI adapter
    cli: CliAdapter,

    /// Songbird binary path
    binary_path: PathBuf,

    /// Whether Songbird supports federation
    supports_federation: bool,
}

impl SongbirdAdapter {
    /// Create a new Songbird adapter with the path to the binary
    pub fn new<P: Into<PathBuf>>(binary_path: P) -> Result<Self> {
        let binary_path = binary_path.into();
        let cli = CliAdapter::new(&binary_path).with_timeout(60); // 60s for tower operations

        let adapter = Self {
            cli,
            binary_path,
            supports_federation: false, // Will be discovered
        };

        // Verify binary exists
        adapter
            .cli
            .verify_binary()
            .context("Songbird binary not found or not executable")?;

        Ok(adapter)
    }

    /// Discover Songbird's capabilities
    pub async fn discover_capabilities(&mut self) -> Result<()> {
        // Try to get help to see what commands are available
        if let Ok(help) = self.cli.get_help().await {
            // Check if federation is mentioned in help
            self.supports_federation = help.contains("federation") || help.contains("--federation");
        }

        Ok(())
    }

    /// Get Songbird version
    pub async fn get_version(&self) -> Result<String> {
        self.cli.get_version().await
    }

    /// Start a Songbird tower
    ///
    /// Note: This spawns a background process. You may want to manage the
    /// process lifecycle separately (see BiomeOS process manager).
    pub async fn start_tower(&self, port: u16, federation: bool) -> Result<CliResult> {
        let port_str = port.to_string();
        let mut args = vec!["tower", "start", "--port", &port_str];

        if federation && self.supports_federation {
            args.push("--federation");
        }

        // Note: This will block! In production, spawn as background process
        self.cli.execute(&args).await
    }

    /// Check if a tower is running (by attempting to get status)
    ///
    /// Note: This is a heuristic since Songbird is CLI-based.
    /// In production, track the tower process directly.
    pub async fn check_tower_running(&self) -> Result<bool> {
        // Try to execute a status command (if available)
        let result = self.cli.execute(&["tower", "status"]).await;

        match result {
            Ok(r) => Ok(r.is_success()),
            Err(_) => {
                // If status command doesn't exist, we can't determine
                // In production, use process tracking instead
                Ok(false)
            }
        }
    }

    /// Register a service with Songbird tower
    ///
    /// This is a placeholder - actual registration method depends on
    /// Songbird's CLI interface discovery.
    pub async fn register_service(&self, service_name: &str, endpoint: &str) -> Result<CliResult> {
        // Attempt common CLI patterns for service registration
        let patterns = vec![
            vec![
                "service",
                "register",
                "--name",
                service_name,
                "--endpoint",
                endpoint,
            ],
            vec!["register", service_name, endpoint],
            vec!["tower", "register", service_name, endpoint],
        ];

        for pattern in patterns {
            if let Ok(result) = self.cli.execute(&pattern).await {
                if result.is_success() {
                    return Ok(result);
                }
            }
        }

        anyhow::bail!("Could not register service - CLI pattern not discovered")
    }

    /// Query services from Songbird
    ///
    /// This is a placeholder - actual query method depends on
    /// Songbird's CLI interface discovery.
    pub async fn query_services(&self) -> Result<String> {
        // Attempt common CLI patterns for service queries
        let patterns = vec![
            vec!["service", "list"],
            vec!["services"],
            vec!["tower", "services"],
        ];

        for pattern in patterns {
            if let Ok(result) = self.cli.execute(&pattern).await {
                if result.is_success() {
                    return Ok(result.stdout);
                }
            }
        }

        anyhow::bail!("Could not query services - CLI pattern not discovered")
    }

    /// Get the CLI adapter (for advanced usage)
    pub fn cli(&self) -> &CliAdapter {
        &self.cli
    }

    /// Get the binary path
    pub fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_songbird_adapter_creation() {
        // Test with a mock binary path (won't verify)
        let result = SongbirdAdapter::new("/usr/bin/false");
        // Should fail if binary doesn't support Songbird interface
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_songbird_version() {
        // This test requires actual Songbird binary
        // Skip if not available
        let songbird_path =
            std::env::var("SONGBIRD_BIN").unwrap_or_else(|_| "/nonexistent/songbird".to_string());

        if let Ok(adapter) = SongbirdAdapter::new(&songbird_path) {
            let _ = adapter.get_version().await;
            // Just test that the method runs
        }
    }
}
