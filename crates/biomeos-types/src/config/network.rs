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

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // Default Implementations
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.tls.is_none());
        assert!(config.websocket.is_none());
        assert!(config.load_balancing.is_none());
        assert!(config.rate_limiting.is_none());
        assert!(config.cors.is_none());
    }

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert!(config.http2);
        assert!(!config.http3);
        assert!(config.compression);
        assert_eq!(config.keep_alive_timeout, Duration::from_secs(60));
        assert_eq!(config.max_header_size, 8192);
        assert_eq!(config.request_timeout, Duration::from_secs(60));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TLS Configuration
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_tls_config_creation() {
        let config = TlsConfig {
            enabled: true,
            cert_file: Some(PathBuf::from("/etc/ssl/cert.pem")),
            key_file: Some(PathBuf::from("/etc/ssl/key.pem")),
            ca_file: Some(PathBuf::from("/etc/ssl/ca.pem")),
            min_version: TlsVersion::V1_3,
            cipher_suites: vec!["TLS_AES_256_GCM_SHA384".to_string()],
            verify_client: true,
        };
        assert!(config.enabled);
        assert!(config.verify_client);
    }

    #[test]
    fn test_tls_version_serialization() {
        for version in [TlsVersion::V1_2, TlsVersion::V1_3] {
            let json = serde_json::to_string(&version).expect("serialize");
            let _: TlsVersion = serde_json::from_str(&json).expect("deserialize");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WebSocket Configuration
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_websocket_config_creation() {
        let config = WebSocketConfig {
            enabled: true,
            max_message_size: 16 * 1024 * 1024,
            max_frame_size: 64 * 1024,
            connection_timeout: Duration::from_secs(30),
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
        };
        assert!(config.enabled);
        assert_eq!(config.max_message_size, 16 * 1024 * 1024);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Load Balancing Configuration
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_load_balancing_algorithm_serialization() {
        for alg in [
            LoadBalancingAlgorithm::RoundRobin,
            LoadBalancingAlgorithm::WeightedRoundRobin,
            LoadBalancingAlgorithm::LeastConnections,
            LoadBalancingAlgorithm::WeightedLeastConnections,
            LoadBalancingAlgorithm::IpHash,
            LoadBalancingAlgorithm::Random,
            LoadBalancingAlgorithm::WeightedRandom,
        ] {
            let json = serde_json::to_string(&alg).expect("serialize");
            let _: LoadBalancingAlgorithm = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_load_balancing_config_creation() {
        let config = LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::LeastConnections,
            health_check: LoadBalancingHealthCheck {
                interval: Duration::from_secs(10),
                timeout: Duration::from_secs(5),
                healthy_threshold: 3,
                unhealthy_threshold: 2,
                path: "/health".to_string(),
            },
            session_affinity: Some(SessionAffinity {
                cookie_name: "BIOMEOS_SESSION".to_string(),
                timeout: Duration::from_secs(3600),
            }),
        };
        assert_eq!(config.health_check.path, "/health");
        assert!(config.session_affinity.is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Rate Limiting Configuration
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_rate_limiting_algorithm_serialization() {
        for alg in [
            RateLimitingAlgorithm::TokenBucket,
            RateLimitingAlgorithm::LeakyBucket,
            RateLimitingAlgorithm::FixedWindow,
            RateLimitingAlgorithm::SlidingWindow,
        ] {
            let json = serde_json::to_string(&alg).expect("serialize");
            let _: RateLimitingAlgorithm = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_rate_limiting_key_serialization() {
        for key in [
            RateLimitingKey::ClientIp,
            RateLimitingKey::ApiKey,
            RateLimitingKey::UserId,
            RateLimitingKey::Custom("x-tenant-id".to_string()),
        ] {
            let json = serde_json::to_string(&key).expect("serialize");
            let _: RateLimitingKey = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_rate_limiting_config_creation() {
        let config = RateLimitingConfig {
            algorithm: RateLimitingAlgorithm::TokenBucket,
            requests_per_second: 100,
            burst_size: 200,
            key: RateLimitingKey::ClientIp,
        };
        assert_eq!(config.requests_per_second, 100);
        assert_eq!(config.burst_size, 200);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CORS Configuration
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_cors_config_creation() {
        let config = CorsConfig {
            allowed_origins: vec!["https://app.example.com".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            exposed_headers: vec!["X-Request-Id".to_string()],
            allow_credentials: true,
            max_age: Duration::from_secs(3600),
        };
        assert!(config.allow_credentials);
        assert_eq!(config.allowed_origins.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Serialization Roundtrip
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_network_config_serialization() {
        let config = NetworkConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.bind_address, "0.0.0.0");
        assert_eq!(deserialized.port, 8080);
    }

    #[test]
    fn test_full_network_config_serialization() {
        let config = NetworkConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 3000,
            tls: Some(TlsConfig {
                enabled: true,
                cert_file: Some(PathBuf::from("/cert.pem")),
                key_file: Some(PathBuf::from("/key.pem")),
                ca_file: None,
                min_version: TlsVersion::V1_3,
                cipher_suites: vec![],
                verify_client: false,
            }),
            http: HttpConfig::default(),
            websocket: Some(WebSocketConfig {
                enabled: true,
                max_message_size: 1024 * 1024,
                max_frame_size: 64 * 1024,
                connection_timeout: Duration::from_secs(30),
                ping_interval: Duration::from_secs(30),
                pong_timeout: Duration::from_secs(10),
            }),
            load_balancing: None,
            rate_limiting: Some(RateLimitingConfig {
                algorithm: RateLimitingAlgorithm::TokenBucket,
                requests_per_second: 100,
                burst_size: 200,
                key: RateLimitingKey::ApiKey,
            }),
            cors: None,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.tls.is_some());
        assert!(deserialized.websocket.is_some());
        assert!(deserialized.rate_limiting.is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Clone
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_network_config_clone() {
        let original = NetworkConfig::default();
        let cloned = original.clone();
        assert_eq!(cloned.port, 8080);
    }

    #[test]
    fn test_http_config_clone() {
        let original = HttpConfig::default();
        let cloned = original.clone();
        assert!(cloned.http2);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Debug Formatting
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_tls_version_debug() {
        let version = TlsVersion::V1_3;
        let debug = format!("{:?}", version);
        assert!(debug.contains("V1_3"));
    }

    #[test]
    fn test_load_balancing_algorithm_debug() {
        let alg = LoadBalancingAlgorithm::LeastConnections;
        let debug = format!("{:?}", alg);
        assert!(debug.contains("LeastConnections"));
    }

    #[test]
    fn test_rate_limiting_key_debug() {
        let key = RateLimitingKey::Custom("tenant-id".to_string());
        let debug = format!("{:?}", key);
        assert!(debug.contains("tenant-id"));
    }
}
