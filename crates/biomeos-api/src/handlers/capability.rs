//! Capability API handlers
//!
//! Provides REST API endpoints for capability discovery and management.
//! Capabilities represent what primals can do (e.g., "crypto.encrypt", "http.get").
//!
//! Note: These handlers are defined and ready to wire into the axum Router.
//! They will be connected when the capability API routes are enabled.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{ApiError, AppState};

/// Capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub name: String,
    pub description: Option<String>,
    pub providers: Vec<CapabilityProvider>,
}

/// Capability provider (primal that provides a capability)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider {
    pub primal_id: String,
    pub primal_name: String,
    pub endpoint: String,
    pub health: String,
    pub trust_level: Option<u8>,
}

/// List capabilities request
#[derive(Debug, Deserialize)]
pub struct ListCapabilitiesQuery {
    #[serde(default)]
    pub filter: Option<String>,
}

/// Discover capability request
#[derive(Debug, Deserialize)]
pub struct DiscoverCapabilityRequest {
    pub capability: String,
}

/// Discover capability response
#[derive(Debug, Serialize)]
pub struct DiscoverCapabilityResponse {
    pub capability: String,
    pub providers: Vec<CapabilityProvider>,
    pub count: usize,
}

/// List capabilities response
#[derive(Debug, Serialize)]
pub struct ListCapabilitiesResponse {
    pub capabilities: Vec<CapabilityInfo>,
    pub count: usize,
}

/// GET /api/v1/capabilities
/// List all available capabilities
pub async fn list_capabilities(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<ListCapabilitiesQuery>,
) -> Result<Json<ListCapabilitiesResponse>, ApiError> {
    info!("📋 Listing capabilities...");

    if state.is_standalone_mode() {
        info!("   Using standalone capabilities (BIOMEOS_STANDALONE_MODE=true)");
        let capabilities = get_standalone_capabilities(&query.filter);
        return Ok(Json(ListCapabilitiesResponse {
            count: capabilities.len(),
            capabilities,
        }));
    }

    // Live mode: Query discovered primals for capabilities
    info!("   Live mode: Querying discovered primals");

    match state.discovery().discover_all().await {
        Ok(discovered) => {
            let capabilities = build_capability_list(&discovered, &query.filter);
            Ok(Json(ListCapabilitiesResponse {
                count: capabilities.len(),
                capabilities,
            }))
        }
        Err(e) => {
            tracing::warn!("   Discovery failed: {}, using standalone fallback", e);
            let capabilities = get_standalone_capabilities(&query.filter);
            Ok(Json(ListCapabilitiesResponse {
                count: capabilities.len(),
                capabilities,
            }))
        }
    }
}

/// POST /api/v1/capabilities/discover
/// Discover primals that provide a specific capability
pub async fn discover_capability(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DiscoverCapabilityRequest>,
) -> Result<Json<DiscoverCapabilityResponse>, ApiError> {
    info!("🔍 Discovering capability: {}", request.capability);

    if state.is_standalone_mode() {
        info!("   Using standalone discovery (BIOMEOS_STANDALONE_MODE=true)");
        let providers = get_standalone_providers(&request.capability);
        return Ok(Json(DiscoverCapabilityResponse {
            capability: request.capability.clone(),
            count: providers.len(),
            providers,
        }));
    }

    // Live mode: Filter discovered primals by capability
    info!("   Live mode: Filtering discovered primals");

    match state.discovery().discover_all().await {
        Ok(discovered) => {
            let providers: Vec<CapabilityProvider> = discovered
                .into_iter()
                .filter(|primal| {
                    primal.capabilities.iter().any(|c| {
                        c.as_str() == request.capability
                            || c.as_str().starts_with(&format!("{}.", request.capability))
                    })
                })
                .map(|primal| {
                    let health = match primal.health {
                        biomeos_core::HealthStatus::Healthy => "healthy",
                        biomeos_core::HealthStatus::Degraded => "degraded",
                        biomeos_core::HealthStatus::Unhealthy => "unhealthy",
                        biomeos_core::HealthStatus::Unknown => "unknown",
                    };

                    CapabilityProvider {
                        primal_id: primal.id.as_str().to_string(),
                        primal_name: primal.name,
                        endpoint: primal.endpoint.as_str().to_string(),
                        health: health.to_string(),
                        trust_level: if primal.family_id.is_some() {
                            Some(3)
                        } else {
                            Some(1)
                        },
                    }
                })
                .collect();

            Ok(Json(DiscoverCapabilityResponse {
                capability: request.capability,
                count: providers.len(),
                providers,
            }))
        }
        Err(e) => {
            tracing::warn!("   Discovery failed: {}, using standalone fallback", e);
            let providers = get_standalone_providers(&request.capability);
            Ok(Json(DiscoverCapabilityResponse {
                capability: request.capability,
                count: providers.len(),
                providers,
            }))
        }
    }
}

/// Build capability list from discovered primals
fn build_capability_list(
    discovered: &[biomeos_core::DiscoveredPrimal],
    filter: &Option<String>,
) -> Vec<CapabilityInfo> {
    use std::collections::HashMap;

    let mut capability_map: HashMap<String, Vec<CapabilityProvider>> = HashMap::new();

    for primal in discovered {
        let health = match primal.health {
            biomeos_core::HealthStatus::Healthy => "healthy",
            biomeos_core::HealthStatus::Degraded => "degraded",
            biomeos_core::HealthStatus::Unhealthy => "unhealthy",
            biomeos_core::HealthStatus::Unknown => "unknown",
        };

        let provider = CapabilityProvider {
            primal_id: primal.id.as_str().to_string(),
            primal_name: primal.name.clone(),
            endpoint: primal.endpoint.as_str().to_string(),
            health: health.to_string(),
            trust_level: if primal.family_id.is_some() {
                Some(3)
            } else {
                Some(1)
            },
        };

        for capability in &primal.capabilities {
            capability_map
                .entry(capability.as_str().to_string())
                .or_default()
                .push(provider.clone());
        }
    }

    let mut capabilities: Vec<CapabilityInfo> = capability_map
        .into_iter()
        .map(|(name, providers)| CapabilityInfo {
            name,
            description: None,
            providers,
        })
        .collect();

    // Apply filter if provided
    if let Some(filter_str) = filter {
        capabilities.retain(|c| c.name.contains(filter_str));
    }

    capabilities.sort_by(|a, b| a.name.cmp(&b.name));
    capabilities
}

/// Get standalone capabilities (for development/demo mode)
fn get_standalone_capabilities(filter: &Option<String>) -> Vec<CapabilityInfo> {
    let mut capabilities = vec![
        CapabilityInfo {
            name: "security".to_string(),
            description: Some("Security and cryptographic operations".to_string()),
            providers: vec![CapabilityProvider {
                primal_id: "beardog-local".to_string(),
                primal_name: "BearDog".to_string(),
                endpoint: "unix:///tmp/biomeos/beardog.sock".to_string(),
                health: "healthy".to_string(),
                trust_level: Some(3),
            }],
        },
        CapabilityInfo {
            name: "crypto.encrypt".to_string(),
            description: Some("Data encryption".to_string()),
            providers: vec![CapabilityProvider {
                primal_id: "beardog-local".to_string(),
                primal_name: "BearDog".to_string(),
                endpoint: "unix:///tmp/biomeos/beardog.sock".to_string(),
                health: "healthy".to_string(),
                trust_level: Some(3),
            }],
        },
        CapabilityInfo {
            name: "orchestration".to_string(),
            description: Some("Primal orchestration and coordination".to_string()),
            providers: vec![CapabilityProvider {
                primal_id: "songbird-local".to_string(),
                primal_name: "Songbird".to_string(),
                endpoint: "unix:///tmp/biomeos/songbird.sock".to_string(),
                health: "healthy".to_string(),
                trust_level: Some(3),
            }],
        },
        CapabilityInfo {
            name: "discovery".to_string(),
            description: Some("Primal discovery and service location".to_string()),
            providers: vec![CapabilityProvider {
                primal_id: "songbird-local".to_string(),
                primal_name: "Songbird".to_string(),
                endpoint: "unix:///tmp/biomeos/songbird.sock".to_string(),
                health: "healthy".to_string(),
                trust_level: Some(3),
            }],
        },
        CapabilityInfo {
            name: "compute".to_string(),
            description: Some("Compute and execution services".to_string()),
            providers: vec![CapabilityProvider {
                primal_id: "toadstool-local".to_string(),
                primal_name: "Toadstool".to_string(),
                endpoint: "unix:///tmp/biomeos/toadstool.sock".to_string(),
                health: "healthy".to_string(),
                trust_level: Some(3),
            }],
        },
        CapabilityInfo {
            name: "storage".to_string(),
            description: Some("Storage and data persistence".to_string()),
            providers: vec![CapabilityProvider {
                primal_id: "nestgate-local".to_string(),
                primal_name: "NestGate".to_string(),
                endpoint: "unix:///tmp/biomeos/nestgate.sock".to_string(),
                health: "healthy".to_string(),
                trust_level: Some(2),
            }],
        },
    ];

    // Apply filter if provided
    if let Some(filter_str) = filter {
        capabilities.retain(|c| c.name.contains(filter_str));
    }

    capabilities
}

/// Get standalone providers for a capability
fn get_standalone_providers(capability: &str) -> Vec<CapabilityProvider> {
    let all_capabilities = get_standalone_capabilities(&None);

    all_capabilities
        .into_iter()
        .filter(|c| c.name == capability || c.name.starts_with(&format!("{}.", capability)))
        .flat_map(|c| c.providers)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn create_test_state(primals: Vec<biomeos_core::DiscoveredPrimal>) -> Arc<AppState> {
        let discovery = MockDiscovery { primals };
        Arc::new(
            AppState::builder()
                .discovery(discovery)
                .build_with_defaults()
                .expect("should build state"),
        )
    }

    #[test]
    fn test_capability_info_serialization() {
        let info = CapabilityInfo {
            name: "crypto.encrypt".to_string(),
            description: Some("Encryption capability".to_string()),
            providers: vec![CapabilityProvider {
                primal_id: "beardog-1".to_string(),
                primal_name: "BearDog".to_string(),
                endpoint: "unix:///tmp/beardog.sock".to_string(),
                health: "healthy".to_string(),
                trust_level: Some(3),
            }],
        };

        let json = serde_json::to_string(&info).expect("should serialize");
        assert!(json.contains("crypto.encrypt"));
        assert!(json.contains("BearDog"));
        assert!(json.contains("healthy"));

        let deserialized: CapabilityInfo = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.name, "crypto.encrypt");
        assert_eq!(deserialized.providers.len(), 1);
    }

    #[test]
    fn test_capability_provider_serialization() {
        let provider = CapabilityProvider {
            primal_id: "test-primal".to_string(),
            primal_name: "Test".to_string(),
            endpoint: "unix:///tmp/test.sock".to_string(),
            health: "healthy".to_string(),
            trust_level: Some(2),
        };

        let json = serde_json::to_string(&provider).expect("should serialize");
        assert!(json.contains("test-primal"));
        assert!(json.contains("healthy"));
        assert!(json.contains("2"));

        let deserialized: CapabilityProvider =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.primal_id, "test-primal");
        assert_eq!(deserialized.trust_level, Some(2));
    }

    #[test]
    fn test_discover_capability_request_deserialization() {
        let json = r#"{"capability": "crypto"}"#;
        let req: DiscoverCapabilityRequest =
            serde_json::from_str(json).expect("should deserialize");
        assert_eq!(req.capability, "crypto");
    }

    #[test]
    fn test_discover_capability_response_serialization() {
        let response = DiscoverCapabilityResponse {
            capability: "crypto".to_string(),
            providers: vec![],
            count: 0,
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("crypto"));
        assert!(json.contains("\"count\":0"));
    }

    #[test]
    fn test_list_capabilities_response_serialization() {
        let response = ListCapabilitiesResponse {
            capabilities: vec![],
            count: 0,
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("capabilities"));
    }

    #[test]
    fn test_get_standalone_capabilities_returns_all() {
        let capabilities = get_standalone_capabilities(&None);
        assert!(!capabilities.is_empty());
        assert!(capabilities.len() >= 5); // At least 5 core capabilities

        // Check that core capabilities are present
        let names: Vec<&str> = capabilities.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"security"));
        assert!(names.contains(&"orchestration"));
    }

    #[test]
    fn test_get_standalone_capabilities_with_filter() {
        let all = get_standalone_capabilities(&None);
        let filtered = get_standalone_capabilities(&Some("crypto".to_string()));

        assert!(filtered.len() <= all.len());
        assert!(filtered.iter().all(|c| c.name.contains("crypto")));
    }

    #[test]
    fn test_get_standalone_providers_finds_matching() {
        let providers = get_standalone_providers("security");
        assert!(!providers.is_empty());
        assert!(providers.iter().any(|p| p.primal_name == "BearDog"));
    }

    #[test]
    fn test_get_standalone_providers_prefix_match() {
        let providers = get_standalone_providers("crypto");
        // Should match "crypto.encrypt" via prefix
        assert!(!providers.is_empty());
    }

    #[test]
    fn test_get_standalone_providers_no_match() {
        let providers = get_standalone_providers("nonexistent");
        assert!(providers.is_empty());
    }

    #[test]
    fn test_build_capability_list_from_primals() {
        use biomeos_types::PrimalId;

        let primals = vec![
            biomeos_core::DiscoveredPrimal {
                id: PrimalId::new_unchecked("beardog-1"),
                name: "BearDog".to_string(),
                primal_type: PrimalType::Security,
                version: Version::parse("1.0.0").expect("valid version"),
                health: HealthStatus::Healthy,
                capabilities: vec![
                    Capability::from("security"),
                    Capability::from("crypto.encrypt"),
                ],
                endpoint: Endpoint::new("unix:///tmp/beardog.sock").expect("valid endpoint"),
                metadata: serde_json::json!({}),
                family_id: Some(FamilyId::new("family-1")),
            },
            biomeos_core::DiscoveredPrimal {
                id: PrimalId::new_unchecked("songbird-1"),
                name: "Songbird".to_string(),
                primal_type: PrimalType::Orchestration,
                version: Version::parse("2.0.0").expect("valid version"),
                health: HealthStatus::Healthy,
                capabilities: vec![Capability::from("orchestration")],
                endpoint: Endpoint::new("unix:///tmp/songbird.sock").expect("valid endpoint"),
                metadata: serde_json::json!({}),
                family_id: Some(FamilyId::new("family-1")),
            },
        ];

        let capabilities = build_capability_list(&primals, &None);

        assert_eq!(capabilities.len(), 3); // security, crypto.encrypt, orchestration
        assert!(capabilities.iter().any(|c| c.name == "security"));
        assert!(capabilities.iter().any(|c| c.name == "crypto.encrypt"));
        assert!(capabilities.iter().any(|c| c.name == "orchestration"));
    }

    #[test]
    fn test_build_capability_list_with_filter() {
        use biomeos_types::PrimalId;

        let primals = vec![biomeos_core::DiscoveredPrimal {
            id: PrimalId::new_unchecked("beardog-1"),
            name: "BearDog".to_string(),
            primal_type: PrimalType::Security,
            version: Version::parse("1.0.0").expect("valid version"),
            health: HealthStatus::Healthy,
            capabilities: vec![
                Capability::from("security"),
                Capability::from("crypto.encrypt"),
            ],
            endpoint: Endpoint::new("unix:///tmp/beardog.sock").expect("valid endpoint"),
            metadata: serde_json::json!({}),
            family_id: Some(FamilyId::new("family-1")),
        }];

        let all = build_capability_list(&primals, &None);
        let filtered = build_capability_list(&primals, &Some("crypto".to_string()));

        assert!(filtered.len() <= all.len());
        assert!(filtered.iter().all(|c| c.name.contains("crypto")));
    }

    #[tokio::test]
    async fn test_list_capabilities_standalone_mode() {
        let state = Arc::new(
            AppState::builder()
                .config({
                    let mut config = crate::Config::default();
                    config.standalone_mode = true;
                    config
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let query = ListCapabilitiesQuery { filter: None };
        let result = list_capabilities(State(state), axum::extract::Query(query)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_discover_capability_standalone_mode() {
        let state = Arc::new(
            AppState::builder()
                .config({
                    let mut config = crate::Config::default();
                    config.standalone_mode = true;
                    config
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let request = DiscoverCapabilityRequest {
            capability: "security".to_string(),
        };
        let result = discover_capability(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.capability, "security");
        assert!(!response.providers.is_empty());
    }

    #[tokio::test]
    async fn test_discover_capability_live_mode() {
        use biomeos_types::PrimalId;

        let primals = vec![biomeos_core::DiscoveredPrimal {
            id: PrimalId::new_unchecked("beardog-1"),
            name: "BearDog".to_string(),
            primal_type: PrimalType::Security,
            version: Version::parse("1.0.0").expect("valid version"),
            health: HealthStatus::Healthy,
            capabilities: vec![Capability::from("security")],
            endpoint: Endpoint::new("unix:///tmp/beardog.sock").expect("valid endpoint"),
            metadata: serde_json::json!({}),
            family_id: Some(FamilyId::new("family-1")),
        }];

        let state = create_test_state(primals);

        let request = DiscoverCapabilityRequest {
            capability: "security".to_string(),
        };
        let result = discover_capability(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.capability, "security");
        assert_eq!(response.providers.len(), 1);
        assert_eq!(response.providers[0].primal_name, "BearDog");
    }

    #[tokio::test]
    async fn test_discover_capability_prefix_match() {
        use biomeos_types::PrimalId;

        let primals = vec![biomeos_core::DiscoveredPrimal {
            id: PrimalId::new_unchecked("beardog-1"),
            name: "BearDog".to_string(),
            primal_type: PrimalType::Security,
            version: Version::parse("1.0.0").expect("valid version"),
            health: HealthStatus::Healthy,
            capabilities: vec![Capability::from("crypto.encrypt")],
            endpoint: Endpoint::new("unix:///tmp/beardog.sock").expect("valid endpoint"),
            metadata: serde_json::json!({}),
            family_id: Some(FamilyId::new("family-1")),
        }];

        let state = create_test_state(primals);

        let request = DiscoverCapabilityRequest {
            capability: "crypto".to_string(),
        };
        let result = discover_capability(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.providers.len(), 1); // Should match crypto.encrypt
    }
}
