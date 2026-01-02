// Topology handler
// Returns network topology (nodes + edges)

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{ApiError, AppState};

/// Topology node (primal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub health: String,
    
    // Trust information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<u8>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,
    
    pub capabilities: Vec<String>,
}

/// Topology edge (connection between primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String, // "federation", "api_call", "trust_relationship"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>, // "http", "tarpc", "grpc"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>, // "limited", "elevated", "highest"
}

/// Topology response
#[derive(Debug, Serialize)]
pub struct TopologyResponse {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
    pub mode: String,
}

/// GET /api/v1/topology
pub async fn get_topology(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TopologyResponse>, ApiError> {
    info!("🌐 Building topology...");

    if state.mock_mode {
        info!("   Using mock topology (BIOMEOS_MOCK_MODE=true)");
        let (nodes, edges) = get_mock_topology();
        return Ok(Json(TopologyResponse {
            nodes,
            edges,
            mode: "mock".to_string(),
        }));
    }

    // Live mode: Build topology from real primals
    // TODO: Use Universal Primal Client to:
    // 1. Get list of primals from discovery
    // 2. Query Songbird for connection graph
    // 3. Query BearDog for trust relationships
    // 4. Aggregate into nodes + edges
    
    info!("   Live mode not yet implemented, falling back to mock");
    let (nodes, edges) = get_mock_topology();
    Ok(Json(TopologyResponse {
        nodes,
        edges,
        mode: "mock_fallback".to_string(),
    }))
}

/// Generate mock topology for testing
fn get_mock_topology() -> (Vec<TopologyNode>, Vec<TopologyEdge>) {
    let nodes = vec![
        TopologyNode {
            id: "beardog-local".to_string(),
            name: "BearDog".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            trust_level: Some(3),
            family_id: Some("iidn".to_string()),
            capabilities: vec!["security".to_string(), "trust_evaluation".to_string()],
        },
        TopologyNode {
            id: "songbird-local".to_string(),
            name: "Songbird".to_string(),
            primal_type: "orchestration".to_string(),
            health: "healthy".to_string(),
            trust_level: Some(3),
            family_id: Some("iidn".to_string()),
            capabilities: vec!["orchestration".to_string(), "discovery".to_string()],
        },
        TopologyNode {
            id: "tower2-remote".to_string(),
            name: "tower2".to_string(),
            primal_type: "tower".to_string(),
            health: "healthy".to_string(),
            trust_level: Some(1),
            family_id: Some("iidn".to_string()),
            capabilities: vec!["orchestration".to_string()],
        },
        TopologyNode {
            id: "nestgate-local".to_string(),
            name: "NestGate".to_string(),
            primal_type: "storage".to_string(),
            health: "healthy".to_string(),
            trust_level: Some(2),
            family_id: Some("iidn".to_string()),
            capabilities: vec!["storage".to_string()],
        },
    ];

    let edges = vec![
        TopologyEdge {
            from: "songbird-local".to_string(),
            to: "beardog-local".to_string(),
            edge_type: "api_call".to_string(),
            protocol: Some("http".to_string()),
            trust: Some("highest".to_string()),
        },
        TopologyEdge {
            from: "songbird-local".to_string(),
            to: "tower2-remote".to_string(),
            edge_type: "federation".to_string(),
            protocol: Some("tarpc".to_string()),
            trust: Some("limited".to_string()),
        },
        TopologyEdge {
            from: "songbird-local".to_string(),
            to: "nestgate-local".to_string(),
            edge_type: "federation".to_string(),
            protocol: Some("http".to_string()),
            trust: Some("elevated".to_string()),
        },
        TopologyEdge {
            from: "beardog-local".to_string(),
            to: "tower2-remote".to_string(),
            edge_type: "trust_relationship".to_string(),
            protocol: None,
            trust: Some("limited".to_string()),
        },
    ];

    (nodes, edges)
}

