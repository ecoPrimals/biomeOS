// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::coordinator::{NucleusClient, NucleusClientBuilder};
use super::family_seed::load_family_seed_from_storage_with;
use super::transport::call_unix_socket_rpc;
use crate::Endpoint;
use crate::EndpointType;
use crate::Error;
use crate::Registry;
use crate::Result;
use crate::capability::{CapabilityInfo, CapabilityLayer, CapabilityVerification};
use crate::discovery::{DiscoveredPrimal, DiscoveryRequest, PhysicalDiscovery};
use crate::identity::{IdentityLayer, IdentityProof, IdentityVerification};
use crate::trust::{TrustEvaluation, TrustLayer, TrustLevel};
use async_trait::async_trait;
use biomeos_test_utils::ready_signal;
use biomeos_types::CapabilityTaxonomy;
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use std::sync::Arc;

fn sample_endpoint() -> Endpoint {
    Endpoint {
        endpoint_type: EndpointType::UnixSocket,
        address: "/tmp/mock-primal.sock".to_string(),
    }
}

fn sample_proof(primal: &str) -> IdentityProof {
    IdentityProof {
        primal_name: primal.to_string(),
        node_id: "node-1".to_string(),
        family_id: "fam-1".to_string(),
        version: "1.0.0".to_string(),
        process_id: 1,
        socket_path: "/tmp/mock-primal.sock".to_string(),
        owner_uid: 1000,
        owner_gid: 1000,
        started_at: "now".to_string(),
        challenge: "c".to_string(),
        signature: "sig".to_string(),
    }
}

fn sample_discovered(primal: &str, endpoints: Vec<Endpoint>) -> DiscoveredPrimal {
    DiscoveredPrimal {
        primal: primal.to_string(),
        node_id: "node-1".to_string(),
        family_id: "fam-1".to_string(),
        capabilities: vec!["encryption".to_string()],
        endpoints,
        signature: "sig".to_string(),
        timestamp: "t".to_string(),
    }
}

struct MockPhysical {
    out: Vec<DiscoveredPrimal>,
}

#[async_trait]
impl PhysicalDiscovery for MockPhysical {
    async fn discover_by_capability(
        &self,
        _request: &DiscoveryRequest,
    ) -> Result<Vec<DiscoveredPrimal>> {
        Ok(self.out.clone())
    }

    async fn discover_by_family(&self, _family_id: &str) -> Result<Vec<DiscoveredPrimal>> {
        Ok(vec![])
    }

    async fn announce(&self, _primal_info: &DiscoveredPrimal) -> Result<()> {
        Ok(())
    }
}

struct MockIdentity {
    ok: bool,
    proof: IdentityProof,
}

#[async_trait]
impl IdentityLayer for MockIdentity {
    async fn request_proof(&self, _endpoint: &str, _challenge: &str) -> Result<IdentityProof> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_proof(&self, _proof: &IdentityProof) -> Result<IdentityVerification> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_identity(
        &self,
        _discovered: &DiscoveredPrimal,
    ) -> Result<IdentityVerification> {
        if self.ok {
            Ok(IdentityVerification {
                verified: true,
                proof: self.proof.clone(),
                message: "ok".to_string(),
            })
        } else {
            Err(Error::discovery_failed("identity failed", None))
        }
    }
}

struct MockCapability;

#[async_trait]
impl CapabilityLayer for MockCapability {
    async fn query_capabilities(&self, _endpoint: &str) -> Result<CapabilityInfo> {
        Ok(CapabilityInfo {
            primal: "p".to_string(),
            version: "1".to_string(),
            family_id: "f".to_string(),
            node_id: "n".to_string(),
            capabilities: vec![],
        })
    }

    async fn verify_capabilities(
        &self,
        _discovered: &DiscoveredPrimal,
        _identity: &IdentityProof,
    ) -> Result<CapabilityVerification> {
        Ok(CapabilityVerification {
            verified: true,
            expected: vec![],
            actual: vec![],
            message: "mock cap ok".to_string(),
        })
    }
}

struct MockCapabilityFail;

#[async_trait]
impl CapabilityLayer for MockCapabilityFail {
    async fn query_capabilities(&self, _endpoint: &str) -> Result<CapabilityInfo> {
        Err(Error::discovery_failed("cap query", None))
    }

    async fn verify_capabilities(
        &self,
        _discovered: &DiscoveredPrimal,
        _identity: &IdentityProof,
    ) -> Result<CapabilityVerification> {
        Err(Error::capability_mismatch(vec![], vec![]))
    }
}

struct MockIdentityAcceptName {
    accept: &'static str,
    proof: IdentityProof,
}

#[async_trait]
impl IdentityLayer for MockIdentityAcceptName {
    async fn request_proof(&self, _endpoint: &str, _challenge: &str) -> Result<IdentityProof> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_proof(&self, _proof: &IdentityProof) -> Result<IdentityVerification> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_identity(&self, discovered: &DiscoveredPrimal) -> Result<IdentityVerification> {
        if discovered.primal == self.accept {
            Ok(IdentityVerification {
                verified: true,
                proof: self.proof.clone(),
                message: "ok".to_string(),
            })
        } else {
            Err(Error::discovery_failed("identity skip", None))
        }
    }
}

struct MockTrust {
    err: bool,
}

#[async_trait]
impl TrustLayer for MockTrust {
    async fn evaluate_trust(
        &self,
        _discovered: &DiscoveredPrimal,
        _identity: &IdentityProof,
        _family_seed: &[u8],
    ) -> Result<TrustEvaluation> {
        if self.err {
            Err(Error::discovery_failed("trust err", None))
        } else {
            Ok(TrustEvaluation {
                level: TrustLevel::Verified,
                relationship: None,
                lineage_verified: true,
                message: "trusted".to_string(),
            })
        }
    }
}

fn test_client(
    primals: Vec<DiscoveredPrimal>,
    identity_ok: bool,
    primal_for_proof: &str,
    cap_fail: bool,
    trust_err: bool,
) -> NucleusClient {
    let proof = sample_proof(primal_for_proof);
    NucleusClient::from_layers_for_test(
        Arc::new(MockPhysical { out: primals }),
        Arc::new(MockIdentity {
            ok: identity_ok,
            proof,
        }),
        if cap_fail {
            Arc::new(MockCapabilityFail) as Arc<dyn CapabilityLayer>
        } else {
            Arc::new(MockCapability) as Arc<dyn CapabilityLayer>
        },
        Arc::new(MockTrust { err: trust_err }),
        Arc::new(Registry::new()),
    )
}

#[tokio::test]
async fn test_discover_happy_path_one_primal() {
    let p = sample_discovered("beardog", vec![sample_endpoint()]);
    let client = test_client(vec![p], true, "beardog", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "beardog");
    assert_eq!(out[0].trust_level, TrustLevel::Verified);
}

#[tokio::test]
async fn test_discover_empty_when_layer1_empty() {
    let client = test_client(vec![], true, "x", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert!(out.is_empty());
}

#[tokio::test]
async fn test_discover_skips_when_identity_fails() {
    let p = sample_discovered("p1", vec![sample_endpoint()]);
    let client = test_client(vec![p], false, "p1", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert!(out.is_empty());
}

#[tokio::test]
async fn test_discover_skips_when_capability_fails() {
    let p = sample_discovered("p1", vec![sample_endpoint()]);
    let client = test_client(vec![p], true, "p1", true, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert!(out.is_empty());
}

#[tokio::test]
async fn test_discover_uses_known_when_trust_fails() {
    let p = sample_discovered("p1", vec![sample_endpoint()]);
    let client = test_client(vec![p], true, "p1", false, true);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].trust_level, TrustLevel::Known);
}

#[tokio::test]
async fn test_discover_err_when_no_endpoints() {
    let p = sample_discovered("p1", vec![]);
    let client = test_client(vec![p], true, "p1", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let err = client.discover(req).await.unwrap_err();
    assert!(err.to_string().contains("No endpoints") || err.to_string().contains("endpoints"));
}

#[tokio::test]
async fn test_discover_second_primal_after_first_skipped() {
    let bad = sample_discovered("bad", vec![sample_endpoint()]);
    let good = sample_discovered("good", vec![sample_endpoint()]);
    let client = NucleusClient::from_layers_for_test(
        Arc::new(MockPhysical {
            out: vec![bad, good],
        }),
        Arc::new(MockIdentityAcceptName {
            accept: "good",
            proof: sample_proof("good"),
        }),
        Arc::new(MockCapability),
        Arc::new(MockTrust { err: false }),
        Arc::new(Registry::new()),
    );
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "good");
}

#[tokio::test]
async fn test_registry_accessor_from_injected_client() {
    let client = test_client(vec![], true, "x", false, false);
    let reg = client.registry();
    assert!(std::sync::Arc::strong_count(&reg) >= 1);
}

#[test]
fn test_nucleus_client_builder_default() {
    let _builder = NucleusClientBuilder::default();
}

#[test]
fn test_nucleus_client_builder_new() {
    let _builder = NucleusClientBuilder::new();
}

#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test_method"));
    assert!(
        request
            .id
            .as_ref()
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0
    );
}

#[test]
fn test_jsonrpc_response_deserialization() {
    let json = r#"{"jsonrpc":"2.0","result":{"success":true},"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id.as_u64().unwrap(), 1);
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_jsonrpc_error_response() {
    let json = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid request");
}

#[test]
fn test_jsonrpc_response_with_null_result() {
    let json = r#"{"jsonrpc":"2.0","result":null,"id":42}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
    assert!(
        response.result.is_none(),
        "serde maps JSON null to None for Option<Value>"
    );
    assert_eq!(response.id.as_u64().unwrap(), 42);
}

#[test]
fn test_jsonrpc_response_nested_result() {
    let json = r#"{"jsonrpc":"2.0","result":{"nested":{"value":123}},"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
    let result = response.result.unwrap();
    assert_eq!(result["nested"]["value"], 123);
}

#[test]
fn test_jsonrpc_request_params_empty_object() {
    let request = JsonRpcRequest::new("ping", serde_json::json!({}));
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"params\":{}"));
    assert!(
        request
            .id
            .as_ref()
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0
    );
}

#[test]
fn test_jsonrpc_request_params_nested() {
    let params = serde_json::json!({"capability": "encryption", "family_id": null});
    let request = JsonRpcRequest::new("discover", params);
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("discover"));
    assert!(json.contains("capability"));
}

#[test]
fn test_nucleus_client_builder_equality() {
    let b1 = NucleusClientBuilder::new();
    let b2 = NucleusClientBuilder::default();
    assert_eq!(std::mem::size_of_val(&b1), std::mem::size_of_val(&b2));
}

#[tokio::test]
async fn test_call_unix_socket_rpc_connection_refused() {
    let result = call_unix_socket_rpc::<serde_json::Value>(
        "/nonexistent/socket/path/12345.sock",
        "ping",
        serde_json::json!({}),
    )
    .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("socket")
            || err.to_string().contains("connect")
            || err.to_string().contains("Connection"),
        "expected connection error: {err}"
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_success() {
    #[derive(serde::Deserialize)]
    struct TestResult {
        success: bool,
        primal: String,
    }

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_test.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"success": true, "primal": "beardog"},
                "id": 1
            });
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<TestResult>(&socket_path, "test_method", serde_json::json!({}))
            .await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.success);
    assert_eq!(value.primal, "beardog");
}

#[tokio::test]
async fn test_call_unix_socket_rpc_jsonrpc_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_error.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": "Method not found"},
                "id": 1
            });
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result = call_unix_socket_rpc::<serde_json::Value>(
        &socket_path,
        "nonexistent",
        serde_json::json!({}),
    )
    .await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Method not found") || err_msg.contains("-32601"),
        "{}",
        err_msg
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_missing_result() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_missing.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": null,
                "id": 1
            });
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "test", serde_json::json!({}))
            .await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Missing") || err_msg.contains("result"),
        "{}",
        err_msg
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_non_json_response() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_bad_json.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let _ = tokio::io::AsyncWriteExt::write_all(&mut stream, b"NOT VALID JSON {{{\n").await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "ping", serde_json::json!({}))
            .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid JSON-RPC") || msg.contains("JSON") || msg.contains("invalid"),
        "{msg}"
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_result_deserialize_mismatch() {
    #[derive(Debug, serde::Deserialize)]
    struct NeedsField {
        // Field is never read: this test only exercises the Err deserialize path.
        #[expect(dead_code, reason = "serde expected shape; only Err path is taken")]
        required_only: String,
    }

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_shape.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"wrong": "shape"},
                "id": 1
            });
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result = call_unix_socket_rpc::<NeedsField>(&socket_path, "m", serde_json::json!({})).await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("deserialize")
            || msg.contains("Failed to deserialize")
            || msg.contains("missing"),
        "{msg}"
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_server_closes_without_response() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_eof.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            drop(stream);
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "ping", serde_json::json!({}))
            .await;
    assert!(result.is_err());
}

#[tokio::test(start_paused = true)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
async fn test_call_unix_socket_rpc_read_times_out_paused_clock() {
    use std::time::Duration;
    use tokio::io::AsyncReadExt;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_hang_paused.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = stream.read(&mut buf).await;
            std::future::pending::<()>().await;
        }
    });

    ready_rx.wait().await.expect("ready");

    let socket_path_clone = socket_path.clone();
    let handle = tokio::spawn(async move {
        call_unix_socket_rpc::<serde_json::Value>(&socket_path_clone, "hang", serde_json::json!({}))
            .await
    });

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(31)).await;

    let result = handle.await.expect("join");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("timeout")
            || msg.contains("30")
            || msg.contains("Timed out")
            || msg.contains("Socket read"),
        "{msg}"
    );
}

#[tokio::test]
#[ignore = "waits for 30s socket read timeout"]
async fn test_call_unix_socket_rpc_read_times_out() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_hang.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            // Never write a response — client should hit 30s timeout
            std::future::pending::<()>().await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "hang", serde_json::json!({}))
            .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("timeout") || msg.contains("30") || msg.contains("Timed out"),
        "{msg}"
    );
}

#[test]
fn test_get_family_seed_from_env_valid_base64() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let seed = b"family-seed-bytes";
    let b64 = STANDARD.encode(seed);
    let out = load_family_seed_from_storage_with(Some(b64.as_str()), None, false);
    assert_eq!(out.as_ref(), seed);
}

#[test]
fn test_get_family_seed_prefers_env_over_xdg_runtime() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let seed = b"env-wins";
    let b64 = STANDARD.encode(seed);
    let temp = tempfile::tempdir().expect("tempdir");
    let seed_path = temp.path().join("biomeos").join("family.seed");
    std::fs::create_dir_all(seed_path.parent().unwrap()).unwrap();
    std::fs::write(&seed_path, b"from-xdg-file").unwrap();

    let out = load_family_seed_from_storage_with(Some(b64.as_str()), Some(temp.path()), false);
    assert_eq!(out.as_ref(), seed);
}

#[test]
fn test_get_family_seed_invalid_base64_ignored() {
    let out = load_family_seed_from_storage_with(Some("@@@not-base64@@@"), None, false);
    assert!(out.is_empty());
}

#[test]
fn test_get_family_seed_from_xdg_file_when_env_unset() {
    let temp = tempfile::tempdir().expect("tempdir");
    let seed_path = temp.path().join("biomeos").join("family.seed");
    std::fs::create_dir_all(seed_path.parent().unwrap()).unwrap();
    std::fs::write(&seed_path, b"seed-from-xdg").unwrap();
    let out = load_family_seed_from_storage_with(None, Some(temp.path()), false);
    assert_eq!(out.as_ref(), b"seed-from-xdg");
}

#[test]
fn test_nucleus_client_builder_build_smoke() {
    // May fail without full stack (Songbird/BearDog paths) — exercise constructor path only when ok.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt");
    let res: std::result::Result<NucleusClient, _> =
        rt.block_on(async { NucleusClientBuilder::new().build().await });
    if let Ok(client) = res {
        let _ = client.registry();
    }
}

#[tokio::test]
async fn test_call_unix_socket_rpc_read_error_empty_after_headers() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("early_eof.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n",
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "m", serde_json::json!({})).await;
    assert!(result.is_err());
}

#[test]
fn test_get_family_seed_empty_when_no_sources() {
    let out = load_family_seed_from_storage_with(None, None, false);
    assert!(out.is_empty());
}
