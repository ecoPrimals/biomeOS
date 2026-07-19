// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::{
    STARTUP_TIMEOUT, coordinated_env, reserve_tcp_port, run_serve_until_tcp_response,
    run_serve_until_uds_response, runtime_env, wait_for_tcp_json_rpc, wait_for_uds_json_rpc,
};
use crate::neural_api_server::NeuralApiServer;
use std::time::Duration;

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
