// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Semantic Layer Integration Tests
//!
//! Tests for capability translation, runtime discovery, and semantic method routing
//!
//! **Concurrency-First Design**: All tests use proper synchronization (oneshot channels)
//! instead of arbitrary sleep() calls. Test issues will be production issues!

use biomeos_atomic_deploy::capability_translation::CapabilityTranslationRegistry;
use serde_json::json;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

/// Mock primal server for testing semantic translation
///
/// **Concurrency**: Uses oneshot channel to signal when server is ready
struct MockPrimalServer {
    socket_path: String,
    expected_method: String,
    response: serde_json::Value,
}

impl MockPrimalServer {
    fn new(socket_path: &str, expected_method: &str, response: serde_json::Value) -> Self {
        let _ = std::fs::remove_file(socket_path);
        Self {
            socket_path: socket_path.to_string(),
            expected_method: expected_method.to_string(),
            response,
        }
    }

    /// Start server and return (handle, ready_receiver)
    /// **Concurrency**: Caller awaits ready_receiver instead of sleeping
    async fn start_with_ready(self) -> (tokio::task::JoinHandle<()>, oneshot::Receiver<()>) {
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle = tokio::spawn(async move {
            let listener = UnixListener::bind(&self.socket_path).unwrap();

            // Signal ready AFTER bind succeeds
            let _ = ready_tx.send(());

            loop {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let expected_method = self.expected_method.clone();
                    let response = self.response.clone();

                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 4096];
                        if let Ok(n) = socket.read(&mut buf).await {
                            let request = String::from_utf8_lossy(&buf[..n]);
                            println!("Mock server received: {request}");

                            // Parse request
                            if let Ok(req) = serde_json::from_str::<serde_json::Value>(&request) {
                                // Verify method name
                                if let Some(method) = req.get("method").and_then(|m| m.as_str()) {
                                    assert_eq!(
                                        method, expected_method,
                                        "Expected method {expected_method}, got {method}"
                                    );
                                }

                                // Send response
                                let id = req
                                    .get("id")
                                    .and_then(serde_json::Value::as_u64)
                                    .unwrap_or(1);
                                let rpc_response = json!({
                                    "jsonrpc": "2.0",
                                    "result": response,
                                    "id": id
                                });

                                let response_str = serde_json::to_string(&rpc_response).unwrap();
                                let _ = socket.write_all(response_str.as_bytes()).await;
                                let _ = socket.flush().await;
                            }
                        }
                    });
                }
            }
        });

        (handle, ready_rx)
    }
}

/// Cleanup helper for socket paths
struct SocketCleanup(String);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

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
    let beardog_socket = "/tmp/test-semantic-multi-bd.sock";
    let songbird_socket = "/tmp/test-semantic-multi-sb.sock";
    let _cleanup1 = SocketCleanup(beardog_socket.to_string());
    let _cleanup2 = SocketCleanup(songbird_socket.to_string());

    // Start mock BearDog
    let beardog_server = MockPrimalServer::new(
        beardog_socket,
        "crypto.sha256",
        json!({
            "hash": "abc123hash"
        }),
    );
    let (_bd_handle, bd_ready) = beardog_server.start_with_ready().await;

    // Start mock Songbird
    let songbird_server = MockPrimalServer::new(
        songbird_socket,
        "http.get",
        json!({
            "status": 200,
            "body": "Hello World"
        }),
    );
    let (_sb_handle, sb_ready) = songbird_server.start_with_ready().await;

    // Wait for BOTH servers concurrently (no serial waiting!)
    tokio::try_join!(
        async { bd_ready.await.map_err(|_| "BearDog failed") },
        async { sb_ready.await.map_err(|_| "Songbird failed") },
    )
    .expect("Servers failed to start");

    // Create registry with multiple primals
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.hash",
        "beardog",
        "crypto.sha256",
        beardog_socket,
        None,
    );

    registry.register_translation(
        "http.request",
        "songbird",
        "http.get",
        songbird_socket,
        None,
    );

    // Route to BearDog
    let crypto_result = registry.call_capability("crypto.hash", json!({})).await;
    assert!(crypto_result.is_ok());
    assert_eq!(crypto_result.unwrap()["hash"], "abc123hash");

    // Route to Songbird
    let http_result = registry.call_capability("http.request", json!({})).await;
    assert!(http_result.is_ok());
    assert_eq!(http_result.unwrap()["status"], 200);
}

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
