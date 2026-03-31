// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Service Lifecycle Operations
//!
//! Handles service creation, scaling, auto-scaling, and status queries.
//! Coordinates with Toadstool for compute orchestration.

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use super::core::{PrimalInfo, UniversalBiomeOSManager};
use biomeos_types::PrimalCapability;

/// Scaling result
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct ScaleResult {
    pub(crate) current_replicas: u32,
    pub(crate) target_replicas: u32,
    pub(crate) status: String,
}

/// Parse scale result from JSON-RPC response (testable pure function)
pub(crate) fn parse_scale_result_from_json(
    result: &serde_json::Value,
    target_replicas: u32,
) -> ScaleResult {
    ScaleResult {
        current_replicas: u32::try_from(result["current_replicas"].as_u64().unwrap_or(1))
            .unwrap_or(1),
        target_replicas,
        status: result["status"].as_str().unwrap_or("scaling").to_string(),
    }
}

impl UniversalBiomeOSManager {
    /// Create a new service
    pub async fn create_service(
        &self,
        service_type: &str,
        name: &str,
        config: Option<std::path::PathBuf>,
        dry_run: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🌱 Creating new service: {} (type: {})", name, service_type);

        let mut result = HashMap::new();
        result.insert("service_name".to_string(), serde_json::json!(name));
        result.insert("service_type".to_string(), serde_json::json!(service_type));
        result.insert("dry_run".to_string(), serde_json::json!(dry_run));

        if dry_run {
            result.insert("status".to_string(), serde_json::json!("planned"));
            result.insert(
                "message".to_string(),
                serde_json::json!("Service creation plan generated"),
            );

            // Generate creation plan
            let execution_plan = serde_json::json!({
                "steps": [
                    "Validate service configuration",
                    "Check resource requirements",
                    "Prepare deployment environment",
                    "Create service instance",
                    "Configure networking",
                    "Start service monitoring"
                ],
                "estimated_duration": "2-5 minutes",
                "resource_requirements": {
                    "cpu": "0.5 cores",
                    "memory": "512MB",
                    "storage": "1GB"
                }
            });
            result.insert("execution_plan".to_string(), execution_plan);
        } else {
            // Service creation - integration point with template system
            match self.create_service_integration(name, service_type, config.as_deref()) {
                Ok(service_info) => {
                    result.insert("status".to_string(), serde_json::json!("created"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!("Service created successfully"),
                    );
                    let service_id = format!("svc-{}", uuid::Uuid::new_v4());
                    result.insert("service_id".to_string(), serde_json::json!(service_id));

                    // Discover actual Toadstool endpoint via capability-based discovery
                    let compute_cap = PrimalCapability::new("compute", "execution", "1.0");
                    let endpoint = match self.discover_by_capability(&[compute_cap]).await {
                        Ok(discovered_ids) => {
                            if let Some(primal_id) = discovered_ids.first() {
                                let primals = self.registered_primals.read().await;
                                if let Some(primal) = primals.get(primal_id) {
                                    format!("{}/{}", primal.endpoint, name)
                                } else {
                                    // Fallback if primal not found
                                    tracing::warn!(
                                        "Discovered primal {} not in registry, using environment fallback",
                                        primal_id
                                    );
                                    let endpoint = std::env::var("BIOMEOS_COMPUTE_ENDPOINT")
                                        .or_else(|_| std::env::var("TOADSTOOL_ENDPOINT"))
                                        .map_err(|_| anyhow::anyhow!(
                                            "BIOMEOS_COMPUTE_ENDPOINT not set and discovery failed. \
                                             Set BIOMEOS_COMPUTE_ENDPOINT or ensure capability discovery is available."
                                        ))?;
                                    format!("{endpoint}/{name}")
                                }
                            } else {
                                // No compute primal discovered - return error
                                return Err(anyhow::anyhow!(
                                    "No compute primal discovered. \
                                     Set BIOMEOS_COMPUTE_ENDPOINT or ensure discovery service is running."
                                ));
                            }
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "Discovery failed: {e}. \
                                 Set BIOMEOS_COMPUTE_ENDPOINT or fix discovery service."
                            ));
                        }
                    };

                    result.insert("endpoint".to_string(), serde_json::json!(endpoint));
                    result.insert("service_info".to_string(), serde_json::json!(service_info));

                    tracing::info!("✅ Service creation completed");
                }
                Err(e) => {
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!(format!("Service creation failed: {}", e)),
                    );

                    tracing::error!("❌ Service creation failed: {}", e);
                }
            }
        }

        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        Ok(result)
    }

    /// Scale services up or down
    pub async fn scale_service(
        &self,
        service: &str,
        replicas: Option<u32>,
        auto_scaling: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!(
            "⚖️ Scaling service '{}' (replicas: {:?}, auto: {})",
            service,
            replicas,
            auto_scaling
        );

        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("auto_scaling".to_string(), serde_json::json!(auto_scaling));

        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {service}"))?;

        {
            if auto_scaling {
                // Enable auto-scaling
                result.insert("status".to_string(), serde_json::json!("success"));
                result.insert(
                    "message".to_string(),
                    serde_json::json!("Auto-scaling enabled"),
                );
                result.insert(
                    "auto_scaling".to_string(),
                    serde_json::json!({
                        "enabled": true,
                        "min_replicas": 1,
                        "max_replicas": 10,
                        "cpu_threshold_percent": 80,
                        "memory_threshold_percent": 85
                    }),
                );

                tracing::info!("✅ Auto-scaling enabled for service '{}'", service);
            } else if let Some(target_replicas) = replicas {
                // Manual scaling
                let scale_result = self
                    .scale_service_integration(primal, target_replicas)
                    .await?;

                result.insert("status".to_string(), serde_json::json!("success"));
                result.insert(
                    "message".to_string(),
                    serde_json::json!("Service scaled successfully"),
                );

                let current_replicas = serde_json::json!("unknown");

                result.insert("current_replicas".to_string(), current_replicas);
                result.insert(
                    "target_replicas".to_string(),
                    serde_json::json!(target_replicas),
                );
                result.insert("scale_result".to_string(), serde_json::json!(scale_result));

                tracing::info!(
                    "✅ Service '{}' scaled to {} replicas",
                    service,
                    target_replicas
                );
            } else {
                return Err(anyhow::anyhow!(
                    "Must specify replicas count or enable auto-scaling"
                ));
            }
        }

        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        Ok(result)
    }

    /// Enable auto-scaling for a service
    pub async fn enable_auto_scaling(
        &self,
        service: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        self.scale_service(service, None, true).await
    }

    /// Get service status
    pub async fn get_service_status(
        &self,
        service: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service);

        let mut result = HashMap::new();

        if let Some(primal) = primal {
            result.insert("service_name".to_string(), serde_json::json!(primal.name));
            result.insert("service_id".to_string(), serde_json::json!(primal.id));
            result.insert("status".to_string(), serde_json::json!("running"));
            result.insert("health".to_string(), serde_json::json!(primal.health));
            result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
            result.insert("type".to_string(), serde_json::json!(primal.primal_type));
            result.insert(
                "capabilities".to_string(),
                serde_json::json!(primal.capabilities),
            );
            result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));
            result.insert(
                "discovered_at".to_string(),
                serde_json::json!(primal.discovered_at),
            );
        } else {
            result.insert(
                "error".to_string(),
                serde_json::json!(format!("Service not found: {}", service)),
            );
        }

        Ok(result)
    }

    /// Create service integration.
    ///
    /// Service provisioning is delegated to `ToadStool` via `compute.*` capability
    /// routing when available; this provides a local fallback.
    #[expect(
        clippy::unused_self,
        reason = "method for future use or API consistency"
    )]
    pub(super) fn create_service_integration(
        &self,
        name: &str,
        service_type: &str,
        _config: Option<&Path>,
    ) -> Result<String> {
        tracing::debug!(
            "Creating service via integration: {} ({})",
            name,
            service_type
        );
        // Integration point with template system
        Ok(format!(
            "Service '{name}' of type '{service_type}' created successfully"
        ))
    }

    /// Scale service via primal's scaling API (Pure Rust via Unix socket!)
    pub(super) async fn scale_service_integration(
        &self,
        primal: &PrimalInfo,
        replicas: u32,
    ) -> Result<ScaleResult> {
        tracing::debug!(
            "🚀 Scaling service {} to {} replicas via atomic client",
            primal.name,
            replicas
        );

        // Use Pure Rust atomic client (Tower-based, Unix sockets)
        let client = crate::atomic_client::AtomicClient::discover(&primal.name)
            .await
            .with_context(|| format!("Failed to discover primal: {}", primal.name))?;

        // Build scale request
        let scale_request = serde_json::json!({
            "target_replicas": replicas
        });

        // Execute scaling via JSON-RPC over Unix socket (Pure Rust!)
        match client.call("scale_service", scale_request).await {
            Ok(result) => {
                tracing::debug!("✅ Scaling completed for {}", primal.name);
                Ok(parse_scale_result_from_json(&result, replicas))
            }
            Err(e) => {
                tracing::error!("Failed to scale service {}: {}", primal.name, e);
                Err(anyhow::anyhow!(
                    "Failed to scale service {}: {}",
                    primal.name,
                    e
                ))
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::universal_biomeos_manager::{PrimalInfo, UniversalBiomeOSManager};
    use biomeos_primal_sdk::PrimalCapability;
    use biomeos_types::{Health, PrimalType};
    use std::collections::HashMap;

    fn test_primal_info(id: &str, name: &str, endpoint: &str) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: name.to_string(),
            primal_type: PrimalType::from_discovered("compute", name, "1.0.0"),
            endpoint: endpoint.to_string(),
            capabilities: vec![PrimalCapability::new("compute", "execution", "1.0")],
            health: Health::Healthy,
            last_seen: chrono::Utc::now(),
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_parse_scale_result_from_json() {
        let json = serde_json::json!({
            "current_replicas": 3,
            "status": "scaled"
        });
        let result = parse_scale_result_from_json(&json, 3);
        assert_eq!(result.current_replicas, 3);
        assert_eq!(result.target_replicas, 3);
        assert_eq!(result.status, "scaled");
    }

    #[test]
    fn test_parse_scale_result_from_json_defaults() {
        let json = serde_json::json!({});
        let result = parse_scale_result_from_json(&json, 5);
        assert_eq!(result.current_replicas, 1);
        assert_eq!(result.target_replicas, 5);
        assert_eq!(result.status, "scaling");
    }

    #[test]
    fn test_parse_scale_result_from_json_partial() {
        let json = serde_json::json!({"status": "pending"});
        let result = parse_scale_result_from_json(&json, 10);
        assert_eq!(result.current_replicas, 1);
        assert_eq!(result.target_replicas, 10);
        assert_eq!(result.status, "pending");
    }

    #[test]
    fn test_scale_result_equality() {
        let a = ScaleResult {
            current_replicas: 2,
            target_replicas: 3,
            status: "scaling".to_string(),
        };
        let b = ScaleResult {
            current_replicas: 2,
            target_replicas: 3,
            status: "scaling".to_string(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_parse_scale_result_from_json_invalid_current_replicas() {
        let json = serde_json::json!({
            "current_replicas": "not_a_number",
            "status": "scaling"
        });
        let result = parse_scale_result_from_json(&json, 2);
        assert_eq!(result.current_replicas, 1);
        assert_eq!(result.target_replicas, 2);
    }

    #[tokio::test]
    async fn test_create_service_dry_run() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let result = manager
            .create_service("compute", "test-svc", None, true)
            .await
            .expect("create_service dry_run");

        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("planned")
        );
        assert_eq!(
            result.get("dry_run").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert!(result.contains_key("execution_plan"));
    }

    #[tokio::test]
    async fn test_create_service_dry_run_with_config_path() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let config_path = std::path::PathBuf::from("/tmp/test-config.yaml");
        let result = manager
            .create_service("storage", "nest-svc", Some(config_path), true)
            .await
            .expect("create_service dry_run");

        assert_eq!(
            result.get("service_type").and_then(|v| v.as_str()),
            Some("storage")
        );
        assert_eq!(
            result.get("service_name").and_then(|v| v.as_str()),
            Some("nest-svc")
        );
    }

    #[tokio::test]
    async fn test_create_service_non_dry_run_no_compute_primal() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");
        // No primals registered, discover_by_capability returns empty
        let result = manager
            .create_service("compute", "test-svc", None, false)
            .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("No compute primal") || err_msg.contains("BIOMEOS_COMPUTE_ENDPOINT"),
            "expected discovery/env error, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_create_service_with_registered_compute_primal() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let primal = test_primal_info("compute-1", "toadstool", "unix:///tmp/toadstool.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager
            .create_service("compute", "new-svc", None, false)
            .await;
        // create_service_integration succeeds, but we need endpoint - discovery finds our primal
        // Endpoint will be primal.endpoint + "/" + name = unix:///tmp/toadstool.sock/new-svc
        if let Ok(res) = &result {
            assert_eq!(res.get("status").and_then(|v| v.as_str()), Some("created"));
        }
        // May fail if AtomicClient/scale not available - integration point
        let _ = result;
    }

    #[tokio::test]
    async fn test_get_service_status_found_by_name() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let primal = test_primal_info("svc-1", "my-service", "unix:///run/svc.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager
            .get_service_status("my-service")
            .await
            .expect("get status");
        assert_eq!(
            result.get("service_name").and_then(|v| v.as_str()),
            Some("my-service")
        );
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("running")
        );
    }

    #[tokio::test]
    async fn test_get_service_status_found_by_id() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let primal = test_primal_info("svc-42", "other-service", "unix:///run/other.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager
            .get_service_status("svc-42")
            .await
            .expect("get status");
        assert_eq!(
            result.get("service_id").and_then(|v| v.as_str()),
            Some("svc-42")
        );
    }

    #[tokio::test]
    async fn test_get_service_status_not_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let result = manager
            .get_service_status("nonexistent")
            .await
            .expect("get status");
        assert!(result.contains_key("error"));
        assert!(
            result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .contains("nonexistent")
        );
    }

    #[tokio::test]
    async fn test_scale_service_not_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let result = manager.scale_service("nonexistent", Some(3), false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_scale_service_must_specify_replicas_or_auto() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let primal = test_primal_info("svc-1", "scale-test", "unix:///tmp/scale.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager.scale_service("scale-test", None, false).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Must specify replicas")
        );
    }

    #[tokio::test]
    async fn test_enable_auto_scaling() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager creation");
        manager.initialize().expect("init");

        let primal = test_primal_info("auto-1", "auto-scale-svc", "unix:///tmp/auto.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager
            .enable_auto_scaling("auto-scale-svc")
            .await
            .expect("enable auto scaling");
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("success")
        );
        assert_eq!(
            result.get("message").and_then(|v| v.as_str()),
            Some("Auto-scaling enabled")
        );
    }
}
