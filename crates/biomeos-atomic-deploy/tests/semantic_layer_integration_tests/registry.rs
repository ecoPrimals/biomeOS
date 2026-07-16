use super::*;

// ============================================================================
// TRANSLATION REGISTRY UNIT TESTS (no sockets needed)
// ============================================================================

#[tokio::test]
async fn test_registry_translation_lookup() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.sign",
        "beardog",
        "crypto.sign_ed25519",
        "/tmp/beardog.sock",
        None,
    );

    // Lookup should find translation
    let translation = registry.get_translation("crypto.sign");
    assert!(translation.is_some());
    let t = translation.unwrap();
    assert_eq!(t.actual_method, "crypto.sign_ed25519");
    assert_eq!(t.socket, "/tmp/beardog.sock");
}

#[tokio::test]
async fn test_registry_multiple_translations() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.sign",
        "beardog",
        "crypto.sign_ed25519",
        "/tmp/beardog.sock",
        None,
    );
    registry.register_translation(
        "crypto.verify",
        "beardog",
        "crypto.verify_ed25519",
        "/tmp/beardog.sock",
        None,
    );
    registry.register_translation(
        "http.get",
        "songbird",
        "http.get",
        "/tmp/songbird.sock",
        None,
    );
    registry.register_translation(
        "http.post",
        "songbird",
        "http.post",
        "/tmp/songbird.sock",
        None,
    );

    // All should be found
    assert!(registry.get_translation("crypto.sign").is_some());
    assert!(registry.get_translation("crypto.verify").is_some());
    assert!(registry.get_translation("http.get").is_some());
    assert!(registry.get_translation("http.post").is_some());

    // Unknown should not be found
    assert!(registry.get_translation("unknown.method").is_none());
}

#[tokio::test]
async fn test_registry_parameter_mapping_storage() {
    let mut registry = CapabilityTranslationRegistry::new();
    let mut param_mappings = HashMap::new();
    param_mappings.insert("our_key".to_string(), "their_key".to_string());

    registry.register_translation(
        "crypto.ecdh",
        "beardog",
        "x25519_derive",
        "/tmp/beardog.sock",
        Some(param_mappings.clone()),
    );

    let translation = registry.get_translation("crypto.ecdh").unwrap();
    // param_mappings is a HashMap, not Option<HashMap>
    assert!(!translation.param_mappings.is_empty());
    assert_eq!(
        translation.param_mappings.get("our_key"),
        Some(&"their_key".to_string())
    );
}

#[tokio::test]
async fn test_registry_error_handling() {
    let socket_path = "/tmp/test-semantic-error.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());

    // Start server that returns an error
    let (ready_tx, ready_rx) = oneshot::channel();
    let _handle = tokio::spawn({
        let socket_path = socket_path.to_string();
        async move {
            let _ = std::fs::remove_file(&socket_path);
            let listener = UnixListener::bind(&socket_path).unwrap();
            let _ = ready_tx.send(());

            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = socket.read(&mut buf).await {
                    let request = String::from_utf8_lossy(&buf[..n]);
                    if let Ok(req) = serde_json::from_str::<serde_json::Value>(&request) {
                        let id = req
                            .get("id")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(1);
                        // Return error response
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32000,
                                "message": "Test error"
                            },
                            "id": id
                        });
                        let _ = socket
                            .write_all(error_response.to_string().as_bytes())
                            .await;
                    }
                }
            }
        }
    });

    // Wait for server ready (no sleep!)
    ready_rx.await.expect("Server failed to start");

    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "test.error",
        "test_primal",
        "test_method",
        socket_path,
        None,
    );

    let result = registry.call_capability("test.error", json!({})).await;
    // Should propagate error from server
    assert!(result.is_err() || result.unwrap().get("error").is_some());
}

#[tokio::test]
async fn test_registry_concurrent_calls() {
    let socket_path = "/tmp/test-semantic-concurrent.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());

    // Start server that can handle multiple concurrent requests
    let server = MockPrimalServer::new(socket_path, "concurrent_test", json!({"success": true}));
    let (_handle, ready_rx) = server.start_with_ready().await;
    ready_rx.await.expect("Server failed to start");

    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "test.concurrent",
        "test_primal",
        "concurrent_test",
        socket_path,
        None,
    );

    // Make 10 concurrent calls
    let mut handles = Vec::new();
    for i in 0..10 {
        let reg = registry.clone();
        handles.push(tokio::spawn(async move {
            reg.call_capability("test.concurrent", json!({"call_id": i}))
                .await
        }));
    }

    // All should succeed - join all handles
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await);
    }
    for (i, result) in results.into_iter().enumerate() {
        let inner = result.expect("Task panicked");
        assert!(inner.is_ok(), "Call {} failed: {:?}", i, inner.err());
    }
}

#[tokio::test]
async fn test_translation_evolution_pattern() {
    let socket_path = "/tmp/test-semantic-evolution.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());

    // Start mock that simulates evolved API
    let server = MockPrimalServer::new(
        socket_path,
        "crypto.sign_ed25519_v2",
        json!({
            "signature": "evolved_signature",
            "algorithm": "ed25519",
            "version": 2
        }),
    );
    let (_handle, ready_rx) = server.start_with_ready().await;
    ready_rx.await.expect("Server failed to start");

    // Old client uses semantic name, gets routed to new API
    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "crypto.sign", // Old semantic name
        "beardog",
        "crypto.sign_ed25519_v2", // New actual method
        socket_path,
        None,
    );

    let result = registry
        .call_capability("crypto.sign", json!({"data": "test"}))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["algorithm"], "ed25519");
    assert_eq!(response["version"], 2);
}
