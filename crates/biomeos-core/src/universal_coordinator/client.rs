//! Primal client implementations for communication
//!
//! This module contains the PrimalClient trait and HTTP client implementation
//! for communicating with Primal instances.

use std::time::Duration;
use async_trait::async_trait;

use crate::universal_primal::{
    CapabilityRequest, CapabilityResponse, CoordinationRequest, CoordinationResponse,
    PrimalMetadata,
};
use crate::{BiomeError, BiomeResult, HealthStatus};

/// Primal client for communication
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Send capability request
    async fn send_capability_request(
        &self,
        request: CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse>;

    /// Send coordination request
    async fn send_coordination_request(
        &self,
        request: CoordinationRequest,
    ) -> BiomeResult<CoordinationResponse>;

    /// Get primal health
    async fn get_health(&self) -> BiomeResult<HealthStatus>;

    /// Get primal metadata
    async fn get_metadata(&self) -> BiomeResult<PrimalMetadata>;
}

/// HTTP primal client implementation
pub struct HttpPrimalClient {
    /// HTTP client
    client: reqwest::Client,
    /// Base URL
    base_url: String,
    /// Timeout
    timeout: Duration,
}

impl HttpPrimalClient {
    /// Create new HTTP client
    pub fn new(base_url: String, timeout: Duration) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            timeout,
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Set a new timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

#[async_trait]
impl PrimalClient for HttpPrimalClient {
    async fn send_capability_request(
        &self,
        request: CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse> {
        let url = format!("{}/capabilities", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::Network(e.to_string()))?;

        if response.status().is_success() {
            let capability_response: CapabilityResponse = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(capability_response)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }

    async fn send_coordination_request(
        &self,
        request: CoordinationRequest,
    ) -> BiomeResult<CoordinationResponse> {
        let url = format!("{}/coordination", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::Network(e.to_string()))?;

        if response.status().is_success() {
            let coordination_response: CoordinationResponse = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(coordination_response)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }

    async fn get_health(&self) -> BiomeResult<HealthStatus> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::Network(e.to_string()))?;

        if response.status().is_success() {
            let health: HealthStatus = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(health)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }

    async fn get_metadata(&self) -> BiomeResult<PrimalMetadata> {
        let url = format!("{}/metadata", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::Network(e.to_string()))?;

        if response.status().is_success() {
            let metadata: PrimalMetadata = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(metadata)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }
} 