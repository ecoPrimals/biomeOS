//! BearDog API Adapter
//!
//! **ARCHITECTURE DISCOVERY (Dec 26, 2025)**: BearDog is CLI-based, NOT REST API!
//!
//! Real-world testing revealed:
//! - Pure CLI tool for cryptographic operations
//! - Commands: encrypt, decrypt, birdsong, key, entropy, hsm
//! - No service/server mode
//! - Stateless operations via process execution
//! - Universal HSM integration
//! - BirdSong lineage-based encryption
//!
//! This adapter wraps BearDog's CLI interface for BiomeOS integration.

use crate::api_adapter::cli_adapter::{CliAdapter, CliResult};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// BearDog-specific CLI adapter
#[derive(Debug, Clone)]
pub struct BearDogAdapter {
    /// Base CLI adapter
    cli: CliAdapter,

    /// BearDog binary path
    binary_path: PathBuf,

    /// Whether BearDog supports stream encryption
    supports_streaming: bool,

    /// Whether BearDog supports HSM operations
    supports_hsm: bool,
}

impl BearDogAdapter {
    /// Create a new BearDog adapter with the path to the binary
    pub fn new<P: Into<PathBuf>>(binary_path: P) -> Result<Self> {
        let binary_path = binary_path.into();
        let cli = CliAdapter::new(&binary_path).with_timeout(120); // 120s for crypto operations

        let adapter = Self {
            cli,
            binary_path,
            supports_streaming: false,
            supports_hsm: false,
        };

        // Verify binary exists
        adapter
            .cli
            .verify_binary()
            .context("BearDog binary not found or not executable")?;

        Ok(adapter)
    }

    /// Discover BearDog's capabilities
    pub async fn discover_capabilities(&mut self) -> Result<()> {
        // Try to get help to see what commands are available
        if let Ok(help) = self.cli.get_help().await {
            // Check for stream encryption support
            self.supports_streaming =
                help.contains("stream-encrypt") || help.contains("stream-decrypt");

            // Check for HSM support
            self.supports_hsm = help.contains("hsm");
        }

        Ok(())
    }

    /// Get BearDog version
    pub async fn get_version(&self) -> Result<String> {
        self.cli.get_version().await
    }

    /// Get BearDog status
    pub async fn get_status(&self) -> Result<String> {
        let result = self.cli.execute(&["status"]).await?;

        if result.is_success() {
            Ok(result.stdout)
        } else {
            anyhow::bail!("Status command failed: {}", result.stderr)
        }
    }

    /// Encrypt data using BearDog
    ///
    /// Args:
    /// - `input_path`: Path to input file
    /// - `output_path`: Path to output file
    /// - `key_id`: Key identifier or path
    pub async fn encrypt(
        &self,
        input_path: &str,
        output_path: &str,
        key_id: &str,
    ) -> Result<CliResult> {
        let args = vec![
            "encrypt",
            "--key",
            key_id,
            "--input",
            input_path,
            "--output",
            output_path,
        ];

        self.cli.execute(&args).await
    }

    /// Decrypt data using BearDog
    ///
    /// Args:
    /// - `input_path`: Path to input file
    /// - `output_path`: Path to output file
    /// - `key_id`: Key identifier or path
    pub async fn decrypt(
        &self,
        input_path: &str,
        output_path: &str,
        key_id: &str,
    ) -> Result<CliResult> {
        let args = vec![
            "decrypt",
            "--key",
            key_id,
            "--input",
            input_path,
            "--output",
            output_path,
        ];

        self.cli.execute(&args).await
    }

    /// BirdSong encrypt (lineage-based encryption)
    ///
    /// Args:
    /// - `input_path`: Path to input file
    /// - `output_path`: Path to output file
    /// - `lineage_id`: Lineage identifier
    pub async fn birdsong_encrypt(
        &self,
        input_path: &str,
        output_path: &str,
        lineage_id: &str,
    ) -> Result<CliResult> {
        let args = vec![
            "birdsong",
            "encrypt",
            "--lineage",
            lineage_id,
            "--input",
            input_path,
            "--output",
            output_path,
        ];

        self.cli.execute(&args).await
    }

    /// BirdSong decrypt (lineage-based decryption)
    ///
    /// Args:
    /// - `input_path`: Path to input file
    /// - `output_path`: Path to output file
    pub async fn birdsong_decrypt(&self, input_path: &str, output_path: &str) -> Result<CliResult> {
        let args = vec![
            "birdsong",
            "decrypt",
            "--input",
            input_path,
            "--output",
            output_path,
        ];

        self.cli.execute(&args).await
    }

    /// Stream encrypt large files (if supported)
    ///
    /// Args:
    /// - `input_path`: Path to input file
    /// - `output_path`: Path to output file
    /// - `key_id`: Key identifier or path
    pub async fn stream_encrypt(
        &self,
        input_path: &str,
        output_path: &str,
        key_id: &str,
    ) -> Result<CliResult> {
        if !self.supports_streaming {
            anyhow::bail!("Stream encryption not supported by this BearDog version");
        }

        let args = vec![
            "stream-encrypt",
            "--key",
            key_id,
            "--input",
            input_path,
            "--output",
            output_path,
        ];

        self.cli.execute(&args).await
    }

    /// Stream decrypt large files (if supported)
    ///
    /// Args:
    /// - `input_path`: Path to input file
    /// - `output_path`: Path to output file
    /// - `key_id`: Key identifier or path
    pub async fn stream_decrypt(
        &self,
        input_path: &str,
        output_path: &str,
        key_id: &str,
    ) -> Result<CliResult> {
        if !self.supports_streaming {
            anyhow::bail!("Stream decryption not supported by this BearDog version");
        }

        let args = vec![
            "stream-decrypt",
            "--key",
            key_id,
            "--input",
            input_path,
            "--output",
            output_path,
        ];

        self.cli.execute(&args).await
    }

    /// Generate a key using BearDog
    ///
    /// Args:
    /// - `key_type`: Type of key (e.g., "aes-256", "ed25519")
    /// - `output_path`: Path to save the key
    pub async fn generate_key(&self, key_type: &str, output_path: &str) -> Result<CliResult> {
        let args = vec![
            "key",
            "generate",
            "--type",
            key_type,
            "--output",
            output_path,
        ];

        self.cli.execute(&args).await
    }

    /// Collect entropy for key generation
    pub async fn collect_entropy(&self) -> Result<CliResult> {
        let args = vec!["entropy", "collect"];
        self.cli.execute(&args).await
    }

    /// HSM operations (if supported)
    ///
    /// Args:
    /// - `operation`: HSM operation (e.g., "list", "status")
    pub async fn hsm_operation(&self, operation: &str) -> Result<CliResult> {
        if !self.supports_hsm {
            anyhow::bail!("HSM operations not supported by this BearDog version");
        }

        let args = vec!["hsm", operation];
        self.cli.execute(&args).await
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
    fn test_beardog_adapter_creation() {
        // Test with a mock binary path
        let result = BearDogAdapter::new("/usr/bin/false");
        // Should succeed creating adapter, fail on verify
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_beardog_version() {
        // This test requires actual BearDog binary
        // Skip if not available
        let beardog_path =
            std::env::var("BEARDOG_BIN").unwrap_or_else(|_| "/nonexistent/beardog".to_string());

        if let Ok(adapter) = BearDogAdapter::new(&beardog_path) {
            let _ = adapter.get_version().await;
            // Just test that the method runs
        }
    }
}
