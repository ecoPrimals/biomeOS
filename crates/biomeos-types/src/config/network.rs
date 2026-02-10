//! Network Configuration
//!
//! Network configuration settings for BiomeOS components.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network interface to bind to
    pub bind_address: String,

    /// Primary port for the system
    pub port: u16,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// HTTP configuration
    pub http: HttpConfig,

    /// WebSocket configuration
    pub websocket: Option<WebSocketConfig>,

    /// Load balancing configuration
    pub load_balancing: Option<LoadBalancingConfig>,

    /// Rate limiting configuration
    pub rate_limiting: Option<RateLimitingConfig>,

    /// CORS configuration
    pub cors: Option<CorsConfig>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,

    /// Certificate file path
    pub cert_file: Option<PathBuf>,

    /// Private key file path
    pub key_file: Option<PathBuf>,

    /// CA certificate file path
    pub ca_file: Option<PathBuf>,

    /// Minimum TLS version
    pub min_version: TlsVersion,

    /// Cipher suites
    pub cipher_suites: Vec<String>,

    /// Client certificate verification
    pub verify_client: bool,
}

/// TLS version specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS version 1.2
    #[serde(rename = "1.2")]
    V1_2,
    /// TLS version 1.3
    #[serde(rename = "1.3")]
    V1_3,
}

/// HTTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Enable HTTP/2
    pub http2: bool,

    /// Enable HTTP/3 (QUIC)
    pub http3: bool,

    /// Enable compression
    pub compression: bool,

    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,

    /// Max header size
    pub max_header_size: usize,

    /// Request timeout
    pub request_timeout: Duration,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Enable WebSocket support
    pub enabled: bool,

    /// Maximum message size
    pub max_message_size: usize,

    /// Maximum frame size
    pub max_frame_size: usize,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Ping interval
    pub ping_interval: Duration,

    /// Pong timeout
    pub pong_timeout: Duration,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,

    /// Health check configuration
    pub health_check: LoadBalancingHealthCheck,

    /// Session affinity configuration
    pub session_affinity: Option<SessionAffinity>,
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Least active connections
    LeastConnections,
    /// Weighted least connections
    WeightedLeastConnections,
    /// IP-based hashing
    IpHash,
    /// Random selection
    Random,
    /// Weighted random selection
    WeightedRandom,
}

/// Load balancing health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingHealthCheck {
    /// Health check interval
    pub interval: Duration,

    /// Health check timeout
    pub timeout: Duration,

    /// Healthy threshold
    pub healthy_threshold: u32,

    /// Unhealthy threshold
    pub unhealthy_threshold: u32,

    /// Health check path
    pub path: String,
}

/// Session affinity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAffinity {
    /// Cookie name for session affinity
    pub cookie_name: String,

    /// Session timeout
    pub timeout: Duration,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Rate limiting algorithm
    pub algorithm: RateLimitingAlgorithm,

    /// Requests per second limit
    pub requests_per_second: u32,

    /// Burst size
    pub burst_size: u32,

    /// Rate limiting key
    pub key: RateLimitingKey,
}

/// Rate limiting algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitingAlgorithm {
    /// Token bucket algorithm
    TokenBucket,
    /// Leaky bucket algorithm
    LeakyBucket,
    /// Fixed window counter
    FixedWindow,
    /// Sliding window counter
    SlidingWindow,
}

/// Rate limiting key types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitingKey {
    /// Rate limit by client IP
    ClientIp,
    /// Rate limit by API key
    ApiKey,
    /// Rate limit by user ID
    UserId,
    /// Custom rate limit key
    Custom(String),
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,

    /// Allowed methods
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    pub allowed_headers: Vec<String>,

    /// Exposed headers
    pub exposed_headers: Vec<String>,

    /// Allow credentials
    pub allow_credentials: bool,

    /// Max age for preflight cache
    pub max_age: Duration,
}

/// Default implementations
impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            tls: None,
            http: HttpConfig::default(),
            websocket: None,
            load_balancing: None,
            rate_limiting: None,
            cors: None,
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            http2: true,
            http3: false,
            compression: true,
            keep_alive_timeout: Duration::from_secs(60),
            max_header_size: 8192,
            request_timeout: Duration::from_secs(60),
        }
    }
}
