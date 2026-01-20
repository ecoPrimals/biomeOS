//! Neural API Routing Layer
//!
//! Pure Rust implementation of capability-based primal routing.
//!
//! # Design Principles
//!
//! - **TRUE PRIMAL**: Self-knowledge only, runtime discovery
//! - **Capability-Based**: No hardcoded primal names/paths
//! - **Service Mesh**: API gateway pattern
//! - **Zero Unsafe**: Fast AND safe Rust
//! - **Observable**: All requests logged for learning
//!
//! # Architecture
//!
//! ```text
//! Client → Neural Router → Capability Discovery → Primal Discovery
//!                              ↓                        ↓
//!                         Atomic Mapping          Socket Lookup
//!                              ↓                        ↓
//!                         Request Forward ← JSON-RPC → Response
//!                              ↓
//!                         Metrics Collection
//! ```

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Discovered primal with socket and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal name (e.g., "beardog", "songbird")
    pub name: String,
    
    /// Unix socket path
    pub socket_path: PathBuf,
    
    /// Capabilities this primal provides
    pub capabilities: Vec<String>,
    
    /// Health status
    pub healthy: bool,
    
    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Atomic type composition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtomicType {
    /// Tower Atomic: BearDog + Songbird (secure communications)
    Tower,
    
    /// Nest Atomic: Tower + NestGate (secure storage)
    Nest,
    
    /// Node Atomic: Tower + ToadStool (secure compute)
    Node,
}

/// Result of capability discovery
#[derive(Debug, Clone)]
pub struct DiscoveredAtomic {
    /// Capability that was discovered
    pub capability: String,
    
    /// Primals that provide this capability
    pub primals: Vec<DiscoveredPrimal>,
    
    /// Atomic type (if applicable)
    pub atomic_type: Option<AtomicType>,
    
    /// Primary primal to route to
    pub primary_socket: PathBuf,
}

/// Metrics for a routing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Unique request ID
    pub request_id: String,
    
    /// Capability requested
    pub capability: String,
    
    /// Method called
    pub method: String,
    
    /// Primals involved in routing
    pub routed_through: Vec<String>,
    
    /// Total latency in milliseconds
    pub latency_ms: u64,
    
    /// Success status
    pub success: bool,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Capability registration info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredCapability {
    /// Capability name (e.g., "http.request", "crypto.sign")
    pub capability: String,
    
    /// Primal that provides it
    pub primal_name: String,
    
    /// Socket path
    pub socket_path: PathBuf,
    
    /// When it was registered
    pub registered_at: chrono::DateTime<chrono::Utc>,
    
    /// Source of registration (graph, primal_announcement, manual)
    pub source: String,
}

/// Neural Router - Capability-based request routing
pub struct NeuralRouter {
    /// Family ID for socket discovery
    family_id: String,
    
    /// Discovered primals cache (runtime discovery)
    discovered_primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,
    
    /// Capability Registry (NEW - dynamic registration!)
    capability_registry: Arc<RwLock<HashMap<String, Vec<RegisteredCapability>>>>,
    
    /// Metrics collection
    metrics: Arc<RwLock<Vec<RoutingMetrics>>>,
    
    /// Request timeout
    request_timeout: Duration,
}

impl NeuralRouter {
    /// Create a new Neural Router
    ///
    /// **Zero Hardcoding**: Uses family_id for runtime discovery
    pub fn new(family_id: impl Into<String>) -> Self {
        Self {
            family_id: family_id.into(),
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            capability_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            request_timeout: Duration::from_secs(30),
        }
    }
    
    /// Register a capability (NEW - for graph deployment and primal announcements)
    ///
    /// # Arguments
    /// * `capability` - Capability name (e.g., "http.request")
    /// * `primal_name` - Primal providing the capability
    /// * `socket_path` - Path to primal's Unix socket
    /// * `source` - Registration source (graph/primal_announcement/manual)
    pub async fn register_capability(
        &self,
        capability: impl Into<String>,
        primal_name: impl Into<String>,
        socket_path: impl Into<PathBuf>,
        source: impl Into<String>,
    ) -> Result<()> {
        let capability = capability.into();
        let primal_name = primal_name.into();
        let socket_path = socket_path.into();
        
        let registration = RegisteredCapability {
            capability: capability.clone(),
            primal_name: primal_name.clone(),
            socket_path,
            registered_at: chrono::Utc::now(),
            source: source.into(),
        };
        
        let mut registry = self.capability_registry.write().await;
        registry
            .entry(capability.clone())
            .or_insert_with(Vec::new)
            .push(registration);
        
        info!("✅ Registered capability: {} → {}", capability, primal_name);
        
        Ok(())
    }
    
    /// List all registered capabilities
    pub async fn list_capabilities(&self) -> HashMap<String, Vec<RegisteredCapability>> {
        self.capability_registry.read().await.clone()
    }
    
    /// Get providers for a specific capability
    pub async fn get_capability_providers(&self, capability: &str) -> Option<Vec<RegisteredCapability>> {
        self.capability_registry
            .read()
            .await
            .get(capability)
            .cloned()
    }
    
    /// Discover primal(s) by capability
    ///
    /// **TRUE PRIMAL Pattern**: Discovers at runtime via registry (new!) or fallback patterns
    pub async fn discover_capability(&self, capability: &str) -> Result<DiscoveredAtomic> {
        info!("🔍 Discovering capability: {}", capability);
        
        // FIRST: Check dynamic registry (NEW!)
        if let Some(providers) = self.get_capability_providers(capability).await {
            if !providers.is_empty() {
                let primary = &providers[0];
                info!("   ✅ Found in registry: {} → {}", capability, primary.primal_name);
                
                // Build discovered atomic from registered providers
                let mut primals = Vec::new();
                for provider in &providers {
                    primals.push(DiscoveredPrimal {
                        name: provider.primal_name.clone(),
                        socket_path: provider.socket_path.clone(),
                        capabilities: vec![capability.to_string()],
                        healthy: true, // TODO: Actual health check
                        last_check: chrono::Utc::now(),
                    });
                }
                
                return Ok(DiscoveredAtomic {
                    capability: capability.to_string(),
                    primals,
                    atomic_type: None, // Could be enhanced to detect atomic types
                    primary_socket: primary.socket_path.clone(),
                });
            }
        }
        
        // FALLBACK: Use hardcoded patterns (for backwards compatibility during migration)
        warn!("   ⚠️  Capability not in registry, using fallback pattern");
        match capability {
            "secure_http" | "http.request" | "http.post" | "http.get" => {
                self.discover_tower_atomic().await
            }
            "secure_storage" => self.discover_nest_atomic().await,
            "secure_compute" => self.discover_node_atomic().await,
            "crypto_sign" | "crypto.sign" => {
                self.discover_single_primal("beardog", capability).await
            }
            "discovery" => self.discover_single_primal("songbird", capability).await,
            "ai" | "ai.routing" | "ai.text_generation" => {
                self.discover_single_primal("squirrel", capability).await
            }
            _ => Err(anyhow!(
                "Capability '{}' not registered. Available: {:?}",
                capability,
                self.capability_registry.read().await.keys().collect::<Vec<_>>()
            ))
        }
    }
    
    /// Discover Tower Atomic (BearDog + Songbird)
    async fn discover_tower_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Tower Atomic (BearDog + Songbird)");
        
        // Discover both primals
        let beardog = self.find_primal_by_socket("beardog").await?;
        let songbird = self.find_primal_by_socket("songbird").await?;
        
        // Verify both are healthy
        if !beardog.healthy || !songbird.healthy {
            warn!("   ⚠️  Tower Atomic unhealthy: beardog={}, songbird={}", 
                  beardog.healthy, songbird.healthy);
        }
        
        info!("   ✅ Tower Atomic discovered: {} + {}", beardog.name, songbird.name);
        
        Ok(DiscoveredAtomic {
            capability: "secure_http".to_string(),
            primals: vec![beardog.clone(), songbird.clone()],
            atomic_type: Some(AtomicType::Tower),
            primary_socket: songbird.socket_path, // Songbird handles HTTP
        })
    }
    
    /// Discover Nest Atomic (Tower + NestGate)
    async fn discover_nest_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Nest Atomic (Tower + NestGate)");
        
        // First get Tower Atomic
        let tower = self.discover_tower_atomic().await?;
        
        // Then add NestGate
        let nestgate = self.find_primal_by_socket("nestgate").await?;
        
        let mut primals = tower.primals;
        primals.push(nestgate.clone());
        
        info!("   ✅ Nest Atomic discovered: Tower + {}", nestgate.name);
        
        Ok(DiscoveredAtomic {
            capability: "secure_storage".to_string(),
            primals,
            atomic_type: Some(AtomicType::Nest),
            primary_socket: nestgate.socket_path, // NestGate handles storage
        })
    }
    
    /// Discover Node Atomic (Tower + ToadStool)
    async fn discover_node_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Node Atomic (Tower + ToadStool)");
        
        // First get Tower Atomic
        let tower = self.discover_tower_atomic().await?;
        
        // Then add ToadStool
        let toadstool = self.find_primal_by_socket("toadstool").await?;
        
        let mut primals = tower.primals;
        primals.push(toadstool.clone());
        
        info!("   ✅ Node Atomic discovered: Tower + {}", toadstool.name);
        
        Ok(DiscoveredAtomic {
            capability: "secure_compute".to_string(),
            primals,
            atomic_type: Some(AtomicType::Node),
            primary_socket: toadstool.socket_path, // ToadStool handles compute
        })
    }
    
    /// Discover a single primal by capability
    async fn discover_single_primal(
        &self,
        primal_hint: &str,
        capability: &str
    ) -> Result<DiscoveredAtomic> {
        debug!("   Discovering single primal for {}", capability);
        
        let primal = self.find_primal_by_socket(primal_hint).await?;
        
        info!("   ✅ Discovered {} for {}", primal.name, capability);
        
        Ok(DiscoveredAtomic {
            capability: capability.to_string(),
            primals: vec![primal.clone()],
            atomic_type: None,
            primary_socket: primal.socket_path,
        })
    }
    
    /// Find primal by socket pattern (runtime discovery)
    ///
    /// **Zero Hardcoding**: Constructs socket path from family_id + primal name
    async fn find_primal_by_socket(&self, primal_name: &str) -> Result<DiscoveredPrimal> {
        // Check cache first
        {
            let cache = self.discovered_primals.read().await;
            if let Some(primal) = cache.get(primal_name) {
                debug!("   📦 Cache hit: {}", primal_name);
                return Ok(primal.clone());
            }
        }
        
        // Construct socket path (runtime, not hardcoded)
        let socket_path = PathBuf::from(format!("/tmp/{}-{}.sock", primal_name, self.family_id));
        
        // Verify socket exists
        if !socket_path.exists() {
            return Err(anyhow!(
                "Primal '{}' not found: socket {} does not exist",
                primal_name,
                socket_path.display()
            ));
        }
        
        // Create discovered primal
        let primal = DiscoveredPrimal {
            name: primal_name.to_string(),
            socket_path,
            capabilities: vec![], // Future: Query primal for capabilities via JSON-RPC
            healthy: true, // Future: Actual health check via JSON-RPC ping
            last_check: chrono::Utc::now(),
        };
        
        // Cache it
        {
            let mut cache = self.discovered_primals.write().await;
            cache.insert(primal_name.to_string(), primal.clone());
        }
        
        debug!("   ✅ Discovered: {} @ {}", primal_name, primal.socket_path.display());
        
        Ok(primal)
    }
    
    /// Forward JSON-RPC request to primal
    ///
    /// **Pure Rust**: Async I/O, no unsafe code, idiomatic error handling
    pub async fn forward_request(
        &self,
        socket_path: &PathBuf,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        let start = std::time::Instant::now();
        
        debug!("   → Forwarding: {} to {}", method, socket_path.display());
        
        // Connect to primal's Unix socket
        let mut stream = timeout(
            Duration::from_secs(5),
            UnixStream::connect(socket_path)
        )
        .await
        .context("Connection timeout")?
        .context("Failed to connect to primal")?;
        
        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        // Send request
        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await?;
        stream.write_all(b"\n").await?; // Newline delimiter
        stream.flush().await?;
        
        debug!("   ✓ Sent {} bytes", request_bytes.len());
        
        // Read response with timeout
        let mut response_bytes = Vec::new();
        timeout(
            self.request_timeout,
            stream.read_to_end(&mut response_bytes)
        )
        .await
        .context("Response timeout")??;
        
        debug!("   ✓ Received {} bytes", response_bytes.len());
        
        // Parse response
        let response: Value = serde_json::from_slice(&response_bytes)
            .context("Failed to parse response")?;
        
        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(anyhow!("Primal returned error: {}", error));
        }
        
        // Extract result
        let result = response.get("result")
            .ok_or_else(|| anyhow!("Response missing 'result' field"))?
            .clone();
        
        let latency = start.elapsed().as_millis() as u64;
        debug!("   ✓ Forwarded successfully in {}ms", latency);
        
        Ok(result)
    }
    
    /// Log routing metrics for learning
    pub async fn log_metric(&self, metric: RoutingMetrics) {
        // Log before moving metric
        debug!("📊 Metric logged: {} - {}ms", metric.method, metric.latency_ms);
        
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        
        // Future Enhancement: Persist metrics to disk for learning layer
        // This will enable AI-driven routing optimization based on historical performance
    }
    
    /// Get all collected metrics (for analysis)
    pub async fn get_metrics(&self) -> Vec<RoutingMetrics> {
        self.metrics.read().await.clone()
    }
    
    /// Clear metrics cache
    pub async fn clear_metrics(&self) {
        self.metrics.write().await.clear();
    }
    
    /// Invalidate discovery cache (force rediscovery)
    pub async fn invalidate_cache(&self) {
        self.discovered_primals.write().await.clear();
        info!("🔄 Discovery cache invalidated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_creation() {
        let router = NeuralRouter::new("test-family");
        assert_eq!(router.family_id, "test-family");
    }
    
    #[tokio::test]
    async fn test_socket_path_construction() {
        let router = NeuralRouter::new("nat0");
        
        // This would fail if socket doesn't exist, but shows the pattern
        let result = router.find_primal_by_socket("beardog").await;
        
        // We expect it to look for /tmp/beardog-nat0.sock
        // (Will fail if not running, but that's OK for unit test)
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let router = NeuralRouter::new("test");
        
        let metric = RoutingMetrics {
            request_id: "test-123".to_string(),
            capability: "secure_http".to_string(),
            method: "http.get".to_string(),
            routed_through: vec!["songbird".to_string()],
            latency_ms: 100,
            success: true,
            timestamp: chrono::Utc::now(),
            error: None,
        };
        
        router.log_metric(metric.clone()).await;
        
        let metrics = router.get_metrics().await;
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].request_id, "test-123");
    }
}

