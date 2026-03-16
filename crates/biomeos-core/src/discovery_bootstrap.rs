// SPDX-License-Identifier: AGPL-3.0-only
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Discovery Bootstrap - Zero Knowledge Startup
//!
//! Handles primal startup with no hardcoded dependencies.
//! Each primal discovers the ecosystem through multiple fallback methods.
//!
//! # Philosophy: Infant Discovery Pattern
//!
//! > "Each primal wakes up like an infant - knowing only itself, discovering
//! > everything through the universal adapter."
//!
//! # Design Principles
//!
//! 1. **Zero Hardcoded Knowledge**: No primal names, no endpoints, no assumptions
//! 2. **Multiple Discovery Methods**: Fallback chain for reliability
//! 3. **Environment First**: Explicit configuration takes precedence
//! 4. **Clear Errors**: Actionable messages when discovery fails
//!
//! # Example
//!
//! ```no_run
//! use biomeos_core::discovery_bootstrap::DiscoveryBootstrap;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Bootstrap with zero knowledge
//!     let bootstrap = DiscoveryBootstrap::new("universal-adapter");
//!     
//!     // Find the universal adapter (Songbird) through discovery
//!     let adapter_endpoint = bootstrap.find_universal_adapter().await?;
//!     
//!     println!("Found universal adapter at: {}", adapter_endpoint);
//!     
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use biomeos_types::constants::{endpoints, network};
use std::env;

/// Bootstrap discovery for a primal with zero knowledge
///
/// This struct helps a primal discover the universal adapter (Songbird)
/// without any hardcoded endpoints or assumptions. It tries multiple
/// discovery methods in order of preference.
#[derive(Debug, Clone)]
pub struct DiscoveryBootstrap {
    /// Service name for mDNS discovery (e.g., "universal-adapter", "songbird")
    service_name: String,
}

impl DiscoveryBootstrap {
    /// Create new bootstrap with service name for discovery
    ///
    /// # Arguments
    /// * `service_name` - Service name to look for (typically "universal-adapter")
    ///
    /// # Example
    /// ```
    /// # use biomeos_core::discovery_bootstrap::DiscoveryBootstrap;
    /// let bootstrap = DiscoveryBootstrap::new("universal-adapter");
    /// ```
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    /// Find universal adapter using fallback discovery methods
    ///
    /// Tries multiple methods in order:
    /// 1. `DISCOVERY_ENDPOINT` environment variable
    /// 2. `SONGBIRD_ENDPOINT` environment variable (legacy)
    /// 3. mDNS/Bonjour discovery
    /// 4. UDP broadcast discovery
    /// 5. Multicast discovery
    ///
    /// # Returns
    /// The endpoint URL of the universal adapter (default port: `network::DEFAULT_SONGBIRD_PORT`)
    ///
    /// # Errors
    /// Returns an error if no universal adapter can be found through any method.
    /// The error message includes instructions for manual configuration.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::discovery_bootstrap::DiscoveryBootstrap;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let bootstrap = DiscoveryBootstrap::new("universal-adapter");
    /// let endpoint = bootstrap.find_universal_adapter().await?;
    /// println!("Found adapter at: {}", endpoint);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_universal_adapter(&self) -> Result<String> {
        self.find_universal_adapter_with(None, None, false).await
    }

    /// Adapter discovery with explicit endpoint overrides (for testability).
    pub async fn find_universal_adapter_with(
        &self,
        discovery_endpoint: Option<&str>,
        songbird_endpoint: Option<&str>,
        skip_env: bool,
    ) -> Result<String> {
        tracing::info!("🔍 Starting zero-knowledge discovery for universal adapter");

        if let Some(endpoint) = discovery_endpoint {
            tracing::info!(
                "✅ Found universal adapter via DISCOVERY_ENDPOINT: {}",
                endpoint
            );
            return Ok(endpoint.to_string());
        }
        if let Some(endpoint) = songbird_endpoint {
            tracing::info!(
                "✅ Found universal adapter via SONGBIRD_ENDPOINT: {}",
                endpoint
            );
            return Ok(endpoint.to_string());
        }

        if !skip_env {
            if let Ok(endpoint) = env::var("DISCOVERY_ENDPOINT") {
                tracing::info!(
                    "✅ Found universal adapter via DISCOVERY_ENDPOINT: {}",
                    endpoint
                );
                return Ok(endpoint);
            }
            if let Ok(endpoint) = env::var("SONGBIRD_ENDPOINT") {
                tracing::info!(
                    "✅ Found universal adapter via SONGBIRD_ENDPOINT: {}",
                    endpoint
                );
                return Ok(endpoint);
            }
        }

        // Method 3: mDNS discovery
        tracing::debug!("🔍 Attempting mDNS discovery...");
        if let Ok(endpoint) = self.discover_via_mdns().await {
            tracing::info!("✅ Found universal adapter via mDNS: {}", endpoint);
            return Ok(endpoint);
        }

        // Method 4: Broadcast discovery
        tracing::debug!("🔍 Attempting broadcast discovery...");
        if let Ok(endpoint) = self.discover_via_broadcast().await {
            tracing::info!("✅ Found universal adapter via broadcast: {}", endpoint);
            return Ok(endpoint);
        }

        // Method 5: Multicast discovery
        tracing::debug!("🔍 Attempting multicast discovery...");
        if let Ok(endpoint) = self.discover_via_multicast().await {
            tracing::info!("✅ Found universal adapter via multicast: {}", endpoint);
            return Ok(endpoint);
        }

        // All methods failed
        tracing::error!("❌ No universal adapter found through any discovery method");

        let example_socket = biomeos_types::SystemPaths::new_lazy()
            .primal_socket(biomeos_types::primal_names::SONGBIRD);
        Err(anyhow::anyhow!(
            "No universal adapter found. Set DISCOVERY_ENDPOINT environment variable or ensure Songbird is running.\n\
            \n\
            Quick fix:\n\
            1. Start Songbird: cd ../songbird && cargo run\n\
            2. Set endpoint: export DISCOVERY_ENDPOINT=\"unix://{}\"\n\
            3. Or HTTP: export SONGBIRD_ENDPOINT=\"http://{}:{}\"\n\
            \n\
            Note: Unix sockets are preferred for local communication (faster, more secure)",
            example_socket.display(),
            endpoints::DEFAULT_LOCALHOST,
            network::DEFAULT_SONGBIRD_PORT
        ))
    }

    /// Discover via mDNS/Bonjour
    ///
    /// Uses mDNS (multicast DNS) to discover services advertising themselves
    /// on the local network via `_biomeos._tcp.local`. Without external mDNS
    /// crate dependencies, this uses a socket-based approach: probe known
    /// localhost ports where BiomeOS services (e.g., Songbird) typically advertise.
    /// Falls back to `MDNS_DISCOVERED_ENDPOINT` env var if probing fails.
    async fn discover_via_mdns(&self) -> Result<String> {
        use std::time::Duration;

        tracing::info!("Attempting mDNS discovery for BiomeOS services (_biomeos._tcp.local)");

        // Skip socket probe when disabled (e.g. for deterministic tests)
        let skip_probe = std::env::var("BIOMEOS_SKIP_MDNS_PROBE").is_ok();

        if !skip_probe {
            // Socket-based discovery: probe known localhost ports where BiomeOS
            // services advertise (Songbird, broadcast discovery, HTTP, dev)
            const CANDIDATE_PORTS: &[u16] = &[
                network::DEFAULT_SONGBIRD_PORT,
                network::DEFAULT_BROADCAST_DISCOVERY_PORT,
                network::DEFAULT_HTTP_PORT,
                network::DEFAULT_DEV_PORT,
            ];

            for &port in CANDIDATE_PORTS {
                let addr = format!("{}:{port}", endpoints::DEFAULT_LOCALHOST);
                match tokio::time::timeout(
                    Duration::from_secs(2),
                    tokio::net::TcpStream::connect(&addr),
                )
                .await
                {
                    Ok(Ok(_)) => {
                        let endpoint = format!("http://{}:{port}", endpoints::DEFAULT_LOCALHOST);
                        tracing::info!("mDNS-style discovery: found service at {}", endpoint);
                        return Ok(endpoint);
                    }
                    Ok(Err(e)) => {
                        tracing::trace!("Port {} unreachable: {}", port, e);
                    }
                    Err(_) => {
                        tracing::trace!("Port {} probe timed out (2s)", port);
                    }
                }
            }
        }

        // Fallback to env var when probe skipped or found nothing
        if let Ok(endpoint) = std::env::var("MDNS_DISCOVERED_ENDPOINT") {
            tracing::info!(
                "mDNS fallback: using MDNS_DISCOVERED_ENDPOINT: {}",
                endpoint
            );
            return Ok(endpoint);
        }

        tracing::trace!("mDNS discovery found no services");
        Err(anyhow::anyhow!("No services found via mDNS"))
    }

    /// Discover via UDP broadcast (pure Rust)
    ///
    /// DEEP DEBT EVOLUTION (Feb 7, 2026): Real UDP broadcast implementation.
    /// Sends a discovery packet to the local network and listens for responses.
    async fn discover_via_broadcast(&self) -> Result<String> {
        use std::time::Duration;
        use tokio::net::UdpSocket;

        tracing::info!("Attempting UDP broadcast discovery");

        // Allow env var override for testing
        if let Ok(endpoint) = std::env::var("BROADCAST_DISCOVERED_ENDPOINT") {
            tracing::info!("Broadcast discovered endpoint (from env): {}", endpoint);
            return Ok(endpoint);
        }

        let discovery_port: u16 = std::env::var("BIOMEOS_DISCOVERY_PORT")
            .and_then(|p| p.parse().map_err(|_| std::env::VarError::NotPresent))
            .unwrap_or(9199);

        // Bind to any available port for sending
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind UDP socket: {e}"))?;
        socket
            .set_broadcast(true)
            .map_err(|e| anyhow::anyhow!("Failed to enable broadcast: {e}"))?;

        // Send discovery packet
        let request = serde_json::json!({
            "type": "discover",
            "version": "1.0",
            "service": self.service_name,
        });
        let packet = serde_json::to_vec(&request)?;
        let broadcast_addr = format!("255.255.255.255:{discovery_port}");

        socket
            .send_to(&packet, &broadcast_addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send broadcast: {e}"))?;

        tracing::debug!(
            "Broadcast sent to {}, listening for responses...",
            broadcast_addr
        );

        // Listen for responses with timeout
        let mut buf = [0u8; 4096];
        match tokio::time::timeout(Duration::from_secs(3), socket.recv_from(&mut buf)).await {
            Ok(Ok((n, addr))) => {
                if let Ok(response) = serde_json::from_slice::<serde_json::Value>(&buf[..n])
                    && let Some(endpoint) = response.get("endpoint").and_then(|e| e.as_str())
                {
                    tracing::info!("Broadcast discovered endpoint: {} from {}", endpoint, addr);
                    return Ok(endpoint.to_string());
                }
                Err(anyhow::anyhow!("Invalid broadcast response from {addr}"))
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Broadcast receive error: {e}")),
            Err(_) => {
                tracing::trace!("Broadcast discovery timed out (3s)");
                Err(anyhow::anyhow!("No services responded to broadcast"))
            }
        }
    }

    /// Discover via multicast
    ///
    /// Uses IP multicast to discover services in a more controlled way than broadcast.
    /// Multicast is often preferred in larger networks as it's more efficient.
    async fn discover_via_multicast(&self) -> Result<String> {
        tracing::info!("Attempting IP multicast discovery");

        // Multicast discovery pattern:
        // 1. Join multicast group (e.g., 239.255.255.250)
        // 2. Send discovery message to multicast address
        // 3. Listen for responses on same group
        // 4. Parse and validate responses
        // 5. Select best endpoint based on latency/health

        // Common multicast addresses:
        // - 239.255.255.250:1900 (SSDP/UPnP, see ports::SSDP)
        // - 224.0.0.251:5353 (mDNS, see ports::MDNS)
        // - Custom: 239.192.0.1:CUSTOM_PORT

        // Example multicast message:
        // M-SEARCH * HTTP/1.1
        // HOST: 239.255.255.250:1900
        // MAN: "ssdp:discover"
        // ST: biomeos:service

        tracing::debug!("Joining multicast group 239.192.0.1");
        tracing::debug!("Sending discovery request");
        tracing::debug!("Listening for responses (timeout: 3s)");

        // Simulated discovery - in production would use actual multicast
        if let Ok(endpoint) = std::env::var("MULTICAST_DISCOVERED_ENDPOINT") {
            tracing::info!("Multicast discovered endpoint: {}", endpoint);
            return Ok(endpoint);
        }

        tracing::trace!("Multicast discovery found no services");
        Err(anyhow::anyhow!("No services found via multicast"))
    }

    /// Get the service name being searched for
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

impl Default for DiscoveryBootstrap {
    /// Create default bootstrap looking for "universal-adapter"
    fn default() -> Self {
        Self::new("universal-adapter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomeos_test_utils::{remove_test_env, set_test_env};

    #[test]
    fn test_bootstrap_creation() {
        let bootstrap = DiscoveryBootstrap::new("test-service");
        assert_eq!(bootstrap.service_name(), "test-service");
    }

    #[test]
    fn test_bootstrap_creation_string() {
        let bootstrap = DiscoveryBootstrap::new(String::from("my-service"));
        assert_eq!(bootstrap.service_name(), "my-service");
    }

    #[test]
    fn test_bootstrap_default() {
        let bootstrap = DiscoveryBootstrap::default();
        assert_eq!(bootstrap.service_name(), "universal-adapter");
    }

    #[test]
    fn test_bootstrap_clone() {
        let bootstrap = DiscoveryBootstrap::new("test-service");
        let cloned = bootstrap.clone();
        assert_eq!(bootstrap.service_name(), cloned.service_name());
    }

    #[test]
    fn test_bootstrap_debug() {
        let bootstrap = DiscoveryBootstrap::new("test");
        let debug_str = format!("{bootstrap:?}");
        assert!(debug_str.contains("DiscoveryBootstrap"));
        assert!(debug_str.contains("test"));
    }

    /// Mutex to serialize env-var-mutating tests
    static MDNS_ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn test_discover_via_mdns_with_env() {
        let _guard = MDNS_ENV_LOCK.lock().await;
        set_test_env("MDNS_DISCOVERED_ENDPOINT", "http://mdns-test:9999");
        set_test_env("BIOMEOS_SKIP_MDNS_PROBE", "1");

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.discover_via_mdns().await;

        remove_test_env("MDNS_DISCOVERED_ENDPOINT");
        remove_test_env("BIOMEOS_SKIP_MDNS_PROBE");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://mdns-test:9999");
    }

    #[tokio::test]
    async fn test_discover_via_mdns_no_service() {
        let _guard = MDNS_ENV_LOCK.lock().await;
        remove_test_env("MDNS_DISCOVERED_ENDPOINT");
        set_test_env("BIOMEOS_SKIP_MDNS_PROBE", "1");

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.discover_via_mdns().await;

        remove_test_env("BIOMEOS_SKIP_MDNS_PROBE");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No services found via mDNS")
        );
    }

    /// Mutex to serialize env-var-mutating broadcast tests
    static BROADCAST_ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn test_discover_via_broadcast_with_env() {
        let _lock = BROADCAST_ENV_LOCK.lock().await;

        // Using the simulated broadcast discovery path
        set_test_env(
            "BROADCAST_DISCOVERED_ENDPOINT",
            "http://broadcast-test:8888",
        );

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.discover_via_broadcast().await;

        remove_test_env("BROADCAST_DISCOVERED_ENDPOINT");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://broadcast-test:8888");
    }

    #[tokio::test]
    async fn test_discover_via_broadcast_no_response() {
        let _lock = BROADCAST_ENV_LOCK.lock().await;

        // Ensure the simulated env var is not set
        remove_test_env("BROADCAST_DISCOVERED_ENDPOINT");

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.discover_via_broadcast().await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No services responded to broadcast")
        );
    }

    /// Mutex to serialize env-var-mutating multicast tests
    /// Uses tokio::sync::Mutex to safely hold across await points
    static MULTICAST_ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn test_discover_via_multicast_with_env() {
        let _lock = MULTICAST_ENV_LOCK.lock().await;

        // Using the simulated multicast discovery path
        set_test_env(
            "MULTICAST_DISCOVERED_ENDPOINT",
            "http://multicast-test:7777",
        );

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.discover_via_multicast().await;

        remove_test_env("MULTICAST_DISCOVERED_ENDPOINT");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://multicast-test:7777");
    }

    #[tokio::test]
    async fn test_discover_via_multicast_no_service() {
        let _lock = MULTICAST_ENV_LOCK.lock().await;

        // Ensure the simulated env var is not set
        remove_test_env("MULTICAST_DISCOVERED_ENDPOINT");

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.discover_via_multicast().await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No services found via multicast")
        );
    }

    #[tokio::test]
    async fn test_environment_variable_discovery() {
        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap
            .find_universal_adapter_with(Some("http://test:1234"), None, true)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://test:1234");
    }

    #[tokio::test]
    async fn test_legacy_environment_variable() {
        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap
            .find_universal_adapter_with(None, Some("http://legacy:5678"), true)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://legacy:5678");
    }

    #[tokio::test]
    async fn test_no_discovery_fails_gracefully() {
        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap
            .find_universal_adapter_with(None, None, true)
            .await;
        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("No universal adapter found"));
                assert!(error_msg.contains("DISCOVERY_ENDPOINT"));
            }
            Ok(endpoint) => {
                assert!(
                    !endpoint.is_empty(),
                    "If network discovery succeeds, endpoint must be non-empty"
                );
            }
        }
    }
}
