//! Universal Adapter
//!
//! Provides universal coordination between Toadstool and Songbird primals

use anyhow::Result;
use biomeos_core::{BiomeError, BiomeResult};
use chrono;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid;

#[derive(Debug, Clone)]
pub struct UniversalAdapter {
    pub config: AdapterConfig,
    pub toadstool_client: Option<ToadstoolClient>,
    pub songbird_client: Option<SongbirdClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub toadstool_endpoint: String,
    pub songbird_endpoint: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            toadstool_endpoint: std::env::var("TOADSTOOL_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string())
                .to_string(),
            songbird_endpoint: std::env::var("SONGBIRD_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToadstoolClient {
    endpoint: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct SongbirdClient {
    endpoint: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    pub operation: String,
    pub payload: serde_json::Value,
    pub metadata: RequestMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub priority: RequestPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub processing_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

impl UniversalAdapter {
    /// Create new universal adapter
    pub fn new(config: AdapterConfig) -> Self {
        Self {
            config,
            toadstool_client: None,
            songbird_client: None,
        }
    }

    /// Initialize the adapter
    pub async fn initialize(&mut self) -> BiomeResult<()> {
        tracing::info!("Initializing Universal Adapter");

        // Initialize Toadstool client
        self.toadstool_client = Some(
            ToadstoolClient::new(&self.config.toadstool_endpoint)
                .await
                .map_err(|e| BiomeError::Configuration {
                    message: format!("Failed to create HTTP client: {}", e),
                })?,
        );

        // Initialize Songbird client
        self.songbird_client = Some(SongbirdClient::new(&self.config.songbird_endpoint).await);

        tracing::info!("Universal Adapter initialized successfully");
        Ok(())
    }

    /// Process universal request
    pub async fn process_request(
        &self,
        request: UniversalRequest,
    ) -> BiomeResult<UniversalResponse> {
        let start_time = std::time::Instant::now();

        tracing::debug!("Processing universal request: {}", request.operation);

        let result = match request.operation.as_str() {
            "parse_and_validate" => self.parse_and_validate(request.payload).await,
            "execute_with_coordination" => self.execute_with_coordination(request.payload).await,
            "discover_and_route" => self.discover_and_route(request.payload).await,
            _ => {
                return Ok(UniversalResponse {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(format!("Unknown operation: {}", request.operation)),
                    metadata: ResponseMetadata {
                        request_id: request.metadata.request_id,
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        source: "universal_adapter".to_string(),
                    },
                });
            }
        };

        match result {
            Ok(data) => Ok(UniversalResponse {
                success: true,
                data,
                error: None,
                metadata: ResponseMetadata {
                    request_id: request.metadata.request_id,
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    source: "universal_adapter".to_string(),
                },
            }),
            Err(e) => Ok(UniversalResponse {
                success: false,
                data: serde_json::Value::Null,
                error: Some(e.to_string()),
                metadata: ResponseMetadata {
                    request_id: request.metadata.request_id,
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    source: "universal_adapter".to_string(),
                },
            }),
        }
    }

    /// Parse and validate using Toadstool
    async fn parse_and_validate(
        &self,
        payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let client = self
            .toadstool_client
            .as_ref()
            .ok_or_else(|| BiomeError::Configuration {
                message: "Toadstool client not initialized".to_string(),
            })?;

        client
            .parse_and_validate(payload)
            .await
            .map_err(|e| BiomeError::Configuration {
                message: format!("Toadstool parsing failed: {}", e),
            })?;

        client
            .discover_primals()
            .await
            .map_err(|e| BiomeError::Discovery {
                message: format!("Songbird discovery failed: {}", e),
            })
    }

    /// Execute with coordination
    async fn execute_with_coordination(
        &self,
        payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let toadstool =
            self.toadstool_client
                .as_ref()
                .ok_or_else(|| BiomeError::Configuration {
                    message: "Toadstool client not initialized".to_string(),
                })?;

        let songbird = self
            .songbird_client
            .as_ref()
            .ok_or_else(|| BiomeError::Configuration {
                message: "Songbird client not initialized".to_string(),
            })?;

        // Execute via Toadstool
        let execution_result =
            toadstool
                .execute(payload)
                .await
                .map_err(|e| BiomeError::Internal {
                    message: format!("Toadstool execution failed: {}", e),
                })?;

        // Coordinate via Songbird
        songbird
            .register_execution(&execution_result)
            .await
            .map_err(|e| BiomeError::Integration {
                message: format!("Songbird registration failed: {}", e),
            })?;

        Ok(execution_result)
    }

    /// Discover and route
    async fn discover_and_route(
        &self,
        payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let songbird = self
            .songbird_client
            .as_ref()
            .ok_or_else(|| BiomeError::Configuration {
                message: "Songbird client not initialized".to_string(),
            })?;

        songbird
            .discover_and_route(payload)
            .await
            .map_err(|e| BiomeError::Discovery {
                message: format!("Failed to discover primals: {}", e),
            })
    }

    /// Get system status
    pub async fn get_system_status(&self) -> BiomeResult<SystemStatus> {
        let toadstool_status = if let Some(client) = &self.toadstool_client {
            client
                .get_status()
                .await
                .unwrap_or(ServiceStatus::Unavailable)
        } else {
            ServiceStatus::NotInitialized
        };

        let songbird_status = if let Some(client) = &self.songbird_client {
            client
                .get_status()
                .await
                .unwrap_or(ServiceStatus::Unavailable)
        } else {
            ServiceStatus::NotInitialized
        };

        Ok(SystemStatus {
            toadstool: toadstool_status,
            songbird: songbird_status,
            adapter_uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

impl ToadstoolClient {
    pub async fn new(endpoint: &str) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| BiomeError::Configuration {
                message: format!("Failed to create Toadstool client: {}", e),
            })?;

        Ok(Self {
            endpoint: endpoint.to_string(),
            http_client,
        })
    }

    pub async fn parse_and_validate(&self, _payload: serde_json::Value) -> Result<()> {
        info!("🍄 Parsing and validating via Toadstool API");

        let validation_url = format!("{}/api/v1/validate", self.endpoint);

        match self
            .http_client
            .post(&validation_url)
            .header("Content-Type", "application/json")
            .json(&_payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Toadstool validation successful");
                    Ok(())
                } else {
                    let error = format!("Toadstool validation failed: {}", response.status());
                    warn!("{}", error);
                    Err(anyhow::anyhow!(error))
                }
            }
            Err(e) => {
                let error = format!("Failed to connect to Toadstool for validation: {}", e);
                warn!("{}", error);
                Err(anyhow::anyhow!(error))
            }
        }
    }

    pub async fn execute(&self, payload: serde_json::Value) -> Result<serde_json::Value> {
        info!("🍄 Executing computation via Toadstool");

        let execution_url = format!("{}/api/v1/execute", self.endpoint);

        match self
            .http_client
            .post(&execution_url)
            .header("Content-Type", "application/json")
            .header("X-BiomeOS-Client", "Universal-Adapter")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120)) // Longer timeout for execution
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(result) => {
                            info!("✅ Toadstool execution completed successfully");
                            Ok(result)
                        }
                        Err(e) => {
                            let error =
                                format!("Failed to parse Toadstool execution response: {}", e);
                            warn!("{}", error);
                            Err(anyhow::anyhow!(error))
                        }
                    }
                } else {
                    let error = format!("Toadstool execution failed: {}", response.status());
                    warn!("{}", error);
                    Err(anyhow::anyhow!(error))
                }
            }
            Err(e) => {
                let error = format!("Failed to connect to Toadstool for execution: {}", e);
                warn!("{}", error);
                Err(anyhow::anyhow!(error))
            }
        }
    }

    pub async fn discover_primals(&self) -> Result<serde_json::Value> {
        info!("🔍 Discovering primals via Toadstool registry");

        let discovery_url = format!("{}/api/v1/discovery/primals", self.endpoint);

        match self
            .http_client
            .get(&discovery_url)
            .header("User-Agent", "BiomeOS-Universal-Adapter")
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(primals_data) => {
                            let primal_count = primals_data
                                .get("primals")
                                .and_then(|p| p.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0);
                            info!("✅ Discovered {} primals via Toadstool", primal_count);
                            Ok(primals_data)
                        }
                        Err(e) => {
                            warn!("Failed to parse Toadstool discovery response: {}", e);
                            Ok(serde_json::json!({ "primals": [], "error": "parse_error" }))
                        }
                    }
                } else {
                    warn!("Toadstool discovery returned error: {}", response.status());
                    Ok(serde_json::json!({ "primals": [], "error": "service_error" }))
                }
            }
            Err(e) => {
                warn!("Failed to connect to Toadstool for discovery: {}", e);
                Ok(serde_json::json!({ "primals": [], "error": "connection_error" }))
            }
        }
    }

    pub async fn get_status(&self) -> Result<ServiceStatus> {
        debug!("🔍 Checking Toadstool health status");

        let health_url = format!("{}/api/v1/health", self.endpoint);

        match self
            .http_client
            .get(&health_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                let status = match response.status().as_u16() {
                    200 => ServiceStatus::Healthy,
                    503 => ServiceStatus::Degraded,
                    _ => ServiceStatus::Degraded,
                };
                debug!("Toadstool status: {:?}", status);
                Ok(status)
            }
            Err(_) => {
                debug!("Toadstool is unavailable");
                Ok(ServiceStatus::Unavailable)
            }
        }
    }
}

impl SongbirdClient {
    pub async fn new(endpoint: &str) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to create Songbird HTTP client: {}", e);
                reqwest::Client::new()
            });

        Self {
            endpoint: endpoint.to_string(),
            http_client,
        }
    }

    pub async fn register_execution(&self, _result: &serde_json::Value) -> Result<()> {
        info!("🎼 Registering execution results with Songbird");

        let registration_url = format!("{}/api/v1/orchestration/register", self.endpoint);

        let registration_payload = serde_json::json!({
            "execution_id": uuid::Uuid::new_v4().to_string(),
            "result": _result,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "biomeOS-universal-adapter",
            "status": "completed"
        });

        match self
            .http_client
            .post(&registration_url)
            .header("Content-Type", "application/json")
            .header("X-BiomeOS-Orchestration", "true")
            .json(&registration_payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Successfully registered execution with Songbird");
                    Ok(())
                } else {
                    let error = format!("Songbird registration failed: {}", response.status());
                    warn!("{}", error);
                    Err(anyhow::anyhow!(error))
                }
            }
            Err(e) => {
                let error = format!("Failed to connect to Songbird for registration: {}", e);
                warn!("{}", error);
                Err(anyhow::anyhow!(error))
            }
        }
    }

    pub async fn discover_and_route(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        info!("🎼 Discovering services and routing via Songbird");

        // First, discover available services
        let discovery_url = format!("{}/api/v1/discovery/services", self.endpoint);

        let available_services = match self
            .http_client
            .get(&discovery_url)
            .header("User-Agent", "BiomeOS-Universal-Adapter")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(services) => services,
                        Err(e) => {
                            warn!("Failed to parse Songbird discovery response: {}", e);
                            serde_json::json!({ "services": [] })
                        }
                    }
                } else {
                    warn!("Songbird discovery failed: {}", response.status());
                    serde_json::json!({ "services": [] })
                }
            }
            Err(e) => {
                warn!("Failed to connect to Songbird for discovery: {}", e);
                serde_json::json!({ "services": [] })
            }
        };

        // Then, find the best service to route to
        let routing_url = format!("{}/api/v1/orchestration/route", self.endpoint);

        let routing_request = serde_json::json!({
            "payload": payload,
            "available_services": available_services.get("services").unwrap_or(&serde_json::json!([])),
            "routing_strategy": "optimal_load",
            "timeout_ms": 30000
        });

        match self
            .http_client
            .post(&routing_url)
            .header("Content-Type", "application/json")
            .header("X-BiomeOS-Orchestration", "true")
            .json(&routing_request)
            .timeout(std::time::Duration::from_secs(35))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(result) => {
                            info!("✅ Songbird routing completed successfully");
                            Ok(result)
                        }
                        Err(e) => {
                            let error = format!("Failed to parse Songbird routing response: {}", e);
                            warn!("{}", error);
                            Err(anyhow::anyhow!(error))
                        }
                    }
                } else {
                    let error = format!("Songbird routing failed: {}", response.status());
                    warn!("{}", error);
                    Err(anyhow::anyhow!(error))
                }
            }
            Err(e) => {
                let error = format!("Failed to connect to Songbird for routing: {}", e);
                warn!("{}", error);
                Err(anyhow::anyhow!(error))
            }
        }
    }

    pub async fn get_status(&self) -> Result<ServiceStatus> {
        debug!("🔍 Checking Songbird orchestration status");

        let health_url = format!("{}/api/v1/health", self.endpoint);

        match self
            .http_client
            .get(&health_url)
            .header("X-BiomeOS-Health-Check", "true")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                let status = match response.status().as_u16() {
                    200 => {
                        // Try to get more detailed status
                        if let Ok(health_data) = response.json::<serde_json::Value>().await {
                            if let Some(services_healthy) = health_data
                                .get("services_healthy")
                                .and_then(|s| s.as_bool())
                            {
                                if services_healthy {
                                    ServiceStatus::Healthy
                                } else {
                                    ServiceStatus::Degraded
                                }
                            } else {
                                ServiceStatus::Healthy
                            }
                        } else {
                            ServiceStatus::Healthy
                        }
                    }
                    503 => ServiceStatus::Degraded,
                    _ => ServiceStatus::Degraded,
                };
                debug!("Songbird status: {:?}", status);
                Ok(status)
            }
            Err(_) => {
                debug!("Songbird is unavailable");
                Ok(ServiceStatus::Unavailable)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub toadstool: ServiceStatus,
    pub songbird: ServiceStatus,
    pub adapter_uptime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unavailable,
    NotInitialized,
}

/// Helper function for coordinated execution
pub async fn coordinate_execution(
    toadstool_endpoint: &str,
    songbird_endpoint: &str,
    request: UniversalRequest,
) -> BiomeResult<UniversalResponse> {
    let config = AdapterConfig {
        toadstool_endpoint: toadstool_endpoint.to_string(),
        songbird_endpoint: songbird_endpoint.to_string(),
        ..Default::default()
    };

    let mut adapter = UniversalAdapter::new(config);
    adapter.initialize().await?;
    adapter.process_request(request).await
}

/// Helper function to get discovered primals
pub async fn get_discovered_primals() -> BiomeResult<Vec<PrimalInfo>> {
    // Use BiomeOS core manager for actual discovery
    use biomeos_core::{BiomeOSConfig, UniversalBiomeOSManager};

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Try actual network discovery
    match manager.discover_network_scan().await {
        Ok(discovered) => {
            if !discovered.is_empty() {
                return Ok(discovered
                    .into_iter()
                    .map(|result| PrimalInfo {
                        name: result.id.clone(),
                        primal_type: "discovered".to_string(), // TODO: Extract from capabilities
                        capabilities: result
                            .capabilities
                            .iter()
                            .map(|cap| cap.name.clone())
                            .collect(),
                        endpoint: result.endpoint.clone(),
                        status: ServiceStatus::Healthy,
                    })
                    .collect());
            }
        }
        Err(e) => {
            tracing::warn!("Discovery failed, falling back to examples: {}", e);
        }
    }
    let primals = vec![
        PrimalInfo {
            name: "example-compute".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["cpu".to_string(), "memory".to_string()],
            endpoint: std::env::var("EXAMPLE_COMPUTE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8001".to_string())
                .to_string(),
            status: ServiceStatus::Healthy,
        },
        PrimalInfo {
            name: "example-storage".to_string(),
            primal_type: "storage".to_string(),
            capabilities: vec!["persistent".to_string(), "distributed".to_string()],
            endpoint: std::env::var("EXAMPLE_STORAGE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8002".to_string())
                .to_string(),
            status: ServiceStatus::Healthy,
        },
    ];

    Ok(primals)
}

/// Find primal by capability
pub async fn find_primal_by_capability(capability: &str) -> BiomeResult<Option<PrimalInfo>> {
    let primals = get_discovered_primals()
        .await
        .map_err(|e| BiomeError::Discovery {
            message: format!("Failed to get discovered primals: {}", e),
        })?;

    for primal in primals {
        if primal.capabilities.contains(&capability.to_string()) {
            return Ok(Some(primal));
        }
    }

    Err(BiomeError::Configuration {
        message: "No suitable primal found".to_string(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub name: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub status: ServiceStatus,
}
