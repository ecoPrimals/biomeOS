// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`crate::atomic_primal_client::AtomicPrimalClient`].

use crate::atomic_primal_client::AtomicPrimalClient;
use crate::socket_discovery::TransportEndpoint;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

#[test]
fn unix_client_exposes_name_and_unix_endpoint() {
    let c = AtomicPrimalClient::unix("beardog", "/tmp/biomeos-test-not-present.sock");
    assert_eq!(c.primal_name(), "beardog");
    match c.endpoint() {
        TransportEndpoint::UnixSocket { path } => {
            assert_eq!(path, &PathBuf::from("/tmp/biomeos-test-not-present.sock"));
        }
        _ => panic!("expected unix endpoint"),
    }
    assert!(!c.is_available());
}

#[test]
fn tcp_client_marks_available_without_filesystem_socket() {
    let c = AtomicPrimalClient::tcp("songbird", "127.0.0.1", 59998);
    assert_eq!(c.primal_name(), "songbird");
    match c.endpoint() {
        TransportEndpoint::TcpSocket { host, port } => {
            assert_eq!(host.as_ref(), "127.0.0.1");
            assert_eq!(*port, 59998);
        }
        _ => panic!("expected tcp endpoint"),
    }
    assert!(c.is_available());
}

#[test]
fn from_endpoint_preserves_transport() {
    let ep = TransportEndpoint::TcpSocket {
        host: Arc::from("10.0.0.5"),
        port: 9000,
    };
    let c = AtomicPrimalClient::from_endpoint("gate", ep.clone());
    assert_eq!(c.endpoint(), &ep);
}

#[tokio::test]
async fn health_check_ok_when_result_status_is_ok() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("health.sock");
    let sock_clone = sock.clone();
    let listener = UnixListener::bind(&sock_clone).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        let id = req.get("id").cloned().unwrap_or(json!(null));
        assert_eq!(req["method"], "health.ping");
        let resp = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "status": "ok" }
        });
        let mut stream = reader.into_inner();
        let out = format!("{}\n", serde_json::to_string(&resp).unwrap());
        stream.write_all(out.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let client = AtomicPrimalClient::unix("p", &sock);
    client.health_check().await.expect("health ok");
    server.await.expect("server");
}

#[tokio::test]
async fn health_check_errors_when_status_not_ok() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("health_bad.sock");
    let sock_clone = sock.clone();
    let listener = UnixListener::bind(&sock_clone).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        let id = req.get("id").cloned().unwrap_or(json!(null));
        let resp = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "status": "degraded" }
        });
        let mut stream = reader.into_inner();
        let out = format!("{}\n", serde_json::to_string(&resp).unwrap());
        stream.write_all(out.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let client = AtomicPrimalClient::unix("p", &sock);
    let err = client
        .health_check()
        .await
        .expect_err("expected health failure");
    assert!(
        err.to_string().contains("Primal health check failed"),
        "{err:?}"
    );
    server.await.expect("server");
}

#[tokio::test]
async fn execute_command_maps_stdout_stderr_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("exec.sock");
    let sock_clone = sock.clone();
    let listener = UnixListener::bind(&sock_clone).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        let id = req.get("id").cloned().unwrap_or(json!(null));
        assert_eq!(req["method"], "execute_command");
        let resp = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "stdout": "out\n",
                "stderr": "err",
                "exit_code": 42
            }
        });
        let mut stream = reader.into_inner();
        let out = format!("{}\n", serde_json::to_string(&resp).unwrap());
        stream.write_all(out.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let client = AtomicPrimalClient::unix("p", &sock);
    let exec = client
        .execute_command("echo hi")
        .await
        .expect("execute_command");
    assert_eq!(exec.stdout, "out\n");
    assert_eq!(exec.stderr, "err");
    assert_eq!(exec.exit_code, Some(42));
    server.await.expect("server");
}

#[tokio::test]
async fn execute_command_defaults_missing_fields_to_empty() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("exec_min.sock");
    let sock_clone = sock.clone();
    let listener = UnixListener::bind(&sock_clone).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        let id = req.get("id").cloned().unwrap_or(json!(null));
        let resp = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {}
        });
        let mut stream = reader.into_inner();
        let out = format!("{}\n", serde_json::to_string(&resp).unwrap());
        stream.write_all(out.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let client = AtomicPrimalClient::unix("p", &sock);
    let exec = client.execute_command("true").await.expect("exec");
    assert_eq!(exec.stdout, "");
    assert_eq!(exec.stderr, "");
    assert_eq!(exec.exit_code, None);
    server.await.expect("server");
}
