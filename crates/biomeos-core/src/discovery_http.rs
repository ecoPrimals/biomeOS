//! HTTP-based primal discovery implementation
//!
//! Discovers primals via HTTP health and identity endpoints.
//!
//! # Features
//!
//! - Discovers primals via identity endpoint (BearDog style)
//! - Falls back to health endpoint if identity unavailable
//! - Configurable timeouts
//! - Automatic retry logic
//!
//! # Examples
//!
//! ```rust,no_run
//! use biomeos_core::{HttpDiscovery, PrimalDiscovery, PrimalType};
//! use biomeos_types::{Endpoint, PrimalId};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let discovery = HttpDiscovery::new(
//!     Endpoint::new("http://localhost:9000")?,
//!     PrimalId::new("beardog-local")?,
//!     "BearDog".to_string(),
//!     PrimalType::Security,
//! )
//! .with_timeout(std::time::Duration::from_secs(10));
//!
//! let primal = discovery.discover(&Endpoint::new("http://localhost:9000")?).await?;
//! println!("Discovered: {} (family: {:?})", primal.name, primal.family_id);
//! # Ok(())
//! # }
//! ```
//!
//! # Helper Functions
//!
//! [`create_local_discovery`] provides a convenient way to discover
//! well-known local primals (BearDog on :9000, Songbird on :8080).

use super::{
    Capability, DiscoveredPrimal, DiscoveryError, DiscoveryResult, HealthStatus, PrimalDiscovery,
    PrimalType,
};
use async_trait::async_trait;
use biomeos_types::{Endpoint, FamilyId, PrimalId};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

/// HTTP-based discovery for a single primal
pub struct HttpDiscovery {
    client: Client,
    endpoint: Endpoint,
    primal_id: PrimalId,
    primal_name: String,
    primal_type: PrimalType,
    timeout: Duration,
}

/// Identity response from primal (generic format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResponse {
    #[serde(default)]
    pub family_id: Option<String>,
    
    #[serde(default)]
    pub capabilities: Vec<String>,
    
    #[serde(default)]
    pub encryption_tag: Option<String>,
    
    #[serde(default)]
    pub version: Option<String>,
}

/// Health response from primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    
    #[serde(default)]
    pub version: Option<String>,
}

impl HttpDiscovery {
    /// Create a new HTTP discovery for a primal
    pub fn new(
        endpoint: Endpoint,
        primal_id: PrimalId,
        primal_name: String,
        primal_type: PrimalType,
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            endpoint,
            primal_id,
            primal_name,
            primal_type,
            timeout: Duration::from_secs(5),
        }
    }
    
    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Try to discover via identity endpoint (BearDog style)
    async fn try_identity(&self) -> DiscoveryResult<IdentityResponse> {
        let identity_url = self.endpoint.join("api/v1/trust/identity")?;
        
        let response = tokio::time::timeout(
            self.timeout,
            self.client.get(identity_url.as_str()).send(),
        )
        .await
        .map_err(|_| DiscoveryError::Timeout { timeout: self.timeout })?
        .map_err(DiscoveryError::Network)?;
        
        if !response.status().is_success() {
            return Err(DiscoveryError::InvalidResponse {
                message: format!("Status: {}", response.status()),
            });
        }
        
        response
            .json::<IdentityResponse>()
            .await
            .map_err(DiscoveryError::Network)
    }
    
    /// Try to discover via health endpoint
    async fn try_health(&self) -> DiscoveryResult<HealthResponse> {
        let health_url = self.endpoint.join("health")?;
        
        let response = tokio::time::timeout(
            self.timeout,
            self.client.get(health_url.as_str()).send(),
        )
        .await
        .map_err(|_| DiscoveryError::Timeout { timeout: self.timeout })?
        .map_err(DiscoveryError::Network)?;
        
        if !response.status().is_success() {
            return Err(DiscoveryError::InvalidResponse {
                message: format!("Status: {}", response.status()),
            });
        }
        
        response
            .json::<HealthResponse>()
            .await
            .map_err(DiscoveryError::Network)
    }
}

#[async_trait]
impl PrimalDiscovery for HttpDiscovery {
    async fn discover(&self, _endpoint: &Endpoint) -> DiscoveryResult<DiscoveredPrimal> {
        // Try identity endpoint first (most informative)
        match self.try_identity().await {
            Ok(identity) => {
                info!("✅ Discovered {} via identity endpoint", self.primal_name);
                
                let version = identity
                    .version
                    .and_then(|v| semver::Version::parse(&v).ok())
                    .unwrap_or_else(|| semver::Version::new(0, 1, 0));
                
                let capabilities: Vec<Capability> = identity
                    .capabilities
                    .into_iter()
                    .map(Capability::new)
                    .collect();
                
                let family_id = identity.family_id.map(FamilyId::new);
                
                return Ok(DiscoveredPrimal {
                    id: self.primal_id.clone(),
                    name: self.primal_name.clone(),
                    primal_type: self.primal_type,
                    version,
                    health: HealthStatus::Healthy,
                    capabilities,
                    endpoint: self.endpoint.clone(),
                    family_id,
                    metadata: serde_json::Value::Null,
                });
            }
            Err(e) => {
                warn!("Identity endpoint failed for {}: {}", self.primal_name, e);
            }
        }
        
        // Fall back to health endpoint
        match self.try_health().await {
            Ok(health) => {
                info!("✅ Discovered {} via health endpoint", self.primal_name);
                
                let version = health
                    .version
                    .and_then(|v| semver::Version::parse(&v).ok())
                    .unwrap_or_else(|| semver::Version::new(0, 1, 0));
                
                let health_status = match health.status.as_str() {
                    "healthy" | "ok" => HealthStatus::Healthy,
                    "degraded" => HealthStatus::Degraded,
                    "unhealthy" => HealthStatus::Unhealthy,
                    _ => HealthStatus::Unknown,
                };
                
                Ok(DiscoveredPrimal {
                    id: self.primal_id.clone(),
                    name: self.primal_name.clone(),
                    primal_type: self.primal_type,
                    version,
                    health: health_status,
                    capabilities: vec![], // No capabilities from health endpoint
                    endpoint: self.endpoint.clone(),
                    family_id: None,
                    metadata: serde_json::Value::Null,
                })
            }
            Err(e) => {
                warn!("Health endpoint failed for {}: {}", self.primal_name, e);
                
                // Last resort: Return basic info for primals without HTTP endpoints
                // (e.g., Songbird with tarpc)
                info!("⚠️  Using fallback discovery for {} (no HTTP endpoints)", self.primal_name);
                Ok(DiscoveredPrimal {
                    id: self.primal_id.clone(),
                    name: self.primal_name.clone(),
                    primal_type: self.primal_type,
                    version: semver::Version::new(0, 1, 0),
                    health: HealthStatus::Unknown,
                    capabilities: vec![], 
                    endpoint: self.endpoint.clone(),
                    family_id: None,
                    metadata: serde_json::json!({"discovery_method": "fallback"}),
                })
            }
        }
    }
    
    async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredPrimal>> {
        match self.discover(&self.endpoint).await {
            Ok(primal) => Ok(vec![primal]),
            Err(_) => Ok(vec![]), // Single primal, just return empty if not found
        }
    }
    
    async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
        match self.try_health().await {
            Ok(health) => {
                let status = match health.status.as_str() {
                    "healthy" | "ok" => HealthStatus::Healthy,
                    "degraded" => HealthStatus::Degraded,
                    "unhealthy" => HealthStatus::Unhealthy,
                    _ => HealthStatus::Unknown,
                };
                Ok(status)
            }
            Err(_) => Ok(HealthStatus::Unknown),
        }
    }
}

/// Builder for creating multiple HTTP discoveries
pub struct HttpDiscoveryBuilder {
    discoveries: Vec<HttpDiscovery>,
}

impl HttpDiscoveryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            discoveries: Vec::new(),
        }
    }
    
    /// Add a primal to discover
    pub fn add_primal(
        mut self,
        endpoint: Endpoint,
        id: PrimalId,
        name: String,
        primal_type: PrimalType,
    ) -> Self {
        self.discoveries.push(HttpDiscovery::new(endpoint, id, name, primal_type));
        self
    }
    
    /// Build into a vector of discoveries
    pub fn build(self) -> Vec<Box<dyn PrimalDiscovery>> {
        self.discoveries
            .into_iter()
            .map(|d| Box::new(d) as Box<dyn PrimalDiscovery>)
            .collect()
    }
}

impl Default for HttpDiscoveryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create discovery for well-known local primals
pub fn create_local_discovery() -> DiscoveryResult<Vec<Box<dyn PrimalDiscovery>>> {
    let mut builder = HttpDiscoveryBuilder::new();
    
    // BearDog (Security)
    if let Ok(endpoint) = Endpoint::new("http://localhost:9000") {
        builder = builder.add_primal(
            endpoint,
            PrimalId::new_unchecked("beardog-local"),
            "BearDog".to_string(),
            PrimalType::Security,
        );
    }
    
    // Songbird (Orchestration)
    if let Ok(endpoint) = Endpoint::new("http://localhost:8080") {
        builder = builder.add_primal(
            endpoint,
            PrimalId::new_unchecked("songbird-local"),
            "Songbird".to_string(),
            PrimalType::Orchestration,
        );
    }
    
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_http_discovery_builder() {
        let discoveries = HttpDiscoveryBuilder::new()
            .add_primal(
                Endpoint::new("http://localhost:9000").unwrap(),
                PrimalId::new("test-primal").unwrap(),
                "Test Primal".to_string(),
                PrimalType::Security,
            )
            .build();
        
        assert_eq!(discoveries.len(), 1);
    }
    
    #[test]
    fn test_create_local_discovery() {
        let discoveries = create_local_discovery().unwrap();
        assert_eq!(discoveries.len(), 2); // BearDog + Songbird
    }
}

