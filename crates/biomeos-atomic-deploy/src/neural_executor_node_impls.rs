// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Node executor implementations for `GraphExecutor`.
//!
//! Extracted from `neural_executor.rs` for maintainability. Contains the
//! heavyweight node handlers: verification, health_check_all, rpc_call,
//! and capability_call — plus the `send_jsonrpc_async` helper.

use anyhow::{Context, Result};
use biomeos_types::JsonRpcRequest;
use serde::Serialize;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::capability_domains::CapabilityRegistry;
use crate::executor::context::ExecutionContext;
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::GraphNode;

impl GraphExecutor {
    /// Node executor: verification
    /// Verifies primal health by checking sockets and optionally querying via JSON-RPC
    pub(crate) async fn node_verification(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let check_sockets = node
            .config
            .get("check_sockets")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let check_health = node
            .config
            .get("check_health")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!("   Verifying ecosystem...");

        if check_sockets {
            let _socket_dir = context
                .env
                .get("SOCKET_DIR")
                .ok_or_else(|| anyhow::anyhow!("SOCKET_DIR not set"))?;

            let mut verified = Vec::new();
            for dep_id in &node.dependencies {
                if let Some(dep_output) = context.get_output(dep_id).await {
                    if let Some(socket) = dep_output.get("socket").and_then(|v| v.as_str()) {
                        let socket_path = std::path::PathBuf::from(socket);
                        if socket_path.exists() {
                            info!("      ✅ {} socket exists", dep_id);
                            verified.push(dep_id.clone());
                        } else {
                            anyhow::bail!("Socket not found for {dep_id}: {socket}");
                        }
                    }
                }
            }

            info!("   ✅ Verified {} primals", verified.len());

            Ok(serde_json::json!({
                "verified_count": verified.len(),
                "verified_primals": verified,
                "check_sockets": true,
                "check_health": check_health
            }))
        } else {
            Ok(serde_json::json!({
                "verified_count": 0,
                "check_sockets": false
            }))
        }
    }

    /// Node executor: health.check_all
    /// Checks health of all primals by scanning socket directory
    pub(crate) async fn node_health_check_all(
        _node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let socket_dir = context
            .env
            .get("SOCKET_DIR")
            .ok_or_else(|| anyhow::anyhow!("SOCKET_DIR not set"))?;

        info!("   Checking health of all primals in {}", socket_dir);

        let socket_dir = PathBuf::from(socket_dir);
        let mut healthy_primals = Vec::new();

        if !socket_dir.exists() {
            warn!(
                "   Socket directory does not exist: {}",
                socket_dir.display()
            );
            return Ok(serde_json::json!({
                "healthy_count": 0,
                "primals": []
            }));
        }

        let entries = std::fs::read_dir(&socket_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                if let Some(primal_name) = path.file_stem().and_then(|s| s.to_str()) {
                    healthy_primals.push(primal_name.to_string());
                }
            }
        }

        info!("   ✅ Found {} healthy primals", healthy_primals.len());

        Ok(serde_json::json!({
            "healthy_count": healthy_primals.len(),
            "primals": healthy_primals
        }))
    }

    /// Node executor: rpc_call
    /// Makes a JSON-RPC call to a target primal, protected by a per-primal circuit breaker.
    ///
    /// The circuit breaker prevents cascade failures: after 5 consecutive RPC
    /// failures to a primal, subsequent calls fail fast for 30 s before retrying.
    pub(crate) async fn node_rpc_call(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use std::time::Duration;

        let target = node
            .config
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("rpc_call requires 'target' config (primal name)"))?;

        let method = node
            .config
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("rpc_call requires 'method' config"))?;

        let params = node
            .config
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let params_str = serde_json::to_string(&params)?;
        let params_expanded = crate::executor::substitute_env(&params_str, context.env());
        let params: serde_json::Value = serde_json::from_str(&params_expanded)?;

        info!("   📞 RPC call to {}: {}({:?})", target, method, params);

        let socket_path = context.get_socket_path(target).await;
        let breaker = context.get_circuit_breaker(target).await;

        let target_owned = target.to_string();
        let method_owned = method.to_string();

        breaker
            .execute(|| {
                let socket_path = socket_path.clone();
                let target = target_owned.clone();
                let method = method_owned.clone();
                let params = params.clone();

                async move {
                    let request = JsonRpcRequest::new(&method, params);

                    let stream = tokio::time::timeout(
                        Duration::from_secs(10),
                        tokio::net::UnixStream::connect(&socket_path),
                    )
                    .await
                    .context(format!("Timeout connecting to {target} at {socket_path}"))?
                    .context(format!("Failed to connect to {target} at {socket_path}"))?;

                    let (read_half, mut write_half) = stream.into_split();

                    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
                    let request_json = serde_json::to_string(&request)?;
                    write_half.write_all(request_json.as_bytes()).await?;
                    write_half.write_all(b"\n").await?;
                    write_half.flush().await?;

                    let mut reader = BufReader::new(read_half);
                    let mut response_line = String::new();
                    tokio::time::timeout(
                        Duration::from_secs(30),
                        reader.read_line(&mut response_line),
                    )
                    .await
                    .context(format!("Timeout waiting for {target} response"))?
                    .context(format!("Failed to read response from {target}"))?;

                    let response: serde_json::Value = serde_json::from_str(&response_line)
                        .context(format!("Invalid JSON response from {target}"))?;

                    if let Some(error) = response.get("error") {
                        let error_msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");
                        anyhow::bail!("RPC error from {target}: {error_msg}");
                    }

                    let result = response
                        .get("result")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);

                    info!("   ✅ RPC call successful: {} → {:?}", method, result);

                    Ok(serde_json::json!({
                        "target": target,
                        "method": method,
                        "result": result,
                        "success": true
                    }))
                }
            })
            .await
    }

    /// Node executor: capability_call with config-driven registry.
    ///
    /// Routes semantic capability calls through the neural-api first, then falls
    /// back to direct primal resolution using the `CapabilityRegistry` (which
    /// itself falls back to the compiled-in `CAPABILITY_DOMAINS` const).
    pub(crate) async fn node_capability_call_with_registry(
        node: &GraphNode,
        context: &ExecutionContext,
        capability_registry: &CapabilityRegistry,
    ) -> Result<serde_json::Value> {
        use std::time::Duration;

        let capability = node
            .config
            .get("capability")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("capability_call requires 'capability' config"))?;

        let params = node
            .config
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let params_str = serde_json::to_string(&params)?;
        let params_expanded = crate::executor::substitute_env(&params_str, context.env());
        let params: serde_json::Value = serde_json::from_str(&params_expanded)?;

        info!("   🔬 Capability call: {}({:?})", capability, params);

        let timeout_ms = node
            .config
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(30_000);

        let (cap_domain, cap_operation) = Self::split_capability(capability);

        // Strategy 1: Route via neural-api capability.call
        let neural_api_socket = context.get_socket_path("neural-api").await;
        let neural_api_path = std::path::PathBuf::from(&neural_api_socket);

        if neural_api_path.exists() {
            let request = JsonRpcRequest::new(
                "capability.call",
                serde_json::json!({
                    "capability": &cap_domain,
                    "operation": &cap_operation,
                    "args": params,
                }),
            );

            match tokio::time::timeout(
                Duration::from_millis(timeout_ms),
                Self::send_jsonrpc_async(&neural_api_socket, &request),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if let Some(error) = response.get("error") {
                        let msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("unknown");
                        warn!(
                            "   ⚠️ capability.call({}) via neural-api failed: {}, trying direct",
                            capability, msg
                        );
                    } else {
                        let result = response
                            .get("result")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        info!(
                            "   ✅ Capability call via neural-api: {} → success",
                            capability
                        );
                        return Ok(serde_json::json!({
                            "capability": capability,
                            "routed_via": "neural-api",
                            "result": result,
                            "success": true,
                        }));
                    }
                }
                Ok(Err(e)) => {
                    warn!(
                        "   ⚠️ neural-api unreachable for {}: {}, trying direct",
                        capability, e
                    );
                }
                Err(_) => {
                    warn!(
                        "   ⚠️ neural-api timeout for {} ({}ms), trying direct",
                        capability, timeout_ms
                    );
                }
            }
        }

        // Strategy 2: Config-driven resolution via CapabilityRegistry
        let provider = capability_registry
            .resolve(capability)
            .or_else(|| capability_registry.resolve(&cap_domain));

        let provider = provider.ok_or_else(|| {
            anyhow::anyhow!(
                "No provider found for capability '{capability}' (neither neural-api nor registry)"
            )
        })?;

        info!(
            "   📞 Direct capability call: {} → {} ({})",
            capability, provider, cap_operation
        );

        let socket_path = context.get_socket_path(&provider).await;
        let breaker = context.get_circuit_breaker(&provider).await;

        let cap_owned = capability.to_string();

        breaker
            .execute(|| {
                let socket_path = socket_path.clone();
                let cap = cap_owned.clone();
                let provider = provider.clone();
                let params = params.clone();

                async move {
                    let request = JsonRpcRequest::new(&cap, params);

                    let response = tokio::time::timeout(
                        Duration::from_millis(timeout_ms),
                        Self::send_jsonrpc_async(&socket_path, &request),
                    )
                    .await
                    .context(format!("Timeout on capability call: {cap}"))?
                    .context(format!(
                        "Failed capability call {cap} → {provider} at {socket_path}"
                    ))?;

                    if let Some(error) = response.get("error") {
                        let msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("unknown");
                        anyhow::bail!("Capability call {cap} failed: {msg}");
                    }

                    let result = response
                        .get("result")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);

                    info!("   ✅ Direct capability call: {} → success", cap);

                    Ok(serde_json::json!({
                        "capability": cap,
                        "routed_via": provider,
                        "result": result,
                        "success": true,
                    }))
                }
            })
            .await
    }

    /// Helper: send a JSON-RPC request over a Unix socket and return the response.
    pub(crate) async fn send_jsonrpc_async(
        socket_path: &str,
        request: &impl Serialize,
    ) -> Result<serde_json::Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream = UnixStream::connect(socket_path)
            .await
            .context(format!("Connecting to {socket_path}"))?;

        let (read_half, mut write_half) = stream.into_split();

        let payload = serde_json::to_string(request)?;
        write_half.write_all(payload.as_bytes()).await?;
        write_half.write_all(b"\n").await?;
        write_half.flush().await?;

        let mut reader = BufReader::new(read_half);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        serde_json::from_str(line.trim()).context("Invalid JSON response")
    }
}
