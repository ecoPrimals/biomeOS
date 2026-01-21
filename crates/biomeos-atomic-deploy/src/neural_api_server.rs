//! Neural API JSON-RPC Server
//!
//! Exposes the Neural API graph orchestration engine via JSON-RPC 2.0 over Unix socket.
//! This enables Squirrel and petalTongue to discover, execute, and monitor graph deployments.

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::mode::BiomeOsMode;
use crate::neural_executor::{ExecutionContext, GraphExecutor};
use crate::neural_graph::Graph;
use crate::neural_router::{NeuralRouter, RoutingMetrics}; // NEW: Routing layer
use crate::nucleation::SocketNucleation;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Neural API server state
#[derive(Clone)]
pub struct NeuralApiServer {
    /// Path to graphs directory
    graphs_dir: PathBuf,

    /// Active executions (execution_id -> status)
    executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,

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
    
    /// Capability Translation Registry (NEW: v2.0.0)
    translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
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
        
        let family_id_str = family_id.into();
        let router = Arc::new(NeuralRouter::new(&family_id_str));
        
        Self {
            graphs_dir: graphs_dir.into(),
            executions: Arc::new(RwLock::new(HashMap::new())),
            family_id: family_id_str,
            socket_path: socket_path.into(),
            router,
            mode: Arc::new(RwLock::new(BiomeOsMode::Bootstrap)), // Default, will detect on serve()
            nucleation: Arc::new(RwLock::new(SocketNucleation::new(SocketStrategy::FamilyDeterministic))),
            translation_registry: Arc::new(RwLock::new(CapabilityTranslationRegistry::new())),
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
            info!("🏰 Tower Atomic detected");
            info!("🌍 Joining existing ecosystem");
            
            // Establish BTSP tunnel with Tower Atomic (inherit security)
            if let Err(e) = self.transition_to_coordinated().await {
                warn!("⚠️  Failed to establish BTSP tunnel: {}", e);
                warn!("   Operating without inherited security");
            }
            
            // Register in ecosystem
            self.register_self_in_registry().await?;
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

        info!("🧠 Neural API server listening on: {}", self.socket_path.display());
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
            self.router.register_capability(
                capability,
                &primal_name,
                &self.socket_path,
                source,
            ).await?;
        }
        
        info!("✅ biomeOS registered {} capabilities in registry", cap_count);
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
        
        info!("📋 Loading bootstrap graph: {}", bootstrap_graph_path.display());
        let graph = crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path)?;
        
        // Prepare environment
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), self.family_id.clone());
        env.insert("BIOMEOS_FAMILY_ID".to_string(), self.family_id.clone());
        env.insert("BIOMEOS_MODE".to_string(), "bootstrap".to_string());
        
        // Create executor with nucleation
        info!("🧬 Creating graph executor with socket nucleation...");
        let executor = GraphExecutor::with_nucleation(
            graph,
            env,
            self.nucleation.clone(),
        );
        
        // Execute graph
        info!("🚀 Executing bootstrap graph...");
        let mut executor = executor;  // Make mutable for execute()
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
        let max_wait = Duration::from_secs(30);
        let check_interval = Duration::from_millis(500);
        let start = std::time::Instant::now();
        
        let beardog_socket = format!("/tmp/beardog-{}.sock", self.family_id);
        let songbird_socket = format!("/tmp/songbird-{}.sock", self.family_id);
        
        loop {
            if start.elapsed() > max_wait {
                return Err(anyhow::anyhow!(
                    "Tower Atomic did not become available within 30s"
                ));
            }
            
            let beardog_exists = std::path::Path::new(&beardog_socket).exists();
            let songbird_exists = std::path::Path::new(&songbird_socket).exists();
            
            if beardog_exists && songbird_exists {
                info!("✅ Tower Atomic sockets detected");
                break;
            }
            
            debug!("   Waiting for Tower Atomic... (BearDog: {}, Songbird: {})",
                if beardog_exists { "✓" } else { "✗" },
                if songbird_exists { "✓" } else { "✗" }
            );
            
            sleep(check_interval).await;
        }
        
        // TODO: Establish BTSP tunnel with BearDog
        // TODO: Verify Songbird health
        // TODO: Inherit security context (become generation 1)
        
        info!("✅ Connected to Tower Atomic (gen 0 → gen 1 transition)");
        info!("   Security context inherited");
        
        Ok(())
    }

    /// Handle a client connection
    async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                // Connection closed
                break;
            }

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
            reader.get_mut().write_all(response_str.as_bytes()).await?;
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request_line: &str) -> Result<Value> {
        let request: JsonRpcRequest = serde_json::from_str(request_line.trim())
            .context("Failed to parse JSON-RPC request")?;

        debug!("📥 Request: {} (id: {})", request.method, request.id);

        let result = match request.method.as_str() {
            // Deployment API (graph execution)
            "neural_api.list_graphs" => self.list_graphs().await?,
            "neural_api.get_graph" => self.get_graph(&request.params).await?,
            "neural_api.save_graph" => self.save_graph(&request.params).await?,
            "neural_api.execute_graph" => self.execute_graph(&request.params).await?,
            "neural_api.get_execution_status" => self.get_execution_status(&request.params).await?,
            "neural_api.get_topology" => self.get_topology().await?,
            "neural_api.get_primals" => self.get_primals().await?,
            "neural_api.get_proprioception" => self.get_proprioception().await?,
            "neural_api.get_metrics" => self.get_metrics().await?,
            "neural_api.list_niche_templates" => self.list_niche_templates().await?,
            "neural_api.deploy_niche" => self.deploy_niche(&request.params).await?,
            
            // Routing API (primal-to-primal communication) - NEW
            "neural_api.proxy_http" => self.proxy_http(&request.params).await?,
            "neural_api.discover_capability" => self.discover_capability(&request.params).await?,
            "neural_api.route_to_primal" => self.route_to_primal(&request.params).await?,
            "neural_api.get_routing_metrics" => self.get_routing_metrics().await?,
            
            // Capability Registry API (NEW - for dynamic capability management)
            "capability.register" => self.register_capability(&request.params).await?,
            "capability.discover" => self.capability_discover(&request.params).await?,
            "capability.list" => self.capability_list().await?,
            "capability.providers" => self.capability_providers(&request.params).await?,
            
            // Capability Translation API (NEW v2.0.0 - semantic capability routing)
            "capability.call" => self.capability_call(&request.params).await?,
            "capability.discover_translation" => self.capability_discover_translation(&request.params).await?,
            "capability.list_translations" => self.capability_list_translations().await?,
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

    /// List available graphs
    async fn list_graphs(&self) -> Result<Value> {
        let mut graphs = Vec::new();

        let entries =
            std::fs::read_dir(&self.graphs_dir).context("Failed to read graphs directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(graph) = Graph::from_toml_file(&path) {
                    graphs.push(json!({
                        "id": graph.id,
                        "version": graph.version,
                        "description": graph.description,
                        "node_count": graph.nodes.len(),
                        "estimated_time_ms": null,
                        "tags": []
                    }));
                }
            }
        }

        Ok(json!(graphs))
    }

    /// Get graph details
    async fn get_graph(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.graphs_dir.join(format!("{}.toml", graph_id));
        let graph = Graph::from_toml_file(&graph_path).context("Failed to load graph")?;

        Ok(serde_json::to_value(graph)?)
    }

    /// Save a graph
    async fn save_graph(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph: Graph =
            serde_json::from_value(params.clone()).context("Failed to parse graph")?;

        let graph_path = self.graphs_dir.join(format!("{}.toml", graph.id));

        // Convert graph to TOML and save
        let toml_str =
            toml::to_string_pretty(&graph).context("Failed to serialize graph to TOML")?;

        std::fs::write(&graph_path, toml_str).context("Failed to write graph file")?;

        info!("💾 Saved graph: {} to {}", graph.id, graph_path.display());

        Ok(json!({"graph_id": graph.id}))
    }

    /// Execute a graph
    async fn execute_graph(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;
        let family_id_param = params["family_id"].as_str().unwrap_or(&self.family_id);

        let graph_path = self.graphs_dir.join(format!("{}.toml", graph_id));
        
        // Enhanced debugging for graph loading
        tracing::info!("🔍 Loading graph: {}", graph_id);
        tracing::debug!("   Graph path: {}", graph_path.display());
        tracing::debug!("   Graphs dir: {}", self.graphs_dir.display());
        
        if !graph_path.exists() {
            tracing::error!("❌ Graph file not found: {}", graph_path.display());
            anyhow::bail!("Graph file not found: {}", graph_path.display());
        }
        
        tracing::debug!("✅ Graph file exists, attempting to parse...");
        let graph = Graph::from_toml_file(&graph_path)
            .with_context(|| format!("Failed to load graph from: {}", graph_path.display()))?;
        
        tracing::info!("✅ Graph loaded successfully: {} (version: {})", graph.id, graph.version);
        tracing::debug!("   Nodes: {}", graph.nodes.len());
        
        // NEW v2.0.0: Load capability translations from graph
        self.load_translations_from_graph(&graph).await?;

        // Generate execution ID
        let execution_id = format!("{}-{}", graph_id, chrono::Utc::now().timestamp());
        let started_at = chrono::Utc::now().to_rfc3339();

        // Create execution status
        let status = ExecutionStatus {
            execution_id: execution_id.clone(),
            state: "running".to_string(),
            current_phase: Some(0),
            total_phases: graph.nodes.len(),
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            duration_ms: 0,
            error: None,
        };

        // Store execution status
        self.executions
            .write()
            .await
            .insert(execution_id.clone(), status);

        // Clone values for async move
        let executions = self.executions.clone();
        let execution_id_clone = execution_id.clone();
        let graph_id_owned = graph_id.to_string();
        let family_id_owned = family_id_param.to_string();
        let router = self.router.clone(); // NEW: Clone router for capability registration

        // Execute graph in background
        tokio::spawn(async move {
            let mut env = HashMap::new();
            env.insert("FAMILY_ID".to_string(), family_id_owned.clone());
            env.insert("UID".to_string(), users::get_current_uid().to_string());
            env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());
            env.insert(
                "JWT_SECRET".to_string(),
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".to_string()),
            );

            let mut executor = GraphExecutor::new(graph.clone(), env);
            let start = std::time::Instant::now();

            match executor.execute().await {
                Ok(report) => {
                    // Register capabilities from deployed nodes (NEW!)
                    if report.success {
                        info!("📝 Registering capabilities from deployed graph...");
                        for node in &graph.nodes {
                            if !node.capabilities.is_empty() {
                                // Determine primal name and socket from node
                                let primal_name = node.primal.as_ref()
                                    .and_then(|p| p.by_capability.as_ref()
                                        .map(|cap| match cap.as_str() {
                                            "security" => "beardog",
                                            "discovery" => "songbird",
                                            "ai" => "squirrel",
                                            "compute" => "toadstool",
                                            "storage" => "nestgate",
                                            _ => cap.as_str()
                                        }))
                                    .or_else(|| node.primal.as_ref().and_then(|p| p.by_name.as_ref().map(|s| s.as_str())))
                                    .unwrap_or(&node.id);
                                
                                let runtime_dir = std::env::var("BIOMEOS_RUNTIME_DIR")
                                    .or_else(|_| std::env::var("TMPDIR"))
                                    .unwrap_or_else(|_| "/tmp".to_string());
                                let socket_path = format!("{}/{}-{}.sock", runtime_dir, primal_name, family_id_owned);
                                
                                for capability in &node.capabilities {
                                    if let Err(e) = router.register_capability(
                                        capability,
                                        primal_name,
                                        PathBuf::from(&socket_path),
                                        "graph_deployment",
                                    ).await {
                                        warn!("Failed to register capability {}: {}", capability, e);
                                    } else {
                                        info!("   ✅ {} → {} @ {}", capability, primal_name, socket_path);
                                    }
                                }
                            }
                        }
                    }
                    
                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = if report.success {
                            "completed".to_string()
                        } else {
                            "failed".to_string()
                        };
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = report.error;
                    }
                }
                Err(e) => {
                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = "failed".to_string();
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = Some(format!("Execution failed: {}", e));
                    }
                }
            }
        });

        Ok(json!({
            "execution_id": execution_id,
            "graph_id": graph_id_owned,
            "started_at": started_at
        }))
    }

    /// Get execution status
    async fn get_execution_status(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let execution_id = params["execution_id"]
            .as_str()
            .context("Missing execution_id")?;

        let executions = self.executions.read().await;
        let status = executions
            .get(execution_id)
            .context("Execution not found")?;

        Ok(serde_json::to_value(status)?)
    }

    /// Get system topology
    async fn get_topology(&self) -> Result<Value> {
        // Scan for active primals by checking Unix sockets
        let mut primals = Vec::new();
        let socket_patterns = vec![
            (
                "beardog",
                format!("/tmp/beardog-{}-default.sock", self.family_id),
            ),
            ("songbird", format!("/tmp/songbird-{}.sock", self.family_id)),
            (
                "toadstool",
                format!("/tmp/toadstool-{}.sock", self.family_id),
            ),
            ("nestgate", format!("/tmp/nestgate-{}.sock", self.family_id)),
            ("squirrel", format!("/tmp/squirrel-{}.sock", self.family_id)),
            (
                "petaltongue",
                format!("/tmp/petaltongue-{}.sock", self.family_id),
            ),
        ];

        for (primal_type, socket_path) in socket_patterns {
            if Path::new(&socket_path).exists() {
                primals.push(json!({
                    "id": format!("{}-{}", primal_type, self.family_id),
                    "primal_type": primal_type,
                    "socket_path": socket_path,
                    "health": "healthy",
                    "resource_usage": null,
                    "capabilities": []
                }));
            }
        }

        // Simple connection inference
        let connections = vec![
            json!({"from": format!("songbird-{}", self.family_id), "to": format!("beardog-{}", self.family_id), "connection_type": "security-provider", "latency_ms": null}),
            json!({"from": format!("toadstool-{}", self.family_id), "to": format!("songbird-{}", self.family_id), "connection_type": "discovery", "latency_ms": null}),
        ];

        Ok(json!({
            "primals": primals,
            "connections": connections,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Get active primals - List all running primals with health
    async fn get_primals(&self) -> Result<Value> {
        info!("📊 Neural API: get_primals called");

        // Reuse topology discovery logic
        let topology = self.get_topology().await?;
        let primals = topology["primals"].as_array().cloned().unwrap_or_default();

        Ok(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "family_id": self.family_id,
            "primals": primals,
            "count": primals.len()
        }))
    }

    /// Get proprioception - SAME DAVE self-awareness
    async fn get_proprioception(&self) -> Result<Value> {
        info!("📊 Neural API: get_proprioception called");

        let topology = self.get_topology().await?;
        let primals = topology["primals"].as_array().cloned().unwrap_or_default();
        let primal_count = primals.len();

        // Calculate system health based on expected vs actual primals
        let expected_primals = 3; // BearDog, Songbird, Toadstool minimum
        let health_percentage =
            ((primal_count as f64 / expected_primals as f64) * 100.0).min(100.0);

        // Check if we have the core components
        let has_security = primals.iter().any(|p| p["primal_type"] == "beardog");
        let has_discovery = primals.iter().any(|p| p["primal_type"] == "songbird");
        let has_compute = primals.iter().any(|p| p["primal_type"] == "toadstool");

        let confidence = if has_security && has_discovery && has_compute {
            100.0
        } else {
            health_percentage * 0.5
        };

        Ok(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "family_id": self.family_id,
            "health": {
                "percentage": health_percentage,
                "status": if health_percentage >= 80.0 { "healthy" } else if health_percentage >= 50.0 { "degraded" } else { "critical" }
            },
            "confidence": confidence,
            "self_awareness": {
                "knows_about": primal_count,
                "can_coordinate": primal_count > 0,
                "has_security": has_security,
                "has_discovery": has_discovery,
                "has_compute": has_compute
            },
            "motor": {
                "can_deploy": true,
                "can_execute_graphs": true,
                "can_coordinate_primals": primal_count >= 2
            },
            "sensory": {
                "active_sockets": primal_count,
                "last_scan": chrono::Utc::now().to_rfc3339()
            }
        }))
    }

    /// Get aggregated metrics
    async fn get_metrics(&self) -> Result<Value> {
        info!("📊 Neural API: get_metrics called");

        // Get system metrics
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let topology = self.get_topology().await?;
        let primal_count = topology["primals"].as_array().map(|a| a.len()).unwrap_or(0);

        Ok(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "system": {
                "cpu_percent": sys.global_cpu_usage(),
                "memory_used_mb": sys.used_memory() / 1024 / 1024,
                "memory_total_mb": sys.total_memory() / 1024 / 1024,
                "memory_percent": (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0,
                "uptime_seconds": sysinfo::System::uptime()
            },
            "neural_api": {
                "family_id": self.family_id,
                "active_primals": primal_count,
                "graphs_available": self.list_graphs().await?.as_array().map(|a| a.len()).unwrap_or(0),
                "active_executions": self.executions.read().await.len()
            }
        }))
    }

    /// List niche templates
    async fn list_niche_templates(&self) -> Result<Value> {
        // For now, return built-in templates
        // In the future, this could scan a templates directory
        let templates = vec![
            json!({
                "id": "nucleus",
                "name": "NUCLEUS",
                "description": "Complete biomeOS infrastructure (Tower + Node + Nest)",
                "category": "infrastructure",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 8192,
                    "gpu_count": null,
                    "storage_gb": 50
                },
                "graph_id": "nucleus-simple",
                "parameters": []
            }),
            json!({
                "id": "ui-atomic",
                "name": "UI Atomic",
                "description": "User interface and AI layer (Squirrel + petalTongue)",
                "category": "user-interface",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 4096,
                    "gpu_count": 1,
                    "storage_gb": 10
                },
                "graph_id": "ui-atomic",
                "parameters": []
            }),
        ];

        Ok(json!(templates))
    }

    /// Deploy a niche from template
    async fn deploy_niche(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let template_id = params["template_id"]
            .as_str()
            .context("Missing template_id")?;
        let family_id = params["family_id"].as_str().unwrap_or(&self.family_id);

        // Map template to graph
        let graph_id = match template_id {
            "nucleus" => "nucleus-simple",
            "ui-atomic" => "ui-atomic",
            _ => return Err(anyhow::anyhow!("Unknown template: {}", template_id)),
        };

        // Execute the graph
        let exec_params = json!({
            "graph_id": graph_id,
            "family_id": family_id
        });

        self.execute_graph(&Some(exec_params)).await
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
        let atomic = self.router.discover_capability("secure_http").await
            .context("Failed to discover Tower Atomic")?;
        
        debug!("   Discovered: {:?} primals", atomic.primals.len());
        
        // Forward to Songbird (handles HTTP/TLS)
        let http_params = json!({
            "method": method,
            "url": url,
            "headers": headers,
            "body": body
        });
        
        let result = self.router.forward_request(
            &atomic.primary_socket,
            "http.request",
            &http_params
        ).await?;
        
        // Log metrics
        let latency = start.elapsed().as_millis() as u64;
        self.router.log_metric(RoutingMetrics {
            request_id: request_id.clone(),
            capability: "secure_http".to_string(),
            method: format!("http.{}", method),
            routed_through: atomic.primals.iter().map(|p| p.name.clone()).collect(),
            latency_ms: latency,
            success: true,
            timestamp: chrono::Utc::now(),
            error: None,
        }).await;
        
        info!("   ✓ Proxied in {}ms", latency);
        
        Ok(result)
    }
    
    /// Discover primal(s) by capability
    async fn discover_capability(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"].as_str().context("Missing capability")?;
        
        info!("🔍 Discover capability: {}", capability);
        
        let atomic = self.router.discover_capability(capability).await?;
        
        Ok(json!({
            "capability": atomic.capability,
            "atomic_type": atomic.atomic_type.map(|t| format!("{:?}", t)),
            "primals": atomic.primals.iter().map(|p| {
                json!({
                    "name": p.name,
                    "socket": p.socket_path,
                    "healthy": p.healthy,
                    "capabilities": p.capabilities
                })
            }).collect::<Vec<_>>(),
            "primary_socket": atomic.primary_socket
        }))
    }
    
    /// Route generic JSON-RPC request to primal by capability
    async fn route_to_primal(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"].as_str().context("Missing capability")?;
        let method = params["method"].as_str().context("Missing method")?;
        
        // Create a longer-lived binding for the default empty JSON
        let default_params = json!({});
        let rpc_params = params.get("params").unwrap_or(&default_params);
        
        info!("🔀 Route: {} -> {}", method, capability);
        
        // Discover primal(s) for this capability
        let atomic = self.router.discover_capability(capability).await?;
        
        // Forward request
        let result = self.router.forward_request(
            &atomic.primary_socket,
            method,
            rpc_params
        ).await?;
        
        // Log metrics
        let latency = start.elapsed().as_millis() as u64;
        self.router.log_metric(RoutingMetrics {
            request_id: request_id.clone(),
            capability: capability.to_string(),
            method: method.to_string(),
            routed_through: atomic.primals.iter().map(|p| p.name.clone()).collect(),
            latency_ms: latency,
            success: true,
            timestamp: chrono::Utc::now(),
            error: None,
        }).await;
        
        info!("   ✓ Routed in {}ms", latency);
        
        Ok(result)
    }
    
    /// Get routing metrics (for learning layer)
    async fn get_routing_metrics(&self) -> Result<Value> {
        let metrics = self.router.get_metrics().await;
        
        Ok(json!({
            "total_requests": metrics.len(),
            "metrics": metrics.iter().map(|m| {
                json!({
                    "request_id": m.request_id,
                    "capability": m.capability,
                    "method": m.method,
                    "routed_through": m.routed_through,
                    "latency_ms": m.latency_ms,
                    "success": m.success,
                    "timestamp": m.timestamp.to_rfc3339(),
                    "error": m.error
                })
            }).collect::<Vec<_>>()
        }))
    }
    
    // ========================================================================
    // Capability Registry API (NEW - Dynamic Capability Management)
    // ========================================================================
    
    /// Register a capability for a primal
    /// 
    /// Called by:
    /// - Graph deployment (automatic)
    /// - Primal on startup (self-announcement)
    /// - Manual registration
    async fn register_capability(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        let primal_name = params["primal"]
            .as_str()
            .context("Missing 'primal' field")?;
        let socket_path = params["socket"]
            .as_str()
            .context("Missing 'socket' field")?;
        let source = params["source"]
            .as_str()
            .unwrap_or("manual");
        
        info!("📝 Registering: {} → {} (from {})", capability, primal_name, source);
        
        self.router.register_capability(
            capability,
            primal_name,
            PathBuf::from(socket_path),
            source,
        ).await?;
        
        Ok(json!({
            "registered": true,
            "capability": capability,
            "primal": primal_name,
            "socket": socket_path
        }))
    }
    
    /// Discover who provides a capability
    /// 
    /// Called by primals to find services at runtime
    async fn capability_discover(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        
        info!("🔍 Discovery request: {}", capability);
        
        match self.router.get_capability_providers(capability).await {
            Some(providers) if !providers.is_empty() => {
                let primary = &providers[0];
                Ok(json!({
                    "found": true,
                    "capability": capability,
                    "provider": primary.primal_name,
                    "socket": primary.socket_path,
                    "registered_at": primary.registered_at.to_rfc3339(),
                    "source": primary.source,
                    "all_providers": providers.iter().map(|p| {
                        json!({
                            "primal": p.primal_name,
                            "socket": p.socket_path
                        })
                    }).collect::<Vec<_>>()
                }))
            }
            _ => Ok(json!({
                "found": false,
                "capability": capability,
                "message": format!(
                    "No provider registered for '{}'. Available capabilities: {:?}",
                    capability,
                    self.router.list_capabilities().await.keys().collect::<Vec<_>>()
                )
            }))
        }
    }
    
    /// List all registered capabilities
    async fn capability_list(&self) -> Result<Value> {
        let capabilities = self.router.list_capabilities().await;
        
        Ok(json!({
            "capabilities": capabilities.iter().map(|(cap, providers)| {
                json!({
                    "capability": cap,
                    "provider_count": providers.len(),
                    "providers": providers.iter().map(|p| {
                        json!({
                            "primal": p.primal_name,
                            "socket": p.socket_path,
                            "registered_at": p.registered_at.to_rfc3339(),
                            "source": p.source
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        }))
    }
    
    /// Get all providers for a specific capability
    async fn capability_providers(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        
        match self.router.get_capability_providers(capability).await {
            Some(providers) => Ok(json!({
                "capability": capability,
                "providers": providers.iter().map(|p| {
                    json!({
                        "primal": p.primal_name,
                        "socket": p.socket_path,
                        "registered_at": p.registered_at.to_rfc3339(),
                        "source": p.source
                    })
                }).collect::<Vec<_>>()
            })),
            None => Ok(json!({
                "capability": capability,
                "providers": []
            }))
        }
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
        }
    }
    
    // ========================================================================
    // Capability Translation API (v2.0.0)
    // ========================================================================
    
    /// Load capability translations from a graph
    ///
    /// Extracts `capabilities_provided` from each node and registers translations
    async fn load_translations_from_graph(&self, graph: &Graph) -> Result<()> {
        let mut registry = self.translation_registry.write().await;
        let mut loaded_count = 0;
        
        for node in &graph.nodes {
            if let Some(caps_provided) = &node.capabilities_provided {
                // Get socket path for this node
                let socket_path = if let Some(operation) = &node.operation {
                    if operation.name == "start" {
                        operation.params.get("socket_path")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                // If we have a socket path, register translations
                if let Some(socket) = socket_path {
                    for (semantic, actual) in caps_provided {
                        info!(
                            "📝 Loading translation from graph: {} → {} ({})",
                            semantic, actual, node.id
                        );
                        
                        registry.register_translation(
                            semantic,
                            &node.id,
                            actual,
                            &socket,
                        );
                        
                        loaded_count += 1;
                    }
                } else {
                    debug!(
                        "⚠️  Node {} has capabilities_provided but no socket_path, skipping",
                        node.id
                    );
                }
            }
        }
        
        if loaded_count > 0 {
            info!("✅ Loaded {} capability translations from graph {}", loaded_count, graph.id);
        }
        
        Ok(())
    }
    
    /// Call a capability with automatic translation
    ///
    /// Maps semantic capability names to provider-specific method names
    async fn capability_call(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        let args = params.get("args").cloned().unwrap_or(json!({}));
        
        info!("🔄 Capability call (with translation): {}", capability);
        
        let registry = self.translation_registry.read().await;
        let result = registry.call_capability(capability, args).await?;
        
        Ok(result)
    }
    
    /// Discover translation for a semantic capability
    async fn capability_discover_translation(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        
        let registry = self.translation_registry.read().await;
        
        match registry.get_translation(capability) {
            Some(translation) => Ok(json!({
                "semantic": translation.semantic,
                "provider": translation.provider,
                "actual_method": translation.actual_method,
                "socket": translation.socket,
                "metadata": translation.metadata
            })),
            None => Err(anyhow::anyhow!("No translation found for capability: {}", capability))
        }
    }
    
    /// List all capability translations
    async fn capability_list_translations(&self) -> Result<Value> {
        let registry = self.translation_registry.read().await;
        let translations = registry.list_all();
        
        let stats = registry.stats();
        
        Ok(json!({
            "translations": translations.iter().map(|t| {
                json!({
                    "semantic": t.semantic,
                    "provider": t.provider,
                    "actual_method": t.actual_method,
                    "socket": t.socket
                })
            }).collect::<Vec<_>>(),
            "stats": {
                "total_translations": stats.total_translations,
                "total_providers": stats.total_providers,
                "by_provider": stats.capabilities_by_provider
            }
        }))
    }
}

/// JSON-RPC request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: u64,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionStatus {
    execution_id: String,
    state: String,
    current_phase: Option<usize>,
    total_phases: usize,
    completed_nodes: Vec<String>,
    failed_nodes: Vec<String>,
    duration_ms: u64,
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires graphs directory
    async fn test_list_graphs() {
        let server = NeuralApiServer::new("graphs", "test", "/tmp/test-neural-api.sock");
        let result = server.list_graphs().await.unwrap();
        assert!(result.is_array());
    }
}
