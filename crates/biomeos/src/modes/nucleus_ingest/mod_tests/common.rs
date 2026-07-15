// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::{Notify, oneshot};

/// Create a valid pseudoSpore 2.0 directory for ingest tests.
pub(super) fn create_valid_pseudospore(dir: &Path) {
    std::fs::write(
        dir.join("scope.toml"),
        r#"[artifact]
name = "test-spore-001"
version = "1.0.0"
type = "pseudoSpore"
date = "2026-05-27"
origin = "biomeOS-test"
license = "AGPL-3.0"
"#,
    )
    .unwrap();

    std::fs::write(
        dir.join("validation.json"),
        r#"{"artifact":"test-spore-001","version":"1.0.0","date":"2026-05-27","modules":[{"name":"structural","status":"PASS","checks_total":3,"checks_passed":3}]}"#,
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("receipts")).unwrap();
    std::fs::write(
        dir.join("receipts/environment.toml"),
        "[hardware]\ncpu = \"x86_64\"\ncores = 8\n\n[software]\nos = \"Linux\"\nrust = \"1.82\"\n",
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("data/payload.bin"), b"hello world").unwrap();

    let hash = biomeos_pseudospore::compute_checksums(dir, &["data"]);
    std::fs::write(
        dir.join("receipts/checksums.blake3"),
        biomeos_pseudospore::format_checksums(&hash),
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("provenance")).unwrap();
    std::fs::write(
        dir.join("provenance/ferment_transcript.json"),
        r#"{"dataset_id":"ds-001","spring":"hotSpring","spring_version":"1.5.0"}"#,
    )
    .unwrap();

    std::fs::write(dir.join("README.md"), "# Test pseudoSpore\n").unwrap();
}

/// Spawn a mock Neural API that returns one fixed JSON-RPC response per accepted connection.
pub(super) async fn spawn_mock_neural_api(
    responses: Vec<serde_json::Value>,
) -> (std::path::PathBuf, tokio::task::JoinHandle<()>) {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("neural-api.sock");
    let path_for_listener = socket_path.clone();
    let (ready_tx, ready_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        let _temp = temp;
        let listener = UnixListener::bind(&path_for_listener).expect("bind mock socket");
        let _ = ready_tx.send(());

        for response in responses {
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response_str = serde_json::to_string(&response).expect("serialize") + "\n";
                    let _ = writer.write_all(response_str.as_bytes()).await;
                    let _ = writer.flush().await;
                }
            }
        }
    });

    ready_rx.await.expect("mock server ready");
    (socket_path, handle)
}

/// Spawn a mock server at an explicit path (for socket discovery tests).
pub(super) async fn spawn_mock_at_path(
    socket_path: std::path::PathBuf,
    response: serde_json::Value,
    ready: Arc<Notify>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket_path).expect("bind discovered socket");
        ready.notify_one();
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                let response_str = serde_json::to_string(&response).expect("serialize") + "\n";
                let _ = writer.write_all(response_str.as_bytes()).await;
            }
        }
    })
}
