// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Typed capability wrappers for primal-to-primal communication.
//!
//! Instead of raw JSON-RPC calls, primals use these typed methods to discover
//! and invoke capabilities. Each method handles discovery, routing, and
//! response parsing — primals only need to know *what* they want, not *who* provides it.
//!
//! Follows groundSpring's typed capability pattern.

use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};

use biomeos_types::{JsonRpcRequest, JsonRpcResponse};

/// Typed capability client for structured primal-to-primal IPC.
///
/// Wraps JSON-RPC calls with domain-specific methods so primals
/// can compose without knowing provider identities.
pub struct CapabilityClient {
    /// Socket path for the Neural API / capability router
    endpoint: PathBuf,
    /// Request timeout
    timeout: Duration,
}

impl CapabilityClient {
    /// Create a new capability client targeting a Neural API endpoint.
    pub fn new(endpoint: impl Into<PathBuf>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Discover from environment: `NEURAL_API_SOCKET` → XDG → fallback
    ///
    /// Uses the same 5-tier resolution as the discovery module:
    /// 1. `NEURAL_API_SOCKET` env var (if set and path exists)
    /// 2. `BIOMEOS_SOCKET_DIR` / `neural-api.sock`
    /// 3. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock`
    /// 4. `/run/user/{uid}/biomeos/neural-api.sock`
    /// 5. Platform temp dir fallback
    pub fn discover() -> Result<Self> {
        let path = resolve_neural_api_socket()?;
        Ok(Self::new(path))
    }

    /// Set request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a `capability.call` request and return the result.
    async fn capability_call(
        &self,
        capability: &str,
        operation: &str,
        args: Value,
    ) -> Result<Value> {
        let params = json!({
            "capability": capability,
            "operation": operation,
            "args": args,
        });
        let request = JsonRpcRequest::new("capability.call", params);
        let response = self.send_request(request).await?;
        if let Some(error) = response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }
        response
            .result
            .ok_or_else(|| anyhow!("No result in response"))
    }

    /// Send a raw JSON-RPC request over the Unix socket.
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let mut stream = timeout(self.timeout, UnixStream::connect(&self.endpoint))
            .await
            .context("Connection timeout")?
            .with_context(|| format!("Failed to connect to {}", self.endpoint.display()))?;

        let request_json = serde_json::to_vec(&request)?;
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut response_buf = Vec::new();
        let _ = timeout(self.timeout, stream.read_to_end(&mut response_buf))
            .await
            .context("Read timeout")?;

        let response: JsonRpcResponse =
            serde_json::from_slice(&response_buf).context("Failed to parse JSON-RPC response")?;
        Ok(response)
    }

    // --- Crypto domain ---

    /// Sign data using the ecosystem's crypto provider (`BearDog`).
    pub async fn crypto_sign(&self, data: &[u8]) -> Result<Bytes> {
        let args = json!({
            "data": base64_encode(data),
        });
        let result = self.capability_call("crypto", "sign", args).await?;
        let sig_b64 = result
            .get("signature")
            .or_else(|| result.get("result"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing signature in response"))?;
        base64_decode(sig_b64)
    }

    /// Verify a signature.
    pub async fn crypto_verify(
        &self,
        data: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool> {
        let args = json!({
            "data": base64_encode(data),
            "signature": base64_encode(signature),
            "public_key": base64_encode(public_key),
        });
        let result = self.capability_call("crypto", "verify", args).await?;
        result
            .get("valid")
            .or_else(|| result.get("result"))
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Missing valid/result in response"))
    }

    /// Generate a cryptographic hash.
    pub async fn crypto_hash(&self, data: &[u8], algorithm: &str) -> Result<Bytes> {
        let args = json!({
            "data": base64_encode(data),
            "algorithm": algorithm,
        });
        let result = self.capability_call("crypto", "hash", args).await?;
        let hash_b64 = result
            .get("hash")
            .or_else(|| result.get("result"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing hash in response"))?;
        base64_decode(hash_b64)
    }

    // --- HTTP domain ---

    /// Make an HTTP request through the ecosystem's network provider (Songbird).
    pub async fn http_request(
        &self,
        method: &str,
        url: &str,
        headers: Option<Value>,
        body: Option<&str>,
    ) -> Result<Value> {
        let mut args = json!({
            "method": method,
            "url": url,
        });
        if let Some(h) = headers {
            args["headers"] = h;
        }
        if let Some(b) = body {
            args["body"] = json!(b);
        }
        self.capability_call("http", "request", args).await
    }

    // --- Storage domain ---

    /// Store data via the ecosystem's storage provider (`NestGate`).
    pub async fn storage_put(&self, key: &str, value: &[u8]) -> Result<()> {
        let args = json!({
            "key": key,
            "value": base64_encode(value),
        });
        self.capability_call("storage", "put", args).await?;
        Ok(())
    }

    /// Retrieve stored data.
    pub async fn storage_get(&self, key: &str) -> Result<Option<Bytes>> {
        let args = json!({ "key": key });
        let result = self.capability_call("storage", "get", args).await?;
        let value_b64 = result.get("value").or_else(|| result.get("result"));
        match value_b64 {
            Some(Value::Null) | None => Ok(None),
            Some(v) => {
                let s = v.as_str().ok_or_else(|| anyhow!("Expected string value"))?;
                Ok(Some(base64_decode(s)?))
            }
        }
    }

    /// Check if a key exists in storage.
    pub async fn storage_exists(&self, key: &str) -> Result<bool> {
        let args = json!({ "key": key });
        let result = self.capability_call("storage", "exists", args).await?;
        result
            .get("exists")
            .or_else(|| result.get("result"))
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Missing exists/result in response"))
    }

    // --- Compute domain ---

    /// Execute a compute task via the ecosystem's compute provider (Toadstool).
    pub async fn compute_execute(&self, task: &str, params: Value) -> Result<Value> {
        let args = json!({
            "task": task,
            "params": params,
        });
        self.capability_call("compute", "execute", args).await
    }

    // --- Discovery domain ---

    /// Discover which primal provides a given capability.
    pub async fn discover_capability(&self, capability: &str) -> Result<Vec<String>> {
        let params = json!({ "capability": capability });
        let request = JsonRpcRequest::new("capability.discover", params);
        let response = self.send_request(request).await?;
        if let Some(error) = response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }
        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let primals = result
            .get("primals")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(primals)
    }

    /// List all available capability translations.
    pub async fn list_translations(&self) -> Result<Value> {
        let request = JsonRpcRequest::new("capabilities.list", json!({}));
        let response = self.send_request(request).await?;
        if let Some(error) = response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }
        response
            .result
            .ok_or_else(|| anyhow!("No result in response"))
    }

    // --- Health domain ---

    /// Check health of a specific primal.
    pub async fn health_check(&self, primal: &str) -> Result<Value> {
        let args = json!({ "primal": primal });
        self.capability_call("health", "check", args).await
    }
}

/// Resolve Neural API socket path using 5-tier discovery.
///
/// If `neural_api_socket` is `Some` and the path exists, it is returned without reading
/// `NEURAL_API_SOCKET` (for tests and explicit wiring).
pub fn resolve_neural_api_socket_with(
    neural_api_socket: Option<&std::path::Path>,
) -> Result<PathBuf> {
    if let Some(p) = neural_api_socket {
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }
    resolve_neural_api_socket()
}

fn resolve_neural_api_socket() -> Result<PathBuf> {
    // Tier 1: Explicit NEURAL_API_SOCKET
    if let Ok(path) = std::env::var("NEURAL_API_SOCKET") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Tier 2–5: Use RuntimeConfig socket dir + neural-api.sock
    let config = biomeos_types::RuntimeConfig::from_env();
    let path = config.neural_api_socket();
    if path.exists() {
        return Ok(path);
    }

    // Also try family_id suffix (neural-api-{family}.sock)
    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".to_string());
    let socket_dir = config.socket_dir();
    let path_with_family = socket_dir.join(format!("neural-api-{family_id}.sock"));
    if path_with_family.exists() {
        return Ok(path_with_family);
    }

    Err(anyhow!(
        "Neural API socket not found. Set NEURAL_API_SOCKET or ensure biomeOS is running."
    ))
}

/// Simple base64 encoding (no external dependency).
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
        let b2 = u32::from(*chunk.get(2).unwrap_or(&0));
        let triple = (b0 << 16) | (b1 << 8) | b2;
        let idx = |shift: u32| usize::try_from((triple >> shift) & 0x3F).unwrap_or(0);
        result.push(char::from(ALPHABET[idx(18)]));
        result.push(char::from(ALPHABET[idx(12)]));
        result.push(if chunk.len() > 1 {
            char::from(ALPHABET[idx(6)])
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            char::from(ALPHABET[idx(0)])
        } else {
            '='
        });
    }
    result
}

/// Simple base64 decoding (no external dependency).
fn base64_decode(s: &str) -> Result<Bytes> {
    let s = s.trim_end_matches('=');
    let decode_char = |c: u8| -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    let bytes: Vec<u8> = s.bytes().filter_map(decode_char).collect();
    let mut result = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let n = chunk.len();
        if n < 2 {
            break;
        }
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(chunk[1]);
        let v = (b0 << 2) | (b1 >> 4);
        result.push(u8::try_from(v & 0xFF).unwrap_or(0));
        if n >= 3 {
            let b2 = u32::from(chunk[2]);
            let v = ((b1 & 0x0F) << 4) | (b2 >> 2);
            result.push(u8::try_from(v & 0xFF).unwrap_or(0));
        }
        if n >= 4 {
            let b2 = u32::from(chunk[2]);
            let b3 = u32::from(chunk[3]);
            let v = ((b2 & 0x03) << 6) | b3;
            result.push(u8::try_from(v & 0xFF).unwrap_or(0));
        }
    }
    Ok(Bytes::from(result))
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
mod tests;
