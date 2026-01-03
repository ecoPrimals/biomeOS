//! Generic primal implementations using capability-based architecture
//!
//! NO hardcoded primal names - everything discovered from environment!
//! Implements the "Infant Model" - primals only know themselves.

use std::{
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use biomeos_types::{
    error::{BiomeError, BiomeResult},
    identifiers::{Endpoint, PrimalId},
};

use crate::{
    capabilities::{Capability, PrimalConfig},
    primal_health::{HttpHealthChecker, PrimalHealthStatus},
    primal_orchestrator::ManagedPrimal,
};

/// Generic managed primal - works for ANY primal!
/// Configuration comes entirely from environment.
pub struct GenericManagedPrimal {
    id: PrimalId,
    config: PrimalConfig,
    process: Arc<Mutex<Option<Child>>>,
    health_checker: Option<HttpHealthChecker>,
}

impl GenericManagedPrimal {
    /// Create from environment (infant model - ZERO hardcoding!)
    pub fn from_env() -> BiomeResult<Self> {
        let config = PrimalConfig::from_env()?;
        let id = PrimalId::new(config.id.clone());
        
        // Only use health checker if we have an HTTP endpoint
        let health_checker = if config.http_port > 0 {
            Some(HttpHealthChecker)
        } else {
            None
        };

        info!("🌱 Created primal from environment:");
        info!("   ID: {}", id);
        info!("   Provides: {:?}", config.provides);
        info!("   Requires: {:?}", config.requires);
        info!("   HTTP Port: {}", if config.http_port == 0 { "auto".to_string() } else { config.http_port.to_string() });

        Ok(Self {
            id,
            config,
            process: Arc::new(Mutex::new(None)),
            health_checker,
        })
    }

    /// Create with explicit config (for testing or manual construction)
    pub fn with_config(config: PrimalConfig) -> BiomeResult<Self> {
        let id = PrimalId::new(config.id.clone());
        
        let health_checker = if config.http_port > 0 {
            Some(HttpHealthChecker)
        } else {
            None
        };

        Ok(Self {
            id,
            config,
            process: Arc::new(Mutex::new(None)),
            health_checker,
        })
    }
}

#[async_trait]
impl ManagedPrimal for GenericManagedPrimal {
    fn id(&self) -> &PrimalId {
        &self.id
    }

    fn provides(&self) -> &[Capability] {
        &self.config.provides
    }

    fn requires(&self) -> &[Capability] {
        &self.config.requires
    }

    async fn endpoint(&self) -> Option<Endpoint> {
        if self.config.http_port > 0 {
            let url = format!("http://127.0.0.1:{}", self.config.http_port);
            Endpoint::new(&url).ok()
        } else {
            None
        }
    }

    async fn start(&self) -> BiomeResult<()> {
        info!("🚀 Starting primal: {}", self.id);

        let mut process_guard = self.process.lock().await;
        if process_guard.is_some() {
            warn!("Primal {} already running", self.id);
            return Ok(());
        }

        // Build environment variables from config
        let mut cmd = Command::new(&self.config.binary_path);
        
        // Add HTTP port if specified
        if self.config.http_port > 0 {
            cmd.env("HTTP_PORT", self.config.http_port.to_string());
        }
        
        // Add all primal-specific environment variables
        for (key, value) in &self.config.env_config {
            cmd.env(key, value);
        }
        
        // Add capabilities for downstream discovery
        if !self.config.provides.is_empty() {
            let provides_str = self.config.provides
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            cmd.env("PRIMAL_PROVIDES", provides_str);
        }

        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                BiomeError::internal_error(format!(
                    "Failed to spawn primal {}: {}",
                    self.id, e
                ))
            })?;

        *process_guard = Some(child);

        info!("✅ Primal {} process started", self.id);
        Ok(())
    }

    async fn stop(&self) -> BiomeResult<()> {
        info!("🛑 Stopping primal: {}", self.id);

        let mut process_guard = self.process.lock().await;
        if let Some(mut child) = process_guard.take() {
            child.kill().map_err(|e| {
                BiomeError::internal_error(format!("Failed to kill {}: {}", self.id, e))
            })?;
            child.wait().map_err(|e| {
                BiomeError::internal_error(format!("Failed to wait for {}: {}", self.id, e))
            })?;
        }

        info!("✅ Primal {} stopped", self.id);
        Ok(())
    }

    async fn health_check(&self) -> BiomeResult<PrimalHealthStatus> {
        // If we have an HTTP health checker, use it
        if let Some(checker) = &self.health_checker {
            if let Some(endpoint) = self.endpoint().await {
                return checker.check_health(&self.id, &endpoint).await;
            }
        }

        // Otherwise just check if process is running
        let process_guard = self.process.lock().await;
        if process_guard.is_some() {
            Ok(PrimalHealthStatus::Healthy)
        } else {
            Err(BiomeError::not_found(format!("Primal {} not running", self.id)))
        }
    }

    fn startup_timeout(&self) -> Duration {
        // Check for custom timeout in config
        self.config
            .env_config
            .get("PRIMAL_STARTUP_TIMEOUT")
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30))
    }
}

/// Builder for constructing primals (for backward compatibility and convenience)
pub struct PrimalBuilder {
    id: Option<String>,
    binary_path: Option<String>,
    provides: Vec<Capability>,
    requires: Vec<Capability>,
    http_port: u16,
    env_vars: std::collections::HashMap<String, String>,
}

impl PrimalBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            binary_path: None,
            provides: Vec::new(),
            requires: Vec::new(),
            http_port: 0,
            env_vars: std::collections::HashMap::new(),
        }
    }

    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn binary_path(mut self, path: String) -> Self {
        self.binary_path = Some(path);
        self
    }

    pub fn provides(mut self, capabilities: Vec<Capability>) -> Self {
        self.provides = capabilities;
        self
    }

    pub fn requires(mut self, capabilities: Vec<Capability>) -> Self {
        self.requires = capabilities;
        self
    }

    pub fn http_port(mut self, port: u16) -> Self {
        self.http_port = port;
        self
    }

    pub fn env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    pub fn build(self) -> BiomeResult<Arc<GenericManagedPrimal>> {
        let config = PrimalConfig {
            id: self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            binary_path: self
                .binary_path
                .ok_or_else(|| BiomeError::configuration_error("Binary path not set".to_string()))?,
            provides: self.provides,
            requires: self.requires,
            http_port: self.http_port,
            env_config: self.env_vars,
        };

        Ok(Arc::new(GenericManagedPrimal::with_config(config)?))
    }
}

impl Default for PrimalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function: Create a security provider (e.g., BearDog-like)
pub fn create_security_provider(binary_path: String, http_port: u16) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Security])
        .requires(vec![])
        .http_port(http_port)
        .build()
}

/// Convenience function: Create a discovery orchestrator (e.g., Songbird-like)
pub fn create_discovery_orchestrator(binary_path: String) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Discovery])
        .requires(vec![Capability::Security])  // Needs a security provider
        .http_port(0)  // No HTTP port (uses UDP)
        .build()
}

/// Convenience function: Create a compute provider (e.g., Toadstool-like)
pub fn create_compute_provider(binary_path: String, http_port: u16) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Compute])
        .requires(vec![])
        .http_port(http_port)
        .build()
}

/// Convenience function: Create an AI service (e.g., Squirrel-like)
pub fn create_ai_service(binary_path: String, http_port: u16) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::AI])
        .requires(vec![Capability::Compute, Capability::Storage])
        .http_port(http_port)
        .build()
}

/// Convenience function: Create a storage provider (e.g., NestGate-like)
pub fn create_storage_provider(binary_path: String, http_port: u16) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Storage])
        .requires(vec![])
        .http_port(http_port)
        .build()
}

// Legacy compatibility (DEPRECATED - use GenericManagedPrimal instead)
pub type ManagedBearDog = GenericManagedPrimal;
pub type ManagedSongbird = GenericManagedPrimal;

#[deprecated(note = "Use PrimalBuilder::new().provides(Security).build() instead")]
pub type BearDogConfig = PrimalConfig;

#[deprecated(note = "Use PrimalBuilder::new().provides(Discovery).requires(Security).build() instead")]
pub type SongbirdConfig = PrimalConfig;

#[deprecated(note = "Use PrimalBuilder directly instead")]
pub type TowerBuilder = PrimalBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let primal = PrimalBuilder::new()
            .id("test-primal".to_string())
            .binary_path("/bin/true".to_string())
            .provides(vec![Capability::Security])
            .requires(vec![])
            .http_port(9000)
            .build()
            .unwrap();

        assert_eq!(primal.id().to_string(), "test-primal");
        assert_eq!(primal.provides(), &[Capability::Security]);
        assert_eq!(primal.requires().len(), 0);
    }

    #[test]
    fn test_convenience_functions() {
        let security = create_security_provider("/path/to/beardog".to_string(), 9000).unwrap();
        assert_eq!(security.provides(), &[Capability::Security]);

        let discovery = create_discovery_orchestrator("/path/to/songbird".to_string()).unwrap();
        assert_eq!(discovery.provides(), &[Capability::Discovery]);
        assert_eq!(discovery.requires(), &[Capability::Security]);
    }

    #[test]
    fn test_capability_composition() {
        let ai_service = create_ai_service("/path/to/squirrel".to_string(), 8080).unwrap();
        assert_eq!(ai_service.provides(), &[Capability::AI]);
        assert!(ai_service.requires().contains(&Capability::Compute));
        assert!(ai_service.requires().contains(&Capability::Storage));
    }
}
