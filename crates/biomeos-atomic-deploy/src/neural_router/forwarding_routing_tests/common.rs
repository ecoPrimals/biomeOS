// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_router::NeuralRouter;
use biomeos_core::TransportEndpoint;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

pub(crate) fn create_router(family_id: &str) -> NeuralRouter {
    NeuralRouter::new(family_id)
}

pub(crate) fn unix_ep(path: &std::path::Path) -> TransportEndpoint {
    TransportEndpoint::UnixSocket {
        path: path.to_path_buf(),
    }
}

pub(crate) async fn run_mock_jsonrpc_server(
    socket_path: &std::path::Path,
    response: serde_json::Value,
    ready_tx: Option<oneshot::Sender<()>>,
) -> tokio::task::JoinHandle<()> {
    let path = socket_path.to_path_buf();
    let response_json = serde_json::to_string(&response).expect("serialize");

    tokio::spawn(async move {
        let listener = UnixListener::bind(&path).expect("bind");
        if let Some(tx) = ready_tx {
            let _ = tx.send(());
        }
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let _request = &buf[..n];

            let response_line = format!("{response_json}\n");
            stream
                .write_all(response_line.as_bytes())
                .await
                .expect("write");
            stream.flush().await.expect("flush");
        }
    })
}

pub(crate) fn tcp_ep() -> TransportEndpoint {
    TransportEndpoint::TcpSocket {
        host: std::sync::Arc::from("192.0.2.100"),
        port: 9001,
    }
}

pub(crate) fn unix_path_ep(path: &str) -> TransportEndpoint {
    TransportEndpoint::UnixSocket {
        path: PathBuf::from(path),
    }
}
