//! Neural API JSON-RPC Server
//!
//! Exposes the Neural API graph orchestration engine via JSON-RPC 2.0 over Unix socket.
//! This enables Squirrel and petalTongue to discover, execute, and monitor graph deployments.
//!
//! # Architecture
//!
//! The server delegates to focused handlers for each domain:
//! - `GraphHandler` - Graph CRUD and execution
//! - `CapabilityHandler` - Capability routing and discovery
//! - `TopologyHandler` - System topology and metrics
//! - `NicheHandler` - Niche template deployment
//!
//! This decomposition keeps each handler under 500 lines while the server
//! focuses on connection handling and request routing.

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::handlers::{CapabilityHandler, GraphHandler, LifecycleHandler, NicheHandler, TopologyHandler};
use crate::mode::BiomeOsMode;
use crate::neural_graph::Graph;
use crate::neural_router::{NeuralRouter, RoutingMetrics};
use crate::nucleation::SocketNucleation;
use biomeos_core::SocketDiscovery;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

/// JSON-RPC 2.0 request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: u64,
}

/// Neural API server state
#[derive(Clone)]
pub struct NeuralApiServer {
    /// Path to graphs directory
    graphs_dir: PathBuf,

    /// Active executions (execution_id -> status)
    executions: Arc<RwLock<HashMap<String, crate::handlers::graph::ExecutionStatus>>>,

    /// Family ID for this server
    family_id: String,

    /// Socket path
    socket_path: PathBuf,

    /// Neural Router for capability-based routing
    router: Arc<NeuralRouter>,

    /// Operating mode (Bootstrap or Coordinated)
    mode: Arc<RwLock<BiomeOsMode>>,

    /// Socket nucleation (deterministic assignment)
    nucleation: Arc<RwLock<SocketNucleation>>,

    /// Capability Translation Registry
    translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,

    // === Handlers (delegated logic) ===
    /// Graph operations handler
    graph_handler: GraphHandler,

    /// Capability routing handler
    capability_handler: CapabilityHandler,

    /// Topology and metrics handler
    topology_handler: TopologyHandler,

    /// Niche deployment handler
    niche_handler: NicheHandler,

    /// Lifecycle management handler (resurrection, apoptosis)
    lifecycle_handler: LifecycleHandler,
}

impl NeuralApiServer {
    /// Create a new Neural API server
    ///
    /// Mode detection happens on first serve() call
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        socket_path: impl Into<PathBuf>,
    ) -> Self {
        use crate::nucleation::SocketStrategy;

        let graphs_dir = graphs_dir.into();
        let family_id_str = family_id.into();
        let router = Arc::new(NeuralRouter::new(&family_id_str));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let translation_registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));

        // Create handlers with shared state
        let graph_handler = GraphHandler::new(
            graphs_dir.clone(),
            family_id_str.clone(),
            executions.clone(),
            router.clone(),
            translation_registry.clone(),
        );

        let capability_handler =
            CapabilityHandler::new(router.clone(), translation_registry.clone());

        let topology_handler = TopologyHandler::new(
            family_id_str.clone(),
            router.clone(),
            executions.clone(),
            graphs_dir.clone(),
        );

        let niche_handler = NicheHandler::new(
            graphs_dir.clone(),
            family_id_str.clone(),
            router.clone(),
            executions.clone(),
        );

        let lifecycle_handler = LifecycleHandler::new(&family_id_str);

        Self {
            graphs_dir,
            executions,
            family_id: family_id_str,
            socket_path: socket_path.into(),
            router,
            mode: Arc::new(RwLock::new(BiomeOsMode::Bootstrap)),
            nucleation: Arc::new(RwLock::new(SocketNucleation::new(
                SocketStrategy::FamilyDeterministic,
            ))),
            translation_registry,
            graph_handler,
            capability_handler,
            topology_handler,
            niche_handler,
            lifecycle_handler,
        }
    }

    /// Start the Neural API server
    pub async fn serve(&self) -> Result<()> {
        // 1. Detect operating mode
        info!("🔍 Detecting biomeOS operating mode...");
        let detected_mode = BiomeOsMode::detect(&self.family_id).await;
        {
            let mut mode = self.mode.write().await;
            *mode = detected_mode;
        }

        // 2. Bootstrap if needed
        if detected_mode == BiomeOsMode::Bootstrap {
            info!("🌱 === BIOMEOS BOOTSTRAP MODE ===");
            info!("🌍 No existing ecosystem detected");
            info!("🏗️  Creating ecosystem foundation...");

            // Register biomeOS in its own capability registry
            self.register_self_in_registry().await?;

            // Execute bootstrap sequence (germinate Tower Atomic)
            info!("");
            info!("🏰 Germinating Tower Atomic (ecosystem genesis)...");
            match self.execute_bootstrap_sequence().await {
                Ok(_) => {
                    info!("✅ Tower Atomic genesis complete!");
                    info!("🔄 Transitioning to COORDINATED MODE...");

                    // Transition to coordinated mode
                    if let Err(e) = self.transition_to_coordinated().await {
                        error!("⚠️  Mode transition failed: {}", e);
                        warn!("   Continuing in bootstrap mode (Tower Atomic may be unhealthy)");
                    } else {
                        // Update mode
                        let mut mode = self.mode.write().await;
                        *mode = BiomeOsMode::Coordinated;
                        info!("✅ biomeOS now operating in COORDINATED MODE (gen 1)");
                    }
                }
                Err(e) => {
                    error!("❌ Bootstrap sequence failed: {}", e);
                    error!("   biomeOS will continue in bootstrap mode");
                    error!("   Manual intervention may be required");
                }
            }
            info!("");
        } else {
            info!("🔄 === BIOMEOS COORDINATED MODE ===");
            info!("🏰 Tower Atomic detected or explicit coordinated mode");
            info!("🌍 Joining existing ecosystem");

            // Check if this is explicit coordinated mode (primals will auto-register)
            let explicit_mode = std::env::var("BIOMEOS_MODE")
                .map(|m| m.to_lowercase())
                .map(|m| m == "coordinated" || m == "coord" || m == "join")
                .unwrap_or(false);

            if explicit_mode {
                // Explicit coordinated mode: don't wait for sockets
                // Primals will register themselves via auto-registration
                info!("📋 Explicit coordinated mode - primals will auto-register");
                info!("   Neural API will accept capability registrations dynamically");
            } else {
                // Auto-detected coordinated mode: establish connection
                if let Err(e) = self.transition_to_coordinated().await {
                    warn!("⚠️  Failed to establish BTSP tunnel: {}", e);
                    warn!("   Operating without inherited security");
                }
            }

            // Register in ecosystem
            self.register_self_in_registry().await?;
        }

        // Start lifecycle monitoring
        info!("🔍 Starting primal lifecycle monitoring...");
        if let Err(e) = self.lifecycle_handler.start_monitoring().await {
            warn!("⚠️ Failed to start lifecycle monitoring: {}", e);
        }

        // ALWAYS load semantic translations from Tower Atomic graph
        // This is ecosystem-wide configuration, not mode-specific
        info!("📝 Loading semantic translations from Tower Atomic graph...");
        let bootstrap_graph_path = self.graphs_dir.join("tower_atomic_bootstrap.toml");
        if bootstrap_graph_path.exists() {
            match crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path) {
                Ok(graph) => match self.load_translations_from_graph(&graph).await {
                    Ok(_) => info!("✅ Semantic translations loaded from graph"),
                    Err(e) => warn!("⚠️  Failed to load translations: {}", e),
                },
                Err(e) => warn!("⚠️  Failed to parse graph: {}", e),
            }
        } else {
            debug!("   No Tower Atomic graph found (will use direct method names)");
        }

        // 3. Remove old socket if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).context("Failed to remove old socket")?;
        }

        // 4. Create Unix socket listener
        let listener =
            UnixListener::bind(&self.socket_path).context("Failed to bind Unix socket")?;

        let mode_str = match detected_mode {
            BiomeOsMode::Bootstrap => "BOOTSTRAP (genesis)",
            BiomeOsMode::Coordinated => "COORDINATED (gen 1)",
        };

        info!(
            "🧠 Neural API server listening on: {}",
            self.socket_path.display()
        );
        info!("   Mode: {}", mode_str);
        info!("   Graphs directory: {}", self.graphs_dir.display());
        info!("   Family ID: {}", self.family_id);

        // 5. Accept connections
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Register biomeOS in the capability registry
    async fn register_self_in_registry(&self) -> Result<()> {
        let mode = self.mode.read().await;
        let source = match *mode {
            BiomeOsMode::Bootstrap => "bootstrap",
            BiomeOsMode::Coordinated => "coordinated",
        };

        let primal_name = format!("biomeos-{}", self.family_id);
        let capabilities = vec![
            "primal.germination",
            "primal.terraria",
            "ecosystem.coordination",
            "ecosystem.nucleation",
            "graph.execution",
        ];

        let cap_count = capabilities.len();

        // Register each capability
        for capability in capabilities {
            self.router
                .register_capability(capability, &primal_name, &self.socket_path, source)
                .await?;
        }

        info!(
            "✅ biomeOS registered {} capabilities in registry",
            cap_count
        );
        Ok(())
    }

    /// Execute bootstrap sequence (germinate Tower Atomic)
    async fn execute_bootstrap_sequence(&self) -> Result<()> {
        use crate::neural_executor::GraphExecutor;
        use std::collections::HashMap;

        // Load tower_atomic_bootstrap.toml
        let bootstrap_graph_path = self.graphs_dir.join("tower_atomic_bootstrap.toml");

        if !bootstrap_graph_path.exists() {
            return Err(anyhow::anyhow!(
                "Bootstrap graph not found: {}",
                bootstrap_graph_path.display()
            ));
        }

        info!(
            "📋 Loading bootstrap graph: {}",
            bootstrap_graph_path.display()
        );
        let graph = crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path)?;

        // Load capability translations from graph
        info!("📝 Loading capability translations from bootstrap graph...");
        self.load_translations_from_graph(&graph).await?;
        info!("✅ Capability translations loaded");

        // Prepare environment
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), self.family_id.clone());
        env.insert("BIOMEOS_FAMILY_ID".to_string(), self.family_id.clone());
        env.insert("BIOMEOS_MODE".to_string(), "bootstrap".to_string());

        // Create executor with nucleation
        info!("🧬 Creating graph executor with socket nucleation...");
        let executor = GraphExecutor::with_nucleation(graph, env, self.nucleation.clone());

        // Execute graph
        info!("🚀 Executing bootstrap graph...");
        let mut executor = executor; // Make mutable for execute()
        let report = executor.execute().await?;

        // Check if successful
        if report.success {
            info!("✅ Bootstrap graph executed successfully");
            info!("   Duration: {}ms", report.duration_ms);
            info!("   Phases: {}", report.phase_results.len());
        } else {
            error!("❌ Bootstrap graph failed");
            if let Some(ref error) = report.error {
                error!("   Error: {}", error);
            }
            return Err(anyhow::anyhow!("Bootstrap graph execution failed"));
        }

        Ok(())
    }

    /// Transition to coordinated mode (establish BTSP tunnel with Tower Atomic)
    async fn transition_to_coordinated(&self) -> Result<()> {
        use tokio::time::{sleep, Duration};

        info!("🔄 Establishing connection with Tower Atomic...");

        // Wait for Tower Atomic to be ready (sockets to exist)
        // Uses SocketNucleation for deterministic paths (no hardcoding)
        let max_wait = Duration::from_secs(30);
        let check_interval = Duration::from_millis(500);
        let start = std::time::Instant::now();

        let mut nucleation = SocketNucleation::default();
        let beardog_socket = nucleation.assign_socket("beardog", &self.family_id);
        let songbird_socket = nucleation.assign_socket("songbird", &self.family_id);

        loop {
            if start.elapsed() > max_wait {
                return Err(anyhow::anyhow!(
                    "Tower Atomic did not become available within 30s"
                ));
            }

            let beardog_exists = beardog_socket.exists();
            let songbird_exists = songbird_socket.exists();

            if beardog_exists && songbird_exists {
                info!("✅ Tower Atomic sockets detected");
                break;
            }

            debug!(
                "   Waiting for Tower Atomic... (BearDog: {}, Songbird: {})",
                if beardog_exists { "✓" } else { "✗" },
                if songbird_exists { "✓" } else { "✗" }
            );

            sleep(check_interval).await;
        }

        // EVOLVED (Jan 27, 2026): Capability-based security context via AtomicClient
        // Layer 1: Verify BearDog health (crypto provider)
        match Self::verify_primal_health(&beardog_socket, "beardog").await {
            Ok(caps) => {
                info!("✅ BearDog healthy with {} capabilities", caps.len());
            }
            Err(e) => {
                warn!(
                    "⚠️ BearDog health check failed: {} (continuing with degraded security)",
                    e
                );
            }
        }

        // Layer 2: Verify Songbird health (discovery/mesh)
        match Self::verify_primal_health(&songbird_socket, "songbird").await {
            Ok(caps) => {
                info!("✅ Songbird healthy with {} capabilities", caps.len());
            }
            Err(e) => {
                warn!(
                    "⚠️ Songbird health check failed: {} (continuing without mesh)",
                    e
                );
            }
        }

        // Layer 3: Establish BTSP tunnel via BearDog (capability: secure_tunneling)
        // This creates a cryptographically secure channel for inter-primal communication
        match Self::establish_btsp_tunnel(&beardog_socket, &self.family_id).await {
            Ok(session_id) => {
                info!("✅ BTSP tunnel established (session: {})", session_id);
            }
            Err(e) => {
                // BTSP is optional for local deployments, warn but continue
                debug!("BTSP tunnel not established: {} (local mode)", e);
            }
        }

        info!("✅ Connected to Tower Atomic (gen 0 → gen 1 transition)");
        info!("   Security context inherited via capability-based discovery");

        Ok(())
    }

    /// Handle a client connection
    async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        use tokio::time::{timeout, Duration};
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();

            // Try to read next request with timeout (client may have shut down write side)
            let read_result =
                timeout(Duration::from_millis(100), reader.read_line(&mut line)).await;

            match read_result {
                Ok(Ok(n)) if n > 0 => {
                    // Request received, handle it
                    let response = match self.handle_request(&line).await {
                        Ok(response) => response,
                        Err(e) => {
                            error!("Request error: {}", e);
                            json!({
                                "jsonrpc": "2.0",
                                "error": {
                                    "code": -32603,
                                    "message": format!("Internal error: {}", e)
                                },
                                "id": null
                            })
                        }
                    };

                    // Write response
                    let response_str = serde_json::to_string(&response)? + "\n";
                    let stream = reader.get_mut();
                    stream.write_all(response_str.as_bytes()).await?;
                    stream.flush().await?;

                    // After sending response, check if we can read more (short timeout)
                    // If client shut down write side, this will timeout quickly
                    continue;
                }
                Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {
                    // EOF, error, or timeout - client is done
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    ///
    /// Delegates to focused handlers for each domain:
    /// - Graph operations → GraphHandler
    /// - Capability routing → CapabilityHandler
    /// - Topology/metrics → TopologyHandler
    /// - Niche templates → NicheHandler
    async fn handle_request(&self, request_line: &str) -> Result<Value> {
        let request: JsonRpcRequest = serde_json::from_str(request_line.trim())
            .context("Failed to parse JSON-RPC request")?;

        debug!("📥 Request: {} (id: {})", request.method, request.id);
        trace!("📥 Full request: {}", request_line.trim());

        let result = match request.method.as_str() {
            // === Graph Operations (delegated to GraphHandler) ===
            "neural_api.list_graphs" | "graph.list" => self.graph_handler.list().await?,
            "neural_api.get_graph" | "graph.get" => self.graph_handler.get(&request.params).await?,
            "neural_api.save_graph" | "graph.save" => {
                self.graph_handler.save(&request.params).await?
            }
            "neural_api.execute_graph" | "graph.execute" => {
                self.graph_handler.execute(&request.params).await?
            }
            "neural_api.get_execution_status" | "graph.status" => {
                self.graph_handler.get_status(&request.params).await?
            }

            // === Topology Operations (delegated to TopologyHandler) ===
            "neural_api.get_topology" | "topology.get" => self.topology_handler.get().await?,
            "neural_api.get_primals" | "topology.primals" => {
                self.topology_handler.get_primals().await?
            }
            "neural_api.get_proprioception" | "topology.proprioception" => {
                self.topology_handler.get_proprioception().await?
            }
            "neural_api.get_metrics" | "topology.metrics" => {
                self.topology_handler.get_metrics().await?
            }

            // === Niche Operations (delegated to NicheHandler) ===
            "neural_api.list_niche_templates" | "niche.list" => self.niche_handler.list().await?,
            "neural_api.deploy_niche" | "niche.deploy" => {
                self.niche_handler.deploy(&request.params).await?
            }

            // === Lifecycle Operations (delegated to LifecycleHandler) ===
            "lifecycle.status" => self.lifecycle_handler.status().await?,
            "lifecycle.get" => self.lifecycle_handler.get(&request.params).await?,
            "lifecycle.register" => self.lifecycle_handler.register(&request.params).await?,
            "lifecycle.resurrect" => self.lifecycle_handler.resurrect(&request.params).await?,
            "lifecycle.apoptosis" => self.lifecycle_handler.apoptosis(&request.params).await?,
            "lifecycle.shutdown_all" => self.lifecycle_handler.shutdown_all().await?,

            // === Capability Operations (delegated to CapabilityHandler) ===
            "capability.register" => self.capability_handler.register(&request.params).await?,
            "capability.discover" | "neural_api.discover_capability" => {
                self.capability_handler.discover(&request.params).await?
            }
            "capability.list" => self.capability_handler.list().await?,
            "capability.providers" => self.capability_handler.providers(&request.params).await?,
            "capability.route" | "neural_api.route_to_primal" => {
                self.capability_handler.route(&request.params).await?
            }
            "capability.metrics" | "neural_api.get_routing_metrics" => {
                self.capability_handler.get_metrics().await?
            }
            "capability.call" => self.capability_handler.call(&request.params).await?,
            "capability.discover_translations" | "capability.discover_translation" => {
                self.capability_handler
                    .discover_translations(&request.params)
                    .await?
            }
            "capability.list_translations" => self.capability_handler.list_translations().await?,

            // === Legacy Routing (still needed for HTTP proxy) ===
            "neural_api.proxy_http" => self.proxy_http(&request.params).await?,

            // === Unknown Method ===
            _ => {
                return Ok(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", request.method)
                    },
                    "id": request.id
                }));
            }
        };

        Ok(json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": request.id
        }))
    }

    // ==================== ROUTING API METHODS (NEW) ====================

    /// Proxy HTTP request through Tower Atomic (Songbird + BearDog)
    ///
    /// This enables primals to make HTTP/HTTPS requests without direct dependencies
    /// on HTTP libraries or C crypto libraries (like ring).
    ///
    /// # TRUE PRIMAL Pattern
    /// Squirrel doesn't know about Songbird or BearDog - it just asks Neural API
    /// for "secure_http" capability, and the router discovers + forwards.
    async fn proxy_http(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        let params = params.as_ref().context("Missing parameters")?;
        let method = params["method"].as_str().context("Missing HTTP method")?;
        let url = params["url"].as_str().context("Missing URL")?;

        // Create a longer-lived binding for default headers
        let default_headers = json!({});
        let headers = params.get("headers").unwrap_or(&default_headers);
        let body = params.get("body");

        info!("🌐 Proxy HTTP: {} {}", method, url);

        // Discover Tower Atomic
        let atomic = self
            .router
            .discover_capability("secure_http")
            .await
            .context("Failed to discover Tower Atomic")?;

        debug!("   Discovered: {:?} primals", atomic.primals.len());

        // Forward to Songbird (handles HTTP/TLS)
        let http_params = json!({
            "method": method,
            "url": url,
            "headers": headers,
            "body": body
        });

        let result = self
            .router
            .forward_request(&atomic.primary_socket, "http.request", &http_params)
            .await?;

        // Log metrics
        let latency = start.elapsed().as_millis() as u64;
        self.router
            .log_metric(RoutingMetrics {
                request_id: request_id.clone(),
                capability: "secure_http".to_string(),
                method: format!("http.{}", method),
                routed_through: atomic.primals.iter().map(|p| p.name.clone()).collect(),
                latency_ms: latency,
                success: true,
                timestamp: chrono::Utc::now(),
                error: None,
            })
            .await;

        info!("   ✓ Proxied in {}ms", latency);

        Ok(result)
    }
    /// Clone for spawning tasks
    fn clone(&self) -> Self {
        Self {
            graphs_dir: self.graphs_dir.clone(),
            executions: self.executions.clone(),
            family_id: self.family_id.clone(),
            socket_path: self.socket_path.clone(),
            router: self.router.clone(),
            mode: self.mode.clone(),
            nucleation: self.nucleation.clone(),
            translation_registry: self.translation_registry.clone(),
            graph_handler: self.graph_handler.clone(),
            capability_handler: self.capability_handler.clone(),
            topology_handler: self.topology_handler.clone(),
            niche_handler: self.niche_handler.clone(),
            lifecycle_handler: self.lifecycle_handler.clone(),
        }
    }

    // ========================================================================
    // Capability Translation API (v2.0.0)
    // ========================================================================

    /// Load capability translations from a graph
    ///
    /// Extracts `capabilities_provided` from each node and registers translations.
    /// Also registers capability categories from `capabilities` field.
    async fn load_translations_from_graph(&self, graph: &Graph) -> Result<()> {
        info!(
            "🔧 load_translations_from_graph called for graph with {} nodes",
            graph.nodes.len()
        );
        let mut registry = self.translation_registry.write().await;
        let mut loaded_count = 0;

        for node in &graph.nodes {
            debug!(
                "   Checking node: {} (capabilities: {:?}, has capabilities_provided: {})",
                node.id,
                node.capabilities,
                node.capabilities_provided.is_some()
            );

            // Get primal name for this node
                let primal_name = if let Some(primal_cfg) = &node.primal {
                    if let Some(cap) = &primal_cfg.by_capability {
                        Some(
                            match cap.as_str() {
                                "security" => "beardog",
                                "discovery" => "songbird",
                                "ai" => "squirrel",
                                "compute" => "toadstool",
                                "storage" => "nestgate",
                                _ => cap.as_str(),
                            }
                            .to_string(),
                        )
                    } else {
                        primal_cfg.by_name.clone()
                    }
                } else {
                    Some(node.id.clone())
                };

            if let Some(ref primal) = primal_name {
                // Get family_id from operation params or use server default
                let family_id = if let Some(operation) = &node.operation {
                    operation
                        .params
                        .get("family_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&self.family_id)
                } else {
                    &self.family_id
                };

                // Build socket path using capability-based discovery
                let socket_discovery = SocketDiscovery::new(family_id.to_string());
                let socket_path = socket_discovery.build_socket_path(primal).to_string_lossy().to_string();

                // Register capability CATEGORIES from the capabilities field
                // This enables capability.call("crypto", "sha256") to route to BearDog
                for capability in &node.capabilities {
                    info!(
                        "📝 Registering capability category: {} → {} @ {}",
                        capability, primal, socket_path
                    );
                    if let Err(e) = self
                        .router
                        .register_capability(capability, primal, &socket_path, "graph_translation")
                        .await
                    {
                        warn!("Failed to register capability {}: {}", capability, e);
                    }
                }
            }

            if let Some(caps_provided) = &node.capabilities_provided {
                // Reuse primal_name extracted above
                if let Some(ref primal) = primal_name {
                    // Get family_id from operation params or use server default
                    let family_id = if let Some(operation) = &node.operation {
                        operation
                            .params
                            .get("family_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&self.family_id)
                    } else {
                        &self.family_id
                    };

                    // Build socket path using capability-based discovery
                    let socket_discovery = SocketDiscovery::new(family_id.to_string());
                    let socket_path = socket_discovery.build_socket_path(primal).to_string_lossy().to_string();

                    // Register all translations for this primal
                    for (semantic, actual) in caps_provided {
                        // Check if there are parameter mappings for this capability
                        let param_mappings = node
                            .parameter_mappings
                            .as_ref()
                            .and_then(|mappings| mappings.get(semantic))
                            .cloned();

                        info!(
                            "📝 Loading translation from graph: {} → {} ({} @ {}) {}",
                            semantic,
                            actual,
                            primal,
                            socket_path,
                            if param_mappings.is_some() {
                                "with param mappings"
                            } else {
                                ""
                            }
                        );

                        registry.register_translation(
                            semantic,
                            primal.as_str(),
                            actual,
                            &socket_path,
                            param_mappings,
                        );

                        loaded_count += 1;
                    }
                }
            }
        }

        if loaded_count > 0 {
            info!(
                "✅ Loaded {} capability translations from graph {}",
                loaded_count, graph.id
            );
        } else {
            debug!("⚠️  No capability translations found in graph {}", graph.id);
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CAPABILITY-BASED PRIMAL COMMUNICATION (Pure Rust, no HTTP)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Verify a primal is healthy and query its capabilities
    ///
    /// Uses Pure Rust JSON-RPC over Unix socket (no C dependencies).
    /// Returns the list of capabilities the primal provides.
    async fn verify_primal_health(
        socket_path: &std::path::Path,
        primal_name: &str,
    ) -> Result<Vec<String>> {
        use tokio::time::{timeout, Duration};

        // Connect to the primal's Unix socket
        let stream = timeout(Duration::from_secs(5), UnixStream::connect(socket_path))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to primal socket")?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send capabilities query (JSON-RPC 2.0)
        let request = json!({
            "jsonrpc": "2.0",
            "method": "primal.capabilities",
            "params": {},
            "id": 1
        });
        let request_str = serde_json::to_string(&request)? + "\n";
        writer.write_all(request_str.as_bytes()).await?;
        writer.flush().await?;

        // Read response
        let mut response_line = String::new();
        timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
            .await
            .context("Response timeout")?
            .context("Failed to read response")?;

        // Parse response
        let response: Value = serde_json::from_str(response_line.trim())?;

        // Check for error
        if let Some(error) = response.get("error") {
            anyhow::bail!(
                "Primal {} returned error: {}",
                primal_name,
                error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown")
            );
        }

        // Extract capabilities from result
        let capabilities = response
            .get("result")
            .and_then(|r| r.get("capabilities"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        debug!(
            "Primal {} provides capabilities: {:?}",
            primal_name, capabilities
        );

        Ok(capabilities)
    }

    /// Establish a BTSP (BearDog Transport Security Protocol) tunnel
    ///
    /// This creates a cryptographically secured channel for inter-primal
    /// communication. The tunnel is authenticated using family lineage.
    ///
    /// # Arguments
    /// * `beardog_socket` - Path to BearDog's Unix socket
    /// * `family_id` - The family identifier for lineage authentication
    ///
    /// # Returns
    /// Session ID for the established tunnel
    async fn establish_btsp_tunnel(
        beardog_socket: &std::path::Path,
        family_id: &str,
    ) -> Result<String> {
        use tokio::time::{timeout, Duration};

        // Connect to BearDog
        let stream = timeout(Duration::from_secs(5), UnixStream::connect(beardog_socket))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to BearDog")?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Request BTSP tunnel establishment
        let request = json!({
            "jsonrpc": "2.0",
            "method": "btsp.establish_tunnel",
            "params": {
                "family_id": family_id,
                "tunnel_type": "local",
                "require_lineage_proof": false  // Local deployments don't require proof
            },
            "id": 1
        });
        let request_str = serde_json::to_string(&request)? + "\n";
        writer.write_all(request_str.as_bytes()).await?;
        writer.flush().await?;

        // Read response
        let mut response_line = String::new();
        timeout(
            Duration::from_secs(10),
            reader.read_line(&mut response_line),
        )
        .await
        .context("BTSP response timeout")?
        .context("Failed to read BTSP response")?;

        // Parse response
        let response: Value = serde_json::from_str(response_line.trim())?;

        // Check for error
        if let Some(error) = response.get("error") {
            anyhow::bail!(
                "BTSP tunnel establishment failed: {}",
                error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown")
            );
        }

        // Extract session ID
        let session_id = response
            .get("result")
            .and_then(|r| r.get("session_id"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing session_id in BTSP response"))?;

        Ok(session_id.to_string())
    }
}
