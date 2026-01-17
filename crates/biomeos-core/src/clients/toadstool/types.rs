// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! ToadStool type definitions
//!
//! This module contains all type definitions for ToadStool client operations.
//! Types are organized by domain: workload management, resource monitoring,
//! and collaborative intelligence.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════
// Resource Monitoring Types
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// Workload Management Types
// ═══════════════════════════════════════════════════════════════════════

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
    pub fn new(
        id: impl Into<String>,
        primal: impl Into<String>,
        capabilities: Vec<String>,
    ) -> Self {
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
    pub fn data_flow(
        from: impl Into<String>,
        to: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type: EdgeType::DataFlow {
                data_flow: data.into(),
            },
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

    #[test]
    fn test_graph_node_creation() {
        let node = GraphNode::new("task1", "toadstool", vec!["compute".to_string()]);
        assert_eq!(node.id, "task1");
        assert_eq!(node.primal, "toadstool");
        assert!(node.resources.is_none());
    }

    #[test]
    fn test_graph_edge_creation() {
        let edge = GraphEdge::data_flow("task1", "task2", "dataset_a");
        assert_eq!(edge.from, "task1");
        assert_eq!(edge.to, "task2");

        let control_edge = GraphEdge::control("task1", "task2");
        assert!(matches!(control_edge.edge_type, EdgeType::Control));
    }
}
