//! STUN Extension - Optional Self-Hosted STUN Support
//!
//! **OPTIONAL**: biomeOS works without this using public STUN
//!
//! This module provides optional integration with self-hosted STUN servers
//! (like coturn) while maintaining fallback to public STUN for robustness.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    STUN Extension (Optional)                        │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │                                                                     │
//! │  check_self_hosted() ──► If available: Use self-hosted             │
//! │         │                                                           │
//! │         └──► If unavailable: Fallback to public STUN               │
//! │                                                                     │
//! │  Benefits of self-hosted:                                           │
//! │  ├── Zero external metadata exposure                               │
//! │  ├── Family-only access (future)                                   │
//! │  └── Eliminates dependency on Google/Cloudflare                    │
//! │                                                                     │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use biomeos_core::stun_extension::StunExtension;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ext = StunExtension::new();
//! let songbird_socket = "/run/user/1000/biomeos/songbird.sock";
//!
//! // Check if self-hosted STUN is available
//! if ext.check_availability(songbird_socket).await {
//!     println!("Self-hosted STUN available");
//! }
//!
//! // Get public address with automatic fallback
//! let addr = ext.get_public_address_with_fallback(songbird_socket).await?;
//! println!("Public address: {}", addr);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// Configuration for the optional STUN extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunExtensionConfig {
    /// Enable self-hosted STUN (if available)
    pub enabled: bool,

    /// Self-hosted STUN server address
    pub self_hosted_address: Option<String>,

    /// Timeout for self-hosted STUN check (milliseconds)
    pub check_timeout_ms: u64,

    /// Fallback to public STUN if self-hosted unavailable
    pub fallback_to_public: bool,

    /// Public STUN servers for fallback
    pub public_servers: Vec<String>,
}

impl Default for StunExtensionConfig {
    fn default() -> Self {
        // Self-hosted STUN takes priority if configured
        let self_hosted_address = std::env::var("BIOMEOS_STUN_SERVER").ok();
        
        // Check if public STUN is disabled (sovereignty mode)
        let no_public_stun = std::env::var("BIOMEOS_NO_PUBLIC_STUN")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);
        
        // Resolve public servers from environment or use defaults
        let public_servers = if no_public_stun {
            Vec::new()
        } else {
            // BIOMEOS_STUN_SERVERS overrides defaults (comma-separated list)
            std::env::var("BIOMEOS_STUN_SERVERS")
                .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
                .unwrap_or_else(|_| vec![
                    "stun.l.google.com:19302".to_string(),
                    "stun.cloudflare.com:3478".to_string(),
                    "stun.nextcloud.com:3478".to_string(),
                ])
        };
        
        Self {
            enabled: true,
            self_hosted_address,
            check_timeout_ms: 2000,
            fallback_to_public: !no_public_stun,
            public_servers,
        }
    }
}

/// STUN Extension for optional self-hosted STUN support
pub struct StunExtension {
    config: StunExtensionConfig,
    /// Cached availability status
    self_hosted_available: Option<bool>,
}

impl StunExtension {
    /// Create new STUN extension with default config
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: StunExtensionConfig::default(),
            self_hosted_available: None,
        }
    }

    /// Create with custom config
    #[must_use]
    pub fn with_config(config: StunExtensionConfig) -> Self {
        Self {
            config,
            self_hosted_available: None,
        }
    }

    /// Check if self-hosted STUN is available
    ///
    /// This performs a quick health check against the configured
    /// self-hosted STUN server. Caches result for subsequent calls.
    pub async fn check_availability(&mut self, songbird_socket: &str) -> bool {
        if let Some(available) = self.self_hosted_available {
            return available;
        }

        let available = self.probe_self_hosted(songbird_socket).await;
        self.self_hosted_available = Some(available);
        available
    }

    /// Probe self-hosted STUN server
    async fn probe_self_hosted(&self, songbird_socket: &str) -> bool {
        let address = match &self.config.self_hosted_address {
            Some(addr) => addr.clone(),
            None => {
                // Try to discover from known_beacons or use default
                self.discover_self_hosted_address().unwrap_or_else(|| {
                    // Default: localhost coturn
                    "127.0.0.1:3478".to_string()
                })
            }
        };

        // Try to get public address from self-hosted STUN
        match self.query_stun(&address, songbird_socket).await {
            Ok(_) => {
                tracing::info!("✅ Self-hosted STUN available at {}", address);
                true
            }
            Err(e) => {
                tracing::debug!("Self-hosted STUN not available at {}: {}", address, e);
                false
            }
        }
    }

    /// Discover self-hosted STUN address from configuration or beacons
    fn discover_self_hosted_address(&self) -> Option<String> {
        // Check environment variable
        if let Ok(addr) = std::env::var("BIOMEOS_STUN_SERVER") {
            return Some(addr);
        }

        // Could also check .known_beacons.json for family STUN servers
        // For now, return None to use default

        None
    }

    /// Query a STUN server via Songbird
    async fn query_stun(
        &self,
        server: &str,
        songbird_socket: &str,
    ) -> Result<SocketAddr, StunExtensionError> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;
        use tokio::time::timeout;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "stun.get_public_address",
            "params": { "server": server },
            "id": 1
        });

        let timeout_duration = Duration::from_millis(self.config.check_timeout_ms);

        let result = timeout(timeout_duration, async {
            let mut stream = UnixStream::connect(songbird_socket).await?;

            let request_bytes = serde_json::to_vec(&request)?;
            stream.write_all(&request_bytes).await?;
            stream.shutdown().await?;

            let mut response = Vec::new();
            stream.read_to_end(&mut response).await?;

            let response: serde_json::Value = serde_json::from_slice(&response)?;

            if let Some(error) = response.get("error") {
                return Err(StunExtensionError::StunError(error.to_string()));
            }

            let public_address = response["result"]["public_address"]
                .as_str()
                .ok_or(StunExtensionError::InvalidResponse)?;

            public_address
                .parse()
                .map_err(|_| StunExtensionError::InvalidResponse)
        })
        .await;

        match result {
            Ok(Ok(addr)) => Ok(addr),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(StunExtensionError::Timeout),
        }
    }

    /// Get public address, preferring self-hosted if available
    ///
    /// # Strategy
    ///
    /// 1. If self-hosted available: use it (maximum sovereignty)
    /// 2. If not and fallback enabled: use public STUN
    /// 3. If both fail: return error
    pub async fn get_public_address_with_fallback(
        &mut self,
        songbird_socket: &str,
    ) -> Result<SocketAddr, StunExtensionError> {
        // Try self-hosted first
        if self.config.enabled {
            if let Some(addr) = &self.config.self_hosted_address {
                match self.query_stun(addr, songbird_socket).await {
                    Ok(public_addr) => {
                        tracing::info!(
                            "📡 Public address via self-hosted STUN: {}",
                            public_addr
                        );
                        return Ok(public_addr);
                    }
                    Err(e) => {
                        tracing::warn!("Self-hosted STUN failed: {}", e);
                    }
                }
            }
        }

        // Fallback to public STUN
        if self.config.fallback_to_public {
            for server in &self.config.public_servers {
                match self.query_stun(server, songbird_socket).await {
                    Ok(public_addr) => {
                        tracing::info!(
                            "📡 Public address via public STUN ({}): {}",
                            server,
                            public_addr
                        );
                        return Ok(public_addr);
                    }
                    Err(e) => {
                        tracing::debug!("Public STUN {} failed: {}", server, e);
                    }
                }
            }
        }

        Err(StunExtensionError::AllServersFailed)
    }

    /// Clear cached availability status (forces re-check)
    pub fn clear_cache(&mut self) {
        self.self_hosted_available = None;
    }

    /// Get current configuration
    #[must_use]
    pub fn config(&self) -> &StunExtensionConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: StunExtensionConfig) {
        self.config = config;
        self.clear_cache();
    }
}

impl Default for StunExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from the STUN extension
#[derive(Debug, thiserror::Error)]
pub enum StunExtensionError {
    #[error("STUN query timed out")]
    Timeout,

    #[error("STUN error: {0}")]
    StunError(String),

    #[error("Invalid response from STUN server")]
    InvalidResponse,

    #[error("All STUN servers failed")]
    AllServersFailed,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StunExtensionConfig::default();
        assert!(config.enabled);
        assert!(config.fallback_to_public);
        assert!(!config.public_servers.is_empty());
    }

    #[test]
    fn test_extension_creation() {
        let ext = StunExtension::new();
        assert!(ext.self_hosted_available.is_none());
    }

    #[test]
    fn test_clear_cache() {
        let mut ext = StunExtension::new();
        ext.self_hosted_available = Some(true);
        ext.clear_cache();
        assert!(ext.self_hosted_available.is_none());
    }
}
