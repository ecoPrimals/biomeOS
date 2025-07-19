//! # Ecosystem Integration Module
//!
//! Core integration layer that unifies biomeOS, Songbird, NestGate, and Toadstool
//! into a single cohesive ecosystem with standardized communication and coordination.
//!
//! This module has been refactored into focused sub-modules for better maintainability:
//! - `types`: Core data structures and enums
//! - `service_registry`: Service registration and health checking
//! - `messaging`: Message bus and communication protocols
//! - `health_monitoring`: Comprehensive health monitoring and coordination
//! - `coordinator`: Main ecosystem coordinator

pub mod coordinator;
pub mod health_monitoring;
pub mod messaging;
pub mod service_registry;
pub mod types;

// Re-export main types and structs for backward compatibility
pub use coordinator::EcosystemCoordinator;
pub use health_monitoring::EcosystemHealthCoordinator;
pub use messaging::{
    EcosystemCommunication, EcosystemMessageBus, InMemoryMessageBus, PrimalClient,
};
pub use service_registry::EcosystemServiceRegistry;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_service_registration() {
        let message_bus = Arc::new(InMemoryMessageBus::new());
        let registry = EcosystemServiceRegistry::new(message_bus);

        let registration = EcosystemServiceRegistration {
            service_id: "primal-songbird-001".to_string(),
            primal_type: "songbird".to_string(),
            biome_id: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            api_version: "biomeOS/v1".to_string(),
            registration_time: Utc::now(),
            endpoints: EcosystemEndpoints {
                primary: "http://localhost:8080".to_string(),
                health: "http://localhost:8080/health".to_string(),
                metrics: "http://localhost:8080/metrics".to_string(),
                admin: None,
                websocket: None,
            },
            capabilities: EcosystemCapabilities {
                core: vec!["orchestration".to_string()],
                extended: vec!["federation".to_string()],
                integrations: vec!["toadstool".to_string()],
            },
            security: EcosystemSecurity {
                authentication_method: "ecosystem_jwt".to_string(),
                tls_enabled: true,
                mtls_required: false,
                trust_domain: "biome.local".to_string(),
            },
            resource_requirements: ResourceRequirements {
                cpu: "2".to_string(),
                memory: "4Gi".to_string(),
                storage: "10Gi".to_string(),
                network: "1Gbps".to_string(),
            },
            health_check: HealthCheckConfig {
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(10),
                retries: 3,
                grace_period: Duration::from_secs(60),
            },
            metadata: HashMap::new(),
        };

        // Register service
        registry
            .register_service(registration.clone())
            .await
            .unwrap();

        // Verify registration
        let services = registry.get_services_by_type("songbird").await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].service_id, registration.service_id);
    }

    #[tokio::test]
    async fn test_ecosystem_health() {
        let message_bus = Arc::new(InMemoryMessageBus::new());
        let registry = EcosystemServiceRegistry::new(message_bus);

        // Check health with no services
        let health = registry.check_ecosystem_health().await.unwrap();
        assert_eq!(health.total_services, 0);

        // Add a service and check again
        let registration = EcosystemServiceRegistration {
            service_id: "test-service".to_string(),
            primal_type: "test".to_string(),
            biome_id: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            api_version: "biomeOS/v1".to_string(),
            registration_time: Utc::now(),
            endpoints: EcosystemEndpoints {
                primary: "http://localhost:8080".to_string(),
                health: "http://localhost:8080/health".to_string(),
                metrics: "http://localhost:8080/metrics".to_string(),
                admin: None,
                websocket: None,
            },
            capabilities: EcosystemCapabilities {
                core: vec![],
                extended: vec![],
                integrations: vec![],
            },
            security: EcosystemSecurity {
                authentication_method: "test".to_string(),
                tls_enabled: false,
                mtls_required: false,
                trust_domain: "test".to_string(),
            },
            resource_requirements: ResourceRequirements {
                cpu: "1".to_string(),
                memory: "1Gi".to_string(),
                storage: "1Gi".to_string(),
                network: "100Mbps".to_string(),
            },
            health_check: HealthCheckConfig {
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                retries: 3,
                grace_period: Duration::from_secs(30),
            },
            metadata: HashMap::new(),
        };

        registry.register_service(registration).await.unwrap();

        let health = registry.check_ecosystem_health().await.unwrap();
        assert_eq!(health.total_services, 1);
        assert_eq!(health.healthy_services, 1);
    }

    #[tokio::test]
    async fn test_ecosystem_coordinator() {
        let coordinator = EcosystemCoordinator::new();

        // Test initialization
        coordinator.initialize_ecosystem().await.unwrap();

        // Test status
        let status = coordinator.get_ecosystem_status().await.unwrap();
        assert_eq!(status.total_services, 0);
        assert_eq!(status.active_primals, 0);

        // Test shutdown
        coordinator.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_health_monitoring() {
        let message_bus = Arc::new(InMemoryMessageBus::new());
        let registry = Arc::new(EcosystemServiceRegistry::new(message_bus));
        let health_coordinator = EcosystemHealthCoordinator::new(registry.clone());

        // Test initialization
        health_coordinator.initialize().await.unwrap();

        // Test health status generation
        let health_status = health_coordinator.get_ecosystem_health().await.unwrap();
        assert_eq!(health_status.service_health.len(), 0);
        assert_eq!(health_status.primal_health.len(), 0);
    }
}
pub use types::EcosystemStatus;
