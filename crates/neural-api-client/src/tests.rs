// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural API Client tests

mod client_tests {
    use super::super::*;
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;
    use tokio::time::Duration;

    async fn run_mock_server_one_shot(
        socket_path: &Path,
        response: serde_json::Value,
    ) -> tokio::task::JoinHandle<()> {
        let path = socket_path.to_path_buf();
        let response_json = serde_json::to_string(&response).expect("serialize response");

        tokio::spawn(async move {
            let listener = UnixListener::bind(&path).expect("bind mock socket");
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.expect("read request");
                let _request = &buf[..n];

                let response_line = format!("{response_json}\n");
                stream
                    .write_all(response_line.as_bytes())
                    .await
                    .expect("write response");
                stream.flush().await.expect("flush");
            }
        })
    }

    #[test]
    fn test_client_new_with_str_path() {
        let client =
            NeuralApiClient::new("/tmp/test.sock").expect("new() with str path should succeed");
        assert_eq!(
            client.socket_path,
            std::path::PathBuf::from("/tmp/test.sock")
        );
    }

    #[test]
    fn test_client_default_timeouts() {
        let client = NeuralApiClient::new("/tmp/test.sock").expect("new() should succeed");
        assert_eq!(client.request_timeout, Duration::from_secs(30));
        assert_eq!(client.connection_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_discover_socket_path_format() {
        let path = NeuralApiClient::discover_socket("1894e909e454");
        assert!(
            path.to_string_lossy()
                .ends_with("neural-api-1894e909e454.sock"),
            "got: {}",
            path.display()
        );
    }

    #[tokio::test]
    async fn test_proxy_http_success() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let http_response = serde_json::json!({
            "status": 200,
            "headers": {"content-type": "application/json"},
            "body": "{\"ok\":true}"
        });
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": http_response,
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let response = client
            .proxy_http("GET", "https://example.com", None, None)
            .await
            .expect("proxy_http should succeed");

        assert_eq!(response.status, 200);
        assert_eq!(response.body_str().unwrap(), "{\"ok\":true}");
    }

    #[tokio::test]
    async fn test_discover_capability_success() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let capability_result = serde_json::json!({
            "capability": "secure_http",
            "atomic_type": "Tower",
            "primals": [{
                "name": "songbird",
                "socket": "/tmp/songbird.sock",
                "healthy": true,
                "capabilities": ["secure_http"]
            }],
            "primary_socket": "/tmp/songbird.sock"
        });
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": capability_result,
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let info = client
            .discover_capability("secure_http")
            .await
            .expect("discover_capability should succeed");

        assert_eq!(info.capability, "secure_http");
        assert_eq!(info.atomic_type, Some("Tower".to_string()));
        assert_eq!(info.primals.len(), 1);
        assert_eq!(info.primals[0].name, "songbird");
    }
}
