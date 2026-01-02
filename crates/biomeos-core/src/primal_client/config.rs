//! Configuration for Universal Primal Client

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::handle::PrimalHandle;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Cache TTLs
    pub cache: CacheConfig,
    
    /// Timeout settings
    pub timeouts: TimeoutConfig,
    
    /// Connection pooling
    pub pooling: PoolConfig,
    
    /// Discovery methods (tried in order)
    pub discovery_methods: Vec<DiscoveryMethod>,
    
    /// Trust policy
    pub trust_policy: TrustPolicy,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Explicit endpoint overrides
    pub endpoints: std::collections::HashMap<String, String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            timeouts: TimeoutConfig::default(),
            pooling: PoolConfig::default(),
            discovery_methods: vec![
                DiscoveryMethod::Environment,
                DiscoveryMethod::MDns,
                DiscoveryMethod::Multicast,
            ],
            trust_policy: TrustPolicy::TrustAll,
            retry: RetryConfig::default(),
            endpoints: std::collections::HashMap::new(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Schema cache TTL
    pub schema_ttl: Duration,
    
    /// Format hint cache TTL
    pub format_ttl: Duration,
    
    /// Discovery cache TTL
    pub discovery_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            schema_ttl: Duration::from_secs(3600), // 1 hour
            format_ttl: Duration::from_secs(300),  // 5 minutes
            discovery_ttl: Duration::from_secs(30), // 30 seconds
        }
    }
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Request timeout
    pub request: Duration,
    
    /// Connect timeout
    pub connect: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request: Duration::from_secs(30),
            connect: Duration::from_secs(10),
        }
    }
}

/// Connection pooling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Max idle connections per host
    pub max_idle_per_host: usize,
    
    /// Pool idle timeout
    pub idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            idle_timeout: Duration::from_secs(90),
        }
    }
}

/// Discovery method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Environment variables
    Environment,
    
    /// mDNS
    MDns,
    
    /// UDP multicast
    Multicast,
    
    /// Consul
    Consul,
    
    /// DHT (future)
    Dht,
}

/// Trust policy for primals
pub enum TrustPolicy {
    /// Trust all primals (development only - INSECURE)
    TrustAll,
    
    /// Trust primals with genetic lineage from same family
    GeneticLineage {
        family_id: String,
    },
    
    /// Trust primals with specific tags
    Tags {
        required_tags: Vec<String>,
    },
    
    /// Custom trust evaluation (cannot derive Clone due to function pointer)
    Custom(Box<dyn Fn(&PrimalHandle) -> bool + Send + Sync>),
}

// Manual Debug implementation
impl std::fmt::Debug for TrustPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustPolicy::TrustAll => write!(f, "TrustAll"),
            TrustPolicy::GeneticLineage { family_id } => {
                f.debug_struct("GeneticLineage")
                    .field("family_id", family_id)
                    .finish()
            }
            TrustPolicy::Tags { required_tags } => {
                f.debug_struct("Tags")
                    .field("required_tags", required_tags)
                    .finish()
            }
            TrustPolicy::Custom(_) => write!(f, "Custom(<fn>)"),
        }
    }
}

// Manual Clone implementation (Custom variant cannot be cloned)
impl Clone for TrustPolicy {
    fn clone(&self) -> Self {
        match self {
            TrustPolicy::TrustAll => TrustPolicy::TrustAll,
            TrustPolicy::GeneticLineage { family_id } => {
                TrustPolicy::GeneticLineage {
                    family_id: family_id.clone(),
                }
            }
            TrustPolicy::Tags { required_tags } => {
                TrustPolicy::Tags {
                    required_tags: required_tags.clone(),
                }
            }
            TrustPolicy::Custom(_) => {
                // Cannot clone function pointers - default to TrustAll
                tracing::warn!("Cloning TrustPolicy::Custom - defaulting to TrustAll");
                TrustPolicy::TrustAll
            }
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    
    /// Backoff strategy
    pub backoff: BackoffStrategy,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential,
        }
    }
}

/// Backoff strategy for retries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    
    /// Exponential backoff
    Exponential,
    
    /// Linear backoff
    Linear,
}

/// Authentication method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// No authentication
    None,
    
    /// Bearer token
    BearerToken(String),
    
    /// API key
    ApiKey {
        key: String,
        header: String,
    },
    
    /// Mutual TLS
    MutualTls {
        cert: Vec<u8>,
        key: Vec<u8>,
    },
    
    /// Genetic lineage proof
    GeneticLineage {
        tag: String,
        proof: Vec<u8>,
    },
}

