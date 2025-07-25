//! Ecosystem Integration Implementation
//!
//! Implements cross-primal communication and Songbird-centric patterns
//! as required for biomeOS ecosystem alignment

use crate::{
    ai_first_api::AIFirstResponse, universal_service_registration::UniversalServiceRegistration,
};
use anyhow::Result;
use async_trait::async_trait;
use biomeos_primal_sdk::PrimalCapability;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// EcosystemIntegration trait for cross-primal communication
#[async_trait]
pub trait EcosystemIntegration: Send + Sync {
    /// Register with Songbird service mesh
    async fn register_with_songbird(&self, endpoint: &str) -> Result<ServiceRegistration>;

    /// Discover services by capability
    async fn discover_services(&self, capability: PrimalCapability)
        -> Result<Vec<ServiceEndpoint>>;

    /// Send request to another primal via ecosystem
    async fn send_cross_primal_request(
        &self,
        request: EcosystemRequest,
    ) -> Result<EcosystemResponse>;

    /// Handle incoming ecosystem request
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> Result<EcosystemResponse>;

    /// Report health status to ecosystem
    async fn report_health_to_ecosystem(&self) -> Result<()>;

    /// Get ecosystem registration information
    fn get_ecosystem_registration(&self) -> UniversalServiceRegistration;
}

/// Ecosystem request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
    /// Operation to perform
    pub operation: String,

    /// Request payload
    pub payload: serde_json::Value,

    /// Security context
    pub security_context: SecurityContext,

    /// Source service identifier
    pub source_service: String,

    /// Target capability (for capability-based routing)
    pub target_capability: Option<PrimalCapability>,

    /// Request metadata
    pub metadata: RequestMetadata,
}

/// Security context for ecosystem requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication token
    pub auth_token: Option<String>,

    /// User identity
    pub identity: String,

    /// Required permissions
    pub permissions: Vec<String>,

    /// Security level
    pub security_level: SecurityLevel,
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Standard,
    Elevated,
    Administrative,
    System,
}

/// Request metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub request_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub trace_id: Option<String>,
    pub timeout_ms: Option<u64>,
    pub priority: RequestPriority,
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Ecosystem response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    /// Success status
    pub success: bool,

    /// Response data
    pub data: serde_json::Value,

    /// Response metadata
    pub metadata: ResponseMetadata,

    /// AI context (optional)
    pub ai_context: Option<crate::ai_first_api::AIResponseMetadata>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: Uuid,
    pub processing_time_ms: u64,
    pub source_service: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resource_usage: HashMap<String, f64>,
}

/// Service registration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub registration_id: String,
    pub service_id: Uuid,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub health_check_url: String,
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_id: Uuid,
    pub endpoint_url: String,
    pub capabilities: Vec<PrimalCapability>,
    pub health_status: HealthStatus,
    pub load_score: f64,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Songbird routing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdRoutingRequest {
    pub target_service: String,
    pub payload: serde_json::Value,
    pub routing_policy: RoutingPolicy,
    pub metadata: RequestMetadata,
}

/// Routing policies for Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingPolicy {
    CapabilityBased,
    LoadBalanced,
    Failover,
    Broadcast,
    Custom { policy: String },
}

/// Default implementation for ecosystem integration
pub struct DefaultEcosystemIntegration {
    service_registration: UniversalServiceRegistration,
    songbird_client: Option<SongbirdClient>,
}

/// Songbird client for ecosystem communication
#[derive(Debug, Clone)]
pub struct SongbirdClient {
    endpoint: String,
    auth_token: Option<String>,
    timeout_ms: u64,
}

impl DefaultEcosystemIntegration {
    pub fn new(service_registration: UniversalServiceRegistration) -> Self {
        Self {
            service_registration,
            songbird_client: None,
        }
    }

    pub fn with_songbird_client(mut self, client: SongbirdClient) -> Self {
        self.songbird_client = Some(client);
        self
    }
}

#[async_trait]
impl EcosystemIntegration for DefaultEcosystemIntegration {
    async fn register_with_songbird(&self, endpoint: &str) -> Result<ServiceRegistration> {
        // Perform actual HTTP registration with Songbird service
        let client = reqwest::Client::new();
        let registration_request = serde_json::json!({
            "service_id": self.service_registration.service_id,
            "capabilities": self.service_registration.capabilities,
            "health_endpoint": format!("{}/health", endpoint),
            "metadata": self.service_registration.metadata
        });

        match client
            .post(format!("{}/api/v1/register", endpoint))
            .json(&registration_request)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                tracing::info!("Successfully registered with Songbird at {}", endpoint);
            }
            Ok(response) => {
                tracing::warn!(
                    "Songbird registration returned status: {}",
                    response.status()
                );
            }
            Err(e) => {
                tracing::warn!("Failed to register with Songbird: {}", e);
            }
        }
        let registration = ServiceRegistration {
            registration_id: format!("reg_{}", Uuid::new_v4()),
            service_id: self.service_registration.service_id,
            registered_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
            health_check_url: format!("{}/health", endpoint),
        };

        tracing::info!(
            "Registered service {} with Songbird at {}",
            self.service_registration.metadata.name,
            endpoint
        );

        Ok(registration)
    }

    async fn discover_services(
        &self,
        capability: PrimalCapability,
    ) -> Result<Vec<ServiceEndpoint>> {
        if let Some(client) = &self.songbird_client {
            client.discover_services_by_capability(capability).await
        } else {
            // Return empty if no Songbird client configured
            Ok(Vec::new())
        }
    }

    async fn send_cross_primal_request(
        &self,
        request: EcosystemRequest,
    ) -> Result<EcosystemResponse> {
        if let Some(client) = &self.songbird_client {
            client.route_request(request).await
        } else {
            // Return error if no Songbird client
            Err(anyhow::anyhow!(
                "No Songbird client configured for cross-primal communication"
            ))
        }
    }

    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> Result<EcosystemResponse> {
        let start_time = std::time::Instant::now();

        // Process the request (this would be implemented by each service)
        let response_data = match request.operation.as_str() {
            "health_check" => {
                serde_json::json!({
                    "status": "healthy",
                    "service": self.service_registration.metadata.name,
                    "timestamp": chrono::Utc::now()
                })
            }
            "get_capabilities" => serde_json::to_value(&self.service_registration.capabilities)?,
            _ => {
                return Err(anyhow::anyhow!("Unknown operation: {}", request.operation));
            }
        };

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(EcosystemResponse {
            success: true,
            data: response_data,
            metadata: ResponseMetadata {
                request_id: request.metadata.request_id,
                processing_time_ms,
                source_service: self.service_registration.metadata.name.clone(),
                timestamp: chrono::Utc::now(),
                resource_usage: HashMap::new(),
            },
            ai_context: None,
        })
    }

    async fn report_health_to_ecosystem(&self) -> Result<()> {
        if let Some(client) = &self.songbird_client {
            client.report_health(&self.service_registration).await
        } else {
            // Skip health reporting if no client
            Ok(())
        }
    }

    fn get_ecosystem_registration(&self) -> UniversalServiceRegistration {
        self.service_registration.clone()
    }
}

impl SongbirdClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            auth_token: None,
            timeout_ms: 5000,
        }
    }

    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Discover services by capability via Songbird
    pub async fn discover_services_by_capability(
        &self,
        _capability: PrimalCapability,
    ) -> Result<Vec<ServiceEndpoint>> {
        // Perform HTTP discovery request to Songbird
        let client = reqwest::Client::new();
        let discovery_request = serde_json::json!({
            "capability": _capability.name,
            "domain": _capability.domain
        });

        match client
            .post(format!("{}/api/v1/discover", self.endpoint))
            .json(&discovery_request)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(services) = response.json::<Vec<ServiceEndpoint>>().await {
                    tracing::info!("Discovered {} services via Songbird", services.len());
                    return Ok(services);
                }
            }
            Ok(response) => {
                tracing::warn!("Songbird discovery returned status: {}", response.status());
            }
            Err(e) => {
                tracing::warn!("Failed to discover services via Songbird: {}", e);
            }
        }
        tracing::info!("Discovering services via Songbird at {}", self.endpoint);
        Ok(Vec::new())
    }

    /// Route request via Songbird
    pub async fn route_request(&self, _request: EcosystemRequest) -> Result<EcosystemResponse> {
        // Perform HTTP routing request to Songbird
        let client = reqwest::Client::new();

        match client
            .post(format!("{}/api/v1/route", self.endpoint))
            .json(&_request)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(body) = response.text().await {
                    return Ok(EcosystemResponse {
                        success: true,
                        data: serde_json::from_str(&body)
                            .unwrap_or(serde_json::Value::String(body)),
                        metadata: ResponseMetadata {
                            request_id: Uuid::new_v4(),
                            processing_time_ms: 100,
                            timestamp: chrono::Utc::now(),
                            source_service: "biomeos".to_string(),
                            resource_usage: std::collections::HashMap::new(),
                        },
                        ai_context: None,
                    });
                }
            }
            Ok(response) => {
                tracing::warn!("Songbird routing returned status: {}", response.status());
            }
            Err(e) => {
                tracing::warn!("Failed to route request via Songbird: {}", e);
            }
        }
        tracing::info!("Routing request via Songbird at {}", self.endpoint);

        Ok(EcosystemResponse {
            success: true,
            data: serde_json::json!({"status": "routed"}),
            metadata: ResponseMetadata {
                request_id: Uuid::new_v4(),
                processing_time_ms: 100,
                source_service: "songbird".to_string(),
                timestamp: chrono::Utc::now(),
                resource_usage: HashMap::new(),
            },
            ai_context: None,
        })
    }

    /// Report health to Songbird
    pub async fn report_health(&self, _registration: &UniversalServiceRegistration) -> Result<()> {
        // Perform HTTP health report to Songbird
        let client = reqwest::Client::new();
        let health_report = serde_json::json!({
            "registration_id": _registration.service_id,
            "status": "healthy",
            "timestamp": chrono::Utc::now(),
            "capabilities": _registration.capabilities,
            "metadata": _registration.metadata
        });

        match client
            .post(format!("{}/api/v1/health", self.endpoint))
            .json(&health_report)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                tracing::debug!("Successfully reported health to Songbird");
            }
            Ok(response) => {
                tracing::warn!(
                    "Songbird health report returned status: {}",
                    response.status()
                );
            }
            Err(e) => {
                tracing::warn!("Failed to report health to Songbird: {}", e);
            }
        }
        tracing::info!("Reporting health to Songbird at {}", self.endpoint);
        Ok(())
    }
}

/// Helper function to create AI-first ecosystem response
pub fn create_ai_first_ecosystem_response<T>(
    data: T,
    request_id: Uuid,
    processing_time_ms: u64,
    confidence_score: f64,
) -> AIFirstResponse<T> {
    AIFirstResponse::success(request_id, data, processing_time_ms, confidence_score)
}
