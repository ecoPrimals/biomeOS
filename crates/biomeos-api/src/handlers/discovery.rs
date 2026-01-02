// Discovery handler
// Returns list of discovered primals with trust levels

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{ApiError, AppState};

/// Discovered primal information (matches PetalTongue's expectations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub version: String,
    pub health: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub last_seen: u64, // Unix timestamp - REQUIRED by PetalTongue
    
    // Trust information (NEW - progressive trust model)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<u8>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_capabilities: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denied_capabilities: Option<Vec<String>>,
}

/// Response structure for discovered primals
#[derive(Debug, Serialize)]
pub struct DiscoveredPrimalsResponse {
    pub primals: Vec<DiscoveredPrimal>,
    pub count: usize,
    pub mode: String,
}

/// GET /api/v1/primals/discovered
/// GET /api/v1/primals/list
/// GET /api/v1/primals
pub async fn get_discovered_primals(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DiscoveredPrimalsResponse>, ApiError> {
    info!("🔍 Discovering primals...");

    if state.mock_mode {
        // Mock mode: Return hardcoded test data
        info!("   Using mock data (BIOMEOS_MOCK_MODE=true)");
        let primals = get_mock_primals();
        return Ok(Json(DiscoveredPrimalsResponse {
            count: primals.len(),
            mode: "mock".to_string(),
            primals,
        }));
    }

    // Live mode: Discover real primals
    info!("   Live mode: Querying real BearDog and Songbird");
    
    let live_primals = super::live_discovery::discover_all_primals().await;
    
    if live_primals.is_empty() {
        info!("   No live primals found, falling back to mock");
        let primals = get_mock_primals();
        return Ok(Json(DiscoveredPrimalsResponse {
            count: primals.len(),
            mode: "mock_fallback".to_string(),
            primals,
        }));
    }
    
    // Convert LivePrimalInfo to DiscoveredPrimal
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let primals: Vec<DiscoveredPrimal> = live_primals
        .into_iter()
        .map(|live| DiscoveredPrimal {
            id: live.id,
            name: live.name,
            primal_type: live.primal_type,
            version: live.version,
            health: live.health,
            capabilities: live.capabilities,
            endpoint: live.endpoint,
            last_seen: now,
            trust_level: Some(3), // Assume highest for local primals (TODO: query BearDog)
            family_id: live.family_id,
            allowed_capabilities: Some(vec!["*".to_string()]),
            denied_capabilities: Some(vec![]),
        })
        .collect();
    
    info!("   Discovered {} live primals", primals.len());
    Ok(Json(DiscoveredPrimalsResponse {
        count: primals.len(),
        mode: "live".to_string(),
        primals,
    }))
}

/// Generate mock primal data for testing
fn get_mock_primals() -> Vec<DiscoveredPrimal> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    vec![
        DiscoveredPrimal {
            id: "beardog-local".to_string(),
            name: "BearDog".to_string(),
            primal_type: "security".to_string(),
            version: "0.11.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![
                "security".to_string(),
                "trust_evaluation".to_string(),
                "genetic_lineage".to_string(),
                "hsm".to_string(),
            ],
            endpoint: "http://localhost:9000".to_string(),
            last_seen: now,
            trust_level: Some(3), // Highest (self)
            family_id: Some("iidn".to_string()),
            allowed_capabilities: Some(vec!["*".to_string()]),
            denied_capabilities: Some(vec![]),
        },
        DiscoveredPrimal {
            id: "songbird-local".to_string(),
            name: "Songbird".to_string(),
            primal_type: "orchestration".to_string(),
            version: "3.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![
                "orchestration".to_string(),
                "discovery".to_string(),
                "federation".to_string(),
                "coordination".to_string(),
            ],
            endpoint: "http://localhost:8080".to_string(),
            last_seen: now,
            trust_level: Some(3), // Highest (self)
            family_id: Some("iidn".to_string()),
            allowed_capabilities: Some(vec!["*".to_string()]),
            denied_capabilities: Some(vec![]),
        },
        DiscoveredPrimal {
            id: "tower2-remote".to_string(),
            name: "tower2".to_string(),
            primal_type: "tower".to_string(),
            version: "1.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![
                "orchestration".to_string(),
                "federation".to_string(),
            ],
            endpoint: "https://192.168.1.134:8080".to_string(),
            last_seen: now - 5, // 5 seconds ago
            trust_level: Some(1), // Limited (same family, not elevated)
            family_id: Some("iidn".to_string()),
            allowed_capabilities: Some(vec![
                "discovery".to_string(),
                "coordination/birdsong".to_string(),
                "health".to_string(),
            ]),
            denied_capabilities: Some(vec![
                "data/*".to_string(),
                "commands/*".to_string(),
                "federation/*".to_string(),
            ]),
        },
        DiscoveredPrimal {
            id: "nestgate-local".to_string(),
            name: "NestGate".to_string(),
            primal_type: "storage".to_string(),
            version: "2.1.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![
                "storage".to_string(),
                "versioning".to_string(),
                "encryption".to_string(),
            ],
            endpoint: "http://localhost:3002".to_string(),
            last_seen: now - 2, // 2 seconds ago
            trust_level: Some(2), // Elevated (human approved)
            family_id: Some("iidn".to_string()),
            allowed_capabilities: Some(vec![
                "discovery".to_string(),
                "coordination/*".to_string(),
                "storage/read".to_string(),
                "storage/write".to_string(),
            ]),
            denied_capabilities: Some(vec![
                "storage/admin".to_string(),
                "keys/*".to_string(),
            ]),
        },
    ]
}

