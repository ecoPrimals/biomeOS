// Live primal discovery - Query real BearDog and Songbird

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

/// BearDog identity response (unwrapped format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogIdentity {
    pub encryption_tag: String,
    pub capabilities: Option<Vec<String>>,
    pub family_id: Option<String>,
    #[serde(default)]
    pub identity_attestations: Vec<IdentityAttestation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAttestation {
    pub provider_capability: String,
    pub format: String,
    pub data: serde_json::Value,
}

/// BearDog health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogHealth {
    pub status: String,
    pub version: String,
}

/// Primal information from live discovery
#[derive(Debug, Clone)]
pub struct LivePrimalInfo {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub version: String,
    pub health: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub family_id: Option<String>,
}

/// Query BearDog for its identity and health
pub async fn discover_beardog(endpoint: &str) -> Result<LivePrimalInfo> {
    info!("🐻 Discovering BearDog at {}", endpoint);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    
    // Query identity (BearDog's primary endpoint)
    let identity_url = format!("{}/api/v1/trust/identity", endpoint);
    let identity_response = match client.get(&identity_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<BeardogIdentity>().await {
                    Ok(i) => {
                        info!("✅ BearDog identity: family={:?}, tag={}", i.family_id, i.encryption_tag);
                        Some(i)
                    }
                    Err(e) => {
                        warn!("Failed to parse BearDog identity: {}", e);
                        None
                    }
                }
            } else {
                error!("BearDog identity check returned: {}", resp.status());
                return Err(anyhow::anyhow!("BearDog not responding"));
            }
        }
        Err(e) => {
            error!("Failed to connect to BearDog: {}", e);
            return Err(e.into());
        }
    };
    
    // Extract capabilities from identity or use defaults
    let capabilities = identity_response
        .as_ref()
        .and_then(|i| i.capabilities.clone())
        .unwrap_or_else(|| vec![
            "security".to_string(),
            "trust_evaluation".to_string(),
            "genetic_lineage".to_string(),
            "hsm".to_string(),
        ]);
    
    let family_id = identity_response.and_then(|i| i.family_id);
    
    Ok(LivePrimalInfo {
        id: "beardog-local".to_string(),
        name: "BearDog".to_string(),
        primal_type: "security".to_string(),
        version: "0.11.0".to_string(), // Future: query /api/identity or /version endpoint
        health: "healthy".to_string(), // Assume healthy if identity responds
        capabilities,
        endpoint: endpoint.to_string(),
        family_id,
    })
}

/// Query Songbird for its health
/// Note: Songbird uses tarpc, not HTTP REST, so this is limited for now
pub async fn discover_songbird(endpoint: &str) -> Result<LivePrimalInfo> {
    info!("🐦 Discovering Songbird at {}", endpoint);
    
    // For now, Songbird doesn't have HTTP health endpoint
    // It uses tarpc RPC, which we'll need to integrate later
    // Return basic info for now
    
    warn!("⚠️  Songbird uses tarpc RPC, not HTTP REST");
    warn!("   Returning basic info. Full integration requires tarpc client.");
    
    Ok(LivePrimalInfo {
        id: "songbird-local".to_string(),
        name: "Songbird".to_string(),
        primal_type: "orchestration".to_string(),
        version: "3.0.0".to_string(),
        health: "assumed_healthy".to_string(), // Can't query yet
        capabilities: vec![
            "orchestration".to_string(),
            "discovery".to_string(),
            "federation".to_string(),
            "coordination".to_string(),
        ],
        endpoint: endpoint.to_string(),
        family_id: None, // Can't query yet
    })
}

/// Discover all configured primals
pub async fn discover_all_primals() -> Vec<LivePrimalInfo> {
    let mut primals = Vec::new();
    
    // Discover BearDog
    let beardog_endpoint = std::env::var("BEARDOG_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());
    
    match discover_beardog(&beardog_endpoint).await {
        Ok(primal) => {
            info!("✅ Discovered BearDog: {} ({})", primal.name, primal.health);
            primals.push(primal);
        }
        Err(e) => {
            error!("❌ Failed to discover BearDog: {}", e);
        }
    }
    
    // Discover Songbird
    let songbird_endpoint = std::env::var("SONGBIRD_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    match discover_songbird(&songbird_endpoint).await {
        Ok(primal) => {
            info!("✅ Discovered Songbird: {} ({})", primal.name, primal.health);
            primals.push(primal);
        }
        Err(e) => {
            error!("❌ Failed to discover Songbird: {}", e);
        }
    }
    
    primals
}

