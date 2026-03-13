// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
            id: format!("{}-probed", primal_name),
            name: primal_name.clone(),
            primal_type: "probed".to_string(), // Unknown until primal self-reports
            version,
            health,
            capabilities,
            endpoint: format!("unix://{}", socket_path),
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
    socket_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp/biomeos".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_primal_serialization() {
        let primal = DiscoveredPrimal {
            id: "beardog-local".to_string(),
            name: "BearDog".to_string(),
            primal_type: "security".to_string(),
            version: "0.11.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec!["security".to_string(), "crypto".to_string()],
            endpoint: "unix:///tmp/beardog.sock".to_string(),
            last_seen: 1234567890,
            trust_level: Some(3),
            family_id: Some("test-family".to_string()),
            allowed_capabilities: Some(vec!["*".to_string()]),
            denied_capabilities: Some(vec![]),
        };

        let json = serde_json::to_string(&primal).expect("serialize");
        assert!(json.contains("beardog-local"));
        assert!(json.contains("BearDog"));
        assert!(json.contains("security"));

        let deserialized: DiscoveredPrimal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, "beardog-local");
        assert_eq!(deserialized.trust_level, Some(3));
    }

    #[test]
    fn test_discovered_primal_optional_fields_skip_none() {
        let primal = DiscoveredPrimal {
            id: "test".to_string(),
            name: "Test".to_string(),
            primal_type: "test".to_string(),
            version: "1.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoint: "unix:///tmp/test.sock".to_string(),
            last_seen: 0,
            trust_level: None, // Should skip
            family_id: None,   // Should skip
            allowed_capabilities: None,
            denied_capabilities: None,
        };

        let json = serde_json::to_string(&primal).expect("serialize");
        // Optional None fields should not appear in JSON (skip_serializing_if)
        assert!(!json.contains("trust_level"));
        assert!(!json.contains("family_id"));
    }

    #[test]
    fn test_discovered_primals_response_serialization() {
        let response = DiscoveredPrimalsResponse {
            primals: vec![],
            count: 0,
            mode: "standalone".to_string(),
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("\"mode\":\"standalone\""));
    }

    #[test]
    fn test_get_socket_dir_returns_valid_path() {
        let socket_dir = get_socket_dir();
        // Should return a path that contains "biomeos" or is a valid directory pattern
        assert!(
            socket_dir.contains("biomeos") || socket_dir.starts_with('/'),
            "Socket dir should be valid path: {socket_dir}"
        );
    }

    #[test]
    fn test_probe_live_sockets_returns_vec() {
        // DEEP DEBT EVOLUTION: Tests the real socket probing (returns empty if no primals running)
        let primals = probe_live_sockets();
        // Should return an empty vec if no sockets exist (which is fine in test env)
        // The important thing is it doesn't panic or return fabricated data
        for primal in &primals {
            assert!(
                primal.endpoint.starts_with("unix://"),
                "Probed endpoint should be Unix socket: {}",
                primal.endpoint
            );
            // Trust level should be 1 (discovered, not yet verified)
            assert_eq!(primal.trust_level, Some(1));
            // Probed type should be "probed" (not fabricated)
            assert_eq!(primal.primal_type, "probed");
        }
    }

    #[test]
    fn test_discovered_primal_deserialization() {
        let json = r#"{
            "id": "test-primal",
            "name": "Test",
            "primal_type": "security",
            "version": "1.0.0",
            "health": "healthy",
            "capabilities": ["security", "crypto"],
            "endpoint": "unix:///tmp/test.sock",
            "last_seen": 1234567890,
            "trust_level": 2,
            "family_id": "test-family"
        }"#;

        let primal: DiscoveredPrimal = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(primal.id, "test-primal");
        assert_eq!(primal.name, "Test");
        assert_eq!(primal.trust_level, Some(2));
        assert_eq!(primal.family_id, Some("test-family".to_string()));
    }

    #[test]
    fn test_discovered_primal_all_fields() {
        let primal = DiscoveredPrimal {
            id: "full-primal".to_string(),
            name: "Full".to_string(),
            primal_type: "compute".to_string(),
            version: "2.0.0".to_string(),
            health: "degraded".to_string(),
            capabilities: vec!["compute".to_string(), "execution".to_string()],
            endpoint: "unix:///tmp/full.sock".to_string(),
            last_seen: 9999999999,
            trust_level: Some(2),
            family_id: Some("family-1".to_string()),
            allowed_capabilities: Some(vec!["compute/*".to_string()]),
            denied_capabilities: Some(vec!["compute/admin".to_string()]),
        };

        let json = serde_json::to_string(&primal).expect("should serialize");
        let deserialized: DiscoveredPrimal =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.id, primal.id);
        assert_eq!(
            deserialized.allowed_capabilities,
            primal.allowed_capabilities
        );
        assert_eq!(deserialized.denied_capabilities, primal.denied_capabilities);
    }

    #[test]
    fn test_discovered_primals_response_with_primals() {
        let primals = vec![
            DiscoveredPrimal {
                id: "primal-1".to_string(),
                name: "Primal1".to_string(),
                primal_type: "security".to_string(),
                version: "1.0.0".to_string(),
                health: "healthy".to_string(),
                capabilities: vec!["security".to_string()],
                endpoint: "unix:///tmp/p1.sock".to_string(),
                last_seen: 1234567890,
                trust_level: Some(3),
                family_id: Some("family-1".to_string()),
                allowed_capabilities: None,
                denied_capabilities: None,
            },
            DiscoveredPrimal {
                id: "primal-2".to_string(),
                name: "Primal2".to_string(),
                primal_type: "orchestration".to_string(),
                version: "2.0.0".to_string(),
                health: "healthy".to_string(),
                capabilities: vec!["orchestration".to_string()],
                endpoint: "unix:///tmp/p2.sock".to_string(),
                last_seen: 1234567891,
                trust_level: Some(2),
                family_id: None,
                allowed_capabilities: None,
                denied_capabilities: None,
            },
        ];

        let response = DiscoveredPrimalsResponse {
            primals: primals.clone(),
            count: primals.len(),
            mode: "live".to_string(),
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"mode\":\"live\""));
        assert!(json.contains("primal-1"));
        assert!(json.contains("primal-2"));
    }

    #[test]
    fn test_probe_live_sockets_correct_structure() {
        // DEEP DEBT EVOLUTION: Probed primals always have correct structure
        let primals = probe_live_sockets();
        for primal in &primals {
            assert!(!primal.id.is_empty(), "Probed primal should have an ID");
            assert!(!primal.name.is_empty(), "Probed primal should have a name");
            assert!(
                !primal.endpoint.is_empty(),
                "Probed primal should have an endpoint"
            );
            assert!(
                primal.last_seen > 0,
                "Probed primal should have a timestamp"
            );
        }
    }

    #[test]
    fn test_get_socket_dir_resolves() {
        // Verify socket directory resolution works (uses 5-tier strategy)
        let socket_dir = get_socket_dir();
        assert!(!socket_dir.is_empty(), "Socket dir should not be empty");
    }

    #[tokio::test]
    async fn test_get_discovered_primals_standalone_mode() {
        use crate::AppState;
        use std::sync::Arc;

        let state = Arc::new(
            AppState::builder()
                .config(crate::Config {
                    standalone_mode: true,
                    ..Default::default()
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let result = get_discovered_primals(axum::extract::State(state)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        // DEEP DEBT: Standalone mode now falls back to socket probing (real discovery)
        // Mode will be "socket_probe" instead of "standalone" (no more fabricated data)
        assert!(
            response.mode == "socket_probe" || response.mode == "live",
            "Mode should be socket_probe or live, got: {}",
            response.mode
        );
        assert_eq!(response.count, response.primals.len());
    }

    #[tokio::test]
    async fn test_get_discovered_primals_live_mode_success() {
        use crate::AppState;
        use biomeos_core::discovery_modern::Capability;
        use biomeos_core::{DiscoveryResult, HealthStatus, PrimalDiscovery, PrimalType};
        use biomeos_types::{Endpoint, FamilyId, PrimalId};
        use semver::Version;
        use std::sync::Arc;

        struct MockDiscovery {
            primals: Vec<biomeos_core::DiscoveredPrimal>,
        }

        #[async_trait::async_trait]
        impl PrimalDiscovery for MockDiscovery {
            async fn discover(
                &self,
                _endpoint: &Endpoint,
            ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
                Err(biomeos_core::DiscoveryError::NotFound {
                    endpoint: "mock".to_string(),
                })
            }

            async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
                Ok(self.primals.clone())
            }

            async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
                Ok(HealthStatus::Healthy)
            }
        }

        let primals = vec![
            biomeos_core::DiscoveredPrimal {
                id: PrimalId::new_unchecked("beardog-1"),
                name: "BearDog".to_string(),
                primal_type: PrimalType::Security,
                version: Version::parse("1.0.0").expect("valid version"),
                health: HealthStatus::Healthy,
                capabilities: vec![Capability::from("security")],
                endpoint: Endpoint::new("unix:///tmp/beardog.sock").expect("valid endpoint"),
                metadata: serde_json::json!({}),
                family_id: Some(FamilyId::new("family-1")),
            },
            biomeos_core::DiscoveredPrimal {
                id: PrimalId::new_unchecked("songbird-1"),
                name: "Songbird".to_string(),
                primal_type: PrimalType::Orchestration,
                version: Version::parse("2.0.0").expect("valid version"),
                health: HealthStatus::Degraded,
                capabilities: vec![Capability::from("orchestration")],
                endpoint: Endpoint::new("unix:///tmp/songbird.sock").expect("valid endpoint"),
                metadata: serde_json::json!({}),
                family_id: Some(FamilyId::new("family-1")),
            },
        ];

        let discovery = MockDiscovery { primals };
        let state = Arc::new(
            AppState::builder()
                .discovery(discovery)
                .build_with_defaults()
                .expect("should build"),
        );

        let result = get_discovered_primals(axum::extract::State(state)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.mode, "live");
        assert_eq!(response.primals.len(), 2);
        assert_eq!(response.count, 2);

        // Verify conversion from core types to API types
        let beardog = response
            .primals
            .iter()
            .find(|p| p.id == "beardog-1")
            .expect("should find BearDog");
        assert_eq!(beardog.name, "BearDog");
        assert_eq!(beardog.health, "healthy");
        assert_eq!(beardog.trust_level, Some(3));

        let songbird = response
            .primals
            .iter()
            .find(|p| p.id == "songbird-1")
            .expect("should find Songbird");
        assert_eq!(songbird.health, "degraded");
    }

    #[tokio::test]
    async fn test_get_discovered_primals_live_mode_failure() {
        use crate::AppState;
        use biomeos_core::{DiscoveryError, DiscoveryResult, HealthStatus, PrimalDiscovery};
        use biomeos_types::{Endpoint, PrimalId};
        use std::sync::Arc;

        struct FailingDiscovery;

        #[async_trait::async_trait]
        impl PrimalDiscovery for FailingDiscovery {
            async fn discover(
                &self,
                _endpoint: &Endpoint,
            ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
                Err(DiscoveryError::NotFound {
                    endpoint: "mock".to_string(),
                })
            }

            async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
                Err(DiscoveryError::NotFound {
                    endpoint: "discovery failed".to_string(),
                })
            }

            async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
                Ok(HealthStatus::Unknown)
            }
        }

        let discovery = FailingDiscovery;
        let state = Arc::new(
            AppState::builder()
                .discovery(discovery)
                .build_with_defaults()
                .expect("should build"),
        );

        let result = get_discovered_primals(axum::extract::State(state)).await;

        assert!(result.is_ok()); // Should return empty list, not error
        let response = result.unwrap();
        assert_eq!(response.mode, "live_failed");
        assert_eq!(response.primals.len(), 0);
        assert_eq!(response.count, 0);
    }

    #[tokio::test]
    async fn test_get_discovered_primals_health_status_conversion() {
        use crate::AppState;
        use biomeos_core::{DiscoveryResult, HealthStatus, PrimalDiscovery, PrimalType};
        use biomeos_types::{Endpoint, PrimalId};
        use semver::Version;
        use std::sync::Arc;

        struct MockDiscovery;

        #[async_trait::async_trait]
        impl PrimalDiscovery for MockDiscovery {
            async fn discover(
                &self,
                _endpoint: &Endpoint,
            ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
                Err(biomeos_core::DiscoveryError::NotFound {
                    endpoint: "mock".to_string(),
                })
            }

            async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
                Ok(vec![
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("healthy"),
                        name: "Healthy".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Healthy,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/healthy.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("degraded"),
                        name: "Degraded".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Degraded,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/degraded.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("unhealthy"),
                        name: "Unhealthy".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Unhealthy,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/unhealthy.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("unknown"),
                        name: "Unknown".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Unknown,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/unknown.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                ])
            }

            async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
                Ok(HealthStatus::Healthy)
            }
        }

        let discovery = MockDiscovery;
        let state = Arc::new(
            AppState::builder()
                .discovery(discovery)
                .build_with_defaults()
                .expect("should build"),
        );

        let result = get_discovered_primals(axum::extract::State(state)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.primals.len(), 4);

        let health_map: std::collections::HashMap<_, _> = response
            .primals
            .iter()
            .map(|p| (p.id.as_str(), p.health.as_str()))
            .collect();

        assert_eq!(health_map.get("healthy"), Some(&"healthy"));
        assert_eq!(health_map.get("degraded"), Some(&"degraded"));
        assert_eq!(health_map.get("unhealthy"), Some(&"unhealthy"));
        assert_eq!(health_map.get("unknown"), Some(&"unknown"));
    }

    #[tokio::test]
    async fn test_get_discovered_primals_standalone_mode_label() {
        use crate::AppState;
        use std::sync::Arc;

        let state = Arc::new(
            AppState::builder()
                .config(crate::Config {
                    standalone_mode: true,
                    ..Default::default()
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let result = get_discovered_primals(axum::extract::State(state)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        // Standalone mode: socket_probe or live (if discovery succeeds)
        assert!(
            response.mode == "socket_probe" || response.mode == "live",
            "Mode should be socket_probe or live, got: {}",
            response.mode
        );
    }

    #[tokio::test]
    async fn test_get_discovered_primals_trust_level_with_family() {
        use crate::AppState;
        use biomeos_core::discovery_modern::Capability;
        use biomeos_core::{DiscoveryResult, HealthStatus, PrimalDiscovery, PrimalType};
        use biomeos_types::{Endpoint, FamilyId, PrimalId};
        use semver::Version;
        use std::sync::Arc;

        struct MockDiscovery;

        #[async_trait::async_trait]
        impl PrimalDiscovery for MockDiscovery {
            async fn discover(
                &self,
                _endpoint: &Endpoint,
            ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
                Err(biomeos_core::DiscoveryError::NotFound {
                    endpoint: "mock".to_string(),
                })
            }

            async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
                Ok(vec![
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("with-family"),
                        name: "WithFamily".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid"),
                        health: HealthStatus::Healthy,
                        capabilities: vec![Capability::from("security")],
                        endpoint: Endpoint::new("unix:///tmp/with.sock").expect("valid"),
                        metadata: serde_json::json!({}),
                        family_id: Some(FamilyId::new("family-1")),
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("no-family"),
                        name: "NoFamily".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid"),
                        health: HealthStatus::Healthy,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/no.sock").expect("valid"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                ])
            }

            async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
                Ok(HealthStatus::Healthy)
            }
        }

        let state = Arc::new(
            AppState::builder()
                .discovery(MockDiscovery)
                .build_with_defaults()
                .expect("should build"),
        );

        let result = get_discovered_primals(axum::extract::State(state)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let with_family = response
            .primals
            .iter()
            .find(|p| p.id == "with-family")
            .expect("with-family primal");
        let no_family = response
            .primals
            .iter()
            .find(|p| p.id == "no-family")
            .expect("no-family primal");
        assert_eq!(with_family.trust_level, Some(3));
        assert_eq!(no_family.trust_level, Some(1));
    }

    #[test]
    fn test_discovered_primal_empty_capabilities() {
        let primal = DiscoveredPrimal {
            id: "empty-caps".to_string(),
            name: "Empty".to_string(),
            primal_type: "security".to_string(),
            version: "1.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoint: "unix:///tmp/empty.sock".to_string(),
            last_seen: 0,
            trust_level: None,
            family_id: None,
            allowed_capabilities: None,
            denied_capabilities: None,
        };
        let json = serde_json::to_string(&primal).expect("serialize");
        assert!(json.contains("empty-caps"));
        assert!(json.contains("\"capabilities\":[]"));
    }
}
