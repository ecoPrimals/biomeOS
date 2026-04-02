// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Device Management Capability Provider
//!
//! Generic implementation of the `device.management` capability.
//! NO primal-specific code - ANY primal can discover and use this!

use anyhow::Result;
use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::CapabilityTaxonomy;
use biomeos_types::primal_names;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{
    Device, DeviceStatus, DeviceType, ManagedPrimal, NicheTemplate, PrimalStatus, ValidationResult,
};

/// Resolve a capability to its runtime provider name via env var or taxonomy default.
fn resolve_provider(env_var: &str, capability: &CapabilityTaxonomy) -> String {
    resolve_provider_with(env_var, capability, None)
}

/// Same as [`resolve_provider`], with an optional simulated env value for tests.
///
/// - `None` — read `std::env::var(env_var)` with taxonomy fallback (production).
/// - `Some(None)` — treat env as unset; use taxonomy default.
/// - `Some(Some(v))` — use `v` without reading the process environment.
#[expect(
    clippy::option_option,
    reason = "three-state: None=read env, Some(None)=unset, Some(Some)=override"
)]
fn resolve_provider_with(
    env_var: &str,
    capability: &CapabilityTaxonomy,
    env_override: Option<Option<&str>>,
) -> String {
    match env_override {
        None => std::env::var(env_var)
            .unwrap_or_else(|_| capability.default_primal().unwrap_or("unknown").to_string()),
        Some(None) => capability.default_primal().unwrap_or("unknown").to_string(),
        Some(Some(v)) => v.to_string(),
    }
}

/// Device Management Capability Provider
///
/// Provides the `device.management` capability for biomeOS.
/// Discovered at runtime - no hardcoded primal names!
pub struct DeviceManagementProvider {
    /// Cache for graceful degradation
    pub(crate) cache: Arc<RwLock<ProviderCache>>,
    /// Socket path for RPC server
    pub(crate) socket_path: String,
}

/// Cached data for offline mode
#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderCache {
    pub(crate) devices: Vec<Device>,
    pub(crate) primals: Vec<ManagedPrimal>,
    pub(crate) templates: Vec<NicheTemplate>,
    pub(crate) last_update: Option<std::time::Instant>,
}

impl DeviceManagementProvider {
    /// Create a new device management provider
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Unix socket path for JSON-RPC server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use biomeos_ui::capabilities::device_management::DeviceManagementProvider;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let provider = DeviceManagementProvider::new("/run/user/1000/biomeos-device-mgmt.sock");
    ///     provider.start().await?;
    ///     Ok(())
    ///     }
    /// ```
    #[must_use]
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(ProviderCache::default())),
            socket_path: socket_path.into(),
        }
    }

    /// Start the capability provider
    ///
    /// Advertises "device.management" capability to Songbird and starts JSON-RPC server.
    pub async fn start(&self) -> Result<()> {
        info!(
            "🔧 Starting device.management capability provider on {}",
            self.socket_path
        );

        // Advertise capability via registry provider (Songbird or env-configured)
        let registry_provider =
            resolve_provider("BIOMEOS_REGISTRY_PROVIDER", &CapabilityTaxonomy::Discovery);
        if let Ok(registry) = AtomicClient::discover(&registry_provider).await {
            match registry
                .call(
                    "registry.register_service",
                    serde_json::json!({
                        "service_name": "device.management",
                        "capabilities": ["device.management"],
                        "endpoint": self.socket_path,
                        "metadata": {
                            "version": env!("CARGO_PKG_VERSION"),
                            "tags": ["ui", "management", "devices"]
                        }
                    }),
                )
                .await
            {
                Ok(_) => info!("✅ Registered device.management capability with registry"),
                Err(e) => warn!(
                    "⚠️ Failed to register with registry provider: {} (continuing anyway)",
                    e
                ),
            }
        } else {
            warn!("⚠️ Songbird not available, capability not advertised");
        }

        // Note: JSON-RPC server is started by the binary (device_management_server.rs)
        // This provider just handles the capability logic

        info!("✅ device.management capability provider ready");
        Ok(())
    }

    /// Get all devices
    ///
    /// Discovers devices via runtime discovery (no hardcoding!)
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("📡 Discovering devices");

        let devices = super::discovery::discover_devices().await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.devices.clone_from(&devices);
        cache.last_update = Some(std::time::Instant::now());

        Ok(devices)
    }

    /// Get all managed primals
    ///
    /// Discovers primals via runtime discovery
    pub async fn get_primals(&self) -> Result<Vec<ManagedPrimal>> {
        debug!("📡 Discovering primals");

        let primals = super::discovery::discover_primals().await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.primals.clone_from(&primals);
        cache.last_update = Some(std::time::Instant::now());

        Ok(primals)
    }

    /// Get niche templates
    ///
    /// Loads templates from storage (NestGate or local)
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        debug!("📚 Loading niche templates");

        let mut templates = Vec::new();

        // Try to load from storage provider (NestGate or env-configured)
        let storage_provider =
            resolve_provider("BIOMEOS_STORAGE_PROVIDER", &CapabilityTaxonomy::DataStorage);
        if let Ok(storage) = AtomicClient::discover(&storage_provider).await {
            match storage
                .call(
                    "storage.list",
                    serde_json::json!({ "key_prefix": "template:" }),
                )
                .await
            {
                Ok(result) => {
                    if let Some(items) = result.as_array() {
                        for item in items {
                            if let Ok(template) =
                                serde_json::from_value::<NicheTemplate>(item.clone())
                            {
                                templates.push(template);
                            }
                        }
                        info!(
                            "📚 Loaded {} templates from storage provider",
                            templates.len()
                        );
                    }
                }
                Err(e) => {
                    debug!(
                        "Storage provider template load failed: {} - using built-in",
                        e
                    );
                }
            }
        }

        // Fall back to built-in templates if none loaded
        if templates.is_empty() {
            templates = Self::get_builtin_templates();
            debug!("📚 Using {} built-in templates", templates.len());
        }

        // Update cache
        let mut cache = self.cache.write().await;
        cache.templates.clone_from(&templates);
        cache.last_update = Some(std::time::Instant::now());

        Ok(templates)
    }

    /// Validate a niche template
    ///
    /// Checks if resources are available for deployment
    pub async fn validate_niche(&self, template: &NicheTemplate) -> Result<ValidationResult> {
        debug!("✅ Validating niche template: {}", template.name);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check available devices
        let devices = self.get_devices().await?;
        let available_devices: Vec<_> = devices
            .iter()
            .filter(|d| matches!(d.status, DeviceStatus::Available))
            .collect();

        // Check GPU requirements
        if template.estimated_resources.gpu_required {
            let available_gpus = available_devices
                .iter()
                .filter(|d| matches!(d.device_type, DeviceType::Gpu))
                .count();

            if available_gpus == 0 {
                errors.push("No available GPUs found, but GPU is required".to_string());
            }
        }

        // Check available primals
        let primals = self.get_primals().await?;
        let healthy_primals: Vec<_> = primals
            .iter()
            .filter(|p| matches!(p.status, PrimalStatus::Healthy))
            .collect();

        // Validate required primals
        for required_role in &template.required_primals {
            let matching_primals = healthy_primals
                .iter()
                .filter(|p| {
                    required_role
                        .capabilities
                        .iter()
                        .all(|cap| p.capabilities.contains(cap))
                        && p.health >= required_role.min_health
                })
                .count();

            if matching_primals == 0 {
                errors.push(format!(
                    "No healthy primals found for required role: {}",
                    required_role.role
                ));
            }
        }

        // Check optional primals (warnings only)
        for optional_role in &template.optional_primals {
            let matching_primals = healthy_primals
                .iter()
                .filter(|p| {
                    optional_role
                        .capabilities
                        .iter()
                        .all(|cap| p.capabilities.contains(cap))
                })
                .count();

            if matching_primals == 0 {
                warnings.push(format!(
                    "No primals found for optional role: {} (may reduce functionality)",
                    optional_role.role
                ));
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Deploy a niche
    ///
    /// Creates and starts a niche from a template
    pub async fn deploy_niche(&self, config: serde_json::Value) -> Result<String> {
        info!("🚀 Deploying niche with config: {:?}", config);

        // Deploy via biomeOS orchestration capability
        if let Ok(biomeos) = AtomicClient::discover(primal_names::BIOMEOS).await {
            match biomeos
                .call("orchestration.deploy_niche", config.clone())
                .await
            {
                Ok(result) => {
                    let niche_id = result
                        .get("niche_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    info!("✅ Niche deployed: {}", niche_id);
                    return Ok(niche_id);
                }
                Err(e) => {
                    warn!("❌ Niche deployment failed: {}", e);
                    return Err(anyhow::anyhow!("Niche deployment failed: {e}"));
                }
            }
        }

        // Try orchestration provider as backup
        let orch_provider =
            resolve_provider("BIOMEOS_REGISTRY_PROVIDER", &CapabilityTaxonomy::Discovery);
        if let Ok(orchestrator) = AtomicClient::discover(&orch_provider).await {
            match orchestrator
                .call("orchestration.deploy_niche", config)
                .await
            {
                Ok(result) => {
                    let niche_id = result
                        .get("niche_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    info!("✅ Niche deployed via Songbird: {}", niche_id);
                    return Ok(niche_id);
                }
                Err(e) => {
                    warn!("❌ Songbird niche deployment failed: {}", e);
                }
            }
        }

        Err(anyhow::anyhow!("No orchestration capability available"))
    }

    /// Assign a device to a primal
    ///
    /// Coordinates device assignment via orchestration
    pub async fn assign_device(&self, device_id: String, primal_id: String) -> Result<()> {
        info!("🔗 Assigning device {} to primal {}", device_id, primal_id);

        // Coordinate via registry provider
        let registry_provider =
            resolve_provider("BIOMEOS_REGISTRY_PROVIDER", &CapabilityTaxonomy::Discovery);
        if let Ok(registry) = AtomicClient::discover(&registry_provider).await {
            match registry
                .call(
                    "registry.assign_device",
                    serde_json::json!({
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Device {} assigned to {}", device_id, primal_id);

                    // Persist assignment to storage provider
                    let storage_prov = resolve_provider(
                        "BIOMEOS_STORAGE_PROVIDER",
                        &CapabilityTaxonomy::DataStorage,
                    );
                    if let Ok(storage) = AtomicClient::discover(&storage_prov).await {
                        let _ = storage
                            .call(
                                "storage.store",
                                serde_json::json!({
                                    "key": format!("assignment:{}:{}", device_id, primal_id),
                                    "value": {
                                        "device_id": device_id,
                                        "primal_id": primal_id,
                                        "assigned_at": chrono::Utc::now().to_rfc3339()
                                    }
                                }),
                            )
                            .await;
                    }

                    return Ok(());
                }
                Err(e) => {
                    warn!("❌ Device assignment failed: {}", e);
                    return Err(anyhow::anyhow!("Device assignment failed: {e}"));
                }
            }
        }

        Err(anyhow::anyhow!(
            "Songbird not available for device assignment"
        ))
    }

    /// Get built-in niche templates
    ///
    /// Delegates to the templates module for standard templates.
    /// This keeps template definitions separate from provider logic.
    pub(crate) fn get_builtin_templates() -> Vec<NicheTemplate> {
        super::templates::builtin_templates()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::capabilities::device_management::{templates, types};
    use biomeos_types::CapabilityTaxonomy;
    use biomeos_types::primal_names;

    #[test]
    fn test_resolve_provider_registry_from_env() {
        let s = resolve_provider_with(
            "BIOMEOS_REGISTRY_PROVIDER",
            &CapabilityTaxonomy::Discovery,
            Some(Some("custom-registry")),
        );
        assert_eq!(s, "custom-registry");
    }

    #[test]
    fn test_resolve_provider_registry_default_without_env() {
        let s = resolve_provider_with(
            "BIOMEOS_REGISTRY_PROVIDER",
            &CapabilityTaxonomy::Discovery,
            Some(None),
        );
        assert_eq!(s, primal_names::SONGBIRD);
    }

    #[test]
    fn test_resolve_provider_storage_default_without_env() {
        let s = resolve_provider_with(
            "BIOMEOS_STORAGE_PROVIDER",
            &CapabilityTaxonomy::DataStorage,
            Some(None),
        );
        assert_eq!(s, primal_names::NESTGATE);
    }

    #[test]
    fn test_provider_cache_default() {
        let cache = ProviderCache::default();
        assert!(cache.devices.is_empty());
        assert!(cache.primals.is_empty());
        assert!(cache.templates.is_empty());
        assert!(cache.last_update.is_none());
    }

    #[test]
    fn test_provider_cache_debug() {
        let cache = ProviderCache::default();
        let s = format!("{cache:?}");
        assert!(s.contains("ProviderCache"));
    }

    #[test]
    fn test_provider_new() {
        let provider = DeviceManagementProvider::new("/run/user/1000/test.sock");
        assert_eq!(provider.socket_path, "/run/user/1000/test.sock");
    }

    #[test]
    fn test_get_builtin_templates() {
        let _provider = DeviceManagementProvider::new("/tmp/test.sock");
        let templates = DeviceManagementProvider::get_builtin_templates();
        assert_eq!(templates.len(), 2);
        assert!(templates.iter().any(|t| t.id == "tower"));
        assert!(templates.iter().any(|t| t.id == "node"));
    }

    #[tokio::test]
    async fn test_get_devices() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let result = provider.get_devices().await;
        assert!(result.is_ok());
        let devices = result.unwrap();
        assert!(devices.iter().all(|d| !d.id.is_empty()));
    }

    #[tokio::test]
    async fn test_get_devices_updates_cache() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let _ = provider.get_devices().await.unwrap();
        let cache = provider.cache.read().await;
        assert!(cache.last_update.is_some());
    }

    #[tokio::test]
    async fn test_get_primals() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let result = provider.get_primals().await;
        assert!(result.is_ok());
        let primals = result.unwrap();
        assert!(primals.iter().any(|p| p.id == "biomeos"));
    }

    #[tokio::test]
    async fn test_get_niche_templates_fallback_to_builtin() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let result = provider.get_niche_templates().await;
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.id == "tower"));
    }

    #[tokio::test]
    async fn test_validate_niche_no_gpu_required() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let template = templates::tower_template();
        let result = provider.validate_niche(&template).await;
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert!(!vr.valid);
        assert!(!vr.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_niche_gpu_required_node_template() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let template = templates::node_template();
        assert!(template.estimated_resources.gpu_required);
        let result = provider.validate_niche(&template).await;
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert!(!vr.valid);
        assert!(!vr.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_niche_with_optional_primals() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let mut template = templates::tower_template();
        template.optional_primals.push(types::PrimalRole {
            role: "optional_compute".to_string(),
            capabilities: vec!["gpu".to_string()],
            min_health: 0.5,
            metadata: serde_json::json!({}),
        });
        let result = provider.validate_niche(&template).await;
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert!(!vr.valid);
        if !vr.warnings.is_empty() {
            assert!(vr.warnings.iter().any(|w| w.contains("optional")));
        }
    }

    #[tokio::test]
    async fn test_deploy_niche_no_orchestration() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let result = provider
            .deploy_niche(serde_json::json!({"template": "tower"}))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("orchestration") || err.to_string().contains("No"));
    }

    #[tokio::test]
    async fn test_assign_device_no_registry() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let result = provider
            .assign_device("gpu-0".to_string(), "toadstool-1".to_string())
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Songbird") || err.to_string().contains("available"));
    }

    #[tokio::test]
    async fn test_start_continues_without_registry() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let result = provider.start().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation_result_structure() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let template = templates::tower_template();
        let vr = provider.validate_niche(&template).await.unwrap();
        assert!(vr.errors.iter().all(|e| !e.is_empty()));
    }

    #[tokio::test]
    async fn test_validate_niche_minimal_template_valid() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let template = types::NicheTemplate {
            id: "minimal".to_string(),
            name: "Minimal".to_string(),
            description: "test".to_string(),
            required_primals: vec![],
            optional_primals: vec![],
            estimated_resources: types::ResourceRequirements {
                cpu_cores: 1,
                memory_mb: 1,
                storage_gb: 1,
                gpu_required: false,
                network_bandwidth_mbps: 1,
            },
            metadata: serde_json::json!({}),
        };
        let vr = provider.validate_niche(&template).await.expect("validate");
        assert!(vr.valid, "expected valid: {:?}", vr.errors);
        assert!(vr.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_niche_optional_role_warning_only() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let mut template = types::NicheTemplate {
            id: "minimal-opt".to_string(),
            name: "Minimal".to_string(),
            description: "test".to_string(),
            required_primals: vec![],
            optional_primals: vec![types::PrimalRole {
                role: "ghost".to_string(),
                capabilities: vec!["no_such_capability_xyz".to_string()],
                min_health: 0.5,
                metadata: serde_json::json!({}),
            }],
            estimated_resources: types::ResourceRequirements {
                cpu_cores: 1,
                memory_mb: 1,
                storage_gb: 1,
                gpu_required: false,
                network_bandwidth_mbps: 1,
            },
            metadata: serde_json::json!({}),
        };
        let vr = provider.validate_niche(&template).await.expect("validate");
        assert!(vr.valid);
        assert!(
            vr.warnings.iter().any(|w| w.contains("optional")),
            "expected optional warning: {:?}",
            vr.warnings
        );

        template.optional_primals.clear();
        let vr2 = provider.validate_niche(&template).await.expect("validate");
        assert!(vr2.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_get_niche_templates_updates_cache() {
        let provider = DeviceManagementProvider::new("/tmp/test.sock");
        let _ = provider.get_niche_templates().await.unwrap();
        let cache = provider.cache.read().await;
        assert!(cache.last_update.is_some());
        assert!(!cache.templates.is_empty());
    }
}
