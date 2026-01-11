//! Graph-based deployment system for biomeOS
//!
//! Provides a capability-based, graph-driven primal orchestration system.
//! Deep Debt Evolution: Uses CapabilityTaxonomy + SystemPaths!

use anyhow::{Context, Result};
use async_trait::async_trait;
use biomeos_graph::{GraphExecutor, GraphParser, GraphResult, GraphValidator, Operation, ExecutionContext};
use biomeos_manifest::niche::NicheManifest;
use biomeos_types::SystemPaths; // Deep Debt Evolution
use std::collections::HashMap;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Registry of discovered primals (capability-based!)
#[derive(Clone)]
pub struct PrimalRegistry {
    /// Map of primal_id → (capabilities, endpoint)
    primals: Arc<RwLock<HashMap<String, PrimalInfo>>>,
}

#[derive(Debug, Clone)]
struct PrimalInfo {
    id: String,
    capabilities: Vec<String>,
    endpoint: Option<String>,
}

impl PrimalRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a primal with its capabilities
    pub async fn register(&self, id: String, capabilities: Vec<String>, endpoint: Option<String>) {
        let mut primals = self.primals.write().await;
        primals.insert(id.clone(), PrimalInfo {
            id,
            capabilities,
            endpoint,
        });
    }
    
    /// Discover primals via Unix socket scanning (XDG-compliant!)
    pub async fn discover_primals(&self) -> Result<Vec<(String, Vec<String>)>> {
        use tokio::fs;
        
        let mut discovered = Vec::new();
        
        // Get XDG-compliant system paths
        let paths = SystemPaths::new()
            .context("Failed to initialize SystemPaths")?;
        
        // Scan runtime directory (no hardcoded /tmp!)
        let runtime_dir = paths.runtime_dir();
        
        info!(
            runtime_dir = %runtime_dir.display(),
            "Scanning for primal sockets in XDG runtime directory"
        );
        
        if let Ok(mut entries) = fs::read_dir(runtime_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                
                if let Ok(metadata) = fs::metadata(&path).await {
                    if metadata.file_type().is_socket() {
                        // Extract primal ID from socket path
                        if let Some(filename) = path.file_name() {
                            if let Some(name) = filename.to_str() {
                                if name.ends_with(".sock") {
                                    let primal_id = name.trim_end_matches(".sock");
                                    
                                    // Query capabilities via Unix socket
                                    match self.query_capabilities_via_socket(&path).await {
                                        Ok(caps) => {
                                            info!(
                                                primal_id = %primal_id,
                                                socket = %path.display(),
                                                capabilities = ?caps,
                                                "Discovered primal via Unix socket (XDG path)"
                                            );
                                            
                                            // Register in our cache
                                            self.register(
                                                primal_id.to_string(),
                                                caps.clone(),
                                                Some(path.to_string_lossy().to_string()),
                                            ).await;
                                            
                                            discovered.push((primal_id.to_string(), caps));
                                        }
                                        Err(e) => {
                                            warn!(
                                                socket = %path.display(),
                                                error = %e,
                                                "Failed to query capabilities from socket"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 2. Check already registered primals
        let primals = self.primals.read().await;
        for info in primals.values() {
            if !discovered.iter().any(|(id, _)| id == &info.id) {
                discovered.push((info.id.clone(), info.capabilities.clone()));
            }
        }
        
        info!("Discovery complete: {} primals found", discovered.len());
        Ok(discovered)
    }
    
    /// Query primal capabilities via Unix socket
    async fn query_capabilities_via_socket(&self, socket_path: &std::path::Path) -> Result<Vec<String>> {
        use tokio::net::UnixStream;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        // Connect to Unix socket
        let mut stream = UnixStream::connect(socket_path).await
            .context("Failed to connect to Unix socket")?;
        
        // Send JSON-RPC request for capabilities
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "get_capabilities",
            "params": {},
            "id": 1
        });
        
        let request_str = serde_json::to_string(&request)?;
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        
        // Read response
        let mut buffer = vec![0u8; 4096];
        let n = stream.read(&mut buffer).await?;
        let response_str = String::from_utf8_lossy(&buffer[..n]);
        
        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_str(&response_str)
            .context("Failed to parse JSON-RPC response")?;
        
        if let Some(result) = response.get("result") {
            if let Some(caps) = result.get("capabilities").and_then(|c| c.as_array()) {
                let capabilities: Vec<String> = caps
                    .iter()
                    .filter_map(|c| c.as_str().map(String::from))
                    .collect();
                return Ok(capabilities);
            }
        }
        
        // TRUE PRIMAL: Cannot infer capabilities from name
        // If primal doesn't respond to capability query, it's unavailable
        warn!(
            socket = %socket_path.display(),
            "Primal did not respond to capability query - may be offline or incompatible"
        );
        
        Ok(vec![])
    }
    
    /// Execute an operation on a primal via Unix socket JSON-RPC
    pub async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use tokio::net::UnixStream;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        info!(
            primal_id = %primal_id,
            operation = %operation.name,
            "Executing operation on primal"
        );
        
        let primals = self.primals.read().await;
        let info = primals.get(primal_id)
            .ok_or_else(|| anyhow::anyhow!("Primal not found: {}", primal_id))?;
        
        let socket_path = info.endpoint.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Primal {} has no endpoint", primal_id))?;
        
        debug!(
            primal_id = %primal_id,
            socket = %socket_path,
            operation = %operation.name,
            "Connecting to primal Unix socket"
        );
        
        // Special handling for 'start' operations - spawn process
        if operation.name == "start" {
            return self.start_primal(primal_id, operation, context).await;
        }
        
        // Connect to Unix socket
        let mut stream = UnixStream::connect(socket_path).await
            .with_context(|| format!("Failed to connect to primal {} at {}", primal_id, socket_path))?;
        
        // Build JSON-RPC request
        let mut params = operation.params.clone();
        
        // Add context data if needed
        if let Some(ctx_data) = context.get_output("previous_node") {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.insert("context".to_string(), ctx_data.clone());
            }
        }
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": operation.name,
            "params": params,
            "id": uuid::Uuid::new_v4().to_string()
        });
        
        debug!(
            primal_id = %primal_id,
            request = %serde_json::to_string(&request)?,
            "Sending JSON-RPC request"
        );
        
        // Send request
        let request_str = serde_json::to_string(&request)?;
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        
        // Read response with timeout
        let mut buffer = vec![0u8; 8192];
        let n = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            stream.read(&mut buffer)
        ).await
            .context("Timeout waiting for primal response")?
            .context("Failed to read from primal socket")?;
        
        if n == 0 {
            anyhow::bail!("Primal closed connection without response");
        }
        
        let response_str = String::from_utf8_lossy(&buffer[..n]);
        debug!(
            primal_id = %primal_id,
            response = %response_str,
            "Received JSON-RPC response"
        );
        
        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_str(&response_str)
            .context("Failed to parse JSON-RPC response")?;
        
        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            let error_msg = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            anyhow::bail!("Primal returned error: {}", error_msg);
        }
        
        // Extract result
        let result = response.get("result")
            .ok_or_else(|| anyhow::anyhow!("No result in JSON-RPC response"))?
            .clone();
        
        info!(
            primal_id = %primal_id,
            operation = %operation.name,
            "Operation completed successfully"
        );
        
        Ok(result)
    }
    
    /// Start a primal process
    async fn start_primal(
        &self,
        primal_id: &str,
        operation: &Operation,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use tokio::process::Command;
        
        info!(primal_id = %primal_id, "Starting primal process");
        
        // Get binary path from params or infer from primal_id
        let binary_path = operation.params.get("binary_path")
            .and_then(|p| p.as_str())
            .map(String::from)
            .unwrap_or_else(|| {
                // Infer from primal_id: songbird-tower-001 → ./primals/songbird
                let base_name = primal_id.split('-').next().unwrap_or(primal_id);
                format!("./primals/{}", base_name)
            });
        
        debug!(
            primal_id = %primal_id,
            binary = %binary_path,
            "Spawning primal process"
        );
        
        // Build command with environment variables
        let mut cmd = Command::new(&binary_path);
        
        // Set node ID from primal_id
        cmd.env("NODE_ID", primal_id);
        
        // Add any additional env vars from params
        if let Some(env_vars) = operation.params.get("env").and_then(|e| e.as_object()) {
            for (key, value) in env_vars {
                if let Some(val_str) = value.as_str() {
                    cmd.env(key, val_str);
                }
            }
        }
        
        // Spawn process in background
        let child = cmd.spawn()
            .with_context(|| format!("Failed to spawn primal: {}", binary_path))?;
        
        let pid = child.id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get PID for spawned process"))?;
        
        info!(
            primal_id = %primal_id,
            pid = pid,
            "Primal process started successfully"
        );
        
        // Wait a moment for process to initialize
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        Ok(serde_json::json!({
            "primal_id": primal_id,
            "operation": "start",
            "status": "started",
            "pid": pid,
            "binary": binary_path
        }))
    }
}

impl Default for PrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement the trait required by GraphExecutor
#[async_trait]
impl biomeos_graph::executor::PrimalOperationExecutor for PrimalRegistry {
    async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &biomeos_graph::Operation,
        context: &biomeos_graph::ExecutionContext,
    ) -> biomeos_graph::Result<serde_json::Value> {
        self.execute_operation(primal_id, operation, context)
            .await
            .map_err(|e| biomeos_graph::GraphError::ExecutionError(e.to_string()))
    }
    
    async fn discover_primals(&self) -> biomeos_graph::Result<Vec<(String, Vec<String>)>> {
        self.discover_primals()
            .await
            .map_err(|e| biomeos_graph::GraphError::ExecutionError(e.to_string()))
    }
}

/// High-level deployment coordinator using graphs
pub struct GraphDeploymentCoordinator {
    registry: PrimalRegistry,
}

impl GraphDeploymentCoordinator {
    /// Create a new coordinator with legacy discovery
    pub fn new() -> Self {
        Self {
            registry: PrimalRegistry::new(),
        }
    }
    
    /// Create with NUCLEUS-based secure discovery
    /// 
    /// **Recommended for production**: Uses 5-layer secure discovery protocol
    #[allow(dead_code)] // TODO: Re-enable after Wave 2 evolution
    pub async fn with_nucleus() -> anyhow::Result<Self> {
        // TODO: Re-enable after nucleus_executor is evolved to use CapabilityTaxonomy
        // use biomeos_graph::NucleusPrimalExecutor;
        
        info!("Initializing deployment coordinator with NUCLEUS (placeholder for Wave 2)");
        
        // Temporary: Use standard registry
        Ok(Self {
            registry: PrimalRegistry::new(),
        })
    }
    
    /// Create with an existing registry
    pub fn with_registry(registry: PrimalRegistry) -> Self {
        Self { registry }
    }
    
    /// Deploy a niche using its default graph
    pub async fn deploy_niche(&self, niche_path: &Path) -> Result<GraphResult> {
        info!(niche_path = %niche_path.display(), "Deploying niche");
        
        // 1. Parse niche manifest
        let manifest = NicheManifest::from_file(niche_path)
            .context("Failed to parse niche manifest")?;
        
        info!(niche = %manifest.niche.name, "Parsed niche manifest");
        
        // 2. Get default graph
        let graph_ref = manifest.get_default_graph()
            .ok_or_else(|| anyhow::anyhow!("Niche has no default graph"))?;
        
        info!(graph = %graph_ref.name, path = %graph_ref.path, "Using default graph");
        
        // 3. Parse graph
        let graph = GraphParser::parse_file(Path::new(&graph_ref.path))
            .context("Failed to parse graph")?;
        
        // 4. Validate graph
        GraphValidator::validate(&graph)
            .context("Graph validation failed")?;
        
        info!(
            graph_name = %graph.name,
            nodes = graph.nodes.len(),
            edges = graph.edges.len(),
            "Graph validated successfully"
        );
        
        // 5. Execute graph
        let executor = GraphExecutor::new(self.registry.clone());
        let result = executor.execute(graph).await
            .context("Graph execution failed")?;
        
        info!(
            success = result.success,
            nodes_executed = result.metrics.len(),
            "Graph execution complete"
        );
        
        Ok(result)
    }
    
    /// Deploy using a specific graph by name
    pub async fn deploy_niche_with_graph(
        &self,
        niche_path: &Path,
        graph_name: &str,
    ) -> Result<GraphResult> {
        info!(
            niche_path = %niche_path.display(),
            graph_name = %graph_name,
            "Deploying niche with specific graph"
        );
        
        // Parse niche manifest
        let manifest = NicheManifest::from_file(niche_path)
            .context("Failed to parse niche manifest")?;
        
        // Get specific graph
        let graph_ref = manifest.get_graph(graph_name)
            .ok_or_else(|| anyhow::anyhow!("Graph '{}' not found in niche", graph_name))?;
        
        info!(graph = %graph_ref.name, path = %graph_ref.path, "Using specified graph");
        
        // Parse graph
        let graph = GraphParser::parse_file(Path::new(&graph_ref.path))
            .context("Failed to parse graph")?;
        
        // Validate graph
        GraphValidator::validate(&graph)
            .context("Graph validation failed")?;
        
        // Execute graph
        let executor = GraphExecutor::new(self.registry.clone());
        let result = executor.execute(graph).await
            .context("Graph execution failed")?;
        
        Ok(result)
    }
    
    /// Get the primal registry
    pub fn registry(&self) -> &PrimalRegistry {
        &self.registry
    }
}

impl Default for GraphDeploymentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_registry_registration() {
        let registry = PrimalRegistry::new();
        
        registry.register(
            "songbird-1".to_string(),
            vec!["discovery".to_string(), "tunneling".to_string()],
            Some("/tmp/songbird.sock".to_string()),
        ).await;
        
        let discovered = registry.discover_primals().await.unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].0, "songbird-1");
    }
    
    #[tokio::test]
    async fn test_deploy_niche_no_default_graph() {
        let coordinator = GraphDeploymentCoordinator::new();
        
        // This should fail - compute-node.toml has no default graph
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap();
        let niche_path = workspace_root.join("niches/compute-node.toml");
        
        let result = coordinator.deploy_niche(&niche_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no default graph"));
    }
}

