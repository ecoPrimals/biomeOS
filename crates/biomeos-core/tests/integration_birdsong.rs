//! Integration tests for BirdSong client and adaptive HTTP client
//!
//! These tests verify:
//! - Adaptive client version detection
//! - BirdSong encryption/decryption roundtrips
//! - Error handling and edge cases
//! - Multi-family scenarios

#[cfg(test)]
mod integration_tests {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use biomeos_core::adaptive_client::BirdSongClient;
    use serde_json::json;
    use tokio::time::Duration;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    /// Test BirdSong client encryption with v2 API
    #[tokio::test]
    async fn test_birdsong_encrypt_v2_api() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();
        let plaintext = STANDARD.encode(b"test message");

        // Mock v2 API response (with wrapper)
        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "ciphertext": STANDARD.encode(b"encrypted_data"),
                    "family_id": "test-family"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let result = client.encrypt(plaintext, family_id).await;

        assert!(result.is_ok(), "Encryption should succeed");
        let ciphertext = result.unwrap();
        assert!(!ciphertext.is_empty());
    }

    /// Test BirdSong client with v1 API
    #[tokio::test]
    async fn test_birdsong_encrypt_v1_fallback() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();
        let plaintext = STANDARD.encode(b"test message");

        // Mock v1 API (actual path used by adaptive client)
        // Note: BearDog API wraps response in {success: true, data: {...}}
        Mock::given(method("POST"))
            .and(path("/api/v1/birdsong/encrypt_discovery"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "encrypted": STANDARD.encode(b"encrypted_data"),
                    "family_id": "test-family"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let result = client.encrypt(plaintext, family_id).await;

        assert!(result.is_ok(), "Should use v1 API and succeed");
    }

    /// Test BirdSong decryption with v2 API
    #[tokio::test]
    async fn test_birdsong_decrypt_v2_api() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();
        let ciphertext = STANDARD.encode(b"encrypted_data");

        // Mock v2 decrypt response
        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/decrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "plaintext": STANDARD.encode(b"test message"),
                    "family_id": "test-family"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let result = client.decrypt(ciphertext, family_id).await;

        assert!(result.is_ok(), "Decryption should succeed");
        let plaintext = result.unwrap();
        assert!(!plaintext.is_empty());

        // Verify we can decode the base64
        let decoded = STANDARD.decode(&plaintext).expect("Should be valid base64");
        assert_eq!(decoded, b"test message");
    }

    /// Test full encryption/decryption roundtrip
    #[tokio::test]
    async fn test_birdsong_full_roundtrip() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();
        let original_message = b"test message for roundtrip";
        let original_plaintext = STANDARD.encode(original_message);

        // Mock encrypt
        let ciphertext_value = STANDARD.encode(b"encrypted_data");

        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "ciphertext": ciphertext_value.clone(),
                    "family_id": "test-family"
                }
            })))
            .mount(&mock_server)
            .await;

        // Mock decrypt
        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/decrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "plaintext": original_plaintext.clone(),
                    "family_id": "test-family"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());

        // Encrypt
        let ciphertext = client
            .encrypt(original_plaintext.clone(), family_id.clone())
            .await
            .expect("Encryption should succeed");

        // Decrypt
        let recovered_plaintext = client
            .decrypt(ciphertext, family_id)
            .await
            .expect("Decryption should succeed");

        assert_eq!(recovered_plaintext, original_plaintext);
    }

    /// Test error handling for API errors
    #[tokio::test]
    async fn test_birdsong_api_error() {
        let mock_server = MockServer::start().await;
        let family_id = "wrong-family".to_string();
        let plaintext = STANDARD.encode(b"test message");

        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(403).set_body_json(json!({
                "success": false,
                "error": "Invalid family credentials"
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let result = client.encrypt(plaintext, family_id).await;

        assert!(result.is_err(), "Should fail with invalid family");
    }

    /// Test network error handling
    #[tokio::test]
    async fn test_birdsong_network_error() {
        // Use invalid endpoint
        let mut client = BirdSongClient::new("http://localhost:1".to_string());
        let family_id = "test-family".to_string();
        let plaintext = STANDARD.encode(b"test message");

        let result = client.encrypt(plaintext, family_id).await;

        assert!(result.is_err(), "Should fail with network error");
    }

    /// Test timeout handling
    #[tokio::test]
    async fn test_birdsong_timeout() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();
        let plaintext = STANDARD.encode(b"test message");

        // Mock slow response (longer than 30s timeout)
        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(35))
                    .set_body_json(json!({
                        "success": true,
                        "data": {
                            "ciphertext": "too_late",
                            "family_id": "test-family"
                        }
                    })),
            )
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let result = client.encrypt(plaintext, family_id).await;

        assert!(result.is_err(), "Should timeout");
    }

    /// Test malformed response handling
    #[tokio::test]
    async fn test_birdsong_malformed_response() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();
        let plaintext = STANDARD.encode(b"test message");

        // Mock invalid JSON response
        Mock::given(method("POST"))
            .and(path("/api/v1/birdsong/encrypt_discovery"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json at all"))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let result = client.encrypt(plaintext, family_id).await;

        assert!(result.is_err(), "Should fail with malformed response");
    }

    /// Test concurrent requests
    #[tokio::test]
    async fn test_birdsong_concurrent_requests() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();

        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "ciphertext": STANDARD.encode(b"encrypted"),
                    "family_id": "test-family"
                }
            })))
            .expect(10) // Expect 10 concurrent requests
            .mount(&mock_server)
            .await;

        let mut handles = vec![];
        for i in 0..10 {
            let endpoint = mock_server.uri().clone();
            let fid = family_id.clone();
            let plaintext = STANDARD.encode(format!("message {}", i).as_bytes());

            let handle = tokio::spawn(async move {
                let mut client = BirdSongClient::new(endpoint);
                client.encrypt(plaintext, fid).await
            });

            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results.len(), 10);
        assert!(
            results.iter().all(|r| r.is_ok()),
            "All concurrent requests should succeed"
        );
    }

    /// Test version detection and caching
    #[tokio::test]
    async fn test_birdsong_version_caching() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();

        // First call: v2 succeeds
        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "ciphertext": STANDARD.encode(b"encrypted1"),
                    "family_id": "test-family"
                }
            })))
            .expect(2) // Should be called twice
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());

        // First call
        let result1 = client
            .encrypt(STANDARD.encode(b"message1"), family_id.clone())
            .await;
        assert!(result1.is_ok());

        // Second call should use same version (no fallback)
        let result2 = client
            .encrypt(STANDARD.encode(b"message2"), family_id)
            .await;
        assert!(result2.is_ok());
    }

    /// Test multi-family support
    #[tokio::test]
    async fn test_birdsong_multi_family() {
        let mock_server = MockServer::start().await;
        let family1 = "family-alpha".to_string();
        let family2 = "family-beta".to_string();

        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "ciphertext": STANDARD.encode(b"encrypted"),
                    "family_id": "dynamic"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());

        // Encrypt for family 1
        let result1 = client
            .encrypt(STANDARD.encode(b"message for family1"), family1)
            .await;
        assert!(result1.is_ok());

        // Encrypt for family 2
        let result2 = client
            .encrypt(STANDARD.encode(b"message for family2"), family2)
            .await;
        assert!(result2.is_ok());

        // Both should succeed
        assert!(result1.is_ok() && result2.is_ok());
    }

    /// Test base64 encoding/decoding with binary data
    #[tokio::test]
    async fn test_birdsong_base64_binary_data() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();

        // Test with binary data (non-UTF8)
        let binary_data: Vec<u8> = vec![0xFF, 0xFE, 0xFD, 0x00, 0x01, 0x02];
        let plaintext = STANDARD.encode(&binary_data);

        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "ciphertext": STANDARD.encode(&binary_data),
                    "family_id": "test-family"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let result = client.encrypt(plaintext, family_id).await;

        assert!(result.is_ok(), "Should handle binary data");
        let ciphertext = result.unwrap();

        // Verify base64 decoding works
        let decoded = STANDARD.decode(&ciphertext).expect("Should decode base64");
        assert_eq!(decoded, binary_data);
    }

    /// Benchmark: Measure typical encryption latency
    #[tokio::test]
    #[ignore] // Run with --ignored for benchmarks
    async fn bench_birdsong_encryption_latency() {
        let mock_server = MockServer::start().await;
        let family_id = "test-family".to_string();

        Mock::given(method("POST"))
            .and(path("/api/v2/birdsong/encrypt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "data": {
                    "ciphertext": STANDARD.encode(b"encrypted"),
                    "family_id": "test-family"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = BirdSongClient::new(mock_server.uri());
        let iterations = 100;
        let plaintext = STANDARD.encode(b"benchmark message");

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = client.encrypt(plaintext.clone(), family_id.clone()).await;
        }
        let elapsed = start.elapsed();

        let avg_latency = elapsed / iterations;
        println!("Average encryption latency: {:?}", avg_latency);

        // Should be under 100ms for local mock server
        assert!(
            avg_latency < Duration::from_millis(100),
            "Encryption should be fast"
        );
    }
}
