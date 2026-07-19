// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{is_explicit_coordinated_mode_str, is_explicit_coordinated_mode_with};
use super::{STARTUP_TIMEOUT, coordinated_env, runtime_env};
use crate::neural_api_server::NeuralApiServer;
use biomeos_types::env_config::vars;

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
async fn serve_log_graph_inventory_handles_missing_graphs_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = temp.path().join("runtime");
    std::fs::create_dir_all(&runtime).expect("runtime dir");
    let graphs_dir = temp.path().join("missing-graphs");
    let port = super::reserve_tcp_port().await;
    let server = NeuralApiServer::new(&graphs_dir, "inv-fam", temp.path().join("n.sock"))
        .with_tcp_only(port)
        .with_bind_address("127.0.0.1".to_string())
        .with_btsp_optional();
    let runtime_str = runtime_env(&runtime);

    temp_env::async_with_vars(coordinated_env(runtime_str, Some("127.0.0.1")), async {
        let response = super::run_serve_until_tcp_response(
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
