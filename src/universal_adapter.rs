//! Universal Adapter
//!
//! Provides universal coordination between Toadstool and Songbird primals

use anyhow::Result;
use biomeos_types::{BiomeError, BiomeResult};
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
                .unwrap_or_else(|_| "http://localhost:8003".to_string())
                .to_string(),
            songbird_endpoint: std::env::var("SONGBIRD_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8004".to_string())
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
                .map_err(|e| {
                    BiomeError::config_error(
                        format!("Failed to create HTTP client: {}", e),
                        Some("toadstool_client"),
                    )
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
        let client = self.toadstool_client.as_ref().ok_or_else(|| {
            BiomeError::config_error("Toadstool client not initialized", Some("toadstool_client"))
        })?;

        client.parse_and_validate(payload).await.map_err(|e| {
            BiomeError::config_error(
                format!("Toadstool parsing failed: {}", e),
                Some("toadstool_parsing"),
            )
        })?;

        client.discover_primals().await.map_err(|e| {
            BiomeError::discovery_failed(
                format!("Failed discovery attempt: {}", e),
                Some("primal_discovery"),
            )
        })?;

        Ok(serde_json::json!({"status": "discovery_complete"}))
    }

    /// Execute with coordination
    async fn execute_with_coordination(
        &self,
        payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let toadstool = self.toadstool_client.as_ref().ok_or_else(|| {
            BiomeError::config_error("Toadstool client not initialized", Some("toadstool_client"))
        })?;

        let songbird = self.songbird_client.as_ref().ok_or_else(|| {
            BiomeError::config_error("Songbird client not initialized", Some("songbird_client"))
        })?;

        // Execute via Toadstool
        let execution_result = toadstool.execute(payload).await.map_err(|e| {
            BiomeError::internal_error(
                format!("Toadstool execution failed: {}", e),
                Some("toadstool_execution"),
            )
        })?;

        // Register with Songbird
        songbird
            .register_execution(&execution_result)
            .await
            .map_err(|e| {
                BiomeError::integration_failed(
                    format!("Songbird registration failed: {}", e),
                    Some("songbird_registration"),
                )
            })?;

        Ok(execution_result)
    }

    /// Discover and route
    async fn discover_and_route(
        &self,
        payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let songbird = self.songbird_client.as_ref().ok_or_else(|| {
            BiomeError::config_error("Songbird client not initialized", Some("songbird_client"))
        })?;

        songbird.discover_and_route(payload).await.map_err(|e| {
            BiomeError::discovery_failed(
                format!("Failed to discover primals: {}", e),
                Some("primal_discovery"),
            )
        })?;

        Ok(serde_json::json!({"status": "execution_complete"}))
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
            .map_err(|e| {
                BiomeError::config_error(
                    format!("Failed to create Toadstool client: {}", e),
                    Some("toadstool_client_creation"),
                )
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
    use biomeos_core::UniversalBiomeOSManager;
    use biomeos_types::BiomeOSConfig;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await.map_err(|e| {
        BiomeError::internal_error(format!("Failed to create manager: {}", e), None::<&str>)
    })?;

    // Try actual network discovery
    match manager.discover_network_scan().await {
        Ok(discovered_endpoints) => {
            if !discovered_endpoints.is_empty() {
                return Ok(discovered_endpoints
                    .into_iter()
                    .enumerate()
                    .map(|(i, endpoint)| PrimalInfo {
                        name: format!("discovered-primal-{}", i),
                        primal_type: "discovered".to_string(),
                        capabilities: vec!["network".to_string()],
                        endpoint,
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
    let primals = get_discovered_primals().await.map_err(|e| {
        BiomeError::discovery_failed(
            format!("Failed to get discovered primals: {}", e),
            None::<&str>,
        )
    })?;

    for primal in primals {
        if primal.capabilities.contains(&capability.to_string()) {
            return Ok(Some(primal));
        }
    }

    Err(BiomeError::config_error(
        "No suitable primal found",
        Some("primal_search"),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub name: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub status: ServiceStatus,
}

#[allow(dead_code)]
fn extract_primal_type_from_capabilities(capabilities: &[String]) -> Option<String> {
    // Check for specific capability patterns to determine primal type
    if capabilities
        .iter()
        .any(|cap| cap.contains("compute") || cap.contains("cpu"))
    {
        Some("compute".to_string())
    } else if capabilities
        .iter()
        .any(|cap| cap.contains("storage") || cap.contains("disk"))
    {
        Some("storage".to_string())
    } else if capabilities
        .iter()
        .any(|cap| cap.contains("network") || cap.contains("routing"))
    {
        Some("network".to_string())
    } else if capabilities
        .iter()
        .any(|cap| cap.contains("ai") || cap.contains("ml"))
    {
        Some("ai".to_string())
    } else if capabilities
        .iter()
        .any(|cap| cap.contains("data") || cap.contains("analytics"))
    {
        Some("data".to_string())
    } else if !capabilities.is_empty() {
        // Use the first capability as a fallback
        Some(capabilities[0].clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_primal_type_compute() {
        let capabilities = vec!["cpu".to_string(), "memory".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("compute".to_string())
        );

        let capabilities = vec!["compute_node".to_string(), "gpu".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("compute".to_string())
        );
    }

    #[test]
    fn test_extract_primal_type_storage() {
        let capabilities = vec!["storage".to_string(), "file_system".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("storage".to_string())
        );

        let capabilities = vec!["disk_io".to_string(), "backup".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("storage".to_string())
        );
    }

    #[test]
    fn test_extract_primal_type_network() {
        let capabilities = vec!["network".to_string(), "load_balancer".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("network".to_string())
        );

        let capabilities = vec!["routing".to_string(), "firewall".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("network".to_string())
        );
    }

    #[test]
    fn test_extract_primal_type_ai() {
        let capabilities = vec!["ai".to_string(), "inference".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("ai".to_string())
        );

        let capabilities = vec!["ml_training".to_string(), "model_serving".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("ai".to_string())
        );
    }

    #[test]
    fn test_extract_primal_type_data() {
        let capabilities = vec!["data_processing".to_string(), "pipeline".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("data".to_string())
        );

        let capabilities = vec!["analytics".to_string(), "reporting".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("data".to_string())
        );
    }

    #[test]
    fn test_extract_primal_type_fallback() {
        let capabilities = vec!["custom_capability".to_string()];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("custom_capability".to_string())
        );
    }

    #[test]
    fn test_extract_primal_type_empty() {
        let capabilities = vec![];
        assert_eq!(extract_primal_type_from_capabilities(&capabilities), None);
    }

    #[test]
    fn test_extract_primal_type_priority() {
        // Should prefer compute over other types when multiple patterns match
        let capabilities = vec![
            "storage".to_string(),
            "cpu".to_string(),
            "network".to_string(),
        ];
        assert_eq!(
            extract_primal_type_from_capabilities(&capabilities),
            Some("compute".to_string())
        );
    }
}
