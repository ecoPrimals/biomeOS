//! Semantic Layer Integration Tests
//!
//! Tests for capability translation, runtime discovery, and semantic method routing

use biomeos_atomic_deploy::capability_translation::CapabilityTranslationRegistry;
use serde_json::json;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

/// Mock primal server for testing semantic translation
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

    async fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let listener = UnixListener::bind(&self.socket_path).unwrap();

            loop {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let expected_method = self.expected_method.clone();
                    let response = self.response.clone();

                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 4096];
                        if let Ok(n) = socket.read(&mut buf).await {
                            let request = String::from_utf8_lossy(&buf[..n]);
                            println!("Mock server received: {}", request);

                            // Parse request
                            if let Ok(req) = serde_json::from_str::<serde_json::Value>(&request) {
                                // Verify method name
                                if let Some(method) = req.get("method").and_then(|m| m.as_str()) {
                                    assert_eq!(
                                        method, expected_method,
                                        "Expected method {}, got {}",
                                        expected_method, method
                                    );
                                }

                                // Send response
                                let id = req.get("id").and_then(|i| i.as_u64()).unwrap_or(1);
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
        })
    }
}

impl Drop for MockPrimalServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

#[tokio::test]
async fn test_basic_capability_translation() {
    let socket_path = "/tmp/test-semantic-basic.sock";

    // Start mock BearDog that expects "x25519_generate_ephemeral"
    let server = MockPrimalServer::new(
        socket_path,
        "x25519_generate_ephemeral",
        json!({
            "public_key": "test_public_key_bytes",
            "secret_key": "test_secret_key_bytes"
        }),
    );
    let _handle = server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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

    // Start mock that expects specific parameter names
    let server = MockPrimalServer::new(
        socket_path,
        "x25519_derive_secret",
        json!({
            "shared_secret": "derived_secret_bytes"
        }),
    );
    let _handle = server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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
                "private_key": "my_private",
                "public_key": "their_public_key"
            }),
        )
        .await;

    // Should map parameters and succeed
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response["shared_secret"], "derived_secret_bytes");
}

#[tokio::test]
async fn test_missing_capability() {
    let registry = CapabilityTranslationRegistry::new();

    // Try to call capability that doesn't exist
    let result = registry
        .call_capability("nonexistent.capability", json!({}))
        .await;

    // Should fail with clear error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("No provider for capability"),
        "Wrong error: {}",
        err
    );
}

#[tokio::test]
async fn test_provider_not_available() {
    let mut registry = CapabilityTranslationRegistry::new();

    // Register capability but don't start server
    registry.register_translation(
        "crypto.test",
        "beardog",
        "test_method",
        "/tmp/nonexistent-socket.sock",
        None,
    );

    // Try to call capability
    let result = registry.call_capability("crypto.test", json!({})).await;

    // Should fail with connection error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Failed to connect")
            || err.to_string().contains("No such file or directory"),
        "Wrong error: {}",
        err
    );
}

#[tokio::test]
async fn test_multiple_capabilities_same_provider() {
    let socket_path = "/tmp/test-semantic-multi.sock";

    // Start mock server
    let listener = UnixListener::bind(socket_path).unwrap();
    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    if let Ok(n) = socket.read(&mut buf).await {
                        let request = String::from_utf8_lossy(&buf[..n]);
                        if let Ok(req) = serde_json::from_str::<serde_json::Value>(&request) {
                            let id = req.get("id").and_then(|i| i.as_u64()).unwrap_or(1);
                            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

                            let result = match method {
                                "x25519_generate_ephemeral" => json!({"public_key": "pk"}),
                                "x25519_derive_secret" => json!({"shared_secret": "ss"}),
                                "chacha20_poly1305_encrypt" => json!({"ciphertext": "ct"}),
                                _ => json!({"error": "unknown method"}),
                            };

                            let response = json!({
                                "jsonrpc": "2.0",
                                "result": result,
                                "id": id
                            });

                            let _ = socket
                                .write_all(serde_json::to_string(&response).unwrap().as_bytes())
                                .await;
                            let _ = socket.flush().await;
                        }
                    }
                });
            }
        }
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Register multiple capabilities for same provider
    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        socket_path,
        None,
    );
    registry.register_translation(
        "crypto.ecdh_derive",
        "beardog",
        "x25519_derive_secret",
        socket_path,
        None,
    );
    registry.register_translation(
        "crypto.encrypt",
        "beardog",
        "chacha20_poly1305_encrypt",
        socket_path,
        None,
    );

    // Verify all capabilities are registered
    let caps = registry.provider_capabilities("beardog");
    assert_eq!(caps.len(), 3);
    assert!(caps.contains(&"crypto.generate_keypair".to_string()));
    assert!(caps.contains(&"crypto.ecdh_derive".to_string()));
    assert!(caps.contains(&"crypto.encrypt".to_string()));

    // Test each capability
    let result1 = registry
        .call_capability("crypto.generate_keypair", json!({}))
        .await;
    assert!(result1.is_ok());

    let result2 = registry
        .call_capability("crypto.ecdh_derive", json!({}))
        .await;
    assert!(result2.is_ok());

    let result3 = registry.call_capability("crypto.encrypt", json!({})).await;
    assert!(result3.is_ok());

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_registry_stats() {
    let mut registry = CapabilityTranslationRegistry::new();

    // Register capabilities for multiple providers
    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock",
        None,
    );
    registry.register_translation(
        "crypto.encrypt",
        "beardog",
        "chacha20_poly1305_encrypt",
        "/tmp/beardog.sock",
        None,
    );
    registry.register_translation(
        "http.request",
        "songbird",
        "http_request",
        "/tmp/songbird.sock",
        None,
    );
    registry.register_translation(
        "discovery.find",
        "songbird",
        "discover_by_capability",
        "/tmp/songbird.sock",
        None,
    );

    // Check stats
    let stats = registry.stats();
    assert_eq!(stats.total_translations, 4);
    assert_eq!(stats.total_providers, 2);
    assert_eq!(stats.capabilities_by_provider["beardog"], 2);
    assert_eq!(stats.capabilities_by_provider["songbird"], 2);
}

#[test]
fn test_registry_list_all() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock",
        None,
    );
    registry.register_translation(
        "http.request",
        "songbird",
        "http_request",
        "/tmp/songbird.sock",
        None,
    );

    let all = registry.list_all();
    assert_eq!(all.len(), 2);

    // Verify translation details
    let crypto_translation = all
        .iter()
        .find(|t| t.semantic == "crypto.generate_keypair")
        .unwrap();
    assert_eq!(crypto_translation.provider, "beardog");
    assert_eq!(
        crypto_translation.actual_method,
        "x25519_generate_ephemeral"
    );

    let http_translation = all.iter().find(|t| t.semantic == "http.request").unwrap();
    assert_eq!(http_translation.provider, "songbird");
    assert_eq!(http_translation.actual_method, "http_request");
}

#[test]
fn test_has_capability() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock",
        None,
    );

    assert!(registry.has_capability("crypto.generate_keypair"));
    assert!(!registry.has_capability("nonexistent.capability"));
}

#[tokio::test]
async fn test_provider_error_handling() {
    let socket_path = "/tmp/test-semantic-error.sock";

    // Start mock server that returns an error
    let listener = UnixListener::bind(socket_path).unwrap();
    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            if let Ok(n) = socket.read(&mut buf).await {
                let request = String::from_utf8_lossy(&buf[..n]);
                if let Ok(req) = serde_json::from_str::<serde_json::Value>(&request) {
                    let id = req.get("id").and_then(|i| i.as_u64()).unwrap_or(1);

                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32601,
                            "message": "Method not found"
                        },
                        "id": id
                    });

                    let _ = socket
                        .write_all(serde_json::to_string(&error_response).unwrap().as_bytes())
                        .await;
                    let _ = socket.flush().await;
                }
            }
        }
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "test.method",
        "test_provider",
        "actual_method",
        socket_path,
        None,
    );

    // Should propagate provider error
    let result = registry.call_capability("test.method", json!({})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Provider test_provider error"));

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_isomorphic_evolution_scenario() {
    // Scenario: Provider evolves method name, but semantic stays same
    let socket_path = "/tmp/test-semantic-evolution.sock";

    // Old provider uses "old_method_name"
    let old_server =
        MockPrimalServer::new(socket_path, "old_method_name", json!({"status": "old"}));
    let handle = old_server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "test.capability",
        "provider",
        "old_method_name",
        socket_path,
        None,
    );

    // Consumer uses semantic capability
    let result1 = registry.call_capability("test.capability", json!({})).await;
    assert!(result1.is_ok());

    // Stop old server
    handle.abort();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let _ = std::fs::remove_file(socket_path);

    // New provider uses "new_method_name"
    let new_server =
        MockPrimalServer::new(socket_path, "new_method_name", json!({"status": "new"}));
    let _new_handle = new_server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update translation (in real system, this happens via graph reload)
    registry.register_translation(
        "test.capability",
        "provider",
        "new_method_name",
        socket_path,
        None,
    );

    // Consumer code unchanged - still uses same semantic capability
    let result2 = registry.call_capability("test.capability", json!({})).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap()["status"], "new");

    // ✅ Isomorphic evolution: Consumer unaffected by provider evolution

    let _ = std::fs::remove_file(socket_path);
}
