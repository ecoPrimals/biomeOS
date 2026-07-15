// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_capability_client_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/path/12345.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.crypto_sign(b"test").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed")
            || err.contains("connect")
            || err.contains("timeout")
            || err.contains("Connection"),
        "expected connection error, got: {err}"
    );
}

#[tokio::test]
async fn test_storage_put_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/456.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.storage_put("key", b"value").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_storage_get_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/789.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.storage_get("key").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_storage_exists_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/exists.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.storage_exists("key").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_http_request_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/http.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client
        .http_request("GET", "https://example.com", None, None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_compute_execute_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/compute.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.compute_execute("task", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_capability_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/discover.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.discover_capability("crypto").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_translations_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/list.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.list_translations().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_health_check_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/health.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.health_check("beardog").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_crypto_verify_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/verify.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.crypto_verify(b"data", b"sig", b"pubkey").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_crypto_hash_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/hash.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.crypto_hash(b"data", "sha256").await;
    assert!(result.is_err());
}
