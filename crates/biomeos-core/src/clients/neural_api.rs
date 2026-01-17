//! Neural API Client
//!
//! Client for interacting with biomeOS Neural API graph orchestration engine.
//! Allows primals (like Squirrel and petalTongue) to:
//! - List available deployment graphs
//! - Execute graphs (deploy niches)
//! - Monitor execution status
//! - Query system topology
//!
//! This enables self-hosted evolution: users can bootstrap niches via UI,
//! AI can generate graphs, and the system orchestrates deployment.

use crate::clients::transport::TransportClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Neural API client for graph orchestration
#[derive(Debug, Clone)]
pub struct NeuralApiClient {
    transport: TransportClient,
    family_id: String,
}

impl NeuralApiClient {
    /// Create a new Neural API client
    pub async fn new(family_id: impl Into<String>) -> Result<Self> {
        let family_id = family_id.into();
        let transport = TransportClient::discover_with_preference(
            "biomeos-neural-api",
            &family_id,
            crate::clients::transport::TransportPreference::UnixSocket,
        )
        .await
        .context("Failed to discover Neural API")?;

        Ok(Self {
            transport,
            family_id,
        })
    }

    /// Create client from a specific socket path
    pub fn from_socket(
        socket_path: impl Into<PathBuf>,
        family_id: impl Into<String>,
    ) -> Result<Self> {
        let socket_path_buf = socket_path.into();
        let family_id_str = family_id.into();

        // Create Unix socket transport directly
        let transport = TransportClient::discover_with_preference(
            "biomeos-neural-api",
            &family_id_str,
            crate::clients::transport::TransportPreference::UnixSocket,
        );

        // Note: For now, we use discover_with_preference. In the future, we can add
        // a direct from_socket constructor to TransportClient if needed.
        Ok(Self {
            transport: futures::executor::block_on(transport)?,
            family_id: family_id_str,
        })
    }

    /// List available Neural API graphs
    pub async fn list_graphs(&self) -> Result<Vec<GraphMetadata>> {
        let response = self
            .transport
            .call("neural_api.list_graphs", None)
            .await
            .context("Failed to list graphs")?;

        serde_json::from_value(response).context("Failed to parse graph list")
    }

    /// Get details of a specific graph
    pub async fn get_graph(&self, graph_id: &str) -> Result<Graph> {
        let response = self
            .transport
            .call(
                "neural_api.get_graph",
                Some(serde_json::json!({
                    "graph_id": graph_id
                })),
            )
            .await
            .context("Failed to get graph")?;

        serde_json::from_value(response).context("Failed to parse graph")
    }

    /// Save a new graph (user-created or AI-generated)
    pub async fn save_graph(&self, graph: &Graph) -> Result<String> {
        let response = self
            .transport
            .call(
                "neural_api.save_graph",
                Some(serde_json::to_value(graph).context("Failed to serialize graph")?),
            )
            .await
            .context("Failed to save graph")?;

        serde_json::from_value(response).context("Failed to parse save response")
    }

    /// Execute a graph (deploy a niche)
    pub async fn execute_graph(&self, graph_id: &str) -> Result<ExecutionHandle> {
        let response = self
            .transport
            .call(
                "neural_api.execute_graph",
                Some(serde_json::json!({
                    "graph_id": graph_id,
                    "family_id": &self.family_id
                })),
            )
            .await
            .context("Failed to execute graph")?;

        serde_json::from_value(response).context("Failed to parse execution handle")
    }

    /// Get execution status for monitoring
    pub async fn get_execution_status(&self, handle: &ExecutionHandle) -> Result<ExecutionStatus> {
        let response = self
            .transport
            .call(
                "neural_api.get_execution_status",
                Some(serde_json::to_value(handle).context("Failed to serialize handle")?),
            )
            .await
            .context("Failed to get execution status")?;

        serde_json::from_value(response).context("Failed to parse execution status")
    }

    /// Cancel a running execution
    pub async fn cancel_execution(&self, handle: &ExecutionHandle) -> Result<()> {
        self.transport
            .call(
                "neural_api.cancel_execution",
                Some(serde_json::to_value(handle).context("Failed to serialize handle")?),
            )
            .await
            .context("Failed to cancel execution")?;

        Ok(())
    }

    /// Get current system topology (for visualization)
    pub async fn get_topology(&self) -> Result<Topology> {
        let response = self
            .transport
            .call("neural_api.get_topology", None)
            .await
            .context("Failed to get topology")?;

        serde_json::from_value(response).context("Failed to parse topology")
    }

    /// List available niche templates
    pub async fn list_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        let response = self
            .transport
            .call("neural_api.list_niche_templates", None)
            .await
            .context("Failed to list niche templates")?;

        serde_json::from_value(response).context("Failed to parse niche templates")
    }

    /// Deploy a niche from a template
    pub async fn deploy_niche(&self, template_id: &str, config: Value) -> Result<ExecutionHandle> {
        let response = self
            .transport
            .call(
                "neural_api.deploy_niche",
                Some(serde_json::json!({
                    "template_id": template_id,
                    "config": config,
                    "family_id": &self.family_id
                })),
            )
            .await
            .context("Failed to deploy niche")?;

        serde_json::from_value(response).context("Failed to parse execution handle")
    }
}

/// Graph metadata (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub id: String,
    pub version: String,
    pub description: String,
    pub node_count: usize,
    pub estimated_time_ms: Option<u64>,
    pub tags: Vec<String>,
}

/// Complete graph definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub id: String,
    pub version: String,
    pub description: String,
    pub nodes: Vec<GraphNode>,
    pub config: GraphConfig,
}

/// Graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub dependencies: Vec<String>,
    pub config: serde_json::Map<String, Value>,
}

/// Graph configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub execution_mode: String,
    pub parallel_phases: bool,
    pub max_parallelism: usize,
    pub timeout_total_ms: u64,
    pub rollback_on_failure: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            execution_mode: "deterministic".to_string(),
            parallel_phases: true,
            max_parallelism: 4,
            timeout_total_ms: 300_000,
            rollback_on_failure: true,
        }
    }
}

/// Execution handle (for tracking deployment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHandle {
    pub execution_id: String,
    pub graph_id: String,
    pub started_at: String,
}

/// Execution status (for monitoring)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatus {
    pub execution_id: String,
    pub state: ExecutionState,
    pub current_phase: Option<usize>,
    pub total_phases: usize,
    pub completed_nodes: Vec<String>,
    pub failed_nodes: Vec<String>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// System topology (for visualization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub primals: Vec<PrimalNode>,
    pub connections: Vec<Connection>,
    pub timestamp: String,
}

/// Primal node in topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalNode {
    pub id: String,
    pub primal_type: String,
    pub socket_path: String,
    pub health: HealthStatus,
    pub resource_usage: Option<ResourceUsage>,
    pub capabilities: Vec<String>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_io_mbps: f64,
}

/// Connection between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub connection_type: String,
    pub latency_ms: Option<f64>,
}

/// Niche template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub required_resources: RequiredResources,
    pub graph_id: String,
    pub parameters: Vec<TemplateParameter>,
}

/// Required resources for a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredResources {
    pub cpu_cores: Option<usize>,
    pub memory_mb: Option<u64>,
    pub gpu_count: Option<usize>,
    pub storage_gb: Option<u64>,
}

/// Template parameter (for user configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub parameter_type: String,
    pub default_value: Option<Value>,
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Neural API server running
    async fn test_list_graphs() {
        let client = NeuralApiClient::new("test").await.unwrap();
        let graphs = client.list_graphs().await.unwrap();
        assert!(!graphs.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires Neural API server running
    async fn test_get_topology() {
        let client = NeuralApiClient::new("test").await.unwrap();
        let topology = client.get_topology().await.unwrap();
        assert!(!topology.primals.is_empty());
    }
}
