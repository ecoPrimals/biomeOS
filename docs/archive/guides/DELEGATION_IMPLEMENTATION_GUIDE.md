# BiomeOS Delegation Implementation Guide

**Status**: 🔄 **IN PROGRESS**  
**Date**: December 24, 2025  
**Purpose**: Practical guide for implementing primal delegation

---

## 🎯 Overview

Now that contamination is removed, we need to implement **proper delegation patterns** to communicate with real primals.

This guide provides concrete examples and patterns for delegating to:
- Songbird (discovery, health, coordination)
- ToadStool (compute, metrics, execution)
- Squirrel (AI, optimization)
- NestGate (storage, persistence)
- BearDog (security, crypto)

---

## 🏗️ Architecture

### Primal Client Layer

```
BiomeOS Manager
       │
       ├─> PrimalClientRegistry
       │     ├─> SongbirdClient
       │     ├─> ToadStoolClient
       │     ├─> SquirrelClient
       │     ├─> NestGateClient
       │     └─> BearDogClient
       │
       └─> Discovers clients at runtime via Songbird
```

---

## 📦 Step 1: Primal Client Trait

Create a common interface for all primal clients:

```rust
// crates/biomeos-core/src/primal_client.rs

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Common interface for primal clients
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Get primal name
    fn name(&self) -> &str;
    
    /// Get primal endpoint
    fn endpoint(&self) -> &str;
    
    /// Check if primal is available
    async fn is_available(&self) -> bool;
    
    /// Perform health check
    async fn health_check(&self) -> Result<HealthStatus>;
    
    /// Execute generic request
    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value>;
}

/// Health status from primal
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    pub details: Option<Value>,
}
```

---

## 📦 Step 2: HTTP Client Base

Create a reusable HTTP client for primal communication:

```rust
// crates/biomeos-core/src/http_client.rs

use anyhow::{Context, Result};
use reqwest::{Client, Method};
use serde_json::Value;
use std::time::Duration;

/// HTTP client for primal communication
pub struct PrimalHttpClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl PrimalHttpClient {
    /// Create new HTTP client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.into(),
            timeout: Duration::from_secs(30),
        }
    }
    
    /// Execute GET request
    pub async fn get(&self, path: &str) -> Result<Value> {
        self.request(Method::GET, path, None).await
    }
    
    /// Execute POST request
    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        self.request(Method::POST, path, Some(body)).await
    }
    
    /// Execute generic request
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut request = self.client.request(method.clone(), &url);
        
        if let Some(body) = body {
            request = request.json(&body);
        }
        
        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to send {} request to {}", method, url))?;
        
        if !response.status().is_success() {
            anyhow::bail!(
                "Primal request failed: {} {}",
                response.status(),
                response.text().await?
            );
        }
        
        response
            .json()
            .await
            .context("Failed to parse primal response")
    }
}
```

---

## 📦 Step 3: Songbird Client

Implement Songbird discovery and coordination:

```rust
// crates/biomeos-core/src/clients/songbird.rs

use crate::http_client::PrimalHttpClient;
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Songbird discovery and coordination client
pub struct SongbirdClient {
    http: PrimalHttpClient,
    endpoint: String,
}

impl SongbirdClient {
    /// Create new Songbird client
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }
    
    /// Discover services by capability
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInfo>> {
        let response = self.http
            .get(&format!("/api/v1/services/query/{}", capability))
            .await?;
        
        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse service list: {}", e))
    }
    
    /// Register a service
    pub async fn register_service(&self, service: &ServiceRegistration) -> Result<String> {
        let response = self.http
            .post("/api/v1/services/register", serde_json::to_value(service)?)
            .await?;
        
        response["service_id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No service_id in registration response"))
    }
    
    /// Get service health
    pub async fn get_service_health(&self, service_id: &str) -> Result<HealthStatus> {
        let response = self.http
            .get(&format!("/api/health/{}", service_id))
            .await?;
        
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"].as_str().unwrap_or("").to_string(),
            details: Some(response),
        })
    }
    
    /// Query services with metadata filter
    pub async fn query_with_metadata<F>(
        &self,
        capability: &str,
        filter: F,
    ) -> Result<Vec<ServiceInfo>>
    where
        F: Fn(&ServiceMetadata) -> bool,
    {
        let all = self.discover_by_capability(capability).await?;
        Ok(all.into_iter().filter(|s| filter(&s.metadata)).collect())
    }
}

#[async_trait]
impl PrimalClient for SongbirdClient {
    fn name(&self) -> &str {
        "songbird"
    }
    
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
    
    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }
    
    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"].as_str().unwrap_or("").to_string(),
            details: Some(response),
        })
    }
    
    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
    }
}

/// Service information from Songbird
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceInfo {
    pub service_id: String,
    pub service_name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub metadata: ServiceMetadata,
}

/// Service metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceMetadata {
    pub version: String,
    #[serde(default)]
    pub location: Option<Location>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Geographic location
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub fn distance_to(&self, lat: f64, lon: f64) -> f64 {
        // Haversine formula for distance in km
        let r = 6371.0; // Earth radius in km
        let d_lat = (lat - self.latitude).to_radians();
        let d_lon = (lon - self.longitude).to_radians();
        
        let a = (d_lat / 2.0).sin().powi(2)
            + self.latitude.to_radians().cos()
            * lat.to_radians().cos()
            * (d_lon / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }
}

/// Service registration request
#[derive(Debug, Clone, Serialize)]
pub struct ServiceRegistration {
    pub service_name: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub metadata: ServiceMetadata,
}
```

---

## 📦 Step 4: ToadStool Client

Implement ToadStool compute and metrics:

```rust
// crates/biomeos-core/src/clients/toadstool.rs

use crate::http_client::PrimalHttpClient;
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// ToadStool compute and execution client
pub struct ToadStoolClient {
    http: PrimalHttpClient,
    endpoint: String,
}

impl ToadStoolClient {
    /// Create new ToadStool client
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }
    
    /// Get resource usage for a service
    pub async fn get_resource_usage(&self, service_id: &str) -> Result<ResourceMetrics> {
        let response = self.http
            .get(&format!("/api/v1/services/{}/metrics", service_id))
            .await?;
        
        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse resource metrics: {}", e))
    }
    
    /// Deploy a workload
    pub async fn deploy_workload(&self, manifest: &WorkloadManifest) -> Result<DeploymentInfo> {
        let response = self.http
            .post("/api/v1/workloads/deploy", serde_json::to_value(manifest)?)
            .await?;
        
        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse deployment info: {}", e))
    }
    
    /// Scale a service
    pub async fn scale_service(&self, service_id: &str, replicas: u32) -> Result<ScaleResult> {
        let body = serde_json::json!({
            "replicas": replicas
        });
        
        let response = self.http
            .post(&format!("/api/v1/services/{}/scale", service_id), body)
            .await?;
        
        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse scale result: {}", e))
    }
    
    /// Get service replicas
    pub async fn get_service_replicas(&self, service_id: &str) -> Result<u32> {
        let response = self.http
            .get(&format!("/api/v1/services/{}/status", service_id))
            .await?;
        
        response["replicas"]
            .as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| anyhow::anyhow!("No replicas field in status response"))
    }
}

#[async_trait]
impl PrimalClient for ToadStoolClient {
    fn name(&self) -> &str {
        "toadstool"
    }
    
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
    
    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }
    
    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"].as_str().unwrap_or("").to_string(),
            details: Some(response),
        })
    }
    
    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
    }
}

/// Resource metrics from ToadStool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceMetrics {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_io: NetworkIO,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkIO {
    pub bytes_in: u64,
    pub bytes_out: u64,
}

/// Workload manifest for deployment
#[derive(Debug, Clone, Serialize)]
pub struct WorkloadManifest {
    pub name: String,
    pub image: String,
    pub replicas: u32,
    pub resources: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f64,
    pub memory_mb: u64,
}

/// Deployment information
#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentInfo {
    pub deployment_id: String,
    pub status: String,
    pub endpoint: Option<String>,
}

/// Scale operation result
#[derive(Debug, Clone, Deserialize)]
pub struct ScaleResult {
    pub previous_replicas: u32,
    pub target_replicas: u32,
    pub status: String,
}
```

---

## 📦 Step 5: Integration into Manager

Update UniversalBiomeOSManager to use clients:

```rust
// crates/biomeos-core/src/universal_biomeos_manager/mod.rs

use crate::clients::{SongbirdClient, ToadStoolClient, SquirrelClient};

pub struct UniversalBiomeOSManager {
    // Primal clients (discovered at runtime)
    songbird: Option<SongbirdClient>,
    toadstool: Option<ToadStoolClient>,
    squirrel: Option<SquirrelClient>,
    // ... other clients
    
    // BiomeOS-specific components
    config: Arc<BiomeOSConfig>,
    // ... other fields
}

impl UniversalBiomeOSManager {
    /// Initialize and discover primals
    pub async fn initialize(&mut self) -> Result<()> {
        // 1. Bootstrap Songbird from environment
        if let Ok(endpoint) = std::env::var("SONGBIRD_ENDPOINT") {
            self.songbird = Some(SongbirdClient::new(endpoint));
        } else {
            // Try mDNS discovery
            if let Ok(endpoint) = self.discover_via_mdns("songbird").await {
                self.songbird = Some(SongbirdClient::new(endpoint));
            }
        }
        
        // 2. Use Songbird to discover other primals
        if let Some(songbird) = &self.songbird {
            // Discover ToadStool
            if let Ok(services) = songbird.discover_by_capability("compute").await {
                if let Some(service) = services.first() {
                    self.toadstool = Some(ToadStoolClient::new(&service.endpoint));
                }
            }
            
            // Discover Squirrel
            if let Ok(services) = songbird.discover_by_capability("ai").await {
                if let Some(service) = services.first() {
                    self.squirrel = Some(SquirrelClient::new(&service.endpoint));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get resource metrics (delegates to ToadStool)
    pub async fn get_resource_metrics(&self, service_id: &str) -> Result<ResourceMetrics> {
        let toadstool = self.toadstool
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!(
                "ToadStool not available. Set TOADSTOOL_ENDPOINT or ensure Songbird discovery."
            ))?;
        
        toadstool.get_resource_usage(service_id).await
    }
}
```

---

## 🧪 Step 6: Integration Tests

Create tests with real primals:

```rust
// tests/real_primal_integration.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test with real Songbird from phase1bins
    #[tokio::test]
    #[ignore] // Only run when primals are running
    async fn test_songbird_discovery() {
        let songbird = SongbirdClient::new("http://localhost:3000");
        
        // Should be able to check health
        let health = songbird.health_check().await.unwrap();
        assert!(health.healthy);
        
        // Should be able to discover services
        let services = songbird.discover_by_capability("compute").await;
        assert!(services.is_ok());
    }
    
    /// Test with real ToadStool from phase1bins
    #[tokio::test]
    #[ignore] // Only run when primals are running
    async fn test_toadstool_metrics() {
        let toadstool = ToadStoolClient::new("http://localhost:8080");
        
        // Should be able to check health
        let health = toadstool.health_check().await.unwrap();
        assert!(health.healthy);
    }
}
```

---

## 📝 Implementation Checklist

### Phase 1: Foundation (Week 1)
- [ ] Create `PrimalClient` trait
- [ ] Create `PrimalHttpClient` base
- [ ] Implement `SongbirdClient`
- [ ] Implement `ToadStoolClient`
- [ ] Update `UniversalBiomeOSManager`

### Phase 2: Integration (Week 2)
- [ ] Replace mock metrics with real ToadStool calls
- [ ] Replace mock discovery with real Songbird calls
- [ ] Add proper error handling
- [ ] Add retry logic

### Phase 3: Testing (Week 3)
- [ ] Create integration test suite
- [ ] Test with real primals from phase1bins
- [ ] Add E2E workflow tests
- [ ] Document usage patterns

### Phase 4: Polish (Week 4)
- [ ] Add connection pooling
- [ ] Add request caching
- [ ] Add circuit breakers
- [ ] Performance optimization

---

## 🎯 Next Steps

1. **Create client module structure**
2. **Implement Songbird client first** (discovery is foundational)
3. **Implement ToadStool client** (most commonly used)
4. **Update manager to use clients**
5. **Add integration tests**
6. **Document patterns**

---

**Status**: Guide complete, ready for implementation  
**Date**: December 24, 2025  
**Next**: Begin implementing primal clients

---

*"Delegate to specialists. Compose with confidence."*

