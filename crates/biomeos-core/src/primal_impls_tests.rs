use super::*;

#[test]
fn test_builder_pattern() {
    let primal = PrimalBuilder::new()
        .id("test-primal".to_string())
        .binary_path("/bin/true".to_string())
        .provides(vec![Capability::Security])
        .requires(vec![])
        .http_port(9000)
        .build()
        .unwrap();

    assert_eq!(primal.id().to_string(), "test-primal");
    assert_eq!(primal.provides(), &[Capability::Security]);
    assert_eq!(primal.requires().len(), 0);
}

#[test]
fn test_builder_without_binary_path_fails() {
    let result = PrimalBuilder::new()
        .id("test".to_string())
        .provides(vec![Capability::Security])
        .build();
    match result {
        Ok(_) => panic!("Expected build to fail without binary path"),
        Err(e) => {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("Binary path") || err_msg.contains("PRIMAL_BINARY"),
                "Expected binary path error, got: {err_msg}"
            );
        }
    }
}

#[test]
fn test_builder_default_id_when_not_set() {
    let primal = PrimalBuilder::new()
        .binary_path("/bin/true".to_string())
        .provides(vec![])
        .requires(vec![])
        .build()
        .unwrap();
    assert!(!primal.id().to_string().is_empty());
}

#[test]
fn test_convenience_functions() {
    let security = create_security_provider("/path/to/beardog".to_string(), 9000).unwrap();
    assert_eq!(security.provides(), &[Capability::Security]);

    let discovery = create_discovery_orchestrator("/path/to/songbird".to_string()).unwrap();
    assert_eq!(discovery.provides(), &[Capability::Discovery]);
    assert_eq!(discovery.requires(), &[Capability::Security]);
}

#[test]
fn test_capability_composition() {
    let ai_service = create_ai_service("/path/to/squirrel".to_string(), 8080).unwrap();
    assert_eq!(ai_service.provides(), &[Capability::AI]);
    assert!(ai_service.requires().contains(&Capability::Compute));
    assert!(ai_service.requires().contains(&Capability::Storage));
}

#[test]
fn test_create_storage_provider() {
    let storage = create_storage_provider("/path/to/nestgate".to_string(), 8002).unwrap();
    assert_eq!(storage.provides(), &[Capability::Storage]);
    assert_eq!(storage.requires().len(), 0);
}

#[test]
fn test_create_compute_provider() {
    let compute = create_compute_provider("/path/to/toadstool".to_string(), 8080).unwrap();
    assert_eq!(compute.provides(), &[Capability::Compute]);
}

#[test]
fn test_primal_builder_env_var() {
    let primal = PrimalBuilder::new()
        .id("env-test".to_string())
        .binary_path("/bin/true".to_string())
        .env_var("CUSTOM_VAR".to_string(), "value".to_string())
        .build()
        .unwrap();
    assert_eq!(primal.id().to_string(), "env-test");
}

#[test]
fn test_legacy_type_aliases() {
    let _beardog: Arc<GenericManagedPrimal> =
        create_security_provider("/bin/true".to_string(), 9000).unwrap();
    let _songbird: Arc<GenericManagedPrimal> =
        create_discovery_orchestrator("/bin/true".to_string()).unwrap();
}

#[tokio::test]
async fn test_endpoint_unix_socket_preferred() {
    let primal = PrimalBuilder::new()
        .id("test".to_string())
        .binary_path("/bin/true".to_string())
        .http_port(9000)
        .unix_socket_path("/run/user/1000/biomeos/test.sock")
        .build()
        .unwrap();
    let endpoint = primal.endpoint().await;
    assert!(endpoint.is_some());
    let ep = endpoint.unwrap();
    assert!(ep.to_string().contains("unix://"));
}

#[tokio::test]
async fn test_endpoint_http_removed() {
    let primal = PrimalBuilder::new()
        .id("test".to_string())
        .binary_path("/bin/true".to_string())
        .http_port(9000)
        .build()
        .unwrap();
    let endpoint = primal.endpoint().await;
    assert!(
        endpoint.is_none(),
        "HTTP transport is removed; should return None"
    );
}

#[tokio::test]
async fn test_health_check_no_process() {
    let primal = PrimalBuilder::new()
        .id("test".to_string())
        .binary_path("/bin/true".to_string())
        .build()
        .unwrap();
    let status = primal.health_check().await.unwrap();
    assert_eq!(status, HealthStatus::Unhealthy);
}

#[tokio::test]
async fn test_startup_timeout_default() {
    let primal = PrimalBuilder::new()
        .id("test".to_string())
        .binary_path("/bin/true".to_string())
        .build()
        .unwrap();
    let timeout = primal.startup_timeout();
    assert_eq!(timeout, Duration::from_secs(30));
}

#[tokio::test]
async fn test_startup_timeout_custom() {
    let primal = PrimalBuilder::new()
        .id("test".to_string())
        .binary_path("/bin/true".to_string())
        .env_var("PRIMAL_STARTUP_TIMEOUT".to_string(), "60".to_string())
        .build()
        .unwrap();
    let timeout = primal.startup_timeout();
    assert_eq!(timeout, Duration::from_secs(60));
}

#[tokio::test]
async fn test_with_config_invalid_id() {
    let config = PrimalConfig {
        id: String::new(),
        binary_path: "/bin/true".to_string(),
        provides: vec![],
        requires: vec![],
        http_port: 0,
        env_config: std::collections::HashMap::new(),
    };
    let result = GenericManagedPrimal::with_config(config);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_provides_requires_accessors() {
    let primal = PrimalBuilder::new()
        .id("test".to_string())
        .binary_path("/bin/true".to_string())
        .provides(vec![Capability::Security, Capability::Discovery])
        .requires(vec![Capability::Compute])
        .build()
        .unwrap();
    assert_eq!(primal.provides().len(), 2);
    assert!(primal.provides().contains(&Capability::Security));
    assert_eq!(primal.requires().len(), 1);
    assert!(primal.requires().contains(&Capability::Compute));
}
