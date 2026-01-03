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

    if state.is_mock_mode() {
        info!("   Using mock topology (BIOMEOS_MOCK_MODE=true)");
        let (nodes, edges) = get_mock_topology();
        return Ok(Json(TopologyResponse {
            nodes,
            edges,
            mode: "mock".to_string(),
        }));
    }

    // Live mode: Build topology from discovered primals
    info!("   Live mode: Building topology from discovered primals");
    
    match build_live_topology(state.discovery()).await {
        Ok((nodes, edges)) => {
            info!("   Built topology with {} nodes, {} edges", nodes.len(), edges.len());
            Ok(Json(TopologyResponse {
                nodes,
                edges,
                mode: "live".to_string(),
            }))
        }
        Err(e) => {
            tracing::warn!("   Failed to build live topology: {}, using mock fallback", e);
            let (nodes, edges) = get_mock_topology();
            Ok(Json(TopologyResponse {
                nodes,
                edges,
                mode: "mock_fallback".to_string(),
            }))
        }
    }
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

/// Build live topology from discovered primals
async fn build_live_topology(
    discovery: &dyn biomeos_core::PrimalDiscovery,
) -> Result<(Vec<TopologyNode>, Vec<TopologyEdge>), Box<dyn std::error::Error + Send + Sync>> {
    // Discover all primals
    let primals = discovery.discover_all().await?;
    
    info!("   Discovered {} primals for topology", primals.len());
    
    // Build nodes from discovered primals
    let nodes: Vec<TopologyNode> = primals
        .iter()
        .map(|primal| {
            let health = match primal.health {
                biomeos_core::HealthStatus::Healthy => "healthy",
                biomeos_core::HealthStatus::Degraded => "degraded",
                biomeos_core::HealthStatus::Unhealthy => "unhealthy",
                biomeos_core::HealthStatus::Unknown => "unknown",
            };
            
            let primal_type = match primal.primal_type {
                biomeos_core::PrimalType::Security => "security",
                biomeos_core::PrimalType::Orchestration => "orchestration",
                biomeos_core::PrimalType::Storage => "storage",
                biomeos_core::PrimalType::Compute => "compute",
                biomeos_core::PrimalType::Ai => "ai",
                biomeos_core::PrimalType::Tower => "tower",
                biomeos_core::PrimalType::Visualization => "visualization",
                biomeos_core::PrimalType::Custom => "custom",
            };
            
            TopologyNode {
                id: primal.id.as_str().to_string(),
                name: primal.name.clone(),
                primal_type: primal_type.to_string(),
                health: health.to_string(),
                trust_level: if primal.family_id.is_some() { Some(3) } else { Some(1) },
                family_id: primal.family_id.as_ref().map(|f| f.as_str().to_string()),
                capabilities: primal
                    .capabilities
                    .iter()
                    .map(|c| c.as_str().to_string())
                    .collect(),
            }
        })
        .collect();
    
    // Build edges based on relationships
    let mut edges = Vec::new();
    
    // For each orchestration primal (Songbird), create edges to other primals
    for primal in &primals {
        if matches!(primal.primal_type, biomeos_core::PrimalType::Orchestration) {
            // Orchestration connects to security for trust evaluation
            for target in &primals {
                if matches!(target.primal_type, biomeos_core::PrimalType::Security) {
                    edges.push(TopologyEdge {
                        from: primal.id.as_str().to_string(),
                        to: target.id.as_str().to_string(),
                        edge_type: "api_call".to_string(),
                        protocol: Some("http".to_string()),
                        trust: Some("highest".to_string()),
                    });
                }
            }
        }
    }
    
    // Add trust relationships between primals with same family
    for (i, primal) in primals.iter().enumerate() {
        if let Some(family) = &primal.family_id {
            for (j, target) in primals.iter().enumerate() {
                if i != j && target.family_id.as_ref() == Some(family) {
                    edges.push(TopologyEdge {
                        from: primal.id.as_str().to_string(),
                        to: target.id.as_str().to_string(),
                        edge_type: "trust_relationship".to_string(),
                        protocol: None,
                        trust: Some("highest".to_string()),
                    });
                }
            }
        }
    }
    
    Ok((nodes, edges))
}
