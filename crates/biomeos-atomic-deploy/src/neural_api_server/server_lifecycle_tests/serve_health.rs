// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::{
    STARTUP_TIMEOUT, coordinated_env, reserve_tcp_port, run_serve_until_tcp_response, runtime_env,
};
use crate::mode::BiomeOsMode;
use crate::neural_api_server::NeuralApiServer;
use biomeos_types::env_config::vars;
use std::time::Duration;
use tokio::net::TcpStream;

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
