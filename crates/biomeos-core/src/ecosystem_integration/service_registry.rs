//! # Service Registry
//!
//! Service registration and management for the ecosystem integration system.
//! This module handles service registration, health checking, and integration
//! with the Songbird orchestrator.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use super::messaging::EcosystemMessageBus;
use super::types::*;
use crate::{BiomeError, BiomeResult, HealthStatus};

/// Ecosystem service registry
#[derive(Clone)]
pub struct EcosystemServiceRegistry {
    pub(crate) services: Arc<RwLock<HashMap<String, EcosystemServiceRegistration>>>,
    message_bus: Arc<dyn EcosystemMessageBus>,
}

impl EcosystemServiceRegistry {
    pub fn new(message_bus: Arc<dyn EcosystemMessageBus>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
        }
    }

    /// Register a service with Songbird orchestrator
    pub async fn register_service_with_songbird(
        &self,
        registration: &EcosystemServiceRegistration,
    ) -> BiomeResult<()> {
        // Determine Songbird endpoint from environment or default
        let songbird_endpoint = std::env::var("SONGBIRD_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                BiomeError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;

        // Convert to Songbird service registration format
        let songbird_registration = serde_json::json!({
            "service_id": registration.service_id,
            "primal_type": registration.primal_type,
            "biome_id": registration.biome_id,
            "version": registration.version,
            "endpoints": {
                "primary": registration.endpoints.primary,
                "health": registration.endpoints.health,
                "metrics": registration.endpoints.metrics,
                "admin": registration.endpoints.admin,
                "websocket": registration.endpoints.websocket
            },
            "capabilities": registration.capabilities,
            "resource_requirements": registration.resource_requirements,
            "health_check": registration.health_check,
            "metadata": registration.metadata
        });

        // Make registration request to Songbird
        let response = client
            .post(format!("{}/api/v1/services", songbird_endpoint))
            .json(&songbird_registration)
            .send()
            .await
            .map_err(|e| {
                BiomeError::NetworkError(format!("Failed to register with Songbird: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BiomeError::NetworkError(format!(
                "Songbird registration failed with status {}: {}",
                status, error_text
            )));
        }

        info!(
            "Successfully registered service {} with Songbird",
            registration.service_id
        );
        Ok(())
    }

    /// Unregister a service from Songbird orchestrator
    pub async fn unregister_service_from_songbird(&self, service_id: &str) -> BiomeResult<()> {
        let songbird_endpoint = std::env::var("SONGBIRD_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                BiomeError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;

        let response = client
            .delete(format!(
                "{}/api/v1/services/{}",
                songbird_endpoint, service_id
            ))
            .send()
            .await
            .map_err(|e| {
                BiomeError::NetworkError(format!("Failed to unregister from Songbird: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BiomeError::NetworkError(format!(
                "Songbird unregistration failed with status {}: {}",
                status, error_text
            )));
        }

        info!(
            "Successfully unregistered service {} from Songbird",
            service_id
        );
        Ok(())
    }

    /// Register a service in the ecosystem
    pub async fn register_service(
        &self,
        mut registration: EcosystemServiceRegistration,
    ) -> BiomeResult<()> {
        info!("Registering ecosystem service: {}", registration.service_id);

        // Set registration timestamp
        registration.registration_time = Utc::now();

        // Validate registration
        self.validate_registration(&registration)?;

        // Register with Songbird first
        self.register_service_with_songbird(&registration).await?;

        // Store registration locally
        {
            let mut services = self.services.write().await;
            services.insert(registration.service_id.clone(), registration.clone());
        }

        // Broadcast registration event
        let message = EcosystemMessage {
            message_id: Uuid::new_v4(),
            from_primal: "biomeos".to_string(),
            to_primal: "all".to_string(),
            message_type: EcosystemMessageType::ServiceRegistration,
            payload: serde_json::to_value(&registration)?,
            timestamp: Utc::now(),
            correlation_id: None,
        };

        self.message_bus.broadcast(message).await?;

        info!(
            "Service registered successfully: {}",
            registration.service_id
        );
        Ok(())
    }

    /// Unregister a service from the ecosystem
    pub async fn unregister_service(&self, service_id: &str) -> BiomeResult<()> {
        info!("Unregistering ecosystem service: {}", service_id);

        // Unregister from Songbird first
        self.unregister_service_from_songbird(service_id).await?;

        // Remove from local registry
        {
            let mut services = self.services.write().await;
            services.remove(service_id);
        }

        // Broadcast unregistration event
        let message = EcosystemMessage {
            message_id: Uuid::new_v4(),
            from_primal: "biomeos".to_string(),
            to_primal: "all".to_string(),
            message_type: EcosystemMessageType::ServiceDeregistration,
            payload: serde_json::json!({ "service_id": service_id }),
            timestamp: Utc::now(),
            correlation_id: None,
        };

        self.message_bus.broadcast(message).await?;

        info!("Service unregistered successfully: {}", service_id);
        Ok(())
    }

    /// Get all services of a specific type
    pub async fn get_services_by_type(
        &self,
        primal_type: &str,
    ) -> Vec<EcosystemServiceRegistration> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| s.primal_type == primal_type)
            .cloned()
            .collect()
    }

    /// Get all services in a biome
    pub async fn get_services_by_biome(&self, biome_id: &str) -> Vec<EcosystemServiceRegistration> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| s.biome_id == biome_id)
            .cloned()
            .collect()
    }

    /// Get service by ID
    pub async fn get_service(&self, service_id: &str) -> Option<EcosystemServiceRegistration> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// List all registered services
    pub async fn list_services(&self) -> BiomeResult<Vec<EcosystemServiceRegistration>> {
        let services = self.services.read().await;
        Ok(services.values().cloned().collect())
    }

    /// Check health of all services
    pub async fn check_ecosystem_health(&self) -> BiomeResult<EcosystemHealthStatus> {
        let services = self.services.read().await;
        let mut healthy_services = 0;
        let mut total_services = 0;
        let mut primal_health = HashMap::new();

        for service in services.values() {
            total_services += 1;

            // Check service health (simplified - would make HTTP call in real implementation)
            let is_healthy = self.check_service_health(service).await.unwrap_or(false);
            if is_healthy {
                healthy_services += 1;
            }

            // Track per-primal health
            let primal_count = primal_health
                .entry(service.primal_type.clone())
                .or_insert((0, 0));
            primal_count.1 += 1; // total
            if is_healthy {
                primal_count.0 += 1; // healthy
            }
        }

        let overall_health = if healthy_services == total_services {
            HealthStatus::Healthy
        } else if healthy_services >= (total_services * 2 / 3) {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };

        Ok(EcosystemHealthStatus {
            overall_health,
            healthy_services,
            total_services,
            primal_health: primal_health
                .into_iter()
                .map(|(primal, (healthy, total))| {
                    let health = if healthy == total {
                        HealthStatus::Healthy
                    } else if healthy >= (total * 2 / 3) {
                        HealthStatus::Warning
                    } else {
                        HealthStatus::Critical
                    };
                    (
                        primal,
                        PrimalHealthInfo {
                            health,
                            healthy_count: healthy,
                            total_count: total,
                        },
                    )
                })
                .collect(),
        })
    }

    /// Check health of a specific service
    pub async fn check_service_health(
        &self,
        service: &EcosystemServiceRegistration,
    ) -> BiomeResult<bool> {
        // Build health check URL
        let health_url = format!("{}/health", service.endpoints.primary);

        // Create HTTP client with reasonable timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| {
                BiomeError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;

        // Make health check request
        match client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Try to parse health response
                    match response.json::<serde_json::Value>().await {
                        Ok(health_data) => {
                            // Check if service reports itself as healthy
                            if let Some(status) = health_data.get("status") {
                                Ok(status.as_str() == Some("healthy")
                                    || status.as_str() == Some("up"))
                            } else {
                                // If no status field, assume healthy if we got a 200 response
                                Ok(true)
                            }
                        }
                        Err(_) => {
                            // If JSON parsing fails, assume healthy if we got a 200 response
                            Ok(true)
                        }
                    }
                } else {
                    debug!(
                        "Health check failed for {}: HTTP {}",
                        service.service_id,
                        response.status()
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                debug!("Health check failed for {}: {}", service.service_id, e);
                Ok(false)
            }
        }
    }

    fn validate_registration(
        &self,
        registration: &EcosystemServiceRegistration,
    ) -> BiomeResult<()> {
        if registration.service_id.is_empty() {
            return Err(BiomeError::InvalidInput(
                "Service ID cannot be empty".to_string(),
            ));
        }

        if registration.primal_type.is_empty() {
            return Err(BiomeError::InvalidInput(
                "Primal type cannot be empty".to_string(),
            ));
        }

        if registration.endpoints.primary.is_empty() {
            return Err(BiomeError::InvalidInput(
                "Primary endpoint cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}
