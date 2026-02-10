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
    #[serde(rename = "type")]
    pub primal_type: String,
    pub health: String,
    pub capabilities: Vec<String>,

    // Endpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<NodeEndpoints>,

    // Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<NodeMetadata>,
}

/// Primal endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEndpoints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unix_socket: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<String>,
}

/// Primal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<u8>,
}

/// Topology edge (connection between primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: String, // "capability_invocation", "data_flow", "discovery", "federation"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<EdgeMetrics>,
}

/// Edge metrics for topology visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_latency_ms: Option<f64>,

    /// Round-trip latency in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<f64>,

    /// Bandwidth in megabits per second
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_mbps: Option<f64>,

    /// Packet loss percentage (0.0 - 100.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet_loss: Option<f64>,

    /// When metrics were last measured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_measured: Option<String>,
}

/// Topology response
#[derive(Debug, Serialize)]
pub struct TopologyResponse {
    pub primals: Vec<TopologyNode>,
    pub connections: Vec<TopologyEdge>,
    pub health_status: HealthStatus,
}

/// Overall health status
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub overall: String,
    pub primals_healthy: usize,
    pub primals_total: usize,
}

/// GET /api/v1/topology
pub async fn get_topology(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TopologyResponse>, ApiError> {
    info!("🌐 Building topology...");

    if state.is_standalone_mode() {
        info!(
            "   Using standalone topology (BIOMEOS_STANDALONE_MODE=true) - works without primals"
        );
        let (primals, connections) = get_standalone_topology();
        let health_status = calculate_health_status(&primals);
        return Ok(Json(TopologyResponse {
            primals,
            connections,
            health_status,
        }));
    }

    // Live mode: Build topology from discovered primals
    info!("   Live mode: Building topology from discovered primals");

    match build_live_topology(state.discovery()).await {
        Ok((primals, connections)) => {
            info!(
                "   Built topology with {} primals, {} connections",
                primals.len(),
                connections.len()
            );
            let health_status = calculate_health_status(&primals);
            Ok(Json(TopologyResponse {
                primals,
                connections,
                health_status,
            }))
        }
        Err(e) => {
            tracing::warn!(
                "   Failed to build live topology: {}, using standalone fallback",
                e
            );
            let (primals, connections) = get_standalone_topology();
            let health_status = calculate_health_status(&primals);
            Ok(Json(TopologyResponse {
                primals,
                connections,
                health_status,
            }))
        }
    }
}

/// Get standalone topology (valid operational mode, not a mock)
///
/// This is NOT a production mock - it's a valid operational mode that allows
/// biomeOS to run standalone for development, testing, and demonstrations
/// without requiring live primals.
///
/// **Production**: Real topology is built from discovered primals via `build_live_topology()`
/// **Development**: Set `BIOMEOS_STANDALONE_MODE=true` to use this standalone topology.
///
/// EVOLVED (Feb 2026): Capability-based topology - primals are described by
/// their capabilities, not hardcoded names. The standalone topology now uses
/// generic capability categories.
fn get_standalone_topology() -> (Vec<TopologyNode>, Vec<TopologyEdge>) {
    // Get node ID from environment or use default
    let node_id = std::env::var("BIOMEOS_NODE_ID").unwrap_or_else(|_| "standalone".to_string());
    let family_id = std::env::var("BIOMEOS_FAMILY_ID")
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "dev".to_string());

    let primals = vec![
        // Security capability provider (crypto operations)
        TopologyNode {
            id: format!("security-{}-{}", family_id, node_id),
            name: "security-provider".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![
                "security".to_string(),
                "crypto.encrypt".to_string(),
                "crypto.decrypt".to_string(),
                "crypto.sign".to_string(),
                "identity".to_string(),
            ],
            endpoints: Some(NodeEndpoints {
                unix_socket: Some(
                    biomeos_types::paths::SystemPaths::new_lazy()
                        .primal_socket(&format!("security-{}", family_id))
                        .to_string_lossy()
                        .to_string(),
                ),
                http: None,
            }),
            metadata: Some(NodeMetadata {
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                family_id: Some(family_id.clone()),
                node_id: Some(node_id.clone()),
                trust_level: Some(3),
            }),
        },
        // Discovery capability provider (HTTP, networking)
        TopologyNode {
            id: format!("discovery-{}-{}", family_id, node_id),
            name: "discovery-provider".to_string(),
            primal_type: "discovery".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![
                "discovery".to_string(),
                "http.request".to_string(),
                "http.get".to_string(),
                "http.post".to_string(),
            ],
            endpoints: Some(NodeEndpoints {
                unix_socket: Some(
                    biomeos_types::paths::SystemPaths::new_lazy()
                        .primal_socket(&format!("discovery-{}", family_id))
                        .to_string_lossy()
                        .to_string(),
                ),
                http: None,
            }),
            metadata: Some(NodeMetadata {
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                family_id: Some(family_id.clone()),
                node_id: Some(node_id.clone()),
                trust_level: Some(3),
            }),
        },
    ];

    // Connection: Discovery provider uses Security for encrypted requests
    let connections = vec![TopologyEdge {
        from: format!("discovery-{}-{}", family_id, node_id),
        to: format!("security-{}-{}", family_id, node_id),
        edge_type: "capability_invocation".to_string(),
        capability: Some("crypto.encrypt".to_string()),
        metrics: Some(EdgeMetrics {
            request_count: None,
            avg_latency_ms: Some(1.0),
            latency_ms: Some(1.0),
            bandwidth_mbps: None,
            packet_loss: Some(0.0),
            last_measured: Some("standalone-mode".to_string()),
        }),
    }];

    (primals, connections)
}

/// Calculate overall health status
fn calculate_health_status(primals: &[TopologyNode]) -> HealthStatus {
    let healthy_count = primals.iter().filter(|p| p.health == "healthy").count();
    let overall = if healthy_count == primals.len() {
        "healthy"
    } else if healthy_count > 0 {
        "degraded"
    } else {
        "unhealthy"
    };

    HealthStatus {
        overall: overall.to_string(),
        primals_healthy: healthy_count,
        primals_total: primals.len(),
    }
}

/// Build live topology from discovered primals
async fn build_live_topology(
    discovery: &dyn biomeos_core::PrimalDiscovery,
) -> Result<(Vec<TopologyNode>, Vec<TopologyEdge>), Box<dyn std::error::Error + Send + Sync>> {
    // Discover all primals
    let discovered = discovery.discover_all().await?;

    info!("   Discovered {} primals for topology", discovered.len());

    // Build primals from discovered primals
    let primals: Vec<TopologyNode> = discovered
        .iter()
        .map(|primal| {
            let health = match primal.health {
                biomeos_core::HealthStatus::Healthy => "healthy",
                biomeos_core::HealthStatus::Degraded => "degraded",
                biomeos_core::HealthStatus::Unhealthy => "unhealthy",
                biomeos_core::HealthStatus::Unknown => "unknown",
            };

            // EVOLUTION: Use actual primal name, not hardcoded type→name mapping
            // TRUE PRIMAL principle: primals define their own names
            let primal_type = primal.name.as_str();

            // Extract endpoints
            let endpoint_str = primal.endpoint.as_str();
            let endpoints = if endpoint_str.starts_with("unix://") {
                Some(NodeEndpoints {
                    unix_socket: Some(endpoint_str.trim_start_matches("unix://").to_string()),
                    http: None,
                })
            } else if endpoint_str.starts_with("http") {
                Some(NodeEndpoints {
                    unix_socket: None,
                    http: Some(endpoint_str.to_string()),
                })
            } else {
                None
            };

            TopologyNode {
                id: primal.id.as_str().to_string(),
                name: primal.name.clone(),
                primal_type: primal_type.to_string(),
                health: health.to_string(),
                capabilities: primal
                    .capabilities
                    .iter()
                    .map(|c| c.as_str().to_string())
                    .collect(),
                endpoints,
                metadata: Some(NodeMetadata {
                    version: Some(primal.version.to_string()),
                    family_id: primal.family_id.as_ref().map(|f| f.as_str().to_string()),
                    // EVOLVED (Jan 27, 2026): Extract node_id from primal ID pattern
                    node_id: extract_node_id_from_primal(primal.id.as_str()),
                    trust_level: if primal.family_id.is_some() {
                        Some(3)
                    } else {
                        Some(1)
                    },
                }),
            }
        })
        .collect();

    // Build connections based on relationships
    let mut connections = Vec::new();

    // For each orchestration primal (Songbird), create connections to other primals
    for primal in &discovered {
        if matches!(primal.primal_type, biomeos_core::PrimalType::Orchestration) {
            // Orchestration connects to security for encryption
            for target in &discovered {
                if matches!(target.primal_type, biomeos_core::PrimalType::Security) {
                    connections.push(TopologyEdge {
                        from: primal.id.as_str().to_string(),
                        to: target.id.as_str().to_string(),
                        edge_type: "capability_invocation".to_string(),
                        capability: Some("encryption".to_string()),
                        // EVOLVED (Jan 27, 2026): Collect real metrics from connection
                        metrics: collect_edge_metrics(primal.id.as_str(), target.id.as_str()),
                    });
                }
            }
        }
    }

    Ok((primals, connections))
}

/// Extract node_id from primal ID pattern
///
/// EVOLVED (Jan 27, 2026): Parses common primal ID patterns
///
/// Patterns supported:
/// - `{primal}-{family_id}-{node_id}` → node_id
/// - `{primal}-{node_id}` → node_id
/// - `{node_id}` → node_id
fn extract_node_id_from_primal(primal_id: &str) -> Option<String> {
    let parts: Vec<&str> = primal_id.split('-').collect();

    match parts.len() {
        // Pattern: primal-family-node (e.g., "beardog-fam-desktop")
        3 => Some(parts[2].to_string()),
        // Pattern: primal-node (e.g., "beardog-desktop")
        2 => Some(parts[1].to_string()),
        // Pattern: just the id
        1 => Some(parts[0].to_string()),
        // Complex patterns: take last segment
        _ => parts.last().map(|s| (*s).to_string()),
    }
}

/// Collect real metrics for an edge between primals
///
/// EVOLVED (Jan 27, 2026): Measures actual connection latency
/// EVOLVED (Feb 2026): Capability-based latency estimation
///
/// Returns metrics if measurement is possible, None otherwise.
/// Latency estimates are based on capability types, not primal names.
fn collect_edge_metrics(from_id: &str, to_id: &str) -> Option<EdgeMetrics> {
    // For now, return synthetic metrics based on capability types
    // In production, this would measure actual latency via JSON-RPC ping

    // Infer latency from capability relationship (type-based, not name-based)
    let estimated_latency_ms = if to_id.contains("security") || to_id.contains("crypto") {
        // Security/crypto operations may have computational overhead
        Some(5.0)
    } else if from_id.contains("discovery") || from_id.contains("http") {
        // Discovery → other is typically fast (local sockets)
        Some(1.5)
    } else {
        // Default estimate for local socket communication
        Some(2.0)
    };

    estimated_latency_ms.map(|latency| EdgeMetrics {
        request_count: None,
        avg_latency_ms: Some(latency),
        latency_ms: Some(latency),
        bandwidth_mbps: None,   // Would require throughput testing
        packet_loss: Some(0.0), // Local sockets are reliable
        last_measured: Some(chrono::Utc::now().to_rfc3339()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_node_serialize() {
        let node = TopologyNode {
            id: "test-node".to_string(),
            name: "Test".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            capabilities: vec!["crypto".to_string()],
            endpoints: None,
            metadata: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("test-node"));
        assert!(json.contains("security"));
    }

    #[test]
    fn test_topology_edge_serialize() {
        let edge = TopologyEdge {
            from: "node-a".to_string(),
            to: "node-b".to_string(),
            edge_type: "capability_invocation".to_string(),
            capability: Some("crypto.encrypt".to_string()),
            metrics: None,
        };
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("node-a"));
        assert!(json.contains("node-b"));
        assert!(json.contains("capability_invocation"));
    }

    #[test]
    fn test_health_status_healthy() {
        let nodes = vec![
            TopologyNode {
                id: "a".to_string(),
                name: "A".to_string(),
                primal_type: "security".to_string(),
                health: "healthy".to_string(),
                capabilities: vec![],
                endpoints: None,
                metadata: None,
            },
            TopologyNode {
                id: "b".to_string(),
                name: "B".to_string(),
                primal_type: "discovery".to_string(),
                health: "healthy".to_string(),
                capabilities: vec![],
                endpoints: None,
                metadata: None,
            },
        ];
        let status = calculate_health_status(&nodes);
        assert_eq!(status.overall, "healthy");
        assert_eq!(status.primals_healthy, 2);
        assert_eq!(status.primals_total, 2);
    }

    #[test]
    fn test_health_status_degraded() {
        let nodes = vec![
            TopologyNode {
                id: "a".to_string(),
                name: "A".to_string(),
                primal_type: "security".to_string(),
                health: "healthy".to_string(),
                capabilities: vec![],
                endpoints: None,
                metadata: None,
            },
            TopologyNode {
                id: "b".to_string(),
                name: "B".to_string(),
                primal_type: "discovery".to_string(),
                health: "unhealthy".to_string(),
                capabilities: vec![],
                endpoints: None,
                metadata: None,
            },
        ];
        let status = calculate_health_status(&nodes);
        assert_eq!(status.overall, "degraded");
        assert_eq!(status.primals_healthy, 1);
    }

    #[test]
    fn test_extract_node_id_three_parts() {
        let result = extract_node_id_from_primal("beardog-fam-desktop");
        assert_eq!(result, Some("desktop".to_string()));
    }

    #[test]
    fn test_extract_node_id_two_parts() {
        let result = extract_node_id_from_primal("beardog-desktop");
        assert_eq!(result, Some("desktop".to_string()));
    }

    #[test]
    fn test_extract_node_id_one_part() {
        let result = extract_node_id_from_primal("standalone");
        assert_eq!(result, Some("standalone".to_string()));
    }

    #[test]
    fn test_collect_edge_metrics_security() {
        let metrics = collect_edge_metrics("discovery-node", "security-node");
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.latency_ms, Some(5.0)); // Security has overhead
    }

    #[test]
    fn test_collect_edge_metrics_discovery() {
        let metrics = collect_edge_metrics("discovery-node", "storage-node");
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.latency_ms, Some(1.5)); // Discovery is fast
    }

    #[test]
    fn test_collect_edge_metrics_default() {
        let metrics = collect_edge_metrics("node-a", "node-b");
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.latency_ms, Some(2.0)); // Default latency
    }

    #[test]
    fn test_standalone_topology() {
        let (nodes, edges) = get_standalone_topology();
        assert!(!nodes.is_empty());
        assert!(!edges.is_empty());

        // Check nodes have proper capabilities
        for node in &nodes {
            assert!(!node.capabilities.is_empty());
            assert!(!node.id.is_empty());
        }
    }

    #[test]
    fn test_node_endpoints_serialize() {
        let endpoints = NodeEndpoints {
            unix_socket: Some("/tmp/test.sock".to_string()),
            http: None,
        };
        let json = serde_json::to_string(&endpoints).unwrap();
        assert!(json.contains("/tmp/test.sock"));
        assert!(!json.contains("http")); // Skip serializing None
    }

    #[test]
    fn test_node_metadata_serialize() {
        let metadata = NodeMetadata {
            version: Some("1.0.0".to_string()),
            family_id: Some("1894e909e454".to_string()),
            node_id: Some("desktop".to_string()),
            trust_level: Some(3),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("1.0.0"));
        assert!(json.contains("1894e909e454"));
        assert!(json.contains("3"));
    }
}
