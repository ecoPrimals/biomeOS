// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Request forwarding - JSON-RPC and tarpc protocol escalation

use anyhow::{Context, Result};
use bytes::Bytes;
use serde_json::Value;
use tracing::debug;

use biomeos_core::TransportEndpoint;
use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::IpcError;
use biomeos_types::tarpc_types::ProtocolPreference;

use crate::living_graph::ProtocolMode;

use biomeos_core::btsp_client;

use super::NeuralRouter;

/// Decode `security.*` tarpc params that carry raw bytes (base64 string or JSON byte array).
pub fn parse_security_bytes_param(params: &Value, key: &str) -> Result<Bytes> {
    let v = params
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("missing param: {key}"))?;
    if let Some(s) = v.as_str() {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(s)
            .map(Bytes::from)
            .context("base64 decode failed")
    } else if let Some(arr) = v.as_array() {
        Ok(arr
            .iter()
            .filter_map(|x| x.as_u64().map(|u| u as u8))
            .collect::<Bytes>())
    } else {
        anyhow::bail!("param {key} must be base64 string or byte array")
    }
}

impl NeuralRouter {
    /// Forward JSON-RPC request to primal via any transport
    ///
    /// **Universal IPC v3.0**: Routes through Unix, abstract, TCP, or HTTP
    /// based on the endpoint's transport type.
    ///
    /// **JSON-RPC AND tarpc first**: Checks protocol availability and preferences.
    pub async fn forward_request(
        &self,
        endpoint: &TransportEndpoint,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        let start = std::time::Instant::now();

        let use_tarpc = self.should_use_tarpc(endpoint).await;

        if use_tarpc {
            if let TransportEndpoint::UnixSocket { path } = endpoint {
                match self.forward_via_tarpc(path, method, params).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        debug!(
                            "tarpc forwarding failed for {}, falling back to JSON-RPC: {e}",
                            endpoint.display_string()
                        );
                    }
                }
            }
        }

        // Secure Socket Architecture: BTSP client handshake for family-scoped sockets
        if let TransportEndpoint::UnixSocket { path } = endpoint {
            if btsp_client::is_family_scoped_socket(path) {
                match btsp_client::security_mode() {
                    btsp_client::SecurityMode::Production { btsp_available } => {
                        if btsp_available {
                            debug!(
                                "   🔒 BTSP: performing client handshake for {}",
                                path.display()
                            );

                            let btsp_client = AtomicClient::from_endpoint(endpoint.clone())
                                .with_timeout(self.request_timeout);
                            match btsp_client.call_btsp(method, params.clone()).await {
                                Ok(value) => {
                                    let latency = start.elapsed().as_millis() as u64;
                                    debug!("   ✓ Forwarded (BTSP) in {}ms", latency);
                                    if let Some(graph) = &self.living_graph {
                                        let primal_label = self.primal_label_for_endpoint(endpoint);
                                        if let Some(label) = primal_label {
                                            graph
                                                .record_request(
                                                    "neural-api",
                                                    &label,
                                                    latency * 1000,
                                                    true,
                                                )
                                                .await;
                                        }
                                    }
                                    return Ok(value);
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "   ⚠️ BTSP handshake failed for {}, falling back to raw JSON-RPC: {e}",
                                        path.display()
                                    );
                                }
                            }
                        } else if btsp_client::btsp_enforce() {
                            tracing::warn!(
                                "   ⚠️ BTSP enforced but security provider unavailable for family-scoped socket: {}",
                                path.display()
                            );
                        } else {
                            debug!(
                                "   ⚠️ BTSP: security provider unavailable, proceeding without handshake to {}",
                                path.display()
                            );
                        }
                    }
                    btsp_client::SecurityMode::Development => {
                        debug!(
                            "   🔓 Development mode — skipping BTSP for {}",
                            path.display()
                        );
                    }
                }
            }
        }

        debug!(
            "   → Forwarding via JSON-RPC: {} to {}",
            method,
            endpoint.display_string()
        );

        let client =
            AtomicClient::from_endpoint(endpoint.clone()).with_timeout(self.request_timeout);

        let result = match client.try_call(method, params.clone()).await {
            Ok(value) => value,
            Err(e @ IpcError::JsonRpcError { .. }) => {
                return Err(e.into());
            }
            Err(e) => {
                return Err(anyhow::Error::from(e).context(format!(
                    "Failed to forward {} to {}",
                    method,
                    endpoint.display_string()
                )));
            }
        };

        let latency = start.elapsed().as_millis() as u64;
        debug!("   ✓ Forwarded successfully in {}ms", latency);

        if let Some(graph) = &self.living_graph {
            let primal_label = self.primal_label_for_endpoint(endpoint);
            if let Some(label) = primal_label {
                graph
                    .record_request("neural-api", &label, latency * 1000, true)
                    .await;
            }
        }

        Ok(result)
    }

    /// Forward with an envelope-derived timeout cap (JH-2).
    ///
    /// If `timeout_cap` is `Some`, the actual forwarding timeout is
    /// `min(self.request_timeout, timeout_cap)`. This ensures scoped
    /// tokens with `timeout_ms` cannot exceed the system default but
    /// *can* impose a tighter deadline.
    pub async fn forward_request_with_timeout(
        &self,
        endpoint: &TransportEndpoint,
        method: &str,
        params: &Value,
        timeout_cap: Option<std::time::Duration>,
    ) -> Result<Value> {
        if let Some(cap) = timeout_cap {
            let effective = self.request_timeout.min(cap);
            return self
                .forward_request_inner(endpoint, method, params, effective)
                .await;
        }
        self.forward_request(endpoint, method, params).await
    }

    /// Inner forwarding with an explicit timeout (used by `forward_request_with_timeout`).
    async fn forward_request_inner(
        &self,
        endpoint: &TransportEndpoint,
        method: &str,
        params: &Value,
        timeout: std::time::Duration,
    ) -> Result<Value> {
        let start = std::time::Instant::now();

        debug!(
            "   → Forwarding via JSON-RPC (timeout {}ms): {} to {}",
            timeout.as_millis(),
            method,
            endpoint.display_string()
        );

        let client = AtomicClient::from_endpoint(endpoint.clone()).with_timeout(timeout);

        let result = match client.try_call(method, params.clone()).await {
            Ok(value) => value,
            Err(e @ IpcError::JsonRpcError { .. }) => {
                return Err(e.into());
            }
            Err(e) => {
                return Err(anyhow::Error::from(e).context(format!(
                    "Failed to forward {} to {} (timeout {}ms)",
                    method,
                    endpoint.display_string(),
                    timeout.as_millis()
                )));
            }
        };

        let latency = start.elapsed().as_millis() as u64;
        debug!("   ✓ Forwarded successfully in {}ms", latency);

        if let Some(graph) = &self.living_graph {
            let primal_label = self.primal_label_for_endpoint(endpoint);
            if let Some(label) = primal_label {
                graph
                    .record_request("neural-api", &label, latency * 1000, true)
                    .await;
            }
        }

        Ok(result)
    }

    /// Extract a human-readable primal label from an endpoint for metrics
    pub(crate) fn primal_label_for_endpoint(&self, endpoint: &TransportEndpoint) -> Option<String> {
        match endpoint {
            TransportEndpoint::UnixSocket { path } => {
                path.file_stem().and_then(|s| s.to_str()).map(String::from)
            }
            TransportEndpoint::AbstractSocket { name } => Some(name.to_string()),
            TransportEndpoint::TcpSocket { host, port } => Some(format!("{host}:{port}")),
            TransportEndpoint::HttpJsonRpc { host, port } => Some(format!("{host}:{port}")),
        }
    }

    /// Forward a request via tarpc binary protocol for high-performance primal communication.
    pub async fn forward_via_tarpc(
        &self,
        socket_path: &std::path::Path,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        use crate::tarpc_client;
        use biomeos_types::tarpc_types::ServiceRegistration;
        use tarpc::context;

        let tarpc_path = biomeos_primal_sdk::tarpc_transport::tarpc_socket_path(socket_path);

        anyhow::ensure!(
            tarpc_path.exists(),
            "tarpc socket not found: {}",
            tarpc_path.display()
        );

        let ctx = context::current();

        // Health methods
        if method == "health.check" || method == "health_check" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .context("tarpc health connect failed")?;
            let status = client
                .health_check(ctx)
                .await
                .context("tarpc health_check failed")?;
            return Ok(serde_json::to_value(&status)?);
        }
        if method == "health.metrics" || method == "health_metrics" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .context("tarpc health connect failed")?;
            let metrics = client
                .health_metrics(ctx)
                .await
                .context("tarpc health_metrics failed")?;
            return Ok(serde_json::to_value(&metrics)?);
        }
        if method == "health.version" || method == "version" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .context("tarpc health connect failed")?;
            let info = client
                .version(ctx)
                .await
                .context("tarpc version failed")?;
            return Ok(serde_json::to_value(&info)?);
        }

        // Discovery methods
        if method.starts_with("discovery.") || method.starts_with("discovery_") {
            let client = tarpc_client::connect_tarpc_discovery(&tarpc_path)
                .await
                .context("tarpc discovery connect failed")?;
            match method {
                "discovery.discover" | "discovery_discover" => {
                    let capability = params
                        .get("capability")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let result = client
                        .discover(ctx, capability)
                        .await
                        .context("tarpc discovery failed")?;
                    return Ok(serde_json::to_value(&result)?);
                }
                "discovery.discover_all" | "discovery_discover_all" => {
                    let result = client
                        .discover_all(ctx)
                        .await
                        .context("tarpc discovery failed")?;
                    return Ok(serde_json::to_value(&result)?);
                }
                "discovery.protocols" | "discovery_protocols" => {
                    let result = client
                        .protocols(ctx)
                        .await
                        .context("tarpc discovery failed")?;
                    return Ok(serde_json::to_value(&result)?);
                }
                "discovery.unregister" | "discovery_unregister" => {
                    let primal_id = params
                        .get("primal_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let ok = client
                        .unregister(ctx, primal_id)
                        .await
                        .context("tarpc unregister failed")?;
                    return Ok(serde_json::to_value(ok)?);
                }
                "discovery.register" | "discovery_register" => {
                    let reg: ServiceRegistration =
                        serde_json::from_value(params.clone()).context("invalid registration")?;
                    let res = client.register(ctx, reg).await.context("tarpc register failed")?;
                    return Ok(serde_json::to_value(&res)?);
                }
                _ => anyhow::bail!("unknown discovery method: {method}"),
            };
        }

        // Security methods
        if method.starts_with("security.") || method.starts_with("security_") {
            let client = tarpc_client::connect_tarpc_security(&tarpc_path)
                .await
                .context("tarpc security connect failed")?;

            match method {
                "security.sign" | "security_sign" => {
                    let data = parse_security_bytes_param(params, "data")?;
                    let result = client
                        .sign(ctx, data)
                        .await
                        .context("tarpc security.sign failed")?;
                    return Ok(serde_json::to_value(&result)?);
                }
                "security.verify" | "security_verify" => {
                    let data = parse_security_bytes_param(params, "data")?;
                    let signature = parse_security_bytes_param(params, "signature")?;
                    let public_key = parse_security_bytes_param(params, "public_key")?;
                    let ok = client
                        .verify(ctx, data, signature, public_key)
                        .await
                        .context("tarpc security.verify failed")?;
                    return Ok(serde_json::to_value(ok)?);
                }
                "security.get_jwt_secret" | "security_get_jwt_secret" => {
                    let service_name = params
                        .get("service_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let result = client
                        .get_jwt_secret(ctx, service_name)
                        .await
                        .context("tarpc get_jwt_secret failed")?;
                    return Ok(serde_json::to_value(&result)?);
                }
                "security.verify_lineage" | "security_verify_lineage" => {
                    let primal_id = params
                        .get("primal_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let result = client
                        .verify_lineage(ctx, primal_id)
                        .await
                        .context("tarpc verify_lineage failed")?;
                    return Ok(serde_json::to_value(&result)?);
                }
                _ => anyhow::bail!("unknown security method: {method}"),
            };
        }

        anyhow::bail!("method {method} has no tarpc mapping, use JSON-RPC")
    }

    /// Check if tarpc should be used for this request.
    ///
    /// tarpc escalation is only available for Unix socket endpoints (tarpc uses
    /// a sibling `.tarpc.sock` file).
    pub(crate) async fn should_use_tarpc(&self, endpoint: &TransportEndpoint) -> bool {
        match self.protocol_preference {
            ProtocolPreference::JsonRpcOnly | ProtocolPreference::PreferJsonRpc => return false,
            ProtocolPreference::TarpcOnly => return true,
            ProtocolPreference::PreferTarpc | ProtocolPreference::Auto => {}
        }

        let primal_name = match endpoint {
            TransportEndpoint::UnixSocket { path } => {
                path.file_stem().and_then(|s| s.to_str()).map(String::from)
            }
            TransportEndpoint::AbstractSocket { name } => Some(name.to_string()),
            _ => None,
        };

        if let (Some(graph), Some(name)) = (&self.living_graph, primal_name) {
            if let Some(state) = graph.get_primal_state(&name).await {
                return state.tarpc_available()
                    && matches!(
                        state.current_mode,
                        ProtocolMode::Tarpc | ProtocolMode::Hybrid
                    );
            }
        }

        false
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use base64::Engine;
    use biomeos_core::TransportEndpoint;
    use biomeos_types::tarpc_types::ProtocolPreference;
    use serde_json::json;

    #[test]
    fn test_parse_security_bytes_param_base64_roundtrip() {
        let params = serde_json::json!({
            "data": base64::engine::general_purpose::STANDARD.encode(b"hello-bytes"),
        });
        let out = parse_security_bytes_param(&params, "data").expect("decode");
        assert_eq!(out.as_ref(), b"hello-bytes");
    }

    #[test]
    fn test_parse_security_bytes_param_byte_array() {
        let params = serde_json::json!({ "data": [1u64, 2, 3] });
        let out = parse_security_bytes_param(&params, "data").expect("bytes");
        assert_eq!(out.as_ref(), &[1u8, 2, 3]);
    }

    #[test]
    fn test_parse_security_bytes_param_missing_key() {
        let msg = parse_security_bytes_param(&serde_json::json!({}), "data")
            .unwrap_err()
            .to_string();
        assert!(msg.contains("missing param"));
    }

    #[test]
    fn test_parse_security_bytes_param_invalid_base64() {
        let params = serde_json::json!({ "data": "@@@not-base64@@@" });
        let msg = parse_security_bytes_param(&params, "data")
            .unwrap_err()
            .to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_parse_security_bytes_param_wrong_json_type() {
        let params = serde_json::json!({ "data": 42 });
        let msg = parse_security_bytes_param(&params, "data")
            .unwrap_err()
            .to_string();
        assert!(msg.contains("base64 string or byte array"));
    }

    #[test]
    fn test_parse_security_bytes_param_empty_array() {
        let params = json!({ "data": [] });
        let out = parse_security_bytes_param(&params, "data").expect("empty");
        assert!(out.is_empty());
    }

    #[test]
    fn primal_label_for_endpoint_unix_socket_stem() {
        let router = NeuralRouter::new("fam");
        let ep = TransportEndpoint::UnixSocket {
            path: std::path::PathBuf::from("/run/biomeos/beardog-f1.sock"),
        };
        assert_eq!(
            router.primal_label_for_endpoint(&ep),
            Some("beardog-f1".to_string())
        );
    }

    #[test]
    fn primal_label_for_endpoint_abstract() {
        let router = NeuralRouter::new("fam");
        let ep = TransportEndpoint::AbstractSocket {
            name: "biomeos-abstract".into(),
        };
        assert_eq!(
            router.primal_label_for_endpoint(&ep),
            Some("biomeos-abstract".to_string())
        );
    }

    #[test]
    fn primal_label_for_endpoint_tcp() {
        let router = NeuralRouter::new("fam");
        let ep = TransportEndpoint::TcpSocket {
            host: "127.0.0.1".into(),
            port: 9000,
        };
        assert_eq!(
            router.primal_label_for_endpoint(&ep),
            Some("127.0.0.1:9000".to_string())
        );
    }

    #[test]
    fn primal_label_for_endpoint_http_jsonrpc() {
        let router = NeuralRouter::new("fam");
        let ep = TransportEndpoint::HttpJsonRpc {
            host: "10.0.0.1".into(),
            port: 8443,
        };
        assert_eq!(
            router.primal_label_for_endpoint(&ep),
            Some("10.0.0.1:8443".to_string())
        );
    }

    #[tokio::test]
    async fn should_use_tarpc_json_rpc_only_returns_false() {
        let router =
            NeuralRouter::new("fam").with_protocol_preference(ProtocolPreference::JsonRpcOnly);
        let ep = TransportEndpoint::UnixSocket {
            path: std::path::PathBuf::from("/tmp/x.sock"),
        };
        assert!(!router.should_use_tarpc(&ep).await);
    }

    #[tokio::test]
    async fn should_use_tarpc_prefer_json_rpc_returns_false() {
        let router =
            NeuralRouter::new("fam").with_protocol_preference(ProtocolPreference::PreferJsonRpc);
        let ep = TransportEndpoint::UnixSocket {
            path: std::path::PathBuf::from("/tmp/x.sock"),
        };
        assert!(!router.should_use_tarpc(&ep).await);
    }

    #[tokio::test]
    async fn should_use_tarpc_tarpc_only_returns_true() {
        let router =
            NeuralRouter::new("fam").with_protocol_preference(ProtocolPreference::TarpcOnly);
        let ep = TransportEndpoint::UnixSocket {
            path: std::path::PathBuf::from("/tmp/x.sock"),
        };
        assert!(router.should_use_tarpc(&ep).await);
    }

    #[tokio::test]
    async fn forward_via_tarpc_errors_when_tarpc_socket_missing() {
        let router = NeuralRouter::new("fam");
        let p = std::path::Path::new("/tmp/biomeos_forward_test_no_such.sock");
        let msg = router
            .forward_via_tarpc(p, "health.check", &json!({}))
            .await
            .expect_err("missing tarpc")
            .to_string();
        assert!(msg.contains("tarpc socket not found"));
    }
}
