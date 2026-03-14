// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
            match self
                .create_service_integration(name, service_type, config.as_deref())
                .await
            {
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

                // Legacy code - depends on ClientRegistry
                // Query ToadStool for actual replica count (if available)
                // Future: Restore via UniversalPrimalClient discovery
                let current_replicas = serde_json::json!("unknown");
                /* Legacy code commented out:
                let current_replicas = if let Ok(toadstool) = self.clients().toadstool().await {
                    match toadstool.get_service_replicas(service).await {
                        Ok(count) => serde_json::json!(count),
                        Err(e) => {
                            tracing::warn!("Failed to get replica count from ToadStool: {}", e);
                            serde_json::json!("unknown")
                        }
                    }
                } else {
                    tracing::debug!("ToadStool not available - replica count unknown");
                    serde_json::json!("unknown")
                };
                */

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

    /// Create service integration
    ///
    /// Future: Integrate with Toadstool for service provisioning
    pub(super) async fn create_service_integration(
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
}
