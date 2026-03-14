// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural API Routing Layer
//!
//! **Universal IPC v3.0 + tarpc**: Uses AtomicClient for multi-transport routing
//! with protocol escalation to tarpc for hot-paths.
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
//! - **Protocol Escalation**: JSON-RPC first, tarpc for hot-paths
//!
//! # Architecture
//!
//! ```text
//! Client → Neural Router → Capability Discovery → Primal Discovery
//!                              ↓                        ↓
//!                         Atomic Mapping          Socket/TCP Lookup
//!                              ↓                        ↓
//!                    Protocol Selection ← LivingGraph Protocol State
//!                              ↓
//!                    [JSON-RPC] or [tarpc]
//!                              ↓
//!                         Metrics Collection
//! ```

#![deny(unsafe_code)] // Fast AND safe: Zero unsafe code, async I/O throughout

use anyhow::{anyhow, Context, Result};
use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::tarpc_types::ProtocolPreference;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, info, warn};

use crate::capability_domains::capability_to_provider_fallback;
use crate::living_graph::{LivingGraph, ProtocolMode};

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
    pub(crate) family_id: String,

    /// Discovered primals cache (runtime discovery)
    discovered_primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,

    /// Capability Registry (NEW - dynamic registration!)
    capability_registry: Arc<RwLock<HashMap<String, Vec<RegisteredCapability>>>>,

    /// Metrics collection
    metrics: Arc<RwLock<Vec<RoutingMetrics>>>,

    /// Request timeout
    request_timeout: Duration,

    /// Living graph for protocol state tracking (JSON-RPC AND tarpc first)
    living_graph: Option<Arc<LivingGraph>>,

    /// Protocol preference from environment
    protocol_preference: ProtocolPreference,
}

impl NeuralRouter {
    /// Create a new Neural Router
    ///
    /// **Zero Hardcoding**: Uses family_id for runtime discovery
    /// **JSON-RPC AND tarpc first**: Protocol preference from environment
    pub fn new(family_id: impl Into<String>) -> Self {
        Self {
            family_id: family_id.into(),
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            capability_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            request_timeout: Duration::from_secs(30),
            living_graph: None,
            protocol_preference: biomeos_types::tarpc_types::protocol_from_env(),
        }
    }

    /// Attach a living graph for protocol-aware routing
    ///
    /// Enables tarpc escalation for hot-path capabilities when primals support it.
    pub fn with_living_graph(mut self, graph: Arc<LivingGraph>) -> Self {
        self.living_graph = Some(graph);
        self
    }

    /// Set protocol preference override
    pub fn with_protocol_preference(mut self, preference: ProtocolPreference) -> Self {
        self.protocol_preference = preference;
        self
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
            .or_default()
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
                    "Capability '{capability}' does not map to a known category (security, discovery, ai)"
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
            if registered_cap == category || registered_cap.starts_with(&format!("{category}.")) {
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

    /// Discover Tower Atomic (security + discovery capabilities)
    ///
    /// **Capability-Based Discovery**: Finds primals by capability, not by name.
    /// Tower Atomic = primal with "security" capability + primal with "discovery" capability
    async fn discover_tower_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Tower Atomic (security + discovery capabilities)");

        // Discover by CAPABILITY, not by name
        // This allows different primals to provide these capabilities in the future
        let security_primal = self
            .find_primal_by_capability("security")
            .await
            .context("Tower Atomic requires a primal with 'security' capability")?;

        let discovery_primal = self
            .find_primal_by_capability("discovery")
            .await
            .context("Tower Atomic requires a primal with 'discovery' capability")?;

        // Verify both are healthy
        if !security_primal.healthy || !discovery_primal.healthy {
            warn!(
                "   ⚠️  Tower Atomic unhealthy: security={}, discovery={}",
                security_primal.healthy, discovery_primal.healthy
            );
        }

        info!(
            "   ✅ Tower Atomic discovered: {} (security) + {} (discovery)",
            security_primal.name, discovery_primal.name
        );

        Ok(DiscoveredAtomic {
            capability: "secure_http".to_string(),
            primals: vec![security_primal.clone(), discovery_primal.clone()],
            atomic_type: Some(AtomicType::Tower),
            primary_socket: discovery_primal.socket_path, // Discovery primal handles HTTP
        })
    }

    /// Discover Nest Atomic (Tower + storage capability)
    ///
    /// **Capability-Based Discovery**: Finds storage primal by capability, not by name.
    async fn discover_nest_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Nest Atomic (Tower + storage capability)");

        // First get Tower Atomic
        let tower = self.discover_tower_atomic().await?;

        // Then find primal with "storage" capability
        let storage_primal = self
            .find_primal_by_capability("storage")
            .await
            .context("Nest Atomic requires a primal with 'storage' capability")?;

        let mut primals = tower.primals;
        primals.push(storage_primal.clone());

        info!(
            "   ✅ Nest Atomic discovered: Tower + {} (storage)",
            storage_primal.name
        );

        Ok(DiscoveredAtomic {
            capability: "secure_storage".to_string(),
            primals,
            atomic_type: Some(AtomicType::Nest),
            primary_socket: storage_primal.socket_path, // Storage primal handles storage
        })
    }

    /// Discover Node Atomic (Tower + compute capability)
    ///
    /// **Capability-Based Discovery**: Finds compute primal by capability, not by name.
    async fn discover_node_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Node Atomic (Tower + compute capability)");

        // First get Tower Atomic
        let tower = self.discover_tower_atomic().await?;

        // Then find primal with "compute" capability
        let compute_primal = self
            .find_primal_by_capability("compute")
            .await
            .context("Node Atomic requires a primal with 'compute' capability")?;

        let mut primals = tower.primals;
        primals.push(compute_primal.clone());

        info!(
            "   ✅ Node Atomic discovered: Tower + {} (compute)",
            compute_primal.name
        );

        Ok(DiscoveredAtomic {
            capability: "secure_compute".to_string(),
            primals,
            atomic_type: Some(AtomicType::Node),
            primary_socket: compute_primal.socket_path, // Compute primal handles compute
        })
    }

    /// Find primal by capability (capability-based discovery)
    ///
    /// **Deep Debt Principle**: Discover by WHAT a primal can do, not WHO it is.
    /// This allows the ecosystem to evolve without changing discovery code.
    async fn find_primal_by_capability(&self, capability: &str) -> Result<DiscoveredPrimal> {
        // First, check the capability registry for registered providers
        let registry = self.capability_registry.read().await;

        if let Some(providers) = registry.get(capability) {
            if let Some(provider) = providers.first() {
                // Found in registry - use the registered socket
                debug!(
                    "   📖 Registry hit: {} provides '{}'",
                    provider.primal_name, capability
                );

                // Perform quick health check via AtomicClient
                let healthy = self.quick_health_check(&provider.socket_path).await;

                return Ok(DiscoveredPrimal {
                    name: provider.primal_name.clone(),
                    socket_path: provider.socket_path.clone(),
                    capabilities: vec![capability.to_string()],
                    healthy,
                    last_check: chrono::Utc::now(),
                });
            }
        }

        // Fallback: Use configurable capability-to-provider mappings
        // See: config/capability_registry.toml for domain definitions
        let fallback_primal = capability_to_provider_fallback(capability);

        if let Some(primal) = fallback_primal {
            debug!(
                "   ⚠️  Registry miss: using fallback mapping {} → {}",
                capability, primal
            );
            self.find_primal_by_socket(primal).await
        } else {
            Err(anyhow!(
                "No primal found for capability '{capability}'. Register a provider or check the capability name."
            ))
        }
    }

    /// Find primal by socket pattern (runtime discovery)
    ///
    /// **Zero Hardcoding**: Constructs socket path from family_id + primal name
    pub(crate) async fn find_primal_by_socket(
        &self,
        primal_name: &str,
    ) -> Result<DiscoveredPrimal> {
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

        // Perform health check
        let healthy = self.quick_health_check(&socket_path).await;

        // Create discovered primal
        let primal = DiscoveredPrimal {
            name: primal_name.to_string(),
            socket_path,
            capabilities: vec![], // Future: Query primal for capabilities via JSON-RPC
            healthy,
            last_check: chrono::Utc::now(),
        };

        // Cache it
        {
            let mut cache = self.discovered_primals.write().await;
            cache.insert(primal_name.to_string(), primal.clone());
        }

        debug!(
            "   ✅ Discovered: {} @ {} (healthy: {})",
            primal_name,
            primal.socket_path.display(),
            healthy
        );

        Ok(primal)
    }

    /// Quick health check via AtomicClient
    ///
    /// Attempts a lightweight health.check RPC call with a short timeout.
    /// Returns true if the primal responds, false otherwise.
    async fn quick_health_check(&self, socket_path: &PathBuf) -> bool {
        use std::time::Duration;

        // Use a short timeout for health checks
        let health_timeout = Duration::from_millis(500);

        let client = AtomicClient::unix(socket_path).with_timeout(health_timeout);

        match client.call("health.check", serde_json::json!({})).await {
            Ok(response) => {
                // Check if response indicates healthy
                response
                    .get("healthy")
                    .and_then(|h| h.as_bool())
                    .unwrap_or(true) // Default to healthy if no explicit field
            }
            Err(_) => {
                // Connection failed or timed out - unhealthy
                debug!("   ⚠️ Health check failed for {}", socket_path.display());
                false
            }
        }
    }

    /// Forward JSON-RPC request to primal
    ///
    /// **Pure Rust**: Async I/O, no unsafe code, idiomatic error handling
    ///
    /// # Safety
    ///
    /// Forward a request to a primal via the appropriate protocol
    ///
    /// **JSON-RPC AND tarpc first**: Checks protocol availability and preferences.
    ///
    /// This function is safe - it uses:
    /// - `AtomicClient` for multi-transport JSON-RPC (Universal IPC v3.0)
    /// - Protocol selection based on `LivingGraph` state and `ProtocolPreference`
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

        // Determine protocol based on preference and availability
        let use_tarpc = self.should_use_tarpc(socket_path).await;

        if use_tarpc {
            // tarpc path - infrastructure ready, primal servers need implementation
            //
            // Current Status (Feb 2026):
            // - tarpc service traits defined in biomeos_types::tarpc_types
            // - NeuralRouter has protocol selection infrastructure
            // - LivingGraph tracks protocol escalation metrics
            // - Primal servers need to implement tarpc endpoints
            //
            // When primals implement tarpc servers:
            // return self.forward_via_tarpc(socket_path, method, params).await;
            debug!(
                "   → tarpc preferred for {} - fallback to JSON-RPC (primal tarpc servers pending)",
                socket_path.display()
            );
        }

        debug!(
            "   → Forwarding via JSON-RPC: {} to {}",
            method,
            socket_path.display()
        );

        // Create AtomicClient with configured timeout (Universal IPC v3.0)
        let client = AtomicClient::unix(socket_path).with_timeout(self.request_timeout);

        // Forward the request via AtomicClient (JSON-RPC)
        let result = client.call(method, params.clone()).await.context(format!(
            "Failed to forward {} to {}",
            method,
            socket_path.display()
        ))?;

        let latency = start.elapsed().as_millis() as u64;
        debug!("   ✓ Forwarded successfully in {}ms", latency);

        // Record metrics for protocol escalation decisions
        if let Some(graph) = &self.living_graph {
            // Extract primal name from socket path for metrics
            if let Some(primal_name) = socket_path.file_stem().and_then(|s| s.to_str()) {
                graph
                    .record_request("neural-api", primal_name, latency * 1000, true)
                    .await;
            }
        }

        Ok(result)
    }

    /// Check if tarpc should be used for this request
    async fn should_use_tarpc(&self, socket_path: &std::path::Path) -> bool {
        // Check protocol preference first
        match self.protocol_preference {
            ProtocolPreference::JsonRpcOnly => return false,
            ProtocolPreference::TarpcOnly => return true,
            ProtocolPreference::PreferJsonRpc => return false, // Default to JSON-RPC
            ProtocolPreference::PreferTarpc | ProtocolPreference::Auto => {
                // Check LivingGraph for tarpc availability
            }
        }

        // Check LivingGraph if available
        if let Some(graph) = &self.living_graph {
            // Extract primal name from socket path
            if let Some(primal_name) = socket_path.file_stem().and_then(|s| s.to_str()) {
                if let Some(state) = graph.get_primal_state(primal_name).await {
                    // Use tarpc if available and connection is in tarpc/hybrid mode
                    return state.tarpc_available()
                        && matches!(
                            state.current_mode,
                            ProtocolMode::Tarpc | ProtocolMode::Hybrid
                        );
                }
            }
        }

        false
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

// Capability domain mappings are in crate::capability_domains

// Tests are in neural_router_tests.rs to keep this file under 1000 lines
