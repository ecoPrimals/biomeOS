// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

use crate::neural_api_server::NeuralApiServer;
use crate::neural_api_server::discovery_init::probe_tcp_capabilities_public;
use biomeos_core::TransportEndpoint;
use biomeos_types::constants::ports::TCP_SPAWN_BASE;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener};

const GRAPH_WITH_CAPABILITIES: &str = r#"
[graph]
id = "disc_init_graph"
version = "1.0.0"
description = "Discovery init graph bootstrap"

[[nodes]]
id = "node_by_name"
capabilities = ["graph.bootstrap.by_name"]

[nodes.primal]
by_name = "graphprimal"

[[nodes]]
id = "node_by_capability"
capabilities = ["graph.bootstrap.by_cap"]

[nodes.primal]
by_capability = "capselector"

[[nodes]]
id = "node_provided"
capabilities = []

[nodes.primal]
by_name = "providedprimal"

[nodes.capabilities_provided]
"graph.bootstrap.provided" = "graph.bootstrap.provided"

[[nodes]]
id = "node_no_caps"
depends_on = []

[nodes.primal]
by_name = "emptyprimal"
"#;

fn spawn_tcp_capabilities_responder(
    port: u16,
    result: serde_json::Value,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .expect("bind tcp mock");
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": result.clone()
            });
            let line = format!("{}\n", serde_json::to_string(&body).expect("serialize"));
            let _ = stream.write_all(line.as_bytes()).await;
            let _ = stream.flush().await;
        }
    })
}

async fn wait_for_tcp_mock() {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

fn spawn_uds_capabilities_responder(
    sock_path: PathBuf,
    capabilities: Vec<String>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = UnixListener::bind(&sock_path).expect("bind uds mock");
        if let Ok((stream, _)) = listener.accept().await {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read request");
            let req_id = serde_json::from_str::<serde_json::Value>(&line)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(json!(1));
            let body = json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "capabilities": capabilities
                }
            });
            let mut stream = reader.into_inner();
            let resp = format!("{}\n", serde_json::to_string(&body).expect("serialize"));
            stream.write_all(resp.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    })
}

#[tokio::test]
async fn probe_tcp_capabilities_public_returns_empty_on_unreachable() {
    let caps = probe_tcp_capabilities_public("127.0.0.1:1").await;
    assert!(caps.is_empty());
}

#[tokio::test]
async fn probe_tcp_capabilities_public_parses_result_capabilities_array() {
    let port = TCP_SPAWN_BASE + 3;
    let mock = spawn_tcp_capabilities_responder(
        port,
        json!({
            "capabilities": ["mockprimal.security", "mockprimal.discovery"]
        }),
    );
    wait_for_tcp_mock().await;

    let caps = probe_tcp_capabilities_public(&format!("127.0.0.1:{port}")).await;
    mock.abort();

    assert_eq!(
        caps,
        vec![
            "mockprimal.security".to_string(),
            "mockprimal.discovery".to_string()
        ]
    );
}

#[tokio::test]
async fn probe_tcp_capabilities_public_parses_flat_result_array() {
    let port = TCP_SPAWN_BASE + 4;
    let mock = spawn_tcp_capabilities_responder(port, json!(["flat.cap.one", "flat.cap.two"]));
    wait_for_tcp_mock().await;

    let caps = probe_tcp_capabilities_public(&format!("127.0.0.1:{port}")).await;
    mock.abort();

    assert_eq!(
        caps,
        vec!["flat.cap.one".to_string(), "flat.cap.two".to_string()]
    );
}

#[tokio::test]
async fn probe_tcp_capabilities_public_returns_empty_on_invalid_json() {
    let port = TCP_SPAWN_BASE + 5;
    let mock = tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .expect("bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 64];
            let _ = stream.read(&mut buf).await;
            stream
                .write_all(b"not-json\n")
                .await
                .expect("write garbage");
        }
    });
    wait_for_tcp_mock().await;

    let caps = probe_tcp_capabilities_public(&format!("127.0.0.1:{port}")).await;
    mock.abort();

    assert!(caps.is_empty());
}

#[tokio::test]
async fn probe_primal_capabilities_delegates_to_standalone_probe() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("delegate.sock");
    let mock =
        spawn_uds_capabilities_responder(sock_path.clone(), vec!["delegate.cap".to_string()]);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let server = NeuralApiServer::new(dir.path(), "fam-delegate", dir.path().join("self.sock"));
    let caps = server
        .probe_primal_capabilities(sock_path.to_string_lossy().as_ref())
        .await;
    mock.abort();

    assert_eq!(caps, vec!["delegate.cap".to_string()]);
}

#[tokio::test]
async fn discover_and_register_primals_uds_registers_responsive_socket() {
    let base = tempfile::tempdir().expect("tempdir");
    let socket_dir = base.path().join("sockets");
    std::fs::create_dir_all(&socket_dir).expect("socket dir");

    let primal_sock = socket_dir.join("mockprimal-discfam.sock");
    let self_sock = socket_dir.join("neural-api.sock");
    let mock = spawn_uds_capabilities_responder(
        primal_sock.clone(),
        vec![
            "mockprimal.cap.a".to_string(),
            "mockprimal.cap.b".to_string(),
        ],
    );
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let socket_dir_str = socket_dir.to_str().expect("utf8 socket dir");
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(socket_dir_str)),
            ("XDG_RUNTIME_DIR", Some(socket_dir_str)),
        ],
        async {
            let server = NeuralApiServer::new(base.path(), "discfam", &self_sock);
            server.discover_and_register_primals().await;

            let caps = server.router.list_capabilities().await;
            assert!(caps.contains_key("mockprimal.cap.a"));
            assert!(caps.contains_key("mockprimal.cap.b"));
            let providers = caps.get("mockprimal.cap.a").expect("provider");
            assert_eq!(providers[0].primal_name.as_ref(), "mockprimal");
            assert_eq!(providers[0].source.as_ref(), "auto-discovery");
        },
    )
    .await;

    mock.abort();
}

#[tokio::test]
async fn discover_and_register_primals_uds_skips_self_socket_and_non_sock_files() {
    let base = tempfile::tempdir().expect("tempdir");
    let socket_dir = base.path().join("sockets");
    std::fs::create_dir_all(&socket_dir).expect("socket dir");

    let self_sock = socket_dir.join("neural-api.sock");
    std::fs::write(socket_dir.join("notes.txt"), "ignore me").expect("write txt");
    std::fs::write(&self_sock, b"").expect("touch self sock file");

    let socket_dir_str = socket_dir.to_str().expect("utf8 socket dir");
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(socket_dir_str)),
            ("XDG_RUNTIME_DIR", Some(socket_dir_str)),
        ],
        async {
            let server = NeuralApiServer::new(base.path(), "skipfam", &self_sock);
            server.discover_and_register_primals().await;
            assert!(server.router.list_capabilities().await.is_empty());
        },
    )
    .await;
}

#[tokio::test]
async fn discover_and_register_primals_tcp_only_registers_spawn_port() {
    let base = tempfile::tempdir().expect("tempdir");
    let port = TCP_SPAWN_BASE + 1;
    let mock = spawn_tcp_capabilities_responder(
        port,
        json!({
            "capabilities": ["tcpprimal.security", "tcpprimal.mesh"]
        }),
    );
    wait_for_tcp_mock().await;

    temp_env::async_with_vars(
        [(
            biomeos_types::env_config::vars::BIND_ADDRESS,
            Some("127.0.0.1"),
        )],
        async {
            let server = NeuralApiServer::new(base.path(), "tcpfam", base.path().join("n.sock"))
                .with_tcp_only(port);
            server.discover_and_register_primals().await;

            let caps = server.router.list_capabilities().await;
            assert!(caps.contains_key("tcpprimal.security"));
            let providers = caps.get("tcpprimal.security").expect("provider");
            assert_eq!(providers[0].primal_name.as_ref(), "tcpprimal");
            assert_eq!(providers[0].source.as_ref(), "tcp-discovery");
            match &providers[0].endpoint {
                TransportEndpoint::TcpSocket {
                    port: bound_port, ..
                } => {
                    assert_eq!(*bound_port, port);
                }
                other => panic!("expected tcp endpoint, got {other:?}"),
            }
        },
    )
    .await;

    mock.abort();
}

#[tokio::test]
async fn register_capabilities_from_graphs_pre_registers_node_capabilities() {
    let base = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        base.path().join("bootstrap_graph.toml"),
        GRAPH_WITH_CAPABILITIES,
    )
    .expect("write graph");
    let sock = base.path().join("neural-api.sock");

    let server = NeuralApiServer::new(base.path(), "graphfam", sock);
    server.register_capabilities_from_graphs().await;

    let caps = server.router.list_capabilities().await;
    assert!(caps.contains_key("graph.bootstrap.by_name"));
    assert!(caps.contains_key("graph.bootstrap.by_cap"));
    assert!(caps.contains_key("graph.bootstrap.provided"));
    assert!(!caps.contains_key("graph.bootstrap.missing"));

    let by_name = caps.get("graph.bootstrap.by_name").expect("by_name");
    assert_eq!(by_name[0].primal_name.as_ref(), "graphprimal");
    assert_eq!(by_name[0].source.as_ref(), "graph-bootstrap");
}

#[tokio::test]
async fn register_capabilities_from_graphs_skips_invalid_toml_and_empty_graphs_dir() {
    let base = tempfile::tempdir().expect("tempdir");
    std::fs::write(base.path().join("broken.toml"), "[[[ not valid").expect("write broken");
    let sock = base.path().join("neural-api.sock");

    let server = NeuralApiServer::new(base.path(), "skipgraph", sock);
    server.register_capabilities_from_graphs().await;
    assert!(server.router.list_capabilities().await.is_empty());

    let missing = base.path().join("missing-graphs");
    let server = NeuralApiServer::new(&missing, "skipgraph", base.path().join("n.sock"));
    server.register_capabilities_from_graphs().await;
    assert!(server.router.list_capabilities().await.is_empty());
}

#[tokio::test]
async fn derive_coordination_key_skips_without_crypto_sign_provider() {
    let base = tempfile::tempdir().expect("tempdir");
    let server = NeuralApiServer::new(base.path(), "coord-none", base.path().join("n.sock"));

    server.derive_coordination_key().await;

    let key = server.coordination_pubkey.read().await;
    assert!(key.is_none());
}

#[tokio::test]
async fn derive_coordination_key_skips_non_uds_provider() {
    let base = tempfile::tempdir().expect("tempdir");
    let server = NeuralApiServer::new(base.path(), "coord-tcp", base.path().join("n.sock"));
    server
        .router
        .register_capability(
            "crypto.sign",
            "beardog",
            TransportEndpoint::TcpSocket {
                host: Arc::from("127.0.0.1"),
                port: 9999,
            },
            "test",
        )
        .await
        .expect("register tcp crypto.sign");

    server.derive_coordination_key().await;

    let key = server.coordination_pubkey.read().await;
    assert!(key.is_none());
}

#[tokio::test]
async fn derive_coordination_key_warns_when_btsp_call_fails() {
    let base = tempfile::tempdir().expect("tempdir");
    let server = NeuralApiServer::new(base.path(), "coord-fail", base.path().join("n.sock"));
    let missing_sock = base.path().join("missing-beardog.sock");
    server
        .router
        .register_capability_unix("crypto.sign", "beardog", &missing_sock, "test")
        .await
        .expect("register uds crypto.sign");

    server.derive_coordination_key().await;

    let key = server.coordination_pubkey.read().await;
    assert!(key.is_none());
}

#[tokio::test]
async fn rescan_primals_returns_json_shape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let sock = temp.path().join("rescan.sock");
    let server = NeuralApiServer::new(temp.path(), "fam-rescan", sock);
    let v = server.rescan_primals().await.expect("rescan");
    assert_eq!(v["rescanned"], true);
    assert!(v.get("new_capabilities_registered").is_some());
    assert!(v.get("total_capabilities").is_some());
}

#[tokio::test]
async fn rescan_primals_reports_capability_totals() {
    let base = tempfile::tempdir().expect("tempdir");
    let server = NeuralApiServer::new(base.path(), "rescanfam", base.path().join("n.sock"));
    server
        .router
        .register_capability_unix(
            "rescanprimal.cap.fixed",
            "rescanprimal",
            base.path().join("rescanprimal-rescanfam.sock"),
            "test",
        )
        .await
        .expect("seed capability");

    let before = server.router.list_capabilities().await.len();
    let result = server.rescan_primals().await.expect("rescan");

    assert_eq!(result["rescanned"], true);
    assert_eq!(
        result["total_capabilities"].as_u64().unwrap() as usize,
        server.router.list_capabilities().await.len()
    );
    assert!(result["total_capabilities"].as_u64().unwrap() >= before as u64);
    assert_eq!(
        result["new_capabilities_registered"].as_u64().unwrap() as usize,
        server
            .router
            .list_capabilities()
            .await
            .len()
            .saturating_sub(before)
    );
}
