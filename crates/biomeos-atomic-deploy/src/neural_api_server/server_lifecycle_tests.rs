// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::expect_used, reason = "test assertions")]

use super::{is_explicit_coordinated_mode_str, is_explicit_coordinated_mode_with};
use crate::mode::BiomeOsMode;
use crate::neural_api_server::NeuralApiServer;
use biomeos_types::constants::ribocipher;
use biomeos_types::env_config::vars;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UnixStream};

const RIBOCIPHER_CLEAR_SIGNAL: [u8; ribocipher::SIGNAL_LEN] =
    [ribocipher::SIGNAL_CLEAR, ribocipher::VERSION_1];

async fn write_ribocipher_json_rpc(stream: &mut (impl AsyncWriteExt + Unpin), request: &str) {
    stream
        .write_all(&RIBOCIPHER_CLEAR_SIGNAL)
        .await
        .expect("write riboCipher signal");
    stream
        .write_all(format!("{request}\n").as_bytes())
        .await
        .expect("write request");
    stream.flush().await.expect("flush request");
}

async fn reserve_tcp_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve tcp port");
    listener.local_addr().expect("local addr").port()
}

async fn wait_for_tcp_json_rpc(port: u16, request: &str, timeout: Duration) -> Value {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        match TcpStream::connect(format!("127.0.0.1:{port}")).await {
            Ok(mut stream) => {
                write_ribocipher_json_rpc(&mut stream, request).await;
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
                    return serde_json::from_str(&line).expect("parse response");
                }
            }
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            Err(e) => panic!("TCP health probe failed: {e}"),
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for TCP JSON-RPC on port {port}"
        );
    }
}

async fn wait_for_uds_json_rpc(socket_path: &Path, request: &str, timeout: Duration) -> Value {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        match UnixStream::connect(socket_path).await {
            Ok(mut stream) => {
                write_ribocipher_json_rpc(&mut stream, request).await;
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
                    return serde_json::from_str(&line).expect("parse response");
                }
            }
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            Err(e) => panic!("UDS health probe failed: {e}"),
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for UDS JSON-RPC on {}",
            socket_path.display()
        );
    }
}

fn runtime_env(base: &Path) -> &str {
    base.to_str().expect("utf8 runtime dir")
}

fn coordinated_env<'a>(
    runtime_str: &'a str,
    bind: Option<&'a str>,
) -> [(&'a str, Option<&'a str>); 5] {
    [
        ("BIOMEOS_SOCKET_DIR", Some(runtime_str)),
        ("XDG_RUNTIME_DIR", Some(runtime_str)),
        (vars::MODE, Some("coordinated")),
        (vars::BIND_ADDRESS, bind),
        (vars::FAMILY_ID, None),
    ]
}

/// Run `serve()` concurrently with a health probe; dropping the returned future
/// cancels the accept loop (shutdown handling).
async fn run_serve_until_tcp_response(
    server: NeuralApiServer,
    port: u16,
    request: &str,
    timeout: Duration,
) -> Value {
    let mut serve = Box::pin(server.serve());
    let mut probe = Box::pin(wait_for_tcp_json_rpc(port, request, timeout));

    tokio::select! {
        response = &mut probe => response,
        result = &mut serve => {
            panic!("serve exited before TCP probe succeeded: {result:?}");
        }
    }
}

async fn run_serve_until_uds_response(
    server: NeuralApiServer,
    socket_path: &Path,
    request: &str,
    timeout: Duration,
) -> Value {
    let mut serve = Box::pin(server.serve());
    let mut probe = Box::pin(wait_for_uds_json_rpc(socket_path, request, timeout));

    tokio::select! {
        response = &mut probe => response,
        result = &mut serve => {
            panic!("serve exited before UDS probe succeeded: {result:?}");
        }
    }
}

const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);

#[test]
fn server_lifecycle_reexports_coordinated_mode_detection() {
    assert!(is_explicit_coordinated_mode_str("coordinated"));
    assert!(is_explicit_coordinated_mode_with(Some("join")));
    assert!(!is_explicit_coordinated_mode_with(Some("bootstrap")));
}

#[tokio::test]
async fn serve_rejects_insecure_guard_when_family_id_and_insecure_set() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = NeuralApiServer::new(temp.path(), "guardfam", temp.path().join("n.sock"))
        .with_btsp_optional();

    temp_env::async_with_vars(
        [
            (vars::FAMILY_ID, Some("guardfam")),
            (vars::INSECURE, Some("1")),
        ],
        async {
            let err = server.serve().await.expect_err("serve should fail");
            let msg = err.to_string();
            assert!(
                msg.contains("INSECURE") || msg.contains("insecure"),
                "unexpected error: {msg}"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn serve_fails_when_no_listeners_configured() {
    let temp = tempfile::tempdir().expect("tempdir");
    let server = NeuralApiServer::new(temp.path(), "nolisteners", temp.path().join("n.sock"))
        .with_btsp_optional();

    temp_env::async_with_vars([(vars::PRIMAL_BIND_MODE, Some("tcp_only"))], async {
        let err = server.serve().await.expect_err("serve should fail");
        assert!(
            err.to_string().contains("No listeners configured"),
            "unexpected error: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn serve_uds_initializes_listener_and_responds_to_health_liveness() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let sock = runtime.join("neural-api-lifecycle.sock");
    let server = NeuralApiServer::new(temp.path(), "uds-lifecycle", &sock).with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, None), async {
        let response = run_serve_until_uds_response(
            server,
            &sock,
            r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#,
            STARTUP_TIMEOUT,
        )
        .await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["result"]["status"], "alive");
        assert_eq!(response["result"]["primal"], "biomeOS");
        assert!(response["result"]["version"].is_string());
    })
    .await;
}

#[tokio::test]
async fn serve_tcp_only_initializes_and_responds_to_health_check() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let port = reserve_tcp_port().await;
    let server = NeuralApiServer::new(temp.path(), "tcp-lifecycle", temp.path().join("n.sock"))
        .with_tcp_only(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let response = run_serve_until_tcp_response(
            server,
            port,
            r#"{"jsonrpc":"2.0","method":"health.check","id":2}"#,
            STARTUP_TIMEOUT,
        )
        .await;

        assert_eq!(response["result"]["status"], "alive");
        assert_eq!(response["result"]["family_id"], "tcp-lifecycle");
        assert_eq!(response["result"]["mode"], "Coordinated");
        assert!(
            response["result"]["registered_capabilities"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );
    })
    .await;
}

#[tokio::test]
async fn serve_degrades_to_tcp_when_uds_bind_fails() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let blocker = temp.path().join("blocker");
    std::fs::write(&blocker, b"not-a-directory").expect("write blocker file");
    let bad_sock = blocker.join("nested.sock");
    let port = reserve_tcp_port().await;

    let server = NeuralApiServer::new(temp.path(), "degrade-tcp", &bad_sock)
        .with_tcp_port(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let response = run_serve_until_tcp_response(
            server,
            port,
            r#"{"jsonrpc":"2.0","method":"health.liveness","id":3}"#,
            STARTUP_TIMEOUT,
        )
        .await;
        assert_eq!(response["result"]["status"], "alive");
    })
    .await;
}

#[tokio::test]
async fn serve_dual_listeners_accept_tcp_and_uds_health_checks() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let sock = runtime.join("dual-lifecycle.sock");
    let port = reserve_tcp_port().await;
    let server = NeuralApiServer::new(temp.path(), "dual-lifecycle", &sock)
        .with_tcp_port(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let mut serve = Box::pin(server.serve());
        let mut probes = Box::pin(async {
            let tcp_response = wait_for_tcp_json_rpc(
                port,
                r#"{"jsonrpc":"2.0","method":"health.liveness","id":8}"#,
                STARTUP_TIMEOUT,
            )
            .await;
            let uds_response = wait_for_uds_json_rpc(
                &sock,
                r#"{"jsonrpc":"2.0","method":"health.liveness","id":9}"#,
                Duration::from_secs(5),
            )
            .await;
            (tcp_response, uds_response)
        });

        let (tcp_response, uds_response) = tokio::select! {
            responses = &mut probes => responses,
            result = &mut serve => {
                panic!("serve exited before dual-listener probe succeeded: {result:?}");
            }
        };
        assert_eq!(tcp_response["result"]["status"], "alive");
        assert_eq!(uds_response["result"]["status"], "alive");
    })
    .await;
}

#[tokio::test]
async fn serve_explicit_coordinated_mode_transitions_mode_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let port = reserve_tcp_port().await;
    let server = NeuralApiServer::new(temp.path(), "coord-mode", temp.path().join("n.sock"))
        .with_tcp_only(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let mode_handle = server.mode.clone();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let response = run_serve_until_tcp_response(
            server,
            port,
            r#"{"jsonrpc":"2.0","method":"health.check","id":4}"#,
            STARTUP_TIMEOUT,
        )
        .await;
        assert_eq!(response["result"]["mode"], "Coordinated");
        assert_eq!(*mode_handle.read().await, BiomeOsMode::Coordinated);
    })
    .await;
}

#[tokio::test]
async fn serve_explicit_bootstrap_mode_keeps_bootstrap_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let port = reserve_tcp_port().await;
    let server = NeuralApiServer::new(temp.path(), "boot-mode", temp.path().join("n.sock"))
        .with_tcp_only(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let mode_handle = server.mode.clone();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(runtime_str)),
            ("XDG_RUNTIME_DIR", Some(runtime_str)),
            (vars::MODE, Some("bootstrap")),
            (vars::BIND_ADDRESS, Some("127.0.0.1")),
            (vars::FAMILY_ID, None),
        ],
        async {
            let response = run_serve_until_tcp_response(
                server,
                port,
                r#"{"jsonrpc":"2.0","method":"health.check","id":5}"#,
                STARTUP_TIMEOUT,
            )
            .await;
            assert_eq!(response["result"]["mode"], "Bootstrap");
            assert_eq!(*mode_handle.read().await, BiomeOsMode::Bootstrap);
        },
    )
    .await;
}

#[tokio::test]
async fn serve_health_readiness_reports_ready_after_self_registration() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let port = reserve_tcp_port().await;
    let server = NeuralApiServer::new(temp.path(), "ready-fam", temp.path().join("n.sock"))
        .with_tcp_only(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let response = run_serve_until_tcp_response(
            server,
            port,
            r#"{"jsonrpc":"2.0","method":"health.readiness","id":6}"#,
            STARTUP_TIMEOUT,
        )
        .await;
        assert_eq!(response["result"]["ready"], true);
        assert!(
            response["result"]["registered_capabilities"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );
    })
    .await;
}

#[tokio::test]
async fn serve_shutdown_dropping_future_stops_accept_loop() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let port = reserve_tcp_port().await;
    let server = NeuralApiServer::new(temp.path(), "shutdown-fam", temp.path().join("n.sock"))
        .with_tcp_only(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let response = run_serve_until_tcp_response(
            server,
            port,
            r#"{"jsonrpc":"2.0","method":"health.liveness","id":7}"#,
            STARTUP_TIMEOUT,
        )
        .await;
        assert_eq!(response["result"]["status"], "alive");
        // `run_serve_until_tcp_response` drops the pinned `serve()` future on return,
        // which cancels the accept loop (shutdown handling).
    })
    .await;

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .is_err(),
        "listener should stop accepting after serve future is dropped"
    );
}

#[tokio::test]
async fn serve_log_graph_inventory_handles_missing_graphs_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let graphs_dir = temp.path().join("missing-graphs");
    let port = reserve_tcp_port().await;
    let server = NeuralApiServer::new(&graphs_dir, "inv-fam", temp.path().join("n.sock"))
        .with_tcp_only(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let response = run_serve_until_tcp_response(
            server,
            port,
            r#"{"jsonrpc":"2.0","method":"health.liveness","id":10}"#,
            STARTUP_TIMEOUT,
        )
        .await;
        assert_eq!(response["result"]["status"], "alive");
    })
    .await;
}
