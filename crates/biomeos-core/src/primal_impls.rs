// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Generic primal implementations using capability-based architecture
//!
//! NO hardcoded primal names - everything discovered from environment!
//! Implements the "Infant Model" - primals only know themselves.

use std::{
    fs::OpenOptions,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{info, warn};

use biomeos_types::{
    SystemPaths,
    error::{BiomeError, BiomeResult},
    identifiers::{Endpoint, PrimalId},
};

use crate::{
    capabilities::{Capability, PrimalConfig},
    discovery_modern::HealthStatus,
    primal_orchestrator::ManagedPrimal,
};

/// Generic managed primal - works for ANY primal!
/// Configuration comes entirely from environment.
pub struct GenericManagedPrimal {
    id: PrimalId,
    config: PrimalConfig,
    process: Arc<Mutex<Option<Child>>>,
}

impl GenericManagedPrimal {
    /// Create from environment (infant model - ZERO hardcoding!)
    pub fn from_env() -> BiomeResult<Self> {
        let config = PrimalConfig::from_env()?;
        let id = PrimalId::new(config.id.clone()).map_err(|e| {
            BiomeError::config_error(format!("Invalid primal ID: {e}"), Some("PRIMAL_ID"))
        })?;

        info!("🌱 Created primal from environment:");
        info!("   ID: {}", id);
        info!("   Provides: {:?}", config.provides);
        info!("   Requires: {:?}", config.requires);
        info!(
            "   HTTP Port: {}",
            if config.http_port == 0 {
                "auto".to_string()
            } else {
                config.http_port.to_string()
            }
        );

        Ok(Self {
            id,
            config,
            process: Arc::new(Mutex::new(None)),
        })
    }

    /// Create with explicit config (for testing or manual construction)
    pub fn with_config(config: PrimalConfig) -> BiomeResult<Self> {
        let id = PrimalId::new(config.id.clone()).map_err(|e| {
            BiomeError::config_error(format!("Invalid primal ID: {e}"), Some("PRIMAL_ID"))
        })?;

        Ok(Self {
            id,
            config,
            process: Arc::new(Mutex::new(None)),
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
        // EVOLUTION: Prefer Unix socket over HTTP
        // Unix sockets are the primary IPC mechanism for biomeOS.
        // HTTP is only for temporary bridge to PetalTongue.
        //
        // Priority:
        // 1. Unix socket (from PRIMAL_SOCKET_PATH env)
        // 2. HTTP (legacy, deprecated)
        //
        // Deep Debt Principle: Unix socket first, HTTP bridge is temporary.

        // Try Unix socket first
        if let Ok(socket_path) = std::env::var("PRIMAL_SOCKET_PATH")
            && let Ok(endpoint) = Endpoint::new(format!("unix://{socket_path}"))
        {
            return Some(endpoint);
        }

        // Fallback to HTTP if configured (deprecated)
        if self.config.http_port > 0 {
            warn!(
                "⚠️  Primal {} using deprecated HTTP endpoint. Evolve to Unix socket!",
                self.id
            );
            warn!(
                "   Set PRIMAL_SOCKET_PATH=$XDG_RUNTIME_DIR/biomeos/{}.sock",
                self.config.id
            );
            // Use RuntimeConfig for bind address and port resolution
            use biomeos_types::defaults::RuntimeConfig;
            let runtime_config = RuntimeConfig::from_env();
            let bind_addr = runtime_config.bind_address();
            let http_port = runtime_config.http_port();
            let url = format!("http://{bind_addr}:{http_port}");
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
            let provides_str = self
                .config
                .provides
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            cmd.env("PRIMAL_PROVIDES", provides_str);
        }

        // ✅ DEEP DEBT FIX (Jan 5, 2026): Redirect logs to per-primal files
        // instead of /dev/null to enable observability and debugging!
        // Use XDG-compliant paths via SystemPaths
        let node_id = std::env::var("SONGBIRD_NODE_ID")
            .or_else(|_| std::env::var("NODE_ID"))
            .unwrap_or_else(|_| "unknown".to_string());

        // Use SystemPaths for XDG-compliant log location
        let log_path = if let Ok(paths) = SystemPaths::new() {
            paths.log_file(&format!("{}-{}", self.id, node_id))
        } else {
            // EVOLVED: Environment-driven fallback (no hardcoded /tmp)
            // Respects BIOMEOS_LOG_DIR or falls back to writable directory
            let log_dir = std::env::var("BIOMEOS_LOG_DIR")
                .or_else(|_| std::env::var("XDG_STATE_HOME").map(|p| format!("{p}/biomeos/logs")))
                .or_else(|_| {
                    std::env::var("HOME").map(|p| format!("{p}/.local/state/biomeos/logs"))
                })
                .unwrap_or_else(|_| {
                    warn!("No XDG paths available, using current directory for logs");
                    "./logs".to_string()
                });

            std::fs::create_dir_all(&log_dir).ok();
            std::path::PathBuf::from(format!("{}/{}-{}.log", log_dir, self.id, node_id))
        };

        // Ensure log directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| {
                BiomeError::internal_error(
                    format!("Failed to create log file {}: {}", log_path.display(), e),
                    Some("log_file_creation_failure"),
                )
            })?;

        info!("📝 Primal logs will be written to: {}", log_path.display());

        let child = cmd
            .stdout(Stdio::from(log_file.try_clone().map_err(|e| {
                BiomeError::internal_error(
                    format!("Failed to clone log file handle: {e}"),
                    Some("log_file_clone_failure"),
                )
            })?))
            .stderr(Stdio::from(log_file))
            .spawn()
            .map_err(|e| {
                BiomeError::internal_error(
                    format!("Failed to spawn primal {}: {}", self.id, e),
                    Some("process_spawn_failure"),
                )
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
                BiomeError::internal_error(
                    format!("Failed to kill {}: {}", self.id, e),
                    Some("process_kill_failure"),
                )
            })?;
            child.wait().map_err(|e| {
                BiomeError::internal_error(
                    format!("Failed to wait for {}: {}", self.id, e),
                    Some("process_wait_failure"),
                )
            })?;
        }

        info!("✅ Primal {} stopped", self.id);
        Ok(())
    }

    async fn health_check(&self) -> BiomeResult<HealthStatus> {
        // Process-based health check with proper zombie reaping
        let mut process_guard = self.process.lock().await;

        if let Some(child) = process_guard.as_mut() {
            // Check if process actually exited (and reap zombie if it did)
            match child.try_wait() {
                Ok(Some(exit_status)) => {
                    // Process exited! Reap the zombie and mark unhealthy
                    info!(
                        "⚠️  Primal {} exited with status: {:?}",
                        self.id, exit_status
                    );
                    *process_guard = None; // Clear the handle

                    Ok(HealthStatus::Unhealthy)
                }
                Ok(None) => {
                    // Process still running - healthy
                    Ok(HealthStatus::Healthy)
                }
                Err(e) => {
                    // Error checking process status
                    warn!("Failed to check process status for {}: {}", self.id, e);
                    Ok(HealthStatus::Unhealthy)
                }
            }
        } else {
            // No process handle - unhealthy
            Ok(HealthStatus::Unhealthy)
        }
    }

    fn startup_timeout(&self) -> Duration {
        // Check for custom timeout in config
        self.config
            .env_config
            .get("PRIMAL_STARTUP_TIMEOUT")
            .and_then(|s| s.parse::<u64>().ok())
            .map_or(Duration::from_secs(30), Duration::from_secs)
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
    /// Create a new builder with default values
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

    /// Set the primal identifier
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the path to the primal binary
    pub fn binary_path(mut self, path: String) -> Self {
        self.binary_path = Some(path);
        self
    }

    /// Set the capabilities this primal provides
    pub fn provides(mut self, capabilities: Vec<Capability>) -> Self {
        self.provides = capabilities;
        self
    }

    /// Set the capabilities this primal requires
    pub fn requires(mut self, capabilities: Vec<Capability>) -> Self {
        self.requires = capabilities;
        self
    }

    /// Set the HTTP port (temporary bridge)
    pub fn http_port(mut self, port: u16) -> Self {
        self.http_port = port;
        self
    }

    /// Add an environment variable for the primal process
    pub fn env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    /// Build the primal from the configured values
    pub fn build(self) -> BiomeResult<Arc<GenericManagedPrimal>> {
        let config = PrimalConfig {
            id: self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            binary_path: self.binary_path.ok_or_else(|| {
                BiomeError::config_error("Binary path not set", Some("PRIMAL_BINARY"))
            })?,
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
pub fn create_security_provider(
    binary_path: String,
    http_port: u16,
) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Security])
        .requires(vec![])
        .http_port(http_port)
        .build()
}

/// Convenience function: Create a discovery orchestrator (e.g., Songbird-like)
pub fn create_discovery_orchestrator(
    binary_path: String,
) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Discovery])
        .requires(vec![Capability::Security]) // Needs a security provider
        .http_port(0) // No HTTP port (uses UDP)
        .build()
}

/// Convenience function: Create a compute provider (e.g., Toadstool-like)
pub fn create_compute_provider(
    binary_path: String,
    http_port: u16,
) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Compute])
        .requires(vec![])
        .http_port(http_port)
        .build()
}

/// Convenience function: Create an AI service (e.g., Squirrel-like)
pub fn create_ai_service(
    binary_path: String,
    http_port: u16,
) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::AI])
        .requires(vec![Capability::Compute, Capability::Storage])
        .http_port(http_port)
        .build()
}

/// Convenience function: Create a storage provider (e.g., NestGate-like)
pub fn create_storage_provider(
    binary_path: String,
    http_port: u16,
) -> BiomeResult<Arc<GenericManagedPrimal>> {
    PrimalBuilder::new()
        .binary_path(binary_path)
        .provides(vec![Capability::Storage])
        .requires(vec![])
        .http_port(http_port)
        .build()
}

// Legacy compatibility (DEPRECATED - use GenericManagedPrimal instead)

/// Legacy alias — use [`GenericManagedPrimal`] instead
pub type ManagedBearDog = GenericManagedPrimal;
/// Legacy alias — use [`GenericManagedPrimal`] instead
pub type ManagedSongbird = GenericManagedPrimal;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_test_utils::{remove_test_env, set_test_env};

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
    fn test_builder_without_binary_path_fails() {
        let result = PrimalBuilder::new()
            .id("test".to_string())
            .provides(vec![Capability::Security])
            .build();
        match result {
            Ok(_) => panic!("Expected build to fail without binary path"),
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Binary path") || err_msg.contains("PRIMAL_BINARY"),
                    "Expected binary path error, got: {err_msg}"
                );
            }
        }
    }

    #[test]
    fn test_builder_default_id_when_not_set() {
        let primal = PrimalBuilder::new()
            .binary_path("/bin/true".to_string())
            .provides(vec![])
            .requires(vec![])
            .build()
            .unwrap();
        assert!(!primal.id().to_string().is_empty());
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

    #[test]
    fn test_create_storage_provider() {
        let storage = create_storage_provider("/path/to/nestgate".to_string(), 8002).unwrap();
        assert_eq!(storage.provides(), &[Capability::Storage]);
        assert_eq!(storage.requires().len(), 0);
    }

    #[test]
    fn test_create_compute_provider() {
        let compute = create_compute_provider("/path/to/toadstool".to_string(), 8080).unwrap();
        assert_eq!(compute.provides(), &[Capability::Compute]);
    }

    #[test]
    fn test_primal_builder_env_var() {
        let primal = PrimalBuilder::new()
            .id("env-test".to_string())
            .binary_path("/bin/true".to_string())
            .env_var("CUSTOM_VAR".to_string(), "value".to_string())
            .build()
            .unwrap();
        assert_eq!(primal.id().to_string(), "env-test");
    }

    #[test]
    fn test_legacy_type_aliases() {
        let _beardog: Arc<GenericManagedPrimal> =
            create_security_provider("/bin/true".to_string(), 9000).unwrap();
        let _songbird: Arc<GenericManagedPrimal> =
            create_discovery_orchestrator("/bin/true".to_string()).unwrap();
    }

    static ENDPOINT_ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn test_endpoint_unix_socket_preferred() {
        let _guard = ENDPOINT_ENV_LOCK.lock().await;
        set_test_env("PRIMAL_SOCKET_PATH", "/run/user/1000/biomeos/test.sock");
        let primal = PrimalBuilder::new()
            .id("test".to_string())
            .binary_path("/bin/true".to_string())
            .http_port(9000)
            .build()
            .unwrap();
        let endpoint = primal.endpoint().await;
        remove_test_env("PRIMAL_SOCKET_PATH");
        assert!(endpoint.is_some());
        let ep = endpoint.unwrap();
        assert!(ep.to_string().contains("unix://"));
    }

    #[tokio::test]
    async fn test_endpoint_http_fallback() {
        let _guard = ENDPOINT_ENV_LOCK.lock().await;
        remove_test_env("PRIMAL_SOCKET_PATH");
        let primal = PrimalBuilder::new()
            .id("test".to_string())
            .binary_path("/bin/true".to_string())
            .http_port(9000)
            .build()
            .unwrap();
        let endpoint = primal.endpoint().await;
        let _ = endpoint;
    }

    #[tokio::test]
    async fn test_health_check_no_process() {
        let primal = PrimalBuilder::new()
            .id("test".to_string())
            .binary_path("/bin/true".to_string())
            .build()
            .unwrap();
        let status = primal.health_check().await.unwrap();
        assert_eq!(status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_startup_timeout_default() {
        let primal = PrimalBuilder::new()
            .id("test".to_string())
            .binary_path("/bin/true".to_string())
            .build()
            .unwrap();
        let timeout = primal.startup_timeout();
        assert_eq!(timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_startup_timeout_custom() {
        let primal = PrimalBuilder::new()
            .id("test".to_string())
            .binary_path("/bin/true".to_string())
            .env_var("PRIMAL_STARTUP_TIMEOUT".to_string(), "60".to_string())
            .build()
            .unwrap();
        let timeout = primal.startup_timeout();
        assert_eq!(timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_with_config_invalid_id() {
        let config = PrimalConfig {
            id: String::new(),
            binary_path: "/bin/true".to_string(),
            provides: vec![],
            requires: vec![],
            http_port: 0,
            env_config: std::collections::HashMap::new(),
        };
        let result = GenericManagedPrimal::with_config(config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_provides_requires_accessors() {
        let primal = PrimalBuilder::new()
            .id("test".to_string())
            .binary_path("/bin/true".to_string())
            .provides(vec![Capability::Security, Capability::Discovery])
            .requires(vec![Capability::Compute])
            .build()
            .unwrap();
        assert_eq!(primal.provides().len(), 2);
        assert!(primal.provides().contains(&Capability::Security));
        assert_eq!(primal.requires().len(), 1);
        assert!(primal.requires().contains(&Capability::Compute));
    }
}
