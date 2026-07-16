use super::*;

#[tokio::test]
async fn test_basic_capability_translation() {
    let socket_path = "/tmp/test-semantic-basic.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());

    // Start mock BearDog that expects "x25519_generate_ephemeral"
    let server = MockPrimalServer::new(
        socket_path,
        "x25519_generate_ephemeral",
        json!({
            "public_key": "test_public_key_bytes",
            "secret_key": "test_secret_key_bytes"
        }),
    );
    let (_handle, ready_rx) = server.start_with_ready().await;

    // Wait for server to be ready (deterministic, no sleep!)
    ready_rx.await.expect("Server failed to start");

    // Create registry and register translation
    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        socket_path,
        None,
    );

    // Call with SEMANTIC name
    let result = registry
        .call_capability("crypto.generate_keypair", json!({}))
        .await;

    // Should succeed with translation
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response["public_key"], "test_public_key_bytes");
}

#[tokio::test]
async fn test_parameter_mapping_translation() {
    let socket_path = "/tmp/test-semantic-params.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());

    // Start mock that expects specific parameter names
    let server = MockPrimalServer::new(
        socket_path,
        "x25519_derive_secret",
        json!({
            "shared_secret": "derived_secret_bytes"
        }),
    );
    let (_handle, ready_rx) = server.start_with_ready().await;

    // Wait for server to be ready (deterministic, no sleep!)
    ready_rx.await.expect("Server failed to start");

    // Create registry with parameter mappings
    let mut registry = CapabilityTranslationRegistry::new();
    let mut param_mappings = HashMap::new();
    param_mappings.insert("private_key".to_string(), "our_secret".to_string());
    param_mappings.insert("public_key".to_string(), "their_public".to_string());

    registry.register_translation(
        "crypto.ecdh_derive",
        "beardog",
        "x25519_derive_secret",
        socket_path,
        Some(param_mappings),
    );

    // Call with SEMANTIC parameter names
    let result = registry
        .call_capability(
            "crypto.ecdh_derive",
            json!({
                "private_key": "my_private_key",
                "public_key": "their_public_key"
            }),
        )
        .await;

    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_translation_not_found() {
    let registry = CapabilityTranslationRegistry::new();

    // Call unregistered capability
    let result = registry
        .call_capability("unknown.capability", json!({}))
        .await;

    // Should fail with clear error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not registered")
            || err.to_string().contains("No translation found")
            || err.to_string().contains("No provider"),
        "Expected 'not registered' or 'No provider' error, got: {err}"
    );
}

#[tokio::test]
async fn test_socket_connection_failure() {
    let mut registry = CapabilityTranslationRegistry::new();

    // Register translation to non-existent socket
    registry.register_translation(
        "test.method",
        "fake_primal",
        "actual_method",
        "/tmp/nonexistent-socket.sock",
        None,
    );

    // Call should fail gracefully
    let result = registry.call_capability("test.method", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_primals_routing() {
    let security_provider_socket = "/tmp/test-semantic-multi-bd.sock";
    let discovery_socket = "/tmp/test-semantic-multi-sb.sock";
    let _cleanup1 = SocketCleanup(security_provider_socket.to_string());
    let _cleanup2 = SocketCleanup(discovery_socket.to_string());

    // Start mock security provider
    let security_provider_server = MockPrimalServer::new(
        security_provider_socket,
        "crypto.sha256",
        json!({
            "hash": "abc123hash"
        }),
    );
    let (_bd_handle, bd_ready) = security_provider_server.start_with_ready().await;

    // Start mock discovery primal
    let discovery_server = MockPrimalServer::new(
        discovery_socket,
        "http.get",
        json!({
            "status": 200,
            "body": "Hello World"
        }),
    );
    let (_sb_handle, sb_ready) = discovery_server.start_with_ready().await;

    // Wait for BOTH servers concurrently (no serial waiting!)
    tokio::try_join!(
        async { bd_ready.await.map_err(|_| "security provider failed") },
        async { sb_ready.await.map_err(|_| "discovery primal failed") },
    )
    .expect("Servers failed to start");

    // Create registry with multiple primals
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.hash",
        "beardog",
        "crypto.sha256",
        security_provider_socket,
        None,
    );

    registry.register_translation(
        "http.request",
        "songbird",
        "http.get",
        discovery_socket,
        None,
    );

    // Route to security provider
    let crypto_result = registry.call_capability("crypto.hash", json!({})).await;
    assert!(crypto_result.is_ok());
    assert_eq!(crypto_result.unwrap()["hash"], "abc123hash");

    // Route to discovery primal
    let http_result = registry.call_capability("http.request", json!({})).await;
    assert!(http_result.is_ok());
    assert_eq!(http_result.unwrap()["status"], 200);
}
