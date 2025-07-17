//! Universal Primal Communication Adapters
//!
//! This module provides universal, agnostic communication adapters that can work
//! with ANY primal (current, future, or community-created) without requiring
//! code changes. Supports both HTTP and tRPC bidirectional communication.

pub mod protocol;
pub mod trpc;
pub mod universal;

use crate::{BiomeError, BiomeResult, PrimalCapability, PrimalContext, PrimalHealth};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Universal communication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalCommConfig {
    /// Base URL or connection string
    pub endpoint: String,
    /// Communication protocol (http, https, trpc, ws, grpc, etc.)
    pub protocol: CommunicationProtocol,
    /// Request timeout
    pub timeout: Duration,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Custom headers/metadata
    pub metadata: HashMap<String, String>,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Bidirectional communication settings
    pub bidirectional: BidirectionalConfig,
}

/// Communication protocols supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationProtocol {
    /// HTTP/HTTPS REST
    Http,
    /// tRPC bidirectional
    Trpc,
    /// WebSocket
    WebSocket,
    /// gRPC
    Grpc,
    /// Custom protocol
    Custom(String),
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication method
    pub method: AuthMethod,
    /// Credentials
    pub credentials: HashMap<String, String>,
    /// Token refresh settings
    pub refresh: Option<TokenRefreshConfig>,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Bearer token
    Bearer,
    /// API key
    ApiKey,
    /// JWT
    Jwt,
    /// mTLS
    Mtls,
    /// Custom auth
    Custom(String),
}

/// Token refresh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshConfig {
    /// Refresh endpoint
    pub endpoint: String,
    /// Refresh interval
    pub interval: Duration,
    /// Auto-refresh enabled
    pub auto_refresh: bool,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Verify certificates
    pub verify: bool,
    /// CA certificate path
    pub ca_cert: Option<String>,
    /// Client certificate path
    pub client_cert: Option<String>,
    /// Client key path
    pub client_key: Option<String>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retries
    pub max_retries: u32,
    /// Initial delay
    pub initial_delay: Duration,
    /// Maximum delay
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter enabled
    pub jitter: bool,
}

/// Bidirectional communication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidirectionalConfig {
    /// Enable bidirectional communication
    pub enabled: bool,
    /// Subscription endpoint
    pub subscription_endpoint: Option<String>,
    /// Event streaming enabled
    pub event_streaming: bool,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Reconnection settings
    pub reconnection: ReconnectionConfig,
}

/// Reconnection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectionConfig {
    /// Auto-reconnect enabled
    pub enabled: bool,
    /// Maximum reconnection attempts
    pub max_attempts: u32,
    /// Reconnection delay
    pub delay: Duration,
    /// Exponential backoff
    pub exponential_backoff: bool,
}

/// Universal request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    /// Request ID
    pub id: String,
    /// Target operation
    pub operation: String,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request context
    pub context: PrimalContext,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Request priority
    pub priority: RequestPriority,
}

/// Universal response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    /// Request ID
    pub request_id: String,
    /// Success status
    pub success: bool,
    /// Response payload
    pub payload: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Processing time
    pub processing_time_ms: u64,
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Universal primal adapter trait
#[async_trait]
pub trait UniversalPrimalAdapter: Send + Sync {
    /// Initialize connection to primal
    async fn initialize(&mut self, config: UniversalCommConfig) -> BiomeResult<()>;

    /// Discover primal capabilities dynamically
    async fn discover_capabilities(&self) -> BiomeResult<Vec<PrimalCapability>>;

    /// Check primal health
    async fn health_check(&self) -> BiomeResult<PrimalHealth>;

    /// Execute operation on primal
    async fn execute_operation(&self, request: UniversalRequest) -> BiomeResult<UniversalResponse>;

    /// Subscribe to primal events (for bidirectional communication)
    async fn subscribe_events(&self, event_types: Vec<String>) -> BiomeResult<()>;

    /// Send event to primal
    async fn send_event(&self, event: PrimalEvent) -> BiomeResult<()>;

    /// Get primal status
    async fn get_status(&self) -> BiomeResult<serde_json::Value>;

    /// Close connection
    async fn close(&mut self) -> BiomeResult<()>;
}

/// Primal event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Event payload
    pub payload: serde_json::Value,
    /// Event context
    pub context: PrimalContext,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Default configurations
impl Default for UniversalCommConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8080".to_string(),
            protocol: CommunicationProtocol::Http,
            timeout: Duration::from_secs(30),
            auth: None,
            metadata: HashMap::new(),
            tls: None,
            retry: RetryConfig::default(),
            bidirectional: BidirectionalConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl Default for BidirectionalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            subscription_endpoint: None,
            event_streaming: false,
            heartbeat_interval: Duration::from_secs(30),
            reconnection: ReconnectionConfig::default(),
        }
    }
}

impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 10,
            delay: Duration::from_secs(1),
            exponential_backoff: true,
        }
    }
}

/// Universal primal adapter factory
pub struct UniversalAdapterFactory;

impl UniversalAdapterFactory {
    /// Create adapter for any primal based on configuration
    pub fn create_adapter(
        config: &UniversalCommConfig,
    ) -> BiomeResult<Box<dyn UniversalPrimalAdapter>> {
        match &config.protocol {
            CommunicationProtocol::Http => Ok(Box::new(universal::HttpUniversalAdapter::new(
                config.clone(),
            )?)),
            CommunicationProtocol::Trpc => {
                Ok(Box::new(trpc::TrpcUniversalAdapter::new(config.clone())?))
            }
            CommunicationProtocol::WebSocket => Ok(Box::new(
                universal::WebSocketUniversalAdapter::new(config.clone())?,
            )),
            CommunicationProtocol::Grpc => Ok(Box::new(universal::GrpcUniversalAdapter::new(
                config.clone(),
            )?)),
            CommunicationProtocol::Custom(protocol) => {
                // Allow custom protocol implementations
                Err(BiomeError::NotImplemented(format!(
                    "Custom protocol not yet implemented: {}",
                    protocol
                )))
            }
        }
    }

    /// Auto-discover primal and create appropriate adapter
    pub async fn auto_discover_and_create(
        endpoint: &str,
    ) -> BiomeResult<Box<dyn UniversalPrimalAdapter>> {
        // Try to auto-discover the best protocol for the primal
        let discovered_protocol = Self::discover_protocol(endpoint).await?;

        let config = UniversalCommConfig {
            endpoint: endpoint.to_string(),
            protocol: discovered_protocol,
            ..Default::default()
        };

        Self::create_adapter(&config)
    }

    /// Discover what protocol a primal supports
    async fn discover_protocol(endpoint: &str) -> BiomeResult<CommunicationProtocol> {
        // Try tRPC first for bidirectional support
        if Self::probe_trpc(endpoint).await.is_ok() {
            return Ok(CommunicationProtocol::Trpc);
        }

        // Fall back to HTTP
        if Self::probe_http(endpoint).await.is_ok() {
            return Ok(CommunicationProtocol::Http);
        }

        // Try WebSocket
        if Self::probe_websocket(endpoint).await.is_ok() {
            return Ok(CommunicationProtocol::WebSocket);
        }

        Err(BiomeError::NetworkError(format!(
            "No supported protocol found for endpoint: {}",
            endpoint
        )))
    }

    async fn probe_trpc(_endpoint: &str) -> BiomeResult<()> {
        // Probe for tRPC endpoint
        // This would attempt to connect via tRPC protocol
        // For now, return not implemented
        Err(BiomeError::NotImplemented(
            "tRPC probing not yet implemented".to_string(),
        ))
    }

    async fn probe_http(endpoint: &str) -> BiomeResult<()> {
        // Simple HTTP probe
        let client = reqwest::Client::new();
        let health_url = format!("{}/health", endpoint);

        client
            .get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| BiomeError::NetworkError(format!("HTTP probe failed: {}", e)))?;

        Ok(())
    }

    async fn probe_websocket(_endpoint: &str) -> BiomeResult<()> {
        // WebSocket probe
        Err(BiomeError::NotImplemented(
            "WebSocket probing not yet implemented".to_string(),
        ))
    }
}
