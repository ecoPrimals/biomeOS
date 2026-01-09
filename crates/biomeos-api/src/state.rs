//! Modern application state with builder pattern
//!
//! Provides a clean, type-safe way to configure the API server.
//!
//! # Architecture
//!
//! The [`AppState`] holds all shared state for the API server, including
//! the discovery service and configuration. It's built using the builder
//! pattern for ergonomic and type-safe initialization.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use biomeos_api::{AppState, Config};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create with defaults
//! let state = AppState::builder()
//!     .config_from_env()
//!     .build_with_defaults()?;
//!
//! println!("Standalone mode: {}", state.is_standalone_mode());
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust,no_run
//! use biomeos_api::{AppState, Config};
//! use biomeos_core::CompositeDiscovery;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = Config::default();
//! config.mock_mode = true;
//!
//! let discovery = CompositeDiscovery::new();
//!
//! let state = AppState::builder()
//!     .discovery(discovery)
//!     .config(config)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Design Philosophy
//!
//! - **Type Safety**: Builder pattern prevents invalid configuration
//! - **Encapsulation**: State fields are private, accessed via methods
//! - **Defaults**: Sensible defaults with `build_with_defaults()`
//! - **Environment**: Load configuration from environment variables

use biomeos_core::{CompositeDiscovery, PrimalDiscovery};
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;

/// Default bind address (const to avoid parsing at runtime)
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";

/// Application state (shared across handlers)
#[derive(Clone)]
pub struct AppState {
    discovery: Arc<dyn PrimalDiscovery>,
    config: Config,
}

impl AppState {
    /// Create a new builder
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::default()
    }
    
    /// Get the discovery service
    pub fn discovery(&self) -> &dyn PrimalDiscovery {
        &*self.discovery
    }
    
    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Check if standalone mode is enabled (graceful degradation)
    pub fn is_standalone_mode(&self) -> bool {
        self.config.standalone_mode
    }
    
    /// Legacy alias for backward compatibility
    #[deprecated(since = "0.2.0", note = "Use is_standalone_mode() instead")]
    pub fn is_mock_mode(&self) -> bool {
        self.is_standalone_mode()
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Enable standalone mode (graceful degradation when primals unavailable)
    ///
    /// When `true`, the API provides standalone fallback responses for demos
    /// and development without requiring full primal infrastructure.
    ///
    /// **Production**: Set to `false` (default) for full primal discovery
    /// **Development**: Set to `true` for standalone operation
    ///
    /// Set via `BIOMEOS_STANDALONE_MODE=true` environment variable.
    pub standalone_mode: bool,
    
    /// Server bind address
    pub bind_addr: SocketAddr,
    
    /// Request timeout
    pub request_timeout: std::time::Duration,
    
    /// Enable CORS
    pub enable_cors: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            standalone_mode: false, // Production default: require primals
            bind_addr: DEFAULT_BIND_ADDR
                .parse()
                .expect("DEFAULT_BIND_ADDR is a valid socket address"),
            request_timeout: std::time::Duration::from_secs(30),
            enable_cors: true,
        }
    }
}

impl Config {
    /// Load configuration from environment
    pub fn from_env() -> Self {
        // Support both old and new env var names for backward compatibility
        let standalone_mode = std::env::var("BIOMEOS_STANDALONE_MODE")
            .or_else(|_| std::env::var("BIOMEOS_STANDALONE_MODE"))
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);
        
        let bind_addr = std::env::var("BIOMEOS_API_BIND_ADDR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                DEFAULT_BIND_ADDR
                    .parse()
                    .expect("DEFAULT_BIND_ADDR is a valid socket address")
            });
        
        Self {
            standalone_mode,
            bind_addr,
            ..Default::default()
        }
    }
}

/// Builder for AppState
#[derive(Default)]
pub struct AppStateBuilder {
    discovery: Option<Arc<dyn PrimalDiscovery>>,
    config: Option<Config>,
}

impl AppStateBuilder {
    /// Set the discovery service
    pub fn discovery(mut self, discovery: impl PrimalDiscovery + 'static) -> Self {
        self.discovery = Some(Arc::new(discovery));
        self
    }
    
    /// Set the discovery service from an Arc
    pub fn discovery_arc(mut self, discovery: Arc<dyn PrimalDiscovery>) -> Self {
        self.discovery = Some(discovery);
        self
    }
    
    /// Set the configuration
    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Load config from environment
    pub fn config_from_env(mut self) -> Self {
        self.config = Some(Config::from_env());
        self
    }
    
    /// Build the AppState
    pub fn build(self) -> Result<AppState, BuildError> {
        let discovery = self
            .discovery
            .ok_or(BuildError::MissingDiscovery)?;
        
        let config = self.config.unwrap_or_default();
        
        Ok(AppState { discovery, config })
    }
    
    /// Build with default local discovery if none provided
    pub fn build_with_defaults(self) -> Result<AppState, BuildError> {
        let discovery = match self.discovery {
            Some(d) => d,
            None => {
                tracing::info!("📡 Creating default local discovery (BearDog + Songbird)");
                let sources = biomeos_core::create_local_discovery()
                    .map_err(|e| BuildError::DiscoveryError(e.to_string()))?;
                
                let mut composite = CompositeDiscovery::new();
                for source in sources {
                    composite = composite.add_boxed_source(source);
                }
                
                Arc::new(composite)
            }
        };
        
        let config = self.config.unwrap_or_default();
        
        Ok(AppState { discovery, config })
    }
}

/// Builder errors
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Discovery service not configured")]
    MissingDiscovery,
    
    #[error("Discovery configuration error: {0}")]
    DiscoveryError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomeos_core::{DiscoveryResult, HealthStatus};
    use biomeos_types::PrimalId;
    
    struct MockDiscovery;
    
    #[async_trait::async_trait]
    impl PrimalDiscovery for MockDiscovery {
        async fn discover(
            &self,
            _endpoint: &biomeos_types::Endpoint,
        ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
            Err(biomeos_core::BiomeError::discovery_failed(
                "MockDiscovery::discover not implemented".to_string(),
                Some("test_mock".to_string())
            ))
        }
        
        async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
            Ok(vec![])
        }
        
        async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
            Ok(HealthStatus::Healthy)
        }
    }
    
    #[test]
    fn test_builder_requires_discovery() {
        let result = AppStateBuilder::default().build();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_builder_with_discovery() {
        let result = AppStateBuilder::default()
            .discovery(MockDiscovery)
            .build();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_config_from_env() {
        let config = Config::from_env();
        // Should not panic and use defaults
        assert_eq!(config.bind_addr.port(), 3000);
    }
}

