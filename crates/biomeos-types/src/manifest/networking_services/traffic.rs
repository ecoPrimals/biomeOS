// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Load balancing, traffic policy, connection pool, and TLS client settings.

use serde::{Deserialize, Serialize};

/// Load balancer specification for traffic policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerSpec {
    /// Simple load balancer
    pub simple: Option<LoadBalancerAlgorithm>,

    /// Consistent hash
    pub consistent_hash: Option<ConsistentHashSpec>,
}

/// Load balancer algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Least connections
    LeastConn,
    /// Random selection
    Random,
    /// Pass through without balancing
    Passthrough,
}

/// Traffic policy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPolicySpec {
    /// Load balancer
    pub load_balancer: Option<LoadBalancerSpec>,

    /// Connection pool
    pub connection_pool: Option<ConnectionPoolSpec>,

    /// Outlier detection
    pub outlier_detection: Option<OutlierDetectionSpec>,

    /// TLS
    pub tls: Option<ClientTlsSettings>,
}

/// Connection pool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolSpec {
    /// TCP settings
    pub tcp: Option<TcpSettingsSpec>,

    /// HTTP settings
    pub http: Option<HttpSettingsSpec>,
}

/// TCP settings specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSettingsSpec {
    /// Max connections
    pub max_connections: Option<u32>,

    /// Connect timeout
    pub connect_timeout: Option<u32>,

    /// TCP no delay
    pub tcp_no_delay: Option<bool>,
}

/// HTTP settings specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSettingsSpec {
    /// HTTP1 max pending requests
    pub http1_max_pending_requests: Option<u32>,

    /// HTTP2 max requests
    pub http2_max_requests: Option<u32>,

    /// Max requests per connection
    pub max_requests_per_connection: Option<u32>,

    /// Max retries
    pub max_retries: Option<u32>,

    /// Idle timeout
    pub idle_timeout: Option<u32>,

    /// H2 upgrade policy
    pub h2_upgrade_policy: Option<H2UpgradePolicy>,
}

/// H2 upgrade policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2UpgradePolicy {
    /// Use default upgrade behavior
    Default,
    /// Do not upgrade to HTTP/2
    DoNotUpgrade,
    /// Upgrade to HTTP/2
    Upgrade,
}

/// Outlier detection specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetectionSpec {
    /// Consecutive errors
    pub consecutive_errors: Option<u32>,

    /// Interval
    pub interval: Option<u32>,

    /// Base ejection time
    pub base_ejection_time: Option<u32>,

    /// Max ejection percent
    pub max_ejection_percent: Option<u32>,

    /// Min health percent
    pub min_health_percent: Option<u32>,
}

/// Client TLS settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTlsSettings {
    /// Mode
    pub mode: ClientTlsMode,

    /// Client certificate
    pub client_certificate: Option<String>,

    /// Private key
    pub private_key: Option<String>,

    /// CA certificates
    pub ca_certificates: Option<String>,

    /// Subject alternative names
    pub subject_alternative_names: Vec<String>,

    /// SNI
    pub sni: Option<String>,
}

/// Client TLS modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientTlsMode {
    /// TLS disabled
    Disable,
    /// Simple TLS (server verification only)
    Simple,
    /// Mutual TLS
    Mutual,
    /// Istio-managed mutual TLS
    IstioMutual,
}

/// Consistent hash specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistentHashSpec {
    /// HTTP header name
    pub http_header_name: Option<String>,

    /// HTTP cookie
    pub http_cookie: Option<HttpCookieSpec>,

    /// Use source IP
    pub use_source_ip: Option<bool>,

    /// Ring hash
    pub ring_hash: Option<RingHashSpec>,
}

/// HTTP cookie specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCookieSpec {
    /// Name
    pub name: String,

    /// Path
    pub path: Option<String>,

    /// TTL
    pub ttl: Option<u32>,
}

/// Ring hash specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingHashSpec {
    /// Minimum ring size
    pub minimum_ring_size: Option<u64>,

    /// Maximum ring size
    pub maximum_ring_size: Option<u64>,
}
