// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

/// connect_to_peer with non-existent neural-api socket — tests error path
#[tokio::test]
async fn test_connect_to_peer_socket_not_found() {
    let result = connect_to_peer("peer-123", "/nonexistent/path/neural-api-12345.sock", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed")
            || err.contains("connect")
            || err.contains("No such file")
            || err.contains("Connection refused"),
        "Expected connection error, got: {err}"
    );
}

/// connect_to_peer with peer_connection_info (uses stun_results for peer NAT)
#[tokio::test]
async fn test_connect_to_peer_with_connection_info() {
    let info = PeerConnectionInfo {
        stun_results: Some(StunResults {
            public_addr: "1.2.3.4:41200".to_string(),
            nat_type: "symmetric".to_string(),
        }),
        relay_endpoint: None,
        stun_server: None,
    };

    let result = connect_to_peer("peer-456", "/nonexistent/neural-api.sock", Some(&info)).await;

    assert!(result.is_err());
}

/// Unix-only: exercises `connect_to_peer` against a local JSON-RPC mock.
#[cfg(unix)]
mod connect_mock_unix {
    use super::*;
    use biomeos_types::JsonRpcResponse;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    fn spawn_sequential_neural_mock(
        socket_path: &std::path::Path,
        results: Vec<serde_json::Value>,
    ) {
        let listener = UnixListener::bind(socket_path).expect("bind neural mock");
        let results = Arc::new(results);
        let idx = Arc::new(AtomicUsize::new(0));
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.expect("accept");
                let results = Arc::clone(&results);
                let idx = Arc::clone(&idx);
                tokio::spawn(async move {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_err() {
                        return;
                    }
                    let Ok(req) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
                        return;
                    };
                    let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
                    let n = idx.fetch_add(1, Ordering::SeqCst);
                    let payload = results
                        .get(n)
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    let response = JsonRpcResponse::success(id, payload);
                    let mut stream = reader.into_inner();
                    let body = serde_json::to_string(&response).expect("serialize");
                    let _ = stream.write_all(format!("{body}\n").as_bytes()).await;
                });
            }
        });
    }

    #[tokio::test]
    async fn connect_to_peer_tier1_lan_direct() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-lan-1";
        spawn_sequential_neural_mock(
            &sock,
            vec![serde_json::json!({
                "peers": [{
                    "node_id": peer_id,
                    "endpoint": "unix:///tmp/mesh.sock"
                }]
            })],
        );

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), None)
            .await
            .expect("tier1");
        assert_eq!(res.tier, ConnectionTier::LanDirect);
        assert!(res.endpoint.contains("mesh") || res.endpoint.contains("unix"));
        assert!(res.tiers_attempted.contains(&ConnectionTier::LanDirect));
    }

    #[tokio::test]
    async fn connect_to_peer_tier2_direct_punch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-punch";
        spawn_sequential_neural_mock(
            &sock,
            vec![
                serde_json::json!({ "peers": [] }),
                serde_json::json!({ "nat_type": "full_cone" }),
                serde_json::json!({
                    "success": true,
                    "endpoint": "udp://punched:1234"
                }),
            ],
        );

        let info = PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.1.1.1:1".to_string(),
                nat_type: "full_cone".to_string(),
            }),
            relay_endpoint: None,
            stun_server: None,
        };

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), Some(&info))
            .await
            .expect("tier2");
        assert_eq!(res.tier, ConnectionTier::DirectPunch);
        assert!(res.tiers_attempted.contains(&ConnectionTier::DirectPunch));
    }

    #[tokio::test]
    async fn connect_to_peer_tier4_pure_relay_after_failed_punch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-relay";
        spawn_sequential_neural_mock(
            &sock,
            vec![
                serde_json::json!({ "peers": [] }),
                serde_json::json!({ "nat_type": "full_cone" }),
                serde_json::json!({ "success": false }),
                serde_json::json!({ "session_id": "relay-final-session" }),
            ],
        );

        let info = PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.1.1.1:1".to_string(),
                nat_type: "full_cone".to_string(),
            }),
            relay_endpoint: None,
            stun_server: None,
        };

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), Some(&info))
            .await
            .expect("relay fallback");
        assert_eq!(res.tier, ConnectionTier::PureRelay);
        assert_eq!(res.endpoint, "relay-final-session");
    }

    #[tokio::test]
    async fn connect_to_peer_tier3_symmetric_pure_relay() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-sym";
        spawn_sequential_neural_mock(
            &sock,
            vec![
                serde_json::json!({ "peers": [] }),
                serde_json::json!({ "nat_type": "symmetric" }),
                serde_json::json!({ "session_id": "relay-sym" }),
                serde_json::json!({}),
            ],
        );

        let info = PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.1.1.1:1".to_string(),
                nat_type: "symmetric".to_string(),
            }),
            relay_endpoint: None,
            stun_server: None,
        };

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), Some(&info))
            .await
            .expect("symmetric relay");
        assert_eq!(res.tier, ConnectionTier::PureRelay);
        assert_eq!(res.endpoint, "relay-sym");
    }
}
