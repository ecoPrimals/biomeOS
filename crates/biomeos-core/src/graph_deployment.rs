// =============================================================================
// Graph-Based Deployment - Neural API Integration
// =============================================================================
//
// Integrates GraphExecutor with biomeOS for adaptive primal orchestration.
//
// Deep Debt Principles:
// - Runtime primal discovery (no hardcoding)
// - Capability-based (discovers by capability)
// - Modern async Rust
//
// =============================================================================

use anyhow::{Context, Result};
use async_trait::async_trait;
use biomeos_graph::{GraphExecutor, GraphParser, GraphResult, GraphValidator, Operation, ExecutionContext};
use biomeos_manifest::niche::NicheManifest;
use std::collections::HashMap;
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
    
    /// Discover primals (placeholder - would scan for Unix sockets, etc.)
    pub async fn discover_primals(&self) -> Result<Vec<(String, Vec<String>)>> {
        // TODO: Real discovery via:
        // - Unix socket scanning (/tmp/songbird-*.sock, /tmp/beardog-*.sock)
        // - UDP multicast announcements
        // - Config file reading
        
        let primals = self.primals.read().await;
        let discovered: Vec<(String, Vec<String>)> = primals
            .values()
            .map(|info| (info.id.clone(), info.capabilities.clone()))
            .collect();
        
        debug!("Discovered {} primals", discovered.len());
        Ok(discovered)
    }
    
    /// Execute an operation on a primal
    pub async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        info!(
            primal_id = %primal_id,
            operation = %operation.name,
            "Executing operation"
        );
        
        // TODO: Real execution via:
        // - Unix socket JSON-RPC
        // - HTTP API calls
        // - Process spawning (for start operations)
        
        let primals = self.primals.read().await;
        if let Some(info) = primals.get(primal_id) {
            debug!("Found primal: {} with endpoint: {:?}", info.id, info.endpoint);
            
            // For now, return success
            // In real implementation, would call actual primal API
            Ok(serde_json::json!({
                "primal_id": primal_id,
                "operation": operation.name,
                "status": "success",
                "mock": true
            }))
        } else {
            warn!("Primal not found: {}", primal_id);
            anyhow::bail!("Primal not found: {}", primal_id)
        }
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
    /// Create a new coordinator
    pub fn new() -> Self {
        Self {
            registry: PrimalRegistry::new(),
        }
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

