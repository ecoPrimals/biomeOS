// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use crate::{CapabilitySet, DiscoveredPrimal, PrimalDiscovery};
use biomeos_test_utils::MockJsonRpcServer;
use std::collections::HashMap;
use std::net::SocketAddr;

#[test]
fn test_security_client_creation_unix() {
    let client = SecurityProviderClient::with_endpoint("unix:///tmp/security-provider.sock")
        .expect("unix endpoint should parse");
    assert!(matches!(client.endpoint, SecurityEndpoint::UnixSocket(_)));
}

#[test]
fn test_security_client_creation_absolute_path() {
    let client = SecurityProviderClient::with_endpoint("/run/biomeos/security-provider.sock")
        .expect("absolute path should parse");
    assert!(matches!(client.endpoint, SecurityEndpoint::UnixSocket(_)));
}

#[test]
fn test_http_endpoint_rejected() {
    let result = SecurityProviderClient::with_endpoint("http://localhost:9000");
    assert!(result.is_err(), "HTTP endpoints should be rejected");
}

#[test]
fn test_invalid_endpoint() {
    let result = SecurityProviderClient::with_endpoint("invalid://endpoint");
    assert!(result.is_err(), "invalid scheme should fail");
}

#[test]
fn test_key_derivation_request_serialization() {
    let req = KeyDerivationRequest {
        parent_family: "family-1".to_string(),
        subfed_name: "gaming".to_string(),
        purpose: "encryption".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize KeyDerivationRequest");
    let restored: KeyDerivationRequest =
        serde_json::from_str(&json).expect("deserialize KeyDerivationRequest");
    assert_eq!(restored.parent_family, req.parent_family);
    assert_eq!(restored.subfed_name, req.subfed_name);
    assert_eq!(restored.purpose, req.purpose);
}

#[test]
fn test_key_derivation_response_serialization() {
    let resp = KeyDerivationResponse {
        key_ref: "key-ref-123".to_string(),
        algorithm: "AES-256-GCM".to_string(),
        created_at: "2026-01-15T12:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&resp).expect("serialize KeyDerivationResponse");
    let restored: KeyDerivationResponse =
        serde_json::from_str(&json).expect("deserialize KeyDerivationResponse");
    assert_eq!(restored.key_ref, resp.key_ref);
    assert_eq!(restored.algorithm, resp.algorithm);
}

#[test]
fn test_encrypt_response_serialization() {
    let resp = EncryptResponse {
        encrypted_data: "base64data".to_string(),
        nonce: "base64nonce".to_string(),
        tag: "base64tag".to_string(),
    };
    let json = serde_json::to_string(&resp).expect("serialize EncryptResponse");
    let restored: EncryptResponse =
        serde_json::from_str(&json).expect("deserialize EncryptResponse");
    assert_eq!(restored.encrypted_data, resp.encrypted_data);
    assert_eq!(restored.nonce, resp.nonce);
    assert_eq!(restored.tag, resp.tag);
}

#[test]
fn test_lineage_verification_request_serialization() {
    let req = LineageVerificationRequest {
        family_id: "family-1".to_string(),
        seed_hash: "sha256hash".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize LineageVerificationRequest");
    let restored: LineageVerificationRequest =
        serde_json::from_str(&json).expect("deserialize LineageVerificationRequest");
    assert_eq!(restored.family_id, req.family_id);
    assert_eq!(restored.seed_hash, req.seed_hash);
}

#[test]
fn test_lineage_verification_response_serialization_and_display() {
    let resp = LineageVerificationResponse {
        is_family_member: true,
        parent_seed_hash: "parent-hash".to_string(),
        relationship: "child".to_string(),
    };
    let json = serde_json::to_string(&resp).expect("serialize LineageVerificationResponse");
    let restored: LineageVerificationResponse =
        serde_json::from_str(&json).expect("deserialize LineageVerificationResponse");
    assert_eq!(restored.is_family_member, resp.is_family_member);
    assert_eq!(restored.relationship, resp.relationship);

    let display = resp.to_string();
    assert!(display.contains("member=true"));
    assert!(display.contains("relationship=child"));
    assert!(display.contains("parent_hash=parent-hash"));
}

#[test]
fn test_with_endpoint_unix_path() {
    let client = SecurityProviderClient::with_endpoint(
        "unix:///run/user/1000/biomeos/security-provider.sock",
    )
    .unwrap();
    assert!(matches!(client.endpoint, SecurityEndpoint::UnixSocket(_)));
}

#[test]
fn test_invalid_endpoint_ftp() {
    let result = SecurityProviderClient::with_endpoint("ftp://localhost/path");
    assert!(result.is_err());
}

#[test]
fn test_invalid_endpoint_empty() {
    let result = SecurityProviderClient::with_endpoint(String::new());
    assert!(result.is_err());
}

#[test]
fn test_key_derivation_request_clone() {
    let req = KeyDerivationRequest {
        parent_family: "fam".to_string(),
        subfed_name: "sub".to_string(),
        purpose: "encryption".to_string(),
    };
    let cloned = req.clone();
    assert_eq!(cloned.parent_family, req.parent_family);
}

#[test]
fn test_lineage_verification_response_not_member() {
    let resp = LineageVerificationResponse {
        is_family_member: false,
        parent_seed_hash: String::new(),
        relationship: "unknown".to_string(),
    };
    let display = resp.to_string();
    assert!(display.contains("member=false"));
    assert!(display.contains("unknown"));
}

#[tokio::test]
async fn test_security_provider_is_available_unix_nonexistent() {
    let client =
        SecurityProviderClient::with_endpoint("unix:///nonexistent/security/socket.sock").unwrap();
    assert!(!client.is_available());
}

#[tokio::test]
async fn test_security_provider_health_check_unix_nonexistent() {
    let client = SecurityProviderClient::with_endpoint("unix:///nonexistent/socket.sock").unwrap();
    let result = client.health_check().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_security_provider_http_rejected_at_construction() {
    let result = SecurityProviderClient::with_endpoint("http://localhost:9000");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("only supports Unix sockets")
    );
}

#[tokio::test]
async fn test_health_check_ok_status_via_mock_unix() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("security-provider-health.sock");
    let _srv =
        MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({ "status": "healthy" }))
            .await;
    let client = SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display()))
        .expect("client");
    assert!(client.is_available());
    client.health_check().await.expect("healthy");
}

#[tokio::test]
async fn test_health_check_ok_status_short() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("security-provider-ok.sock");
    let _srv =
        MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({ "status": "ok" })).await;
    let client =
        SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display())).unwrap();
    client.health_check().await.expect("ok");
}

#[tokio::test]
async fn test_health_check_unhealthy_status() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bd-bad.sock");
    let _srv =
        MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({ "status": "degraded" }))
            .await;
    let client =
        SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display())).unwrap();
    let e = client.health_check().await.expect_err("unhealthy");
    assert!(e.to_string().contains("unhealthy") || format!("{e:#}").contains("unhealthy"));
}

#[tokio::test]
async fn test_verify_same_family_via_mock() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bd-verify.sock");
    let _srv = MockJsonRpcServer::spawn_echo_success(
        &sock,
        serde_json::json!({
            "is_family_member": true,
            "parent_seed_hash": "ph",
            "relationship": "child"
        }),
    )
    .await;
    let client =
        SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display())).unwrap();
    let r = client
        .verify_same_family("f1", "sh", "n1")
        .await
        .expect("verify");
    assert!(r.is_family_member);
    assert_eq!(r.relationship, "child");
}

#[tokio::test]
async fn test_derive_subfed_key_via_mock() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bd-derive.sock");
    let _srv = MockJsonRpcServer::spawn_echo_success(
        &sock,
        serde_json::json!({
            "key_ref": "ref1",
            "algorithm": "AES-256-GCM",
            "created_at": "2026-01-01T00:00:00Z"
        }),
    )
    .await;
    let client =
        SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display())).unwrap();
    let r = client
        .derive_subfed_key(KeyDerivationRequest {
            parent_family: "p".to_string(),
            subfed_name: "s".to_string(),
            purpose: "enc".to_string(),
        })
        .await
        .expect("derive");
    assert_eq!(r.key_ref, "ref1");
}

#[tokio::test]
async fn test_encrypt_decrypt_roundtrip_via_mock() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bd-enc.sock");
    use base64::Engine;
    let enc = base64::engine::general_purpose::STANDARD.encode(b"hello");
    let _srv = MockJsonRpcServer::spawn(&sock, move |line| {
        let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
        let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = v["method"].as_str().unwrap_or("");
        let result = if method == "encryption.encrypt" {
            serde_json::json!({
                "encrypted_data": "e",
                "nonce": "n",
                "tag": "t"
            })
        } else if method == "encryption.decrypt" {
            serde_json::json!({ "data": enc })
        } else {
            serde_json::json!(null)
        };
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
            serde_json::to_string(&id).unwrap(),
            serde_json::to_string(&result).unwrap()
        )
    })
    .await;
    let client =
        SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display())).unwrap();
    let e = client
        .encrypt_data(b"hello", "kref")
        .await
        .expect("encrypt");
    assert_eq!(e.nonce, "n");
    let plain = client
        .decrypt_data(&e.encrypted_data, &e.nonce, &e.tag, "kref")
        .await
        .expect("decrypt");
    assert_eq!(plain.as_ref(), b"hello");
}

#[tokio::test]
async fn test_decrypt_invalid_base64_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bd-dec-bad.sock");
    let _srv = MockJsonRpcServer::spawn_echo_success(
        &sock,
        serde_json::json!({ "data": "!!!not-base64!!!" }),
    )
    .await;
    let client =
        SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display())).unwrap();
    let r = client.decrypt_data("x", "n", "t", "k").await;
    assert!(r.is_err());
}

#[test]
fn test_http_encrypt_rejected_at_construction() {
    let r = SecurityProviderClient::with_endpoint("http://localhost:1");
    assert!(r.is_err());
    assert!(
        r.unwrap_err()
            .to_string()
            .contains("only supports Unix sockets")
    );
}

#[tokio::test]
async fn test_health_check_no_status_field_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bd-h-empty.sock");
    let _srv = MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({})).await;
    let client =
        SecurityProviderClient::with_endpoint(format!("unix://{}", sock.display())).unwrap();
    client.health_check().await.expect("ok without status");
}

#[tokio::test]
async fn test_from_primal_discovery_unix_socket() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("security-provider-discovery.sock");
    let _l = std::os::unix::net::UnixListener::bind(&sock).expect("bind");
    let mut pd = PrimalDiscovery::new();
    pd.discovered_primals.insert(
        biomeos_types::primal_names::BEARDOG.into(),
        DiscoveredPrimal {
            name: biomeos_types::primal_names::BEARDOG.into(),
            primal_type: "security".into(),
            capabilities: CapabilitySet::new(),
            endpoints: vec![PrimalEndpoint::UnixSocket { path: sock }],
            metadata: HashMap::new(),
        },
    );
    let client = SecurityProviderClient::from_primal_discovery(&pd).expect("from discovery");
    assert!(client.is_available());
}

#[tokio::test]
async fn test_from_primal_discovery_udp_not_supported() {
    let addr: SocketAddr = "127.0.0.1:9".parse().expect("addr");
    let mut pd = PrimalDiscovery::new();
    pd.discovered_primals.insert(
        biomeos_types::primal_names::BEARDOG.into(),
        DiscoveredPrimal {
            name: biomeos_types::primal_names::BEARDOG.into(),
            primal_type: "security".into(),
            capabilities: CapabilitySet::new(),
            endpoints: vec![PrimalEndpoint::Udp { addr }],
            metadata: HashMap::new(),
        },
    );
    let r = SecurityProviderClient::from_primal_discovery(&pd);
    assert!(r.is_err(), "expected UDP endpoint to fail");
    let err = r.unwrap_err();
    let s = err.to_string().to_lowercase();
    assert!(s.contains("udp") || s.contains("unix sockets"), "{s}");
}

#[test]
fn test_https_rejected_at_construction() {
    let r = SecurityProviderClient::with_endpoint("https://localhost:1");
    assert!(r.is_err());
    assert!(
        r.unwrap_err()
            .to_string()
            .contains("only supports Unix sockets")
    );
}
