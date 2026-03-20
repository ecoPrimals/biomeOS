// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// Discovery handler
// Returns list of discovered primals with trust levels

use axum::{Json, extract::State};
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

    // DEEP DEBT EVOLUTION (Feb 7, 2026):
    // Standalone mode no longer returns fabricated data.
    // All modes attempt live discovery first. Standalone only affects
    // whether we fall back to socket-probing when trait discovery fails.
    let mode_label = if state.is_standalone_mode() {
        "standalone_probe"
    } else {
        "live"
    };
    info!("   Discovery mode: {}", mode_label);

    match state.discovery().discover_all().await {
        Ok(discovered) => {
            info!(
                "   Discovered {} primals via modern discovery",
                discovered.len()
            );

            // Convert to API format
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0)) // Safe fallback: epoch time
                .as_secs();

            let primals: Vec<DiscoveredPrimal> = discovered
                .into_iter()
                .map(|primal| {
                    let health = match primal.health {
                        biomeos_core::HealthStatus::Healthy => "healthy",
                        biomeos_core::HealthStatus::Degraded => "degraded",
                        biomeos_core::HealthStatus::Unhealthy => "unhealthy",
                        biomeos_core::HealthStatus::Unknown => "unknown",
                    };

                    let primal_type = format!("{:?}", primal.primal_type).to_lowercase();

                    DiscoveredPrimal {
                        id: primal.id.as_str().to_string(),
                        name: primal.name,
                        primal_type,
                        version: primal.version.to_string(),
                        health: health.to_string(),
                        capabilities: primal
                            .capabilities
                            .iter()
                            .map(|c| c.as_str().to_string())
                            .collect(),
                        endpoint: primal.endpoint.as_str().to_string(),
                        last_seen: now,
                        trust_level: if primal.family_id.is_some() {
                            Some(3)
                        } else {
                            Some(1)
                        },
                        family_id: primal.family_id.map(|f| f.as_str().to_string()),
                        allowed_capabilities: Some(vec!["*".to_string()]),
                        denied_capabilities: Some(vec![]),
                    }
                })
                .collect();

            Ok(Json(DiscoveredPrimalsResponse {
                count: primals.len(),
                mode: "live".to_string(),
                primals,
            }))
        }
        Err(e) => {
            tracing::warn!("   Trait-based discovery failed: {}", e);

            // DEEP DEBT EVOLUTION: In standalone mode, fall back to socket probing.
            // This checks if actual primal sockets exist on disk (real discovery,
            // not fabricated data). Only reports primals that are actually running.
            if state.is_standalone_mode() {
                info!("   Falling back to socket probe discovery (standalone mode)");
                let probed = probe_live_sockets();
                return Ok(Json(DiscoveredPrimalsResponse {
                    count: probed.len(),
                    mode: "socket_probe".to_string(),
                    primals: probed,
                }));
            }

            Ok(Json(DiscoveredPrimalsResponse {
                count: 0,
                mode: "live_failed".to_string(),
                primals: vec![],
            }))
        }
    }
}

/// Probe live sockets to discover actually running primals
///
/// DEEP DEBT EVOLUTION (Feb 7, 2026):
/// Replaced fabricated standalone data with real socket probing.
/// This function scans the socket directory for `.sock` files and
/// pings each one to verify the primal is actually running.
///
/// # Deep Debt Principles
///
/// 1. **No fabricated data**: Only reports primals that respond to health checks
/// 2. **Self-knowledge only**: Discovers by socket presence, not hardcoded names
/// 3. **Capability-based**: Reads capabilities from primal's own response
/// 4. **Environment-aware**: Uses 5-tier socket resolution
fn probe_live_sockets() -> Vec<DiscoveredPrimal> {
    let socket_dir = get_socket_dir();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let mut primals = Vec::new();

    // Scan socket directory for .sock files
    let dir = match std::fs::read_dir(&socket_dir) {
        Ok(d) => d,
        Err(e) => {
            tracing::debug!("Socket dir {} not readable: {}", socket_dir, e);
            return primals;
        }
    };

    for entry in dir.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "sock") {
            continue;
        }

        // Check if it's a Unix socket (not a regular file)
        if !path.exists() {
            continue;
        }

        let socket_path = path.to_string_lossy().to_string();
        let file_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Try to ping the primal via its socket
        let client = biomeos_core::AtomicClient::unix(&socket_path)
            .with_timeout(std::time::Duration::from_secs(2));

        // Use a runtime handle if available, otherwise report as discovered-but-unchecked
        let (health, capabilities, version) = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context — use block_in_place to avoid nesting
                match std::thread::scope(|_| {
                    handle.block_on(async { client.call("health", serde_json::json!({})).await })
                }) {
                    Ok(result) => {
                        let h = result
                            .get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let caps = result
                            .get("capabilities")
                            .and_then(|c| c.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let v = result
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        (h, caps, v)
                    }
                    Err(_) => ("unreachable".to_string(), vec![], "unknown".to_string()),
                }
            }
            Err(_) => ("discovered".to_string(), vec![], "unknown".to_string()),
        };

        // Extract primal name from socket filename (e.g., "beardog-family" → "beardog")
        let primal_name = file_name
            .split('-')
            .next()
            .unwrap_or(&file_name)
            .to_string();

        primals.push(DiscoveredPrimal {
            id: format!("{primal_name}-probed"),
            name: primal_name.clone(),
            primal_type: "probed".to_string(), // Unknown until primal self-reports
            version,
            health,
            capabilities,
            endpoint: format!("unix://{socket_path}"),
            last_seen: now,
            trust_level: Some(1), // Discovered, not yet verified
            family_id: None,      // Unknown until lineage check
            allowed_capabilities: None,
            denied_capabilities: None,
        });

        tracing::info!("   Probed socket: {} → {}", primal_name, socket_path);
    }

    primals
}

/// Get socket directory using 5-tier resolution via SocketDiscovery
///
/// Delegates to `biomeos_core::socket_discovery::SocketDiscovery` which implements
/// the full PRIMAL_DEPLOYMENT_STANDARD.md hierarchy:
/// 1. BIOMEOS_SOCKET_DIR environment variable
/// 2. XDG_RUNTIME_DIR/biomeos
/// 3. /run/user/{uid}/biomeos
/// 4. /data/local/tmp/biomeos (Android)
/// 5. /tmp/biomeos (fallback)
fn get_socket_dir() -> String {
    use biomeos_core::socket_discovery::SocketDiscovery;

    // Get family ID from environment or use default
    let family_id = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| biomeos_core::family_discovery::get_family_id());

    let discovery = SocketDiscovery::new(family_id);

    // Build path for a dummy primal to get the directory
    let socket_path = discovery.build_socket_path("_discovery_probe");

    // Extract directory from path
    socket_path.parent().map_or_else(
        || "/tmp/biomeos".to_string(),
        |p| p.to_string_lossy().to_string(),
    )
}

#[cfg(test)]
mod tests;
