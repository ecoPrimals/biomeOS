// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Request forwarding - JSON-RPC and tarpc protocol escalation

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::tarpc_types::ProtocolPreference;

use crate::living_graph::ProtocolMode;

use super::NeuralRouter;

impl NeuralRouter {
    /// Forward JSON-RPC request to primal
    ///
    /// **Pure Rust**: Async I/O, no unsafe code, idiomatic error handling
    ///
    /// **JSON-RPC AND tarpc first**: Checks protocol availability and preferences.
    pub async fn forward_request(
        &self,
        socket_path: &PathBuf,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        let start = std::time::Instant::now();

        // Determine protocol based on preference and availability
        let use_tarpc = self.should_use_tarpc(socket_path).await;

        if use_tarpc {
            match self.forward_via_tarpc(socket_path, method, params).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    debug!(
                        "tarpc forwarding failed for {}, falling back to JSON-RPC: {e}",
                        socket_path.display()
                    );
                }
            }
        }

        debug!(
            "   → Forwarding via JSON-RPC: {} to {}",
            method,
            socket_path.display()
        );

        let client = AtomicClient::unix(socket_path).with_timeout(self.request_timeout);

        let result = client.call(method, params.clone()).await.context(format!(
            "Failed to forward {} to {}",
            method,
            socket_path.display()
        ))?;

        let latency = start.elapsed().as_millis() as u64;
        debug!("   ✓ Forwarded successfully in {}ms", latency);

        if let Some(graph) = &self.living_graph {
            if let Some(primal_name) = socket_path.file_stem().and_then(|s| s.to_str()) {
                graph
                    .record_request("neural-api", primal_name, latency * 1000, true)
                    .await;
            }
        }

        Ok(result)
    }

    /// Forward a request via tarpc binary protocol for high-performance primal communication.
    pub async fn forward_via_tarpc(
        &self,
        socket_path: &std::path::Path,
        method: &str,
        params: &Value,
    ) -> Result<Value, String> {
        use crate::tarpc_client;
        use biomeos_types::tarpc_types::ServiceRegistration;
        use bytes::Bytes;
        use tarpc::context;

        let tarpc_path = biomeos_primal_sdk::tarpc_transport::tarpc_socket_path(socket_path);

        if !tarpc_path.exists() {
            return Err(format!("tarpc socket not found: {}", tarpc_path.display()));
        }

        let ctx = context::current();

        // Health methods
        if method == "health.check" || method == "health_check" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc health connect failed: {e}"))?;
            let status = client
                .health_check(ctx)
                .await
                .map_err(|e| format!("tarpc health_check failed: {e}"))?;
            return serde_json::to_value(&status).map_err(|e| e.to_string());
        }
        if method == "health.metrics" || method == "health_metrics" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc health connect failed: {e}"))?;
            let metrics = client
                .health_metrics(ctx)
                .await
                .map_err(|e| format!("tarpc health_metrics failed: {e}"))?;
            return serde_json::to_value(&metrics).map_err(|e| e.to_string());
        }
        if method == "health.version" || method == "version" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc health connect failed: {e}"))?;
            let info = client
                .version(ctx)
                .await
                .map_err(|e| format!("tarpc version failed: {e}"))?;
            return serde_json::to_value(&info).map_err(|e| e.to_string());
        }

        // Discovery methods
        if method.starts_with("discovery.") || method.starts_with("discovery_") {
            let client = tarpc_client::connect_tarpc_discovery(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc discovery connect failed: {e}"))?;
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
                        .map_err(|e| format!("tarpc discovery failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "discovery.discover_all" | "discovery_discover_all" => {
                    let result = client
                        .discover_all(ctx)
                        .await
                        .map_err(|e| format!("tarpc discovery failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "discovery.protocols" | "discovery_protocols" => {
                    let result = client
                        .protocols(ctx)
                        .await
                        .map_err(|e| format!("tarpc discovery failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
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
                        .map_err(|e| e.to_string())?;
                    return serde_json::to_value(ok).map_err(|e| e.to_string());
                }
                "discovery.register" | "discovery_register" => {
                    let reg: ServiceRegistration =
                        serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
                    let res = client.register(ctx, reg).await.map_err(|e| e.to_string())?;
                    return serde_json::to_value(&res).map_err(|e| e.to_string());
                }
                _ => return Err(format!("unknown discovery method: {method}")),
            };
        }

        // Security methods
        if method.starts_with("security.") || method.starts_with("security_") {
            let client = tarpc_client::connect_tarpc_security(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc security connect failed: {e}"))?;

            let bytes_from_param = |key: &str| -> Result<Bytes, String> {
                let v = params
                    .get(key)
                    .ok_or_else(|| format!("missing param: {key}"))?;
                if let Some(s) = v.as_str() {
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD
                        .decode(s)
                        .map(Bytes::from)
                        .map_err(|e| e.to_string())
                } else if let Some(arr) = v.as_array() {
                    let bytes: Vec<u8> = arr
                        .iter()
                        .filter_map(|x| x.as_u64().map(|u| u as u8))
                        .collect();
                    Ok(Bytes::from(bytes))
                } else {
                    Err(format!("param {key} must be base64 string or byte array"))
                }
            };

            match method {
                "security.sign" | "security_sign" => {
                    let data = bytes_from_param("data")?;
                    let result = client
                        .sign(ctx, data)
                        .await
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "security.verify" | "security_verify" => {
                    let data = bytes_from_param("data")?;
                    let signature = bytes_from_param("signature")?;
                    let public_key = bytes_from_param("public_key")?;
                    let ok = client
                        .verify(ctx, data, signature, public_key)
                        .await
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(ok).map_err(|e| e.to_string());
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
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
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
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                _ => return Err(format!("unknown security method: {method}")),
            };
        }

        Err(format!(
            "method {method} has no tarpc mapping, use JSON-RPC"
        ))
    }

    /// Check if tarpc should be used for this request
    pub(super) async fn should_use_tarpc(&self, socket_path: &std::path::Path) -> bool {
        match self.protocol_preference {
            ProtocolPreference::JsonRpcOnly => return false,
            ProtocolPreference::TarpcOnly => return true,
            ProtocolPreference::PreferJsonRpc => return false,
            ProtocolPreference::PreferTarpc | ProtocolPreference::Auto => {}
        }

        if let Some(graph) = &self.living_graph {
            if let Some(primal_name) = socket_path.file_stem().and_then(|s| s.to_str()) {
                if let Some(state) = graph.get_primal_state(primal_name).await {
                    return state.tarpc_available()
                        && matches!(
                            state.current_mode,
                            ProtocolMode::Tarpc | ProtocolMode::Hybrid
                        );
                }
            }
        }

        false
    }
}
