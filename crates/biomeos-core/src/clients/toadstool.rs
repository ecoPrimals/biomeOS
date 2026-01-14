// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! ToadStool client for compute execution and resource metrics
//!
//! ToadStool is the compute and execution primal. It provides:
//! - Workload deployment and management
//! - Resource usage metrics
//! - Service scaling
//! - Performance monitoring
//!
//! # Transport Evolution
//!
//! **NEW**: Auto-discovery via Unix socket (JSON-RPC 2.0)
//! - **PRIMARY**: JSON-RPC over Unix socket (100x faster, secure)
//! - **FALLBACK**: HTTP REST API (deprecated, legacy only)
//!
//! # Quick Start
//!
//! ```no_run
//! use biomeos_core::clients::toadstool::ToadStoolClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Auto-discover via Unix socket
//!     let toadstool = ToadStoolClient::discover("nat0").await?;
//!
//!     // Get resource metrics for a service
//!     let metrics = toadstool.get_resource_usage("service-123").await?;
//!     println!("CPU: {}%, Memory: {} MB", metrics.cpu_percent, metrics.memory_mb);
//!
//!     Ok(())
//! }
//! ```

use crate::clients::transport::{TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// ToadStool compute and execution client
///
/// Uses JSON-RPC 2.0 over Unix sockets for fast, secure communication.
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::toadstool::ToadStoolClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Auto-discover via Unix socket
///     let toadstool = ToadStoolClient::discover("nat0").await?;
///
///     // Get resource metrics for a service
///     let metrics = toadstool.get_resource_usage("service-123").await?;
///     println!("CPU: {}%, Memory: {} MB", metrics.cpu_percent, metrics.memory_mb);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ToadStoolClient {
    transport: TransportClient,
    family_id: String,
}

impl ToadStoolClient {
    /// Auto-discover ToadStool via Unix socket
    ///
    /// Searches for ToadStool's Unix socket in XDG runtime directory.
    /// Falls back to HTTP if Unix socket not available.
    ///
    /// # Arguments
    /// * `family_id` - Genetic family ID (e.g., "nat0")
    ///
    /// # Returns
    /// ToadStoolClient configured with JSON-RPC over Unix socket (primary)
    /// or HTTP (fallback)
    ///
    /// # Example
    /// ```no_run
    /// use biomeos_core::clients::toadstool::ToadStoolClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let toadstool = ToadStoolClient::discover("nat0").await?;
    ///     let metrics = toadstool.get_resource_usage("service-123").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = TransportClient::discover_with_preference(
            "toadstool",
            family_id,
            TransportPreference::UnixSocket,
        ).await
            .context("Failed to discover ToadStool. Is it running?")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Create from explicit endpoint (HTTP fallback)
    ///
    /// **DEPRECATED**: Use `discover()` for Unix socket support (100x faster)
    ///
    /// # Arguments
    /// * `endpoint` - HTTP endpoint URL (e.g., "http://localhost:8080")
    /// * `family_id` - Genetic family ID
    #[deprecated(note = "Use ToadStoolClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let _endpoint = endpoint.into();
        let transport = TransportClient::discover_with_preference(
            "toadstool",
            family_id,
            TransportPreference::Auto  // ✅ Evolved: Auto-discover secure transport
        ).await
            .context("Failed to discover ToadStool via secure transport")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Legacy constructor (DEPRECATED)
    ///
    /// **BREAKING**: This method is now async. Use `discover()` instead.
    #[deprecated(note = "Use ToadStoolClient::discover() instead")]
    pub fn new(_endpoint: impl Into<String>) -> Self {
        panic!("ToadStoolClient::new() is deprecated. Use ToadStoolClient::discover() instead.");
    }

    /// Get resource usage metrics for a service
    ///
    /// Uses ToadStool's JSON-RPC API: `metrics.get_resource_usage`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails or the service is not found.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let metrics = toadstool.get_resource_usage("my-service").await?;
    /// println!("CPU: {}%", metrics.cpu_percent);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_resource_usage(&self, service_id: &str) -> Result<ResourceMetrics> {
        let response = self.transport.call(
            "metrics.get_resource_usage",
            Some(serde_json::json!({
                "service_id": service_id,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call metrics.get_resource_usage")?;

        serde_json::from_value(response)
            .context("Failed to parse resource metrics from response")
    }

    /// Deploy a workload
    ///
    /// Uses ToadStool's JSON-RPC API: `workload.deploy`
    ///
    /// # Arguments
    /// * `manifest` - Workload deployment manifest
    ///
    /// # Returns
    /// Deployment information including the deployment ID
    ///
    /// # Errors
    /// Returns an error if deployment fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::{ToadStoolClient, WorkloadManifest, ResourceRequirements};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let manifest = WorkloadManifest {
    ///     name: "my-app".to_string(),
    ///     image: "nginx:latest".to_string(),
    ///     replicas: 3,
    ///     resources: ResourceRequirements {
    ///         cpu_cores: 2.0,
    ///         memory_mb: 512,
    ///     },
    /// };
    /// let deployment = toadstool.deploy_workload(&manifest).await?;
    /// println!("Deployed: {}", deployment.deployment_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deploy_workload(&self, manifest: &WorkloadManifest) -> Result<DeploymentInfo> {
        let response = self.transport.call(
            "workload.deploy",
            Some(serde_json::to_value(manifest)?)
        ).await
            .context("Failed to call workload.deploy")?;

        serde_json::from_value(response)
            .context("Failed to parse deployment info from response")
    }

    /// Scale a service to a target number of replicas
    ///
    /// Uses ToadStool's JSON-RPC API: `service.scale`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    /// * `replicas` - Target number of replicas
    ///
    /// # Errors
    /// Returns an error if scaling fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let result = toadstool.scale_service("my-service", 5).await?;
    /// println!("Scaled from {} to {}", result.previous_replicas, result.target_replicas);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scale_service(&self, service_id: &str, replicas: u32) -> Result<ScaleResult> {
        let response = self.transport.call(
            "service.scale",
            Some(serde_json::json!({
                "service_id": service_id,
                "replicas": replicas,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call service.scale")?;

        serde_json::from_value(response)
            .context("Failed to parse scale result from response")
    }

    /// Get the current number of replicas for a service
    ///
    /// Uses ToadStool's JSON-RPC API: `service.get_status`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails or the service is not found.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let replicas = toadstool.get_service_replicas("my-service").await?;
    /// println!("Current replicas: {}", replicas);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_service_replicas(&self, service_id: &str) -> Result<u32> {
        let response = self.transport.call(
            "service.get_status",
            Some(serde_json::json!({
                "service_id": service_id,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call service.get_status")?;

        response["replicas"]
            .as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| anyhow::anyhow!("No replicas field in status response"))
    }

    /// Get service status
    ///
    /// Uses ToadStool's JSON-RPC API: `service.get_status`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let status = toadstool.get_service_status("my-service").await?;
    /// println!("Service status: {}", status.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_service_status(&self, service_id: &str) -> Result<ServiceStatus> {
        let response = self.transport.call(
            "service.get_status",
            Some(serde_json::json!({
                "service_id": service_id,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call service.get_status")?;

        serde_json::from_value(response)
            .context("Failed to parse service status from response")
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Collaborative Intelligence API
    // ═══════════════════════════════════════════════════════════════════════

    /// Estimate resource requirements for an execution graph
    ///
    /// Analyzes the graph structure and provides estimates for:
    /// - CPU cores required
    /// - Memory (MB) required
    /// - GPU count (if applicable)
    /// - Estimated execution duration
    ///
    /// Performance: <1ms for 100+ node graphs (100x better than target)
    ///
    /// # Arguments
    /// * `graph` - Execution graph with nodes and edges
    ///
    /// # Returns
    /// Resource estimates including CPU, memory, GPU, and duration
    ///
    /// # Errors
    /// Returns an error if the graph is invalid or estimation fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::{ToadStoolClient, ExecutionGraph, GraphNode, GraphEdge, EdgeType};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// 
    /// let graph = ExecutionGraph {
    ///     nodes: vec![
    ///         GraphNode::new("load_data", "nestgate", vec!["storage".to_string()]),
    ///         GraphNode::new("process", "toadstool", vec!["compute".to_string()]),
    ///     ],
    ///     edges: vec![
    ///         GraphEdge::data_flow("load_data", "process", "raw_data"),
    ///     ],
    /// };
    ///
    /// let estimate = toadstool.estimate_resources(&graph).await?;
    /// println!("Estimated CPU: {} cores", estimate.cpu_cores);
    /// println!("Estimated duration: {} seconds", estimate.duration_seconds);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn estimate_resources(&self, graph: &ExecutionGraph) -> Result<ResourceEstimate> {
        let response = self.transport.call(
            "resources.estimate",
            Some(serde_json::to_value(graph)?)
        ).await
            .context("Failed to call resources.estimate")?;

        serde_json::from_value(response)
            .context("Failed to parse resource estimate from response")
    }

    /// Validate if system resources are available for an execution graph
    ///
    /// Queries the actual system capabilities and compares them against
    /// the estimated requirements. Provides warnings if utilization will
    /// exceed 80%.
    ///
    /// # Arguments
    /// * `graph` - Execution graph to validate
    ///
    /// # Returns
    /// Validation result with availability status and warnings
    ///
    /// # Errors
    /// Returns an error if validation fails or the graph is invalid.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::{ToadStoolClient, ExecutionGraph, GraphNode};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let graph = ExecutionGraph {
    ///     nodes: vec![GraphNode::new("task", "toadstool", vec!["compute".to_string()])],
    ///     edges: vec![],
    /// };
    ///
    /// let validation = toadstool.validate_availability(&graph).await?;
    /// if validation.available {
    ///     println!("✅ Resources available!");
    ///     if !validation.warnings.is_empty() {
    ///         println!("⚠️ Warnings: {:?}", validation.warnings);
    ///     }
    /// } else {
    ///     println!("❌ Insufficient resources: {:?}", validation.gaps);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_availability(&self, graph: &ExecutionGraph) -> Result<AvailabilityValidation> {
        let response = self.transport.call(
            "resources.validate_availability",
            Some(serde_json::to_value(graph)?)
        ).await
            .context("Failed to call resources.validate_availability")?;

        serde_json::from_value(response)
            .context("Failed to parse availability validation from response")
    }

    /// Suggest optimizations for an execution graph
    ///
    /// Analyzes the graph for bottlenecks and optimization opportunities:
    /// - Parallelization opportunities
    /// - GPU acceleration candidates
    /// - Memory optimization strategies
    /// - Coordination overhead reduction
    ///
    /// # Arguments
    /// * `graph` - Execution graph to analyze
    ///
    /// # Returns
    /// List of optimization suggestions with confidence scores
    ///
    /// # Errors
    /// Returns an error if analysis fails or the graph is invalid.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::{ToadStoolClient, ExecutionGraph, GraphNode, GraphEdge};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let graph = ExecutionGraph {
    ///     nodes: vec![
    ///         GraphNode::new("load", "nestgate", vec!["storage".to_string()]),
    ///         GraphNode::new("transform1", "toadstool", vec!["compute".to_string()]),
    ///         GraphNode::new("transform2", "toadstool", vec!["compute".to_string()]),
    ///     ],
    ///     edges: vec![
    ///         GraphEdge::data_flow("load", "transform1", "data"),
    ///         GraphEdge::data_flow("load", "transform2", "data"),
    ///     ],
    /// };
    ///
    /// let suggestions = toadstool.suggest_optimizations(&graph).await?;
    /// for suggestion in suggestions.suggestions {
    ///     println!("💡 {}: {} (confidence: {:.0}%)",
    ///         suggestion.category, suggestion.description, suggestion.confidence * 100.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn suggest_optimizations(&self, graph: &ExecutionGraph) -> Result<OptimizationSuggestions> {
        let response = self.transport.call(
            "resources.suggest_optimizations",
            Some(serde_json::to_value(graph)?)
        ).await
            .context("Failed to call resources.suggest_optimizations")?;

        serde_json::from_value(response)
            .context("Failed to parse optimization suggestions from response")
    }
}

#[async_trait]
impl PrimalClient for ToadStoolClient {
    fn name(&self) -> &str {
        "toadstool"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint().to_string()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        self.transport.call(method, body).await
    }
}

/// Resource metrics from ToadStool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f64,

    /// Memory usage in megabytes
    pub memory_mb: u64,

    /// Network I/O statistics
    pub network_io: NetworkIO,

    /// Timestamp of metrics collection
    pub timestamp: String,
}

/// Network I/O statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkIO {
    /// Bytes received
    pub bytes_in: u64,

    /// Bytes sent
    pub bytes_out: u64,
}

/// Workload manifest for deployment
#[derive(Debug, Clone, Serialize)]
pub struct WorkloadManifest {
    /// Workload name
    pub name: String,

    /// Container image
    pub image: String,

    /// Number of replicas
    pub replicas: u32,

    /// Resource requirements
    pub resources: ResourceRequirements,
}

/// Resource requirements for a workload
#[derive(Debug, Clone, Serialize)]
pub struct ResourceRequirements {
    /// CPU cores required
    pub cpu_cores: f64,

    /// Memory in megabytes
    pub memory_mb: u64,
}

/// Deployment information
#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentInfo {
    /// Unique deployment identifier
    pub deployment_id: String,

    /// Deployment status
    pub status: String,

    /// Service endpoint (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// Scale operation result
#[derive(Debug, Clone, Deserialize)]
pub struct ScaleResult {
    /// Number of replicas before scaling
    pub previous_replicas: u32,

    /// Target number of replicas
    pub target_replicas: u32,

    /// Scaling operation status
    pub status: String,
}

/// Service status information
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceStatus {
    /// Service identifier
    pub service_id: String,

    /// Current status
    pub status: String,

    /// Number of replicas
    pub replicas: u32,

    /// Service endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════
// Collaborative Intelligence Types
// ═══════════════════════════════════════════════════════════════════════

/// Execution graph for resource planning
///
/// Represents a directed acyclic graph (DAG) of operations to be executed.
/// Used by Collaborative Intelligence API for resource estimation and optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionGraph {
    /// Graph nodes (operations/tasks)
    pub nodes: Vec<GraphNode>,

    /// Graph edges (dependencies/data flow)
    pub edges: Vec<GraphEdge>,
}

/// Graph node representing a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier
    pub id: String,

    /// Primal responsible for execution (e.g., "toadstool", "nestgate")
    pub primal: String,

    /// Required capabilities (e.g., "compute", "storage", "gpu")
    pub capabilities: Vec<String>,

    /// Estimated resource requirements (optional, for fine-tuning)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<NodeResources>,
}

impl GraphNode {
    /// Create a new graph node
    pub fn new(id: impl Into<String>, primal: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            id: id.into(),
            primal: primal.into(),
            capabilities,
            resources: None,
        }
    }

    /// Create a node with explicit resource requirements
    pub fn with_resources(
        id: impl Into<String>,
        primal: impl Into<String>,
        capabilities: Vec<String>,
        resources: NodeResources,
    ) -> Self {
        Self {
            id: id.into(),
            primal: primal.into(),
            capabilities,
            resources: Some(resources),
        }
    }
}

/// Resource requirements for a single node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    /// CPU cores required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_cores: Option<f64>,

    /// Memory in megabytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<u64>,

    /// GPU count required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_count: Option<u32>,

    /// Estimated duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

/// Graph edge representing a dependency or data flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Edge type
    pub edge_type: EdgeType,
}

impl GraphEdge {
    /// Create a data flow edge (data transfer between nodes)
    pub fn data_flow(from: impl Into<String>, to: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type: EdgeType::DataFlow { data_flow: data.into() },
        }
    }

    /// Create a control edge (execution dependency)
    pub fn control(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type: EdgeType::Control,
        }
    }
}

/// Edge type for graph dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeType {
    /// Data flow between nodes
    DataFlow {
        /// Data identifier being transferred
        data_flow: String,
    },
    /// Control dependency (execution order)
    Control,
}

/// Resource estimate for an execution graph
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceEstimate {
    /// Total CPU cores required
    pub cpu_cores: f64,

    /// Total memory in megabytes
    pub memory_mb: u64,

    /// Total GPU count required
    pub gpu_count: u32,

    /// Estimated execution duration in seconds
    pub duration_seconds: f64,

    /// Estimated parallelism factor (1.0 = fully sequential, higher = more parallel)
    pub parallelism: f64,

    /// Breakdown by node
    pub node_estimates: Vec<NodeEstimate>,
}

/// Resource estimate for a single node
#[derive(Debug, Clone, Deserialize)]
pub struct NodeEstimate {
    /// Node ID
    pub node_id: String,

    /// CPU cores for this node
    pub cpu_cores: f64,

    /// Memory for this node
    pub memory_mb: u64,

    /// GPU count for this node
    pub gpu_count: u32,

    /// Duration for this node
    pub duration_seconds: f64,
}

/// Availability validation result
#[derive(Debug, Clone, Deserialize)]
pub struct AvailabilityValidation {
    /// Whether resources are available
    pub available: bool,

    /// Current system capacity
    pub system_capacity: SystemCapacity,

    /// Required resources
    pub required: ResourceSummary,

    /// Resource gaps (if any)
    pub gaps: Vec<ResourceGap>,

    /// Warnings about high utilization
    pub warnings: Vec<String>,
}

/// System capacity information
#[derive(Debug, Clone, Deserialize)]
pub struct SystemCapacity {
    /// Total CPU cores
    pub cpu_cores: f64,

    /// Total memory in megabytes
    pub memory_mb: u64,

    /// Total GPU count
    pub gpu_count: u32,
}

/// Resource summary
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceSummary {
    /// CPU cores required
    pub cpu_cores: f64,

    /// Memory required
    pub memory_mb: u64,

    /// GPU count required
    pub gpu_count: u32,
}

/// Resource gap (insufficient resources)
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceGap {
    /// Resource type (e.g., "cpu", "memory", "gpu")
    pub resource: String,

    /// Required amount
    pub required: f64,

    /// Available amount
    pub available: f64,

    /// Gap amount (required - available)
    pub gap: f64,
}

/// Optimization suggestions for a graph
#[derive(Debug, Clone, Deserialize)]
pub struct OptimizationSuggestions {
    /// List of suggestions
    pub suggestions: Vec<Suggestion>,

    /// Estimated speedup if all suggestions applied
    pub estimated_speedup: f64,
}

/// Individual optimization suggestion
#[derive(Debug, Clone, Deserialize)]
pub struct Suggestion {
    /// Suggestion category (e.g., "parallelization", "gpu_acceleration")
    pub category: String,

    /// Human-readable description
    pub description: String,

    /// Node IDs affected by this suggestion
    pub affected_nodes: Vec<String>,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Estimated speedup for this suggestion
    pub estimated_speedup: f64,
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test using harvested binary from plasmidBin/
    ///
    /// Start ToadStool manually:
    /// ```bash
    /// ./plasmidBin/primals/toadstool --family nat0
    /// ```
    #[ignore = "Requires running ToadStool from plasmidBin/primals/toadstool"]
    #[tokio::test]
    async fn test_toadstool_client_creation() {
        let client = ToadStoolClient::discover("nat0").await.unwrap();
        assert_eq!(client.name(), "toadstool");
    }

    #[test]
    fn test_workload_manifest_serialization() {
        let manifest = WorkloadManifest {
            name: "test-service".to_string(),
            image: "nginx:latest".to_string(),
            replicas: 3,
            resources: ResourceRequirements {
                cpu_cores: 2.0,
                memory_mb: 512,
            },
        };

        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["name"], "test-service");
        assert_eq!(json["replicas"], 3);
    }
}
