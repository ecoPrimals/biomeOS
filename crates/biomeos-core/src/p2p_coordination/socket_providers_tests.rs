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
use crate::p2p_coordination::{
    DiscoveryMode, EncryptedDiscoveryConfig, HealthStatus, LineageInfo, LineageProof, RelayOffer,
    RelayStatus, TransportEndpoint,
};
use biomeos_test_utils::MockJsonRpcServer;
use bytes::Bytes;
use std::time::Duration;

#[test]
fn test_socket_rpc_client_creation() {
    let client = SocketRpcClient::new(PathBuf::from("/tmp/test.sock"));
    assert_eq!(client.socket_path(), &PathBuf::from("/tmp/test.sock"));
}

#[test]
fn test_socket_rpc_client_timeout() {
    let client =
        SocketRpcClient::new(PathBuf::from("/tmp/test.sock")).with_timeout(Duration::from_secs(30));
    assert_eq!(client.socket_path(), &PathBuf::from("/tmp/test.sock"));
}

#[test]
fn test_socket_security_provider_new() {
    let provider = SocketSecurityProvider::new(PathBuf::from("/run/security.sock"));
    assert_eq!(
        provider.rpc.socket_path(),
        &PathBuf::from("/run/security.sock")
    );
}

#[test]
fn test_socket_discovery_provider_new() {
    let provider = SocketDiscoveryProvider::new(PathBuf::from("/run/discovery.sock"));
    assert_eq!(
        provider.rpc.socket_path(),
        &PathBuf::from("/run/discovery.sock")
    );
}

#[test]
fn test_socket_routing_provider_new() {
    let provider = SocketRoutingProvider::new(PathBuf::from("/run/relay.sock"));
    assert_eq!(
        provider.rpc.socket_path(),
        &PathBuf::from("/run/relay.sock")
    );
}

#[test]
fn test_jsonrpc_request_format() {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "test.method",
        "params": {"key": "value"}
    });
    let bytes = serde_json::to_vec(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "test.method");
    assert_eq!(parsed["params"]["key"], "value");
}

#[test]
fn test_socket_rpc_client_call_nonexistent_socket() {
    let client = SocketRpcClient::new(PathBuf::from("/nonexistent/path/to/socket.sock"));
    let result = client.call("test.method", serde_json::json!({}));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed") || err.contains("connect") || err.contains("No such file"),
        "Expected connection error, got: {err}"
    );
}

#[tokio::test]
async fn test_socket_rpc_client_call_async_nonexistent() {
    let client = SocketRpcClient::new(PathBuf::from("/nonexistent/socket.sock"));
    let result = client
        .call_async("test.method", serde_json::json!({}))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_socket_rpc_call_success_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rpc.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({"answer": 42})).await;
    let client = SocketRpcClient::new(sock);
    let v = tokio::task::spawn_blocking(move || client.call("any.method", serde_json::json!({})))
        .await
        .expect("join")
        .expect("ok");
    assert_eq!(v["answer"], 42);
}

#[tokio::test]
async fn test_socket_rpc_call_jsonrpc_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rpc-err.sock");
    let _server = MockJsonRpcServer::spawn_echo_error(&sock, -32000, "boom").await;
    let client = SocketRpcClient::new(sock);
    let r = tokio::task::spawn_blocking(move || client.call("m", serde_json::json!({})))
        .await
        .expect("join");
    assert!(r.is_err());
    let chain = format!("{:#}", r.unwrap_err());
    assert!(chain.contains("boom"), "got {chain}");
}

#[tokio::test]
async fn test_socket_rpc_call_missing_result() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rpc-nores.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |req| {
        let id = serde_json::from_str::<serde_json::Value>(req.trim())
            .ok()
            .and_then(|v| v.get("id").cloned())
            .unwrap_or(serde_json::Value::Null);
        format!(r#"{{"jsonrpc":"2.0","id":{},"not_result":true}}"#, id)
    })
    .await;
    let client = SocketRpcClient::new(sock);
    let r = tokio::task::spawn_blocking(move || client.call("m", serde_json::json!({})))
        .await
        .expect("join");
    assert!(r.is_err());
    let chain = format!("{:#}", r.unwrap_err());
    assert!(chain.contains("No result"), "got {chain}");
}

fn sample_proof() -> LineageProof {
    LineageProof {
        lineage_id: "fam".to_string(),
        depth: 0,
        proof: Bytes::new(),
        timestamp: std::time::SystemTime::UNIX_EPOCH,
    }
}

#[tokio::test]
async fn test_socket_security_provider_tunnel_request_and_defaults() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("sec.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |line| {
        let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
        let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = v["method"].as_str().unwrap_or("");
        let result = match method {
            "tunnel.request" => serde_json::json!({
                "tunnel_id": "tid",
                "endpoint_a_address": "192.0.2.1",
                "endpoint_b_address": "192.0.2.2",
                "endpoint_a_port": 111,
                "endpoint_b_port": 222
            }),
            "tunnel.health" => serde_json::json!({"status": "degraded"}),
            "crypto.broadcast_keys" => serde_json::json!({"encryption_key": "abcd"}),
            "lineage.verify" => serde_json::json!({"is_ancestor": true, "depth": 3}),
            _ => serde_json::json!({}),
        };
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
            serde_json::to_string(&id).expect("id"),
            serde_json::to_string(&result).expect("result")
        )
    })
    .await;
    let p = SocketSecurityProvider::new(sock);
    let proof = sample_proof();
    let tunnel = p.request_tunnel("na", "nb", &proof).await.expect("tunnel");
    assert_eq!(tunnel.id, "tid");
    assert_eq!(tunnel.endpoint_a.port, 111);
    let health = p.check_tunnel_health("x").await.expect("health");
    assert_eq!(health.status, HealthStatus::Degraded);
    let keys = p.generate_broadcast_keys("fam").await.expect("keys");
    assert_eq!(keys.broadcast_key.as_ref(), b"abcd");
    let lin = p.verify_lineage("r", "t").await.expect("lin");
    assert!(lin.is_ancestor);
    assert_eq!(lin.depth, 3);
}

#[tokio::test]
async fn test_socket_security_tunnel_health_status_branches() {
    for (status_str, expected) in [
        ("healthy", HealthStatus::Healthy),
        ("degraded", HealthStatus::Degraded),
        ("broken", HealthStatus::Unhealthy),
    ] {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join(format!("h-{status_str}.sock"));
        let _server = MockJsonRpcServer::spawn_echo_success(
            &sock,
            serde_json::json!({ "status": status_str }),
        )
        .await;
        let p = SocketSecurityProvider::new(sock);
        let h = p.check_tunnel_health("t").await.expect("h");
        assert_eq!(h.status, expected);
    }
}

#[tokio::test]
async fn test_socket_discovery_provider_methods() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("disc.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |line| {
        let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
        let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = v["method"].as_str().unwrap_or("");
        let result = match method {
            "transport.health" => serde_json::json!({"status": "healthy", "latency_ms": 12}),
            "discovery.test_broadcast" => serde_json::json!({"encrypted": true, "success": true}),
            _ => serde_json::json!(null),
        };
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
            serde_json::to_string(&id).expect("id"),
            serde_json::to_string(&result).expect("result")
        )
    })
    .await;
    let p = SocketDiscoveryProvider::new(sock.clone());
    let ep = TransportEndpoint {
        node_id: "n".to_string(),
        address: "127.0.0.1".to_string(),
        port: 1,
        protocol: "tcp".to_string(),
        secure: true,
    };
    p.register_transport(&ep).await.expect("reg");
    let cfg = EncryptedDiscoveryConfig {
        encryption_key: Bytes::from("k"),
        lineage_filter: sample_proof(),
        mode: DiscoveryMode::Encrypted,
    };
    p.enable_encrypted_mode(cfg).await.expect("enc");
    let th = p.check_transport_health("t1").await.expect("th");
    assert_eq!(th.status, HealthStatus::Healthy);
    assert_eq!(th.latency_ms, Some(12));
    let bt = p.test_encrypted_broadcast().await.expect("bt");
    assert!(bt.encrypted && bt.success);
}

#[tokio::test]
async fn test_socket_routing_provider_relay_and_accept() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("route.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |line| {
        let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
        let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = v["method"].as_str().unwrap_or("");
        let result = match method {
            "relay.request" => serde_json::json!({
                "relay_node": "rn",
                "address": "10.0.0.9",
                "port": 7777
            }),
            "relay.accept" => serde_json::json!({
                "connection_id": "cid-1",
                "status": "establishing"
            }),
            _ => serde_json::json!(null),
        };
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
            serde_json::to_string(&id).expect("id"),
            serde_json::to_string(&result).expect("result")
        )
    })
    .await;
    let p = SocketRoutingProvider::new(sock);
    let lineage = LineageInfo {
        is_ancestor: true,
        depth: 1,
        proof: sample_proof(),
    };
    let offer = p.request_relay("a", "b", lineage).await.expect("offer");
    assert_eq!(offer.relay_endpoint.port, 7777);
    let conn = p.accept_relay(&offer).await.expect("accept");
    assert_eq!(conn.connection_id, "cid-1");
    assert_eq!(conn.status, RelayStatus::Establishing);
}

#[tokio::test]
async fn test_socket_routing_accept_status_failed_branch() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("r2.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        serde_json::json!({"connection_id": "x", "status": "dead"}),
    )
    .await;
    let p = SocketRoutingProvider::new(sock);
    let offer = RelayOffer {
        relay_node: "n".to_string(),
        relay_endpoint: TransportEndpoint {
            node_id: "n".to_string(),
            address: String::new(),
            port: 0,
            protocol: "tcp".to_string(),
            secure: true,
        },
        expires_at: std::time::SystemTime::UNIX_EPOCH,
        lineage_verified: false,
    };
    let c = p.accept_relay(&offer).await.expect("c");
    assert_eq!(c.status, RelayStatus::Failed);
}

#[tokio::test]
async fn test_socket_discovery_transport_health_unknown_defaults_unhealthy() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("d-unhealthy.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        serde_json::json!({"status": "unknown", "latency_ms": null}),
    )
    .await;
    let p = SocketDiscoveryProvider::new(sock);
    let th = p.check_transport_health("t").await.expect("th");
    assert_eq!(th.status, HealthStatus::Unhealthy);
}

#[tokio::test]
async fn test_socket_discovery_test_broadcast_defaults_false() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("d-bc.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({})).await;
    let p = SocketDiscoveryProvider::new(sock);
    let bt = p.test_encrypted_broadcast().await.expect("bt");
    assert!(!bt.encrypted && !bt.success);
}

#[tokio::test]
async fn test_socket_security_tunnel_request_minimal_json_defaults() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("sec-min.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({})).await;
    let p = SocketSecurityProvider::new(sock);
    let proof = sample_proof();
    let t = p.request_tunnel("a", "b", &proof).await.expect("t");
    assert_eq!(t.id, "pending");
    assert_eq!(t.endpoint_a.port, 0);
}

#[tokio::test]
async fn test_socket_routing_relay_defaults_and_active_accept() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("r-def.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |line| {
        let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
        let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = v["method"].as_str().unwrap_or("");
        let result = match method {
            "relay.request" => serde_json::json!({}),
            "relay.accept" => serde_json::json!({"status": "active"}),
            _ => serde_json::json!(null),
        };
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
            serde_json::to_string(&id).expect("id"),
            serde_json::to_string(&result).expect("result")
        )
    })
    .await;
    let p = SocketRoutingProvider::new(sock);
    let lineage = LineageInfo {
        is_ancestor: false,
        depth: 0,
        proof: sample_proof(),
    };
    let offer = p.request_relay("x", "y", lineage).await.expect("o");
    assert_eq!(offer.relay_node, "relay");
    let conn = p.accept_relay(&offer).await.expect("c");
    assert_eq!(conn.status, RelayStatus::Active);
}

#[test]
fn test_socket_rpc_call_invalid_json_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bad-json.sock");
    std::os::unix::net::UnixListener::bind(&sock).expect("bind");
    let client = SocketRpcClient::new(sock);
    let r = client.call("m", serde_json::json!({}));
    assert!(r.is_err());
}
