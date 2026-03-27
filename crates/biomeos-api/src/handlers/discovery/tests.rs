use super::*;
use serial_test::serial;

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
        last_seen: 1_234_567_890,
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

#[tokio::test]
async fn test_probe_live_sockets_returns_vec() {
    let primals = probe_live_sockets().await;
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
        last_seen: 9_999_999_999,
        trust_level: Some(2),
        family_id: Some("family-1".to_string()),
        allowed_capabilities: Some(vec!["compute/*".to_string()]),
        denied_capabilities: Some(vec!["compute/admin".to_string()]),
    };

    let json = serde_json::to_string(&primal).expect("should serialize");
    let deserialized: DiscoveredPrimal = serde_json::from_str(&json).expect("should deserialize");
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
            last_seen: 1_234_567_890,
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
            last_seen: 1_234_567_891,
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

#[tokio::test]
async fn test_probe_live_sockets_correct_structure() {
    let primals = probe_live_sockets().await;
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
                    endpoint: Endpoint::new("unix:///tmp/healthy.sock").expect("valid endpoint"),
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
                    endpoint: Endpoint::new("unix:///tmp/degraded.sock").expect("valid endpoint"),
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
                    endpoint: Endpoint::new("unix:///tmp/unhealthy.sock").expect("valid endpoint"),
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
                    endpoint: Endpoint::new("unix:///tmp/unknown.sock").expect("valid endpoint"),
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

#[tokio::test]
#[serial]
async fn test_probe_live_sockets_with_sock_files_no_runtime() {
    use biomeos_test_utils::env_helpers::TestEnvGuard;

    let temp = tempfile::tempdir().expect("tempdir");
    let sock_dir = temp.path();

    std::fs::write(sock_dir.join("beardog-family1.sock"), "").expect("write");
    std::fs::write(sock_dir.join("songbird-family1.sock"), "").expect("write");
    std::fs::write(sock_dir.join("not-a-socket.txt"), "").expect("write");
    std::fs::write(sock_dir.join("another.log"), "").expect("write");

    let _guard = TestEnvGuard::set("PRIMAL_SOCKET", sock_dir.to_str().unwrap());

    let primals = probe_live_sockets().await;

    assert_eq!(primals.len(), 2, "should find exactly 2 .sock files");

    let mut names: Vec<&str> = primals.iter().map(|p| p.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["beardog", "songbird"]);

    for primal in &primals {
        assert_eq!(primal.health, "unreachable");
        assert_eq!(primal.version, "unknown");
        assert!(primal.capabilities.is_empty());
        assert_eq!(primal.trust_level, Some(1));
        assert_eq!(primal.primal_type, "probed");
        assert!(primal.endpoint.starts_with("unix://"));
        assert!(primal.id.ends_with("-probed"));
        assert!(primal.family_id.is_none());
    }
}

#[tokio::test]
#[serial]
async fn test_probe_live_sockets_controlled_empty_dir() {
    use biomeos_test_utils::env_helpers::TestEnvGuard;

    let temp = tempfile::tempdir().expect("tempdir");
    let _guard = TestEnvGuard::set("PRIMAL_SOCKET", temp.path().to_str().unwrap());

    let primals = probe_live_sockets().await;
    assert!(primals.is_empty());
}

#[tokio::test]
#[serial]
async fn test_probe_live_sockets_nonexistent_override_dir() {
    use biomeos_test_utils::env_helpers::TestEnvGuard;

    let _guard = TestEnvGuard::set("PRIMAL_SOCKET", "/nonexistent/probe/dir/xyz123");

    let primals = probe_live_sockets().await;
    assert!(primals.is_empty());
}

#[tokio::test]
#[serial]
async fn test_probe_live_sockets_extracts_primal_name_from_hyphenated_filename() {
    use biomeos_test_utils::env_helpers::TestEnvGuard;

    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("beardog-family-a.sock"), "").expect("write");
    std::fs::write(temp.path().join("simple.sock"), "").expect("write");

    let _guard = TestEnvGuard::set("PRIMAL_SOCKET", temp.path().to_str().unwrap());

    let primals = probe_live_sockets().await;
    assert_eq!(primals.len(), 2);

    let by_id: std::collections::HashMap<&str, &DiscoveredPrimal> =
        primals.iter().map(|p| (p.name.as_str(), p)).collect();

    assert!(
        by_id.contains_key("beardog"),
        "hyphenated name → first segment"
    );
    assert!(
        by_id.contains_key("simple"),
        "unhyphenated name → full stem"
    );
}

#[test]
#[serial]
fn test_get_socket_dir_respects_primal_socket_env() {
    use biomeos_test_utils::env_helpers::TestEnvGuard;

    let temp = tempfile::tempdir().expect("tempdir");
    let _guard = TestEnvGuard::set("PRIMAL_SOCKET", temp.path().to_str().unwrap());

    let dir = get_socket_dir();
    assert!(
        dir.starts_with(temp.path().to_str().unwrap()),
        "socket dir should use PRIMAL_SOCKET override: got {dir}"
    );
}
