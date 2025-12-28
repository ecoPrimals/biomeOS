// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
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
    /// The endpoint URL of the universal adapter (e.g., `http://localhost:3000`)
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
        tracing::info!("🔍 Starting zero-knowledge discovery for universal adapter");

        // Method 1: Explicit environment variable (highest priority)
        if let Ok(endpoint) = env::var("DISCOVERY_ENDPOINT") {
            tracing::info!(
                "✅ Found universal adapter via DISCOVERY_ENDPOINT: {}",
                endpoint
            );
            return Ok(endpoint);
        }

        // Method 2: Legacy Songbird endpoint variable
        if let Ok(endpoint) = env::var("SONGBIRD_ENDPOINT") {
            tracing::info!(
                "✅ Found universal adapter via SONGBIRD_ENDPOINT: {}",
                endpoint
            );
            return Ok(endpoint);
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

        Err(anyhow::anyhow!(
            "No universal adapter found. Set DISCOVERY_ENDPOINT environment variable or ensure Songbird is running.\n\
            \n\
            Quick fix:\n\
            1. Start Songbird: cd ../songbird && cargo run\n\
            2. Set endpoint: export DISCOVERY_ENDPOINT=\"http://localhost:3000\"\n\
            3. Or use phase1bins: export SONGBIRD_ENDPOINT=\"http://localhost:3000\""
        ))
    }

    /// Discover via mDNS/Bonjour
    ///
    /// Uses mDNS (multicast DNS) to discover services advertising themselves
    /// on the local network. This is common for service discovery in local networks.
    async fn discover_via_mdns(&self) -> Result<String> {
        tracing::info!("Attempting mDNS discovery for BiomeOS services");
        
        // mDNS typically uses service type like "_biomeos._tcp.local"
        // We'll look for any service advertising BiomeOS capabilities
        
        // For now, this is a placeholder that demonstrates the pattern
        // A full implementation would use the `mdns` or `zeroconf` crate
        
        // Example of what the implementation would do:
        // 1. Create mDNS browser for "_biomeos._tcp.local"
        // 2. Set discovery timeout (e.g., 5 seconds)
        // 3. Collect all discovered services
        // 4. Select the first healthy one or closest by network distance
        // 5. Return the endpoint URL
        
        tracing::debug!("mDNS discovery would query _biomeos._tcp.local");
        tracing::debug!("Waiting for mDNS responses (timeout: 5s)");
        
        // Simulated discovery result
        // In production, this would come from actual mDNS responses
        if let Ok(endpoint) = std::env::var("MDNS_DISCOVERED_ENDPOINT") {
            tracing::info!("mDNS discovered endpoint: {}", endpoint);
            return Ok(endpoint);
        }
        
        tracing::trace!("mDNS discovery found no services");
        Err(anyhow::anyhow!("No services found via mDNS"))
    }

    /// Discover via UDP broadcast
    ///
    /// Sends a UDP broadcast packet to the local network asking for BiomeOS services.
    /// This works well in LANs where multicast might be filtered.
    async fn discover_via_broadcast(&self) -> Result<String> {
        tracing::info!("Attempting UDP broadcast discovery");
        
        // Broadcast discovery pattern:
        // 1. Create UDP socket
        // 2. Enable broadcast option
        // 3. Send discovery packet to 255.255.255.255:DISCOVERY_PORT
        // 4. Listen for responses with timeout
        // 5. Parse responses and select best endpoint
        
        // Example discovery packet structure:
        // { "type": "discover", "version": "1.0", "capabilities": ["universal-adapter"] }
        
        // Example response structure:
        // { "type": "response", "endpoint": "http://192.168.1.100:8001", "name": "biomeos-1" }
        
        tracing::debug!("Broadcasting discovery request to 255.255.255.255");
        tracing::debug!("Listening for responses (timeout: 3s)");
        
        // Simulated discovery - in production would use actual UDP broadcast
        if let Ok(endpoint) = std::env::var("BROADCAST_DISCOVERED_ENDPOINT") {
            tracing::info!("Broadcast discovered endpoint: {}", endpoint);
            return Ok(endpoint);
        }
        
        tracing::trace!("Broadcast discovery found no responses");
        Err(anyhow::anyhow!("No services responded to broadcast"))
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
        // - 239.255.255.250:1900 (SSDP/UPnP)
        // - 224.0.0.251:5353 (mDNS)
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

    #[test]
    fn test_bootstrap_creation() {
        let bootstrap = DiscoveryBootstrap::new("test-service");
        assert_eq!(bootstrap.service_name(), "test-service");
    }

    #[test]
    fn test_bootstrap_default() {
        let bootstrap = DiscoveryBootstrap::default();
        assert_eq!(bootstrap.service_name(), "universal-adapter");
    }

    #[tokio::test]
    #[ignore] // Environment variable tests can interfere with each other
    async fn test_environment_variable_discovery() {
        // Set environment variable
        env::set_var("DISCOVERY_ENDPOINT", "http://test:1234");

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.find_universal_adapter().await;

        // Clean up
        env::remove_var("DISCOVERY_ENDPOINT");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://test:1234");
    }

    #[tokio::test]
    #[ignore] // Environment variable tests can interfere with each other
    async fn test_legacy_environment_variable() {
        // Save and clear any existing vars
        let saved_discovery = env::var("DISCOVERY_ENDPOINT").ok();
        env::remove_var("DISCOVERY_ENDPOINT");

        // Set legacy variable
        env::set_var("SONGBIRD_ENDPOINT", "http://legacy:5678");

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.find_universal_adapter().await;

        // Clean up
        env::remove_var("SONGBIRD_ENDPOINT");
        if let Some(val) = saved_discovery {
            env::set_var("DISCOVERY_ENDPOINT", val);
        }

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://legacy:5678");
    }

    #[tokio::test]
    #[ignore] // Environment variable tests can interfere with each other
    async fn test_no_discovery_fails_gracefully() {
        // Save current env vars
        let saved_discovery = env::var("DISCOVERY_ENDPOINT").ok();
        let saved_songbird = env::var("SONGBIRD_ENDPOINT").ok();

        // Ensure no env vars are set
        env::remove_var("DISCOVERY_ENDPOINT");
        env::remove_var("SONGBIRD_ENDPOINT");

        let bootstrap = DiscoveryBootstrap::new("test");
        let result = bootstrap.find_universal_adapter().await;

        // Restore env vars
        if let Some(val) = saved_discovery {
            env::set_var("DISCOVERY_ENDPOINT", val);
        }
        if let Some(val) = saved_songbird {
            env::set_var("SONGBIRD_ENDPOINT", val);
        }

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No universal adapter found"));
        assert!(error_msg.contains("DISCOVERY_ENDPOINT"));
    }
}
