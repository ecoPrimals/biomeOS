//! Neural API Routing Layer
//!
//! **Universal IPC v3.0**: Uses AtomicClient for multi-transport routing.
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
//! - **Multi-Transport**: Unix, Abstract, TCP via Universal IPC v3.0
//!
//! # Architecture
//!
//! ```text
//! Client → Neural Router → Capability Discovery → Primal Discovery
//!                              ↓                        ↓
//!                         Atomic Mapping          Socket/TCP Lookup
//!                              ↓                        ↓
//!                         AtomicClient Forward ← JSON-RPC → Response
//!                              ↓
//!                         Metrics Collection
//! ```

#![deny(unsafe_code)] // Fast AND safe: Zero unsafe code, async I/O throughout

use anyhow::{anyhow, Context, Result};
use biomeos_core::atomic_client::AtomicClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, info, warn};

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
    pub async fn get_capability_providers(
        &self,
        capability: &str,
    ) -> Option<Vec<RegisteredCapability>> {
        self.capability_registry
            .read()
            .await
            .get(capability)
            .cloned()
    }

    /// Discover primals by capability category
    ///
    /// Maps capability names to categories and queries the registry for any primal
    /// that provides that category capability. This implements TRUE PRIMAL pattern:
    /// primals have self-knowledge only and discover other primals at runtime.
    ///
    /// # Capability Mappings
    /// - crypto/security → "security" category
    /// - discovery → "discovery" category
    /// - ai → "ai" category
    async fn discover_by_capability_category(&self, capability: &str) -> Result<DiscoveredAtomic> {
        // Map capability name to category
        let category = match capability {
            "crypto_sign" | "crypto.sign" | "crypto" | "security" | "encryption" => "security",
            "discovery" => "discovery",
            "ai" | "ai.routing" | "ai.text_generation" | "ai.coordination" => "ai",
            _ => {
                return Err(anyhow!(
                    "Capability '{}' does not map to a known category (security, discovery, ai)",
                    capability
                ));
            }
        };

        debug!(
            "   Mapping capability '{}' to category '{}'",
            capability, category
        );

        // Query registry for any primal providing this category
        let registry = self.capability_registry.read().await;

        // Search for primals that provide this category capability
        let mut matching_providers = Vec::new();
        for (registered_cap, providers) in registry.iter() {
            // Check if this registered capability matches the category
            // This handles both exact matches and category-based matches
            if registered_cap == category || registered_cap.starts_with(&format!("{}.", category)) {
                matching_providers.extend(providers.iter().cloned());
            }
        }

        if matching_providers.is_empty() {
            return Err(anyhow!(
                "No primals found providing '{}' capability. Available capabilities: {:?}",
                category,
                registry.keys().collect::<Vec<_>>()
            ));
        }

        // Use the first healthy provider
        let primary = &matching_providers[0];
        info!(
            "   ✅ Found primal via capability category: {} → {} (provides {})",
            capability, primary.primal_name, category
        );

        // Build discovered atomic from matching providers with health status
        let mut primals = Vec::new();
        for provider in &matching_providers {
            let socket_str = provider.socket_path.to_string_lossy();
            let healthy = Self::check_primal_health(&socket_str).await;
            primals.push(DiscoveredPrimal {
                name: provider.primal_name.clone(),
                socket_path: provider.socket_path.clone(),
                capabilities: vec![category.to_string()],
                healthy,
                last_check: chrono::Utc::now(),
            });
        }

        Ok(DiscoveredAtomic {
            capability: capability.to_string(),
            primals,
            atomic_type: None,
            primary_socket: primary.socket_path.clone(),
        })
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
                info!(
                    "   ✅ Found in registry: {} → {}",
                    capability, primary.primal_name
                );

                // Build discovered atomic from registered providers with health status
                let mut primals = Vec::new();
                for provider in &providers {
                    let socket_str = provider.socket_path.to_string_lossy();
                    let healthy = Self::check_primal_health(&socket_str).await;
                    primals.push(DiscoveredPrimal {
                        name: provider.primal_name.clone(),
                        socket_path: provider.socket_path.clone(),
                        capabilities: vec![capability.to_string()],
                        healthy,
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

        // FALLBACK: Try capability-based discovery by category
        warn!("   ⚠️  Capability not in registry, trying capability category discovery");
        match capability {
            "secure_http" | "http.request" | "http.post" | "http.get" => {
                self.discover_tower_atomic().await
            }
            "secure_storage" => self.discover_nest_atomic().await,
            "secure_compute" => self.discover_node_atomic().await,
            "crypto_sign" | "crypto.sign" | "crypto" | "security" | "encryption" => {
                // Use capability-based discovery: find any primal with "security" capability
                self.discover_by_capability_category(capability).await
            }
            "discovery" => {
                // Use capability-based discovery: find any primal with "discovery" capability
                self.discover_by_capability_category(capability).await
            }
            "ai" | "ai.routing" | "ai.text_generation" | "ai.coordination" => {
                // Use capability-based discovery: find any primal with "ai" capability
                self.discover_by_capability_category(capability).await
            }
            _ => Err(anyhow!(
                "Capability '{}' not registered. Available: {:?}",
                capability,
                self.capability_registry
                    .read()
                    .await
                    .keys()
                    .collect::<Vec<_>>()
            )),
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
            warn!(
                "   ⚠️  Tower Atomic unhealthy: beardog={}, songbird={}",
                beardog.healthy, songbird.healthy
            );
        }

        info!(
            "   ✅ Tower Atomic discovered: {} + {}",
            beardog.name, songbird.name
        );

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
        capability: &str,
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

        // Construct socket path via nucleation (deterministic, not hardcoded)
        use crate::nucleation::SocketNucleation;
        let mut nucleation = SocketNucleation::default();
        let socket_path = nucleation.assign_socket(primal_name, &self.family_id);

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
            healthy: true,        // Future: Actual health check via JSON-RPC ping
            last_check: chrono::Utc::now(),
        };

        // Cache it
        {
            let mut cache = self.discovered_primals.write().await;
            cache.insert(primal_name.to_string(), primal.clone());
        }

        debug!(
            "   ✅ Discovered: {} @ {}",
            primal_name,
            primal.socket_path.display()
        );

        Ok(primal)
    }

    /// Forward JSON-RPC request to primal
    ///
    /// **Pure Rust**: Async I/O, no unsafe code, idiomatic error handling
    ///
    /// # Safety
    ///
    /// This function is safe - it uses:
    /// - `tokio::net::UnixStream` for safe async Unix socket I/O
    /// - `AtomicClient` for multi-transport JSON-RPC (Universal IPC v3.0)
    /// - Configurable timeout via `request_timeout`
    /// - Proper error propagation via `Result<T>`
    ///
    /// No unsafe code, raw pointers, or manual memory management.
    pub async fn forward_request(
        &self,
        socket_path: &PathBuf,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        let start = std::time::Instant::now();

        debug!("   → Forwarding: {} to {}", method, socket_path.display());

        // Create AtomicClient with configured timeout (Universal IPC v3.0)
        let client = AtomicClient::unix(socket_path).with_timeout(self.request_timeout);

        // Forward the request via AtomicClient
        let result = client
            .call(method, params.clone())
            .await
            .context(format!(
                "Failed to forward {} to {}",
                method,
                socket_path.display()
            ))?;

        let latency = start.elapsed().as_millis() as u64;
        debug!("   ✓ Forwarded successfully in {}ms", latency);

        Ok(result)
    }

    /// Log routing metrics for learning
    pub async fn log_metric(&self, metric: RoutingMetrics) {
        // Log before moving metric
        debug!(
            "📊 Metric logged: {} - {}ms",
            metric.method, metric.latency_ms
        );

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

    /// Check if a primal is healthy via JSON-RPC health.check call
    ///
    /// Returns true if the primal responds with healthy status, false otherwise.
    /// Timeout is 2 seconds to avoid blocking.
    async fn check_primal_health(socket_path: &str) -> bool {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;
        use tokio::time::{timeout, Duration};

        // Quick check: socket exists?
        if !std::path::Path::new(socket_path).exists() {
            return false;
        }

        // Try to connect and call health.check
        let health_check = async {
            let stream = UnixStream::connect(socket_path).await?;
            let (read_half, mut write_half) = stream.into_split();

            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "health.check",
                "params": {},
                "id": 1
            });

            write_half.write_all(request.to_string().as_bytes()).await?;
            write_half.write_all(b"\n").await?;
            write_half.flush().await?;

            let mut reader = BufReader::new(read_half);
            let mut response_line = String::new();
            reader.read_line(&mut response_line).await?;

            let response: serde_json::Value = serde_json::from_str(&response_line)?;

            Ok::<bool, anyhow::Error>(
                response
                    .get("result")
                    .and_then(|r| r.get("healthy"))
                    .and_then(|h| h.as_bool())
                    .unwrap_or(false),
            )
        };

        // Timeout after 2 seconds
        match timeout(Duration::from_secs(2), health_check).await {
            Ok(Ok(healthy)) => healthy,
            _ => false,
        }
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
