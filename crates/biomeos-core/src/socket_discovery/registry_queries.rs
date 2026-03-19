// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
