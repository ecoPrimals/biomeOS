//! Universal Adapter Implementations
//!
//! This module provides concrete implementations of universal adapters that can
//! communicate with any primal regardless of its specific type or implementation.

use super::{
    AuthMethod, PrimalEvent, UniversalCommConfig, UniversalPrimalAdapter, UniversalRequest,
    UniversalResponse,
};
use crate::health::HealthMetrics;
#[cfg(test)]
use crate::primal_clients::{CapabilityCategory, CapabilityResponse, PrimalHealth};
use crate::{BiomeError, BiomeResult, HealthStatus, primal_clients::CapabilityResponse, PrimalHealth};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// Universal HTTP adapter - works with any primal exposing HTTP endpoints
pub struct HttpUniversalAdapter {
    client: Client,
    config: UniversalCommConfig,
    capabilities_cache: Option<Vec<primal_clients::CapabilityResponse>>,
}

impl HttpUniversalAdapter {
    pub fn new(config: UniversalCommConfig) -> BiomeResult<Self> {
        let mut client_builder = Client::builder().timeout(config.timeout);

        // Configure TLS
        if let Some(tls_config) = &config.tls {
            client_builder = client_builder.danger_accept_invalid_certs(!tls_config.verify);
        }

        // Add authentication headers
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(auth_config) = &config.auth {
            match &auth_config.method {
                AuthMethod::Bearer => {
                    if let Some(token) = auth_config.credentials.get("token") {
                        headers.insert(
                            reqwest::header::AUTHORIZATION,
                            format!("Bearer {}", token).parse().map_err(|e| {
                                BiomeError::ConfigError(format!("Invalid auth token: {}", e))
                            })?,
                        );
                    }
                }
                AuthMethod::ApiKey => {
                    if let Some(key) = auth_config.credentials.get("api_key") {
                        headers.insert(
                            "X-API-Key",
                            key.parse().map_err(|e| {
                                BiomeError::ConfigError(format!("Invalid API key: {}", e))
                            })?,
                        );
                    }
                }
                AuthMethod::Jwt => {
                    if let Some(token) = auth_config.credentials.get("jwt") {
                        headers.insert(
                            reqwest::header::AUTHORIZATION,
                            format!("Bearer {}", token).parse().map_err(|e| {
                                BiomeError::ConfigError(format!("Invalid JWT: {}", e))
                            })?,
                        );
                    }
                }
                _ => {
                    // Other auth methods handled elsewhere
                }
            }
        }

        // Add custom metadata as headers
        for (key, value) in &config.metadata {
            if let (Ok(header_name), Ok(header_value)) = (
                key.parse::<reqwest::header::HeaderName>(),
                value.parse::<reqwest::header::HeaderValue>(),
            ) {
                headers.insert(header_name, header_value);
            }
        }

        client_builder = client_builder.default_headers(headers);

        let client = client_builder.build().map_err(|e| {
            BiomeError::NetworkError(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self {
            client,
            config,
            capabilities_cache: None,
        })
    }

    /// Execute HTTP request with retry logic
    async fn execute_with_retry<T>(
        &self,
        request_func: impl Fn() -> reqwest::RequestBuilder,
        operation: &str,
    ) -> BiomeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut delay = self.config.retry.initial_delay;
        let mut last_error = None;

        for attempt in 0..=self.config.retry.max_retries {
            match self.execute_request(&request_func(), operation).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.config.retry.max_retries {
                        // Add jitter if enabled
                        let actual_delay = if self.config.retry.jitter {
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};

                            let mut hasher = DefaultHasher::new();
                            std::thread::current().id().hash(&mut hasher);
                            let hash = hasher.finish();
                            let jitter = ((hash % 21) as f64 / 100.0) - 0.1; // ±10% jitter
                            Duration::from_millis(
                                ((delay.as_millis() as f64) * (1.0 + jitter)) as u64,
                            )
                        } else {
                            delay
                        };

                        sleep(actual_delay).await;
                        delay = std::cmp::min(
                            Duration::from_millis(
                                (delay.as_millis() as f64 * self.config.retry.backoff_multiplier)
                                    as u64,
                            ),
                            self.config.retry.max_delay,
                        );
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| BiomeError::NetworkError("Unknown error during retry".to_string())))
    }

    /// Execute single HTTP request
    async fn execute_request<T>(
        &self,
        request_builder: &reqwest::RequestBuilder,
        operation: &str,
    ) -> BiomeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let _start_time = std::time::Instant::now();

        let response = request_builder
            .try_clone()
            .ok_or_else(|| BiomeError::NetworkError("Failed to clone request".to_string()))?
            .send()
            .await
            .map_err(|e| {
                BiomeError::NetworkError(format!("HTTP request failed for {}: {}", operation, e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BiomeError::NetworkError(format!(
                "HTTP {} failed with status {}: {}",
                operation, status, error_text
            )));
        }

        response.json().await.map_err(|e| {
            BiomeError::NetworkError(format!("Failed to parse response for {}: {}", operation, e))
        })
    }

    /// Standard primal endpoint URLs
    fn health_url(&self) -> String {
        format!("{}/api/v1/health", self.config.endpoint)
    }

    fn capabilities_url(&self) -> String {
        format!("{}/api/v1/capabilities", self.config.endpoint)
    }

    fn status_url(&self) -> String {
        format!("{}/api/v1/status", self.config.endpoint)
    }

    fn operation_url(&self) -> String {
        format!("{}/api/v1/operation", self.config.endpoint)
    }
}

#[async_trait]
impl UniversalPrimalAdapter for HttpUniversalAdapter {
    async fn initialize(&mut self, config: UniversalCommConfig) -> BiomeResult<()> {
        self.config = config;
        // Test connection
        let _health = self.health_check().await?;
        Ok(())
    }

    async fn discover_capabilities(&self) -> BiomeResult<Vec<primal_clients::CapabilityResponse>> {
        if let Some(cached) = &self.capabilities_cache {
            return Ok(cached.clone());
        }

        let url = self.capabilities_url();
        let response: serde_json::Value = self
            .execute_with_retry(|| self.client.get(&url), "discover_capabilities")
            .await?;

        let capabilities = response["capabilities"]
            .as_array()
            .ok_or_else(|| BiomeError::InvalidResponse("No capabilities found".to_string()))?;

        let mut primal_capabilities = Vec::new();
        for cap in capabilities {
            if let Some(cap_obj) = cap.as_object() {
                let capability = primal_clients::CapabilityResponse {
                    name: cap_obj["name"].as_str().unwrap_or("unknown").to_string(),
                    version: cap_obj["version"].as_str().unwrap_or("1.0.0").to_string(),
                    description: cap_obj["description"].as_str().unwrap_or("").to_string(),
                    parameters: HashMap::new(),
                };
                primal_capabilities.push(capability);
            }
        }

        Ok(primal_capabilities)
    }

    async fn health_check(&self) -> BiomeResult<PrimalHealth> {
        let url = self.health_url();
        let response: serde_json::Value = self
            .execute_with_retry(|| self.client.get(&url), "health_check")
            .await?;

        // Parse health response using standard format
        let status = match response["status"].as_str() {
            Some("healthy") => crate::primal_clients::HealthStatus::Healthy,
            Some("degraded") => crate::primal_clients::HealthStatus::Degraded,
            Some("unhealthy") => crate::primal_clients::HealthStatus::Unhealthy,
            _ => crate::primal_clients::HealthStatus::Unknown,
        };

        let health_score = response["health_score"].as_f64().unwrap_or(0.0);
        let metrics = &response["metrics"];

        Ok(PrimalHealth {
            status,
            details: HashMap::new(),
            last_check: chrono::Utc::now(),
        })
    }

    async fn execute_operation(&self, request: UniversalRequest) -> BiomeResult<UniversalResponse> {
        let url = self.operation_url();
        let start_time = std::time::Instant::now();

        let response: serde_json::Value = self
            .execute_with_retry(
                || self.client.post(&url).json(&request),
                "execute_operation",
            )
            .await?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(UniversalResponse {
            request_id: request.id,
            success: response["success"].as_bool().unwrap_or(false),
            payload: response["payload"].clone(),
            error: response["error"].as_str().map(|s| s.to_string()),
            metadata: response["metadata"]
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            processing_time_ms: processing_time,
        })
    }

    async fn subscribe_events(&self, _event_types: Vec<String>) -> BiomeResult<()> {
        if !self.config.bidirectional.enabled {
            return Err(BiomeError::NotImplemented(
                "Bidirectional communication not enabled".to_string(),
            ));
        }

        // For HTTP, we might use Server-Sent Events or WebSocket upgrade
        // This is a placeholder implementation
        Err(BiomeError::NotImplemented(
            "HTTP event subscription not yet implemented".to_string(),
        ))
    }

    async fn send_event(&self, event: PrimalEvent) -> BiomeResult<()> {
        let url = format!("{}/api/v1/events", self.config.endpoint);

        let _response: serde_json::Value = self
            .execute_with_retry(|| self.client.post(&url).json(&event), "send_event")
            .await?;

        Ok(())
    }

    async fn get_status(&self) -> BiomeResult<serde_json::Value> {
        let url = self.status_url();

        self.execute_with_retry(|| self.client.get(&url), "get_status")
            .await
    }

    async fn close(&mut self) -> BiomeResult<()> {
        // HTTP clients don't need explicit closing
        Ok(())
    }
}

/// Universal WebSocket adapter - works with any primal exposing WebSocket endpoints
pub struct WebSocketUniversalAdapter {
    config: UniversalCommConfig,
}

impl WebSocketUniversalAdapter {
    pub fn new(config: UniversalCommConfig) -> BiomeResult<Self> {
        Ok(Self { config })
    }
}

#[async_trait]
impl UniversalPrimalAdapter for WebSocketUniversalAdapter {
    async fn initialize(&mut self, config: UniversalCommConfig) -> BiomeResult<()> {
        self.config = config;
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }

    async fn discover_capabilities(&self) -> BiomeResult<Vec<primal_clients::CapabilityResponse>> {
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }

    async fn health_check(&self) -> BiomeResult<PrimalHealth> {
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }

    async fn execute_operation(
        &self,
        _request: UniversalRequest,
    ) -> BiomeResult<UniversalResponse> {
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }

    async fn subscribe_events(&self, _event_types: Vec<String>) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }

    async fn send_event(&self, _event: PrimalEvent) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }

    async fn get_status(&self) -> BiomeResult<serde_json::Value> {
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }

    async fn close(&mut self) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "WebSocket adapter not yet implemented".to_string(),
        ))
    }
}

/// Universal gRPC adapter - works with any primal exposing gRPC endpoints
pub struct GrpcUniversalAdapter {
    config: UniversalCommConfig,
}

impl GrpcUniversalAdapter {
    pub fn new(config: UniversalCommConfig) -> BiomeResult<Self> {
        Ok(Self { config })
    }
}

#[async_trait]
impl UniversalPrimalAdapter for GrpcUniversalAdapter {
    async fn initialize(&mut self, config: UniversalCommConfig) -> BiomeResult<()> {
        self.config = config;
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }

    async fn discover_capabilities(&self) -> BiomeResult<Vec<primal_clients::CapabilityResponse>> {
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }

    async fn health_check(&self) -> BiomeResult<PrimalHealth> {
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }

    async fn execute_operation(
        &self,
        _request: UniversalRequest,
    ) -> BiomeResult<UniversalResponse> {
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }

    async fn subscribe_events(&self, _event_types: Vec<String>) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }

    async fn send_event(&self, _event: PrimalEvent) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }

    async fn get_status(&self) -> BiomeResult<serde_json::Value> {
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }

    async fn close(&mut self) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "gRPC adapter not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_universal_adapter_creation() {
        let config = crate::adapters::UniversalCommConfig {
            endpoint: "http://localhost:8080".to_string(),
            protocol: crate::CommunicationProtocol::Http,
            timeout: Duration::from_secs(30),
            auth: None,
            metadata: HashMap::new(),
            tls: None,
            retry: crate::adapters::RetryConfig::default(),
            bidirectional: crate::adapters::BidirectionalConfig::default(),
        };

        let adapter = HttpUniversalAdapter::new(config);
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_health_check_with_mock_context() {
        let _config = crate::adapters::UniversalCommConfig {
            endpoint: "http://localhost:8080".to_string(),
            protocol: crate::CommunicationProtocol::Http,
            timeout: Duration::from_secs(30),
            auth: None,
            metadata: HashMap::new(),
            tls: None,
            retry: crate::adapters::RetryConfig::default(),
            bidirectional: crate::adapters::BidirectionalConfig::default(),
        };

        let _context = PrimalContext {
            session_id: "test-session".to_string(),
            user_id: "test-user".to_string(),
            device_id: "test-device".to_string(),
            network_location: NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: crate::SecurityLevel::Standard,
            biome_id: Some("test-biome".to_string()),
            team_id: Some("test-team".to_string()),
            metadata: HashMap::new(),
        };

        // Mock adapter would return a health response
        let health_response = crate::primal_clients::PrimalHealth {
            status: crate::primal_clients::HealthStatus::Healthy,
            details: HashMap::new(),
            last_check: chrono::Utc::now(),
        };
        
    }
}
