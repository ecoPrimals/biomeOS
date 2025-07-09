//! # Ecosystem Integration Module
//!
//! Core integration layer that unifies biomeOS, Songbird, NestGate, and Toadstool
//! into a single cohesive ecosystem with standardized communication and coordination.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{BiomeResult, BiomeError, HealthStatus, PrimalType};

/// Unified service registration for all Primals in the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique service identifier: "primal-{type}-{instance}"
    pub service_id: String,
    /// Type of Primal (toadstool, songbird, nestgate, etc.)
    pub primal_type: PrimalType,
    /// Biome instance this service belongs to
    pub biome_id: String,
    /// Semantic version of the service
    pub version: String,
    /// API version (e.g., "biomeOS/v1")
    pub api_version: String,
    /// When this service was registered
    pub registration_time: DateTime<Utc>,
    
    /// Service endpoints
    pub endpoints: EcosystemEndpoints,
    /// Service capabilities
    pub capabilities: EcosystemCapabilities,
    /// Security configuration
    pub security: EcosystemSecurity,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Standardized endpoints for ecosystem services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEndpoints {
    /// Primary API endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: String,
    /// Admin interface (optional)
    pub admin: Option<String>,
    /// WebSocket endpoint for real-time updates (optional)
    pub websocket: Option<String>,
}

/// Capabilities provided by a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemCapabilities {
    /// Core capabilities (always available)
    pub core: Vec<String>,
    /// Extended features (may be optional)
    pub extended: Vec<String>,
    /// Integration points with other Primals
    pub integrations: Vec<String>,
}

/// Security configuration for ecosystem services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemSecurity {
    /// Authentication method
    pub authentication_method: String,
    /// Whether TLS is enabled
    pub tls_enabled: bool,
    /// Whether mutual TLS is required
    pub mtls_required: bool,
    /// Trust domain for this service
    pub trust_domain: String,
}

/// Resource requirements for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU requirement
    pub cpu: String,
    /// Memory requirement
    pub memory: String,
    /// Storage requirement
    pub storage: String,
    /// Network requirement
    pub network: String,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Check interval
    pub interval: Duration,
    /// Check timeout
    pub timeout: Duration,
    /// Number of retries before marking unhealthy
    pub retries: u32,
    /// Grace period for startup
    pub grace_period: Duration,
}

/// Inter-Primal communication message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMessage {
    /// Unique message identifier
    pub message_id: Uuid,
    /// Source Primal
    pub from_primal: PrimalType,
    /// Destination Primal
    pub to_primal: PrimalType,
    /// Message type
    pub message_type: EcosystemMessageType,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for request/response tracking
    pub correlation_id: Option<Uuid>,
}

/// Types of ecosystem messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemMessageType {
    // Service coordination
    ServiceRegistration,
    ServiceDeregistration,
    HealthCheck,
    
    // Resource coordination
    ResourceRequest,
    ResourceAllocation,
    ResourceRelease,
    
    // Workload coordination
    WorkloadRequest,
    WorkloadStatus,
    WorkloadComplete,
    
    // Storage coordination
    VolumeProvisionRequest,
    VolumeProvisionComplete,
    MountRequest,
    MountComplete,
    
    // Ecosystem events
    EcosystemStateChange,
    PrimalStatusUpdate,
    ErrorNotification,
}

/// Trait for ecosystem communication
#[async_trait]
pub trait EcosystemCommunication: Send + Sync {
    /// Send a message to another Primal
    async fn send_message(&self, message: EcosystemMessage) -> BiomeResult<()>;
    
    /// Handle an incoming message
    async fn handle_message(&mut self, message: EcosystemMessage) -> BiomeResult<Option<EcosystemMessage>>;
    
    /// Broadcast status to the ecosystem
    async fn broadcast_status(&self) -> BiomeResult<()>;
    
    /// Register with the ecosystem
    async fn register_service(&self, registration: EcosystemServiceRegistration) -> BiomeResult<()>;
    
    /// Deregister from the ecosystem
    async fn deregister_service(&self, service_id: &str) -> BiomeResult<()>;
}

/// Ecosystem service registry
pub struct EcosystemServiceRegistry {
    services: Arc<RwLock<HashMap<String, EcosystemServiceRegistration>>>,
    message_bus: Arc<dyn EcosystemMessageBus>,
}

impl EcosystemServiceRegistry {
    pub fn new(message_bus: Arc<dyn EcosystemMessageBus>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
        }
    }
    
    /// Register a service in the ecosystem
    pub async fn register_service(&self, registration: EcosystemServiceRegistration) -> BiomeResult<()> {
        info!("Registering ecosystem service: {}", registration.service_id);
        
        // Validate registration
        self.validate_registration(&registration)?;
        
        // Store registration
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
        
        info!("Service registered successfully: {}", registration.service_id);
        Ok(())
    }
    
    /// Get all services of a specific type
    pub async fn get_services_by_type(&self, primal_type: &str) -> Vec<EcosystemServiceRegistration> {
        let services = self.services.read().await;
        services.values()
            .filter(|s| s.primal_type == primal_type)
            .cloned()
            .collect()
    }
    
    /// Get all services in a biome
    pub async fn get_services_by_biome(&self, biome_id: &str) -> Vec<EcosystemServiceRegistration> {
        let services = self.services.read().await;
        services.values()
            .filter(|s| s.biome_id == biome_id)
            .cloned()
            .collect()
    }
    
    /// Get service by ID
    pub async fn get_service(&self, service_id: &str) -> Option<EcosystemServiceRegistration> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
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
            let primal_count = primal_health.entry(service.primal_type.clone()).or_insert((0, 0));
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
            primal_health: primal_health.into_iter()
                .map(|(primal, (healthy, total))| {
                    let health = if healthy == total {
                        HealthStatus::Healthy
                    } else if healthy >= (total * 2 / 3) {
                        HealthStatus::Warning
                    } else {
                        HealthStatus::Critical
                    };
                    (primal, PrimalHealthInfo { health, healthy_count: healthy, total_count: total })
                })
                .collect(),
        })
    }
    
    async fn check_service_health(&self, service: &EcosystemServiceRegistration) -> BiomeResult<bool> {
        // In a real implementation, this would make an HTTP request to the health endpoint
        // For now, we'll assume services are healthy if they're registered
        Ok(true)
    }
    
    fn validate_registration(&self, registration: &EcosystemServiceRegistration) -> BiomeResult<()> {
        if registration.service_id.is_empty() {
            return Err(BiomeError::InvalidInput("Service ID cannot be empty".to_string()));
        }
        
        if registration.primal_type.is_empty() {
            return Err(BiomeError::InvalidInput("Primal type cannot be empty".to_string()));
        }
        
        if registration.endpoints.primary.is_empty() {
            return Err(BiomeError::InvalidInput("Primary endpoint cannot be empty".to_string()));
        }
        
        Ok(())
    }
}

/// Ecosystem health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealthStatus {
    pub overall_health: HealthStatus,
    pub healthy_services: usize,
    pub total_services: usize,
    pub primal_health: HashMap<String, PrimalHealthInfo>,
}

/// Health information for a specific Primal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealthInfo {
    pub health: HealthStatus,
    pub healthy_count: usize,
    pub total_count: usize,
}

/// Message bus for ecosystem communication
#[async_trait]
pub trait EcosystemMessageBus: Send + Sync {
    /// Send a message to a specific Primal
    async fn send(&self, message: EcosystemMessage) -> BiomeResult<()>;
    
    /// Broadcast a message to all Primals
    async fn broadcast(&self, message: EcosystemMessage) -> BiomeResult<()>;
    
    /// Subscribe to messages of a specific type
    async fn subscribe(&self, message_type: EcosystemMessageType) -> BiomeResult<()>;
}

/// In-memory message bus implementation for testing and development
pub struct InMemoryMessageBus {
    subscribers: Arc<RwLock<HashMap<String, Vec<tokio::sync::mpsc::UnboundedSender<EcosystemMessage>>>>>,
}

impl InMemoryMessageBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EcosystemMessageBus for InMemoryMessageBus {
    async fn send(&self, message: EcosystemMessage) -> BiomeResult<()> {
        debug!("Sending message: {} -> {}", message.from_primal, message.to_primal);
        
        let subscribers = self.subscribers.read().await;
        if let Some(senders) = subscribers.get(&message.to_primal) {
            for sender in senders {
                if let Err(e) = sender.send(message.clone()) {
                    warn!("Failed to send message to subscriber: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn broadcast(&self, message: EcosystemMessage) -> BiomeResult<()> {
        debug!("Broadcasting message from: {}", message.from_primal);
        
        let subscribers = self.subscribers.read().await;
        for senders in subscribers.values() {
            for sender in senders {
                if let Err(e) = sender.send(message.clone()) {
                    warn!("Failed to broadcast message to subscriber: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn subscribe(&self, message_type: EcosystemMessageType) -> BiomeResult<()> {
        // Implementation would set up subscription channels
        // For now, this is a placeholder
        Ok(())
    }
}

/// Ecosystem coordinator that manages the integration between all Primals
pub struct EcosystemCoordinator {
    pub service_registry: EcosystemServiceRegistry,
    message_bus: Arc<dyn EcosystemMessageBus>,
    primal_clients: HashMap<String, Arc<dyn PrimalClient>>,
}

impl EcosystemCoordinator {
    pub fn new() -> Self {
        let message_bus = Arc::new(InMemoryMessageBus::new());
        let service_registry = EcosystemServiceRegistry::new(message_bus.clone());
        
        Self {
            service_registry,
            message_bus,
            primal_clients: HashMap::new(),
        }
    }
    
    /// Register a Primal client
    pub fn register_primal_client(&mut self, primal_type: String, client: Arc<dyn PrimalClient>) {
        self.primal_clients.insert(primal_type, client);
    }
    
    /// Initialize the ecosystem
    pub async fn initialize_ecosystem(&self) -> BiomeResult<()> {
        info!("Initializing ecosystem coordination");
        
        // Initialize all registered Primal clients
        for (primal_type, client) in &self.primal_clients {
            info!("Initializing {} client", primal_type);
            client.initialize().await?;
        }
        
        info!("Ecosystem coordination initialized successfully");
        Ok(())
    }
    
    /// Get ecosystem status
    pub async fn get_ecosystem_status(&self) -> BiomeResult<EcosystemStatus> {
        let health = self.service_registry.check_ecosystem_health().await?;
        let services = {
            let services = self.service_registry.services.read().await;
            services.len()
        };
        
        Ok(EcosystemStatus {
            health,
            total_services: services,
            active_primals: self.primal_clients.len(),
            uptime: Duration::from_secs(0), // Would track actual uptime
        })
    }
}

/// Overall ecosystem status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    pub health: EcosystemHealthStatus,
    pub total_services: usize,
    pub active_primals: usize,
    pub uptime: Duration,
}

/// Trait for Primal clients
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Initialize the Primal client
    async fn initialize(&self) -> BiomeResult<()>;
    
    /// Get Primal health status
    async fn health_check(&self) -> BiomeResult<HealthStatus>;
    
    /// Send a message to this Primal
    async fn send_message(&self, message: EcosystemMessage) -> BiomeResult<Option<EcosystemMessage>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
        registry.register_service(registration.clone()).await.unwrap();
        
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
} 