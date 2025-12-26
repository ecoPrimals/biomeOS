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
    /// # Future Implementation
    /// This will use mDNS to discover services on the local network.
    /// Services advertise themselves using mDNS with their service type.
    async fn discover_via_mdns(&self) -> Result<String> {
        // TODO: Implement mDNS discovery
        // - Use mdns crate
        // - Query for _biomeos._tcp.local or similar
        // - Return first responsive service

        tracing::trace!("mDNS discovery not yet implemented");
        Err(anyhow::anyhow!("mDNS discovery not yet implemented"))
    }

    /// Discover via UDP broadcast
    ///
    /// # Future Implementation
    /// This will broadcast a discovery request on the local network
    /// and wait for responses from universal adapters.
    async fn discover_via_broadcast(&self) -> Result<String> {
        // TODO: Implement broadcast discovery
        // - Send UDP broadcast to 255.255.255.255
        // - Listen for responses with service info
        // - Return first valid response

        tracing::trace!("Broadcast discovery not yet implemented");
        Err(anyhow::anyhow!("Broadcast discovery not yet implemented"))
    }

    /// Discover via multicast
    ///
    /// # Future Implementation
    /// This will use IP multicast to discover services without
    /// flooding the entire network with broadcasts.
    async fn discover_via_multicast(&self) -> Result<String> {
        // TODO: Implement multicast discovery
        // - Join multicast group (e.g., 224.0.0.251 for mDNS)
        // - Send discovery query
        // - Process responses

        tracing::trace!("Multicast discovery not yet implemented");
        Err(anyhow::anyhow!("Multicast discovery not yet implemented"))
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
