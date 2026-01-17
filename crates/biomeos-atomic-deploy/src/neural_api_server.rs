//! Neural API JSON-RPC Server
//!
//! Exposes the Neural API graph orchestration engine via JSON-RPC 2.0 over Unix socket.
//! This enables Squirrel and petalTongue to discover, execute, and monitor graph deployments.

use crate::neural_executor::{ExecutionContext, GraphExecutor};
use crate::neural_graph::Graph;
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
pub struct NeuralApiServer {
    /// Path to graphs directory
    graphs_dir: PathBuf,

    /// Active executions (execution_id -> status)
    executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,

    /// Family ID for this server
    family_id: String,

    /// Socket path
    socket_path: PathBuf,
}

impl NeuralApiServer {
    /// Create a new Neural API server
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        socket_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            graphs_dir: graphs_dir.into(),
            executions: Arc::new(RwLock::new(HashMap::new())),
            family_id: family_id.into(),
            socket_path: socket_path.into(),
        }
    }

    /// Start the Neural API server
    pub async fn serve(&self) -> Result<()> {
        // Remove old socket if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).context("Failed to remove old socket")?;
        }

        // Create Unix socket listener
        let listener =
            UnixListener::bind(&self.socket_path).context("Failed to bind Unix socket")?;

        info!(
            "🧠 Neural API server listening on: {}",
            self.socket_path.display()
        );
        info!("   Graphs directory: {}", self.graphs_dir.display());
        info!("   Family ID: {}", self.family_id);

        // Accept connections
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
        let graph = Graph::from_toml_file(&graph_path).context("Failed to load graph")?;

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

        // Execute graph in background
        tokio::spawn(async move {
            let mut env = HashMap::new();
            env.insert("FAMILY_ID".to_string(), family_id_owned);
            env.insert("UID".to_string(), users::get_current_uid().to_string());
            env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());
            env.insert(
                "JWT_SECRET".to_string(),
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".to_string()),
            );

            let mut executor = GraphExecutor::new(graph, env);
            let start = std::time::Instant::now();

            match executor.execute().await {
                Ok(report) => {
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

    /// Clone for spawning tasks
    fn clone(&self) -> Self {
        Self {
            graphs_dir: self.graphs_dir.clone(),
            executions: self.executions.clone(),
            family_id: self.family_id.clone(),
            socket_path: self.socket_path.clone(),
        }
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
