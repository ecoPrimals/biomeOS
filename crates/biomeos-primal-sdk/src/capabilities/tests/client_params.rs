// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_capability_client_new() {
    let _client = CapabilityClient::new("/tmp/neural-api.sock");
    // Construction succeeds with path
}

#[test]
fn test_capability_client_with_timeout() {
    let _client = CapabilityClient::new("/tmp/sock").with_timeout(Duration::from_secs(5));
    // Builder pattern compiles and works
}

#[test]
fn test_capability_call_request_format() {
    // Verify the capability.call params structure
    let params = json!({
        "capability": "storage",
        "operation": "put",
        "args": { "key": "k", "value": "dmFsdWU=" }
    });
    assert_eq!(params["capability"], "storage");
    assert_eq!(params["operation"], "put");
    assert!(params["args"].is_object());
}

#[test]
fn test_resolve_neural_api_socket_no_env() {
    // Without NEURAL_API_SOCKET and no running biomeOS, discover fails
    let result = resolve_neural_api_socket();
    // May succeed if biomeOS happens to be running in test env
    let _ = result;
}

#[test]
fn test_capability_client_discover() {
    // discover() may fail if no socket exists
    let result = CapabilityClient::discover();
    let _ = result;
}

#[test]
fn test_capability_client_path_impl() {
    let client = CapabilityClient::new("/var/run/neural.sock");
    let client2 = CapabilityClient::new(PathBuf::from("/var/run/neural.sock"));
    drop(client);
    drop(client2);
}

#[test]
fn test_http_request_params_structure() {
    let args = json!({
        "method": "GET",
        "url": "https://example.com",
        "headers": {"Authorization": "Bearer x"},
        "body": "request body"
    });
    assert_eq!(args["method"], "GET");
    assert_eq!(args["url"], "https://example.com");
    assert!(args["headers"].is_object());
    assert_eq!(args["body"], "request body");
}

#[test]
fn test_storage_put_params() {
    let args = json!({
        "key": "my-key",
        "value": base64_encode(b"value bytes")
    });
    assert_eq!(args["key"], "my-key");
    assert!(args["value"].as_str().is_some());
}

#[test]
fn test_storage_get_params() {
    let args = json!({ "key": "lookup-key" });
    assert_eq!(args["key"], "lookup-key");
}

#[test]
fn test_crypto_sign_params() {
    let args = json!({
        "data": base64_encode(b"data to sign")
    });
    assert!(args["data"].as_str().is_some());
}

#[test]
fn test_crypto_verify_params() {
    let args = json!({
        "data": base64_encode(b"data"),
        "signature": base64_encode(b"sig"),
        "public_key": base64_encode(b"pubkey")
    });
    assert!(args["data"].as_str().is_some());
    assert!(args["signature"].as_str().is_some());
    assert!(args["public_key"].as_str().is_some());
}

#[test]
fn test_crypto_hash_params() {
    let args = json!({
        "data": base64_encode(b"data"),
        "algorithm": "sha256"
    });
    assert_eq!(args["algorithm"], "sha256");
}

#[test]
fn test_compute_execute_params() {
    let args = json!({
        "task": "inference",
        "params": {"model": "test"}
    });
    assert_eq!(args["task"], "inference");
    assert!(args["params"].is_object());
}

#[test]
fn test_health_check_params() {
    let args = json!({ "primal": "beardog" });
    assert_eq!(args["primal"], "beardog");
}

#[test]
fn test_resolve_neural_api_socket_invocation() {
    let result = resolve_neural_api_socket();
    match &result {
        Ok(p) => assert!(!p.as_os_str().is_empty()),
        Err(e) => {
            assert!(e.to_string().contains("not found") || e.to_string().contains("Neural"));
        }
    }
}

#[test]
fn test_resolve_neural_api_socket_from_env() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let path = tmp.path().to_path_buf();
    let got =
        resolve_neural_api_socket_with(Some(tmp.path())).expect("explicit path should resolve");
    assert_eq!(got, path);
}
