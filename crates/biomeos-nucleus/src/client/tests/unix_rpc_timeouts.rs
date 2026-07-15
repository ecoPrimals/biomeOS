// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::transport::call_unix_socket_rpc;
use biomeos_test_utils::ready_signal;

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
#[ignore = "Slow: exercises ~30s socket read timeout path; run with --ignored"]
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
