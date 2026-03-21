// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Neural API Registry Queries
//!
//! Discovery via the Neural API capability registry. Queries `primal.discover`
//! and `capability.discover` JSON-RPC methods over the neural-api Unix socket.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::debug;

use super::engine::SocketDiscovery;
use super::result::{DiscoveredSocket, DiscoveryMethod};
use super::transport::TransportEndpoint;

impl SocketDiscovery {
    pub(crate) async fn discover_via_registry_by_name(
        &self,
        primal_name: &str,
    ) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self
            .query_registry(
                "primal.discover",
                &serde_json::json!({ "name": primal_name }),
                &neural_api,
            )
            .await
        {
            Ok(result) => {
                let endpoint =
                    if let Some(socket_path) = result.get("socket_path").and_then(|s| s.as_str()) {
                        TransportEndpoint::UnixSocket {
                            path: PathBuf::from(socket_path),
                        }
                    } else if let Some(tcp) = result.get("tcp_endpoint").and_then(|s| s.as_str()) {
                        TransportEndpoint::parse(tcp)?
                    } else if let Some(abstract_name) =
                        result.get("abstract_socket").and_then(|s| s.as_str())
                    {
                        TransportEndpoint::AbstractSocket {
                            name: Arc::from(abstract_name),
                        }
                    } else {
                        return None;
                    };

                let capabilities = result
                    .get("capabilities")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                return Some(
                    DiscoveredSocket::from_endpoint(endpoint, DiscoveryMethod::CapabilityRegistry)
                        .with_primal_name(primal_name)
                        .with_capabilities(capabilities),
                );
            }
            Err(e) => {
                debug!("Registry query failed for {}: {}", primal_name, e);
            }
        }

        None
    }

    pub(crate) async fn discover_via_registry_by_capability(
        &self,
        capability: &str,
    ) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self
            .query_registry(
                "capability.discover",
                &serde_json::json!({ "capability": capability }),
                &neural_api,
            )
            .await
        {
            Ok(result) => {
                let endpoint = if let Some(socket_path) =
                    result.get("primary_socket").and_then(|s| s.as_str())
                {
                    TransportEndpoint::UnixSocket {
                        path: PathBuf::from(socket_path),
                    }
                } else if let Some(tcp) = result.get("tcp_endpoint").and_then(|s| s.as_str()) {
                    TransportEndpoint::parse(tcp)?
                } else {
                    return None;
                };

                let primal_name: Option<Arc<str>> = result
                    .get("provider")
                    .and_then(|p| p.as_str())
                    .map(Arc::from);

                let mut socket =
                    DiscoveredSocket::from_endpoint(endpoint, DiscoveryMethod::CapabilityRegistry)
                        .with_capabilities(vec![capability.to_string()]);

                if let Some(name) = primal_name {
                    socket = socket.with_primal_name(name.as_ref());
                }

                return Some(socket);
            }
            Err(e) => {
                debug!("Registry query failed for capability {}: {}", capability, e);
            }
        }

        None
    }

    pub(crate) async fn query_registry(
        &self,
        method: &str,
        params: &serde_json::Value,
        neural_api_socket: &Path,
    ) -> Result<serde_json::Value, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;
        use tokio::time::{Duration, timeout};

        let stream = timeout(
            Duration::from_secs(5),
            UnixStream::connect(neural_api_socket),
        )
        .await
        .map_err(|_| "Connection timeout")?
        .map_err(|e| format!("Connection failed: {e}"))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let request = biomeos_types::JsonRpcRequest::new(method, params.clone());

        let request_str = serde_json::to_string(&request).map_err(|e| e.to_string())? + "\n";
        writer
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        writer.flush().await.map_err(|e| e.to_string())?;

        let mut response_line = String::new();
        timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
            .await
            .map_err(|_| "Response timeout")?
            .map_err(|e| format!("Read failed: {e}"))?;

        let response: serde_json::Value =
            serde_json::from_str(response_line.trim()).map_err(|e| format!("Parse failed: {e}"))?;

        if let Some(error) = response.get("error") {
            return Err(format!("Registry error: {error}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| "No result in response".to_string())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::super::engine::SocketDiscovery;
    use super::super::result::DiscoveryMethod;
    use super::super::transport::TransportEndpoint;
    use std::path::PathBuf;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::oneshot;

    /// Bind a one-shot mock Neural API that reads one JSON-RPC line and writes `response_line`.
    async fn spawn_neural_api_mock(response_line: String) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("neural-api-mock.sock");
        let (ready_tx, ready_rx) = oneshot::channel();

        let path_for_task = path.clone();
        tokio::spawn(async move {
            let _ = tokio::fs::remove_file(&path_for_task).await;
            let listener = UnixListener::bind(&path_for_task).expect("bind mock neural API");
            ready_tx.send(()).expect("signal mock ready");
            let (stream, _) = listener.accept().await.expect("accept");
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut request = String::new();
            reader
                .read_line(&mut request)
                .await
                .expect("read request line");
            assert!(
                !request.is_empty(),
                "client should send at least one line: {request:?}"
            );
            write_half
                .write_all(response_line.as_bytes())
                .await
                .expect("write response");
        });

        ready_rx.await.expect("mock server failed to start");
        (dir, path)
    }

    #[tokio::test]
    async fn query_registry_success_returns_result_object() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"result":{"socket_path":"/tmp/x.sock"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let out = discovery
            .query_registry(
                "primal.discover",
                &serde_json::json!({ "name": "beardog" }),
                &sock,
            )
            .await
            .expect("query ok");
        assert_eq!(
            out.get("socket_path").and_then(|v| v.as_str()),
            Some("/tmp/x.sock")
        );
    }

    #[tokio::test]
    async fn query_registry_json_rpc_error() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"nope"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let err = discovery
            .query_registry("x", &serde_json::json!({}), &sock)
            .await
            .unwrap_err();
        assert!(err.contains("Registry error"), "got: {err}");
    }

    #[tokio::test]
    async fn query_registry_malformed_json() {
        let line = "not-json-at-all\n".to_string();
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let err = discovery
            .query_registry("x", &serde_json::json!({}), &sock)
            .await
            .unwrap_err();
        assert!(err.contains("Parse failed"), "got: {err}");
    }

    #[tokio::test]
    async fn query_registry_missing_result_field() {
        let line = r#"{"jsonrpc":"2.0","id":1}"#.to_string() + "\n";
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let err = discovery
            .query_registry("x", &serde_json::json!({}), &sock)
            .await
            .unwrap_err();
        assert!(err.contains("No result"), "got: {err}");
    }

    #[tokio::test]
    async fn query_registry_connection_fails_for_missing_socket() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("definitely-missing.sock");
        let discovery = SocketDiscovery::new("fam").with_neural_api(missing.clone());
        let err = discovery
            .query_registry("x", &serde_json::json!({}), &missing)
            .await
            .unwrap_err();
        assert!(
            err.contains("Connection") || err.contains("timeout"),
            "got: {err}"
        );
    }

    #[tokio::test]
    async fn discover_via_registry_by_name_unix_socket_path() {
        let result_json = serde_json::json!({
            "socket_path": "/run/biomeos/primal.sock",
            "capabilities": ["security", "crypto"]
        });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let found = discovery
            .discover_via_registry_by_name("nestgate")
            .await
            .expect("discovered");
        assert_eq!(found.discovered_via, DiscoveryMethod::CapabilityRegistry);
        assert_eq!(found.primal_name.as_deref(), Some("nestgate"));
        assert!(found.capabilities.contains(&"security".to_string()));
        match &found.endpoint {
            TransportEndpoint::UnixSocket { path } => {
                assert_eq!(path, &PathBuf::from("/run/biomeos/primal.sock"));
            }
            other => panic!("expected UnixSocket, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn discover_via_registry_by_name_tcp_endpoint() {
        let result_json = serde_json::json!({
            "tcp_endpoint": "127.0.0.1:9100",
            "capabilities": []
        });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let found = discovery
            .discover_via_registry_by_name("remote")
            .await
            .expect("discovered");
        assert!(matches!(
            found.endpoint,
            TransportEndpoint::TcpSocket { .. }
        ));
    }

    #[tokio::test]
    async fn discover_via_registry_by_name_abstract_socket() {
        let result_json = serde_json::json!({
            "abstract_socket": "biomeos_primal_abc",
            "capabilities": []
        });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let found = discovery
            .discover_via_registry_by_name("abs")
            .await
            .expect("discovered");
        assert!(matches!(
            found.endpoint,
            TransportEndpoint::AbstractSocket { .. }
        ));
    }

    #[tokio::test]
    async fn discover_via_registry_by_name_no_endpoint_fields() {
        let result_json = serde_json::json!({ "capabilities": [] });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        assert!(discovery.discover_via_registry_by_name("x").await.is_none());
    }

    #[tokio::test]
    async fn discover_via_registry_by_name_invalid_tcp_returns_none() {
        let result_json = serde_json::json!({ "tcp_endpoint": "not-a-valid-tcp" });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        assert!(discovery.discover_via_registry_by_name("x").await.is_none());
    }

    #[tokio::test]
    async fn discover_via_registry_by_name_query_error_returns_none() {
        let line = r#"{"jsonrpc":"2.0","id":1,"error":{"message":"fail"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        assert!(discovery.discover_via_registry_by_name("p").await.is_none());
    }

    #[tokio::test]
    async fn discover_via_registry_by_capability_primary_socket() {
        let result_json = serde_json::json!({
            "primary_socket": "/tmp/cap.sock",
            "provider": "beardog"
        });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let found = discovery
            .discover_via_registry_by_capability("security.auth")
            .await
            .expect("discovered");
        assert_eq!(found.primal_name.as_deref(), Some("beardog"));
        assert!(found.capabilities.contains(&"security.auth".to_string()));
        match &found.endpoint {
            TransportEndpoint::UnixSocket { path } => {
                assert_eq!(path, &PathBuf::from("/tmp/cap.sock"));
            }
            other => panic!("expected UnixSocket, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn discover_via_registry_by_capability_tcp_only() {
        let result_json = serde_json::json!({ "tcp_endpoint": "10.0.0.2:4433" });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        let found = discovery
            .discover_via_registry_by_capability("mesh")
            .await
            .expect("discovered");
        assert!(found.primal_name.is_none());
        assert!(matches!(
            found.endpoint,
            TransportEndpoint::TcpSocket { .. }
        ));
    }

    #[tokio::test]
    async fn discover_via_registry_by_capability_no_socket_fields() {
        let result_json = serde_json::json!({ "provider": "x" });
        let line = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}}\n",
            result_json
        );
        let (_dir, sock) = spawn_neural_api_mock(line).await;
        let discovery = SocketDiscovery::new("fam").with_neural_api(sock.clone());
        assert!(
            discovery
                .discover_via_registry_by_capability("anything")
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn discover_via_registry_by_name_returns_none_without_neural_socket() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("no-neural.sock");
        let discovery = SocketDiscovery::new("fam").with_neural_api(missing);
        assert!(discovery.discover_via_registry_by_name("p").await.is_none());
    }
}
