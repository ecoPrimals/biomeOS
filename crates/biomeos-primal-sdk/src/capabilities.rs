// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

    /// Discover from environment: NEURAL_API_SOCKET → XDG → fallback
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
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
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

    /// Sign data using the ecosystem's crypto provider (BearDog).
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

    /// Store data via the ecosystem's storage provider (NestGate).
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
        let request = JsonRpcRequest::new("capability.list", json!({}));
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
    use serial_test::serial;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    #[test]
    fn test_capability_client_new() {
        let _client = CapabilityClient::new("/tmp/neural-api.sock");
        // Construction succeeds with path
    }

    #[test]
    fn test_capability_client_with_timeout() {
        let _client = CapabilityClient::new("/tmp/sock").with_timeout(Duration::from_secs(5));
        // Builder pattern compiles and works
    }

    #[test]
    fn test_capability_call_request_format() {
        // Verify the capability.call params structure
        let params = json!({
            "capability": "storage",
            "operation": "put",
            "args": { "key": "k", "value": "dmFsdWU=" }
        });
        assert_eq!(params["capability"], "storage");
        assert_eq!(params["operation"], "put");
        assert!(params["args"].is_object());
    }

    #[test]
    fn test_base64_encode_decode_roundtrip() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded.as_ref(), data);
    }

    #[test]
    fn test_base64_encode_empty() {
        let encoded = base64_encode(b"");
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_base64_encode_single_byte() {
        let encoded = base64_encode(b"a");
        assert_eq!(encoded.len(), 4);
        assert!(encoded.ends_with("=="));
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded.as_ref(), b"a");
    }

    #[test]
    fn test_base64_encode_two_bytes() {
        let encoded = base64_encode(b"ab");
        assert_eq!(encoded.len(), 4);
        assert!(encoded.ends_with('='));
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded.as_ref(), b"ab");
    }

    #[test]
    fn test_base64_encode_three_bytes() {
        let encoded = base64_encode(b"abc");
        assert_eq!(encoded.len(), 4);
        assert!(!encoded.ends_with('='));
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded.as_ref(), b"abc");
    }

    #[test]
    fn test_base64_decode_with_padding() {
        let decoded = base64_decode("YQ==").unwrap();
        assert_eq!(decoded.as_ref(), b"a");
    }

    #[test]
    fn test_base64_decode_without_padding() {
        let decoded = base64_decode("YQ").unwrap();
        assert_eq!(decoded.as_ref(), b"a");
    }

    #[test]
    fn test_base64_decode_ignores_invalid_chars() {
        // Invalid chars are filtered out
        let decoded = base64_decode("YQ==\n\t ").unwrap();
        assert_eq!(decoded.as_ref(), b"a");
    }

    #[test]
    fn test_base64_decode_empty() {
        let decoded = base64_decode("").unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_resolve_neural_api_socket_no_env() {
        // Without NEURAL_API_SOCKET and no running biomeOS, discover fails
        let result = resolve_neural_api_socket();
        // May succeed if biomeOS happens to be running in test env
        let _ = result;
    }

    #[test]
    fn test_capability_client_discover() {
        // discover() may fail if no socket exists
        let result = CapabilityClient::discover();
        let _ = result;
    }

    #[test]
    fn test_capability_client_path_impl() {
        let client = CapabilityClient::new("/var/run/neural.sock");
        let client2 = CapabilityClient::new(PathBuf::from("/var/run/neural.sock"));
        drop(client);
        drop(client2);
    }

    #[test]
    fn test_base64_encode_decode_large_data() {
        let data: Vec<u8> = (0u8..200).collect();
        let encoded = base64_encode(&data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded.as_ref(), data.as_slice());
    }

    #[test]
    fn test_base64_decode_invalid_characters_filtered() {
        let decoded = base64_decode("Y\nQ\t=\r\n=").unwrap();
        assert_eq!(decoded.as_ref(), b"a");
    }

    #[test]
    fn test_base64_decode_plus_slash() {
        let decoded = base64_decode("+/+").unwrap();
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_http_request_params_structure() {
        let args = json!({
            "method": "GET",
            "url": "https://example.com",
            "headers": {"Authorization": "Bearer x"},
            "body": "request body"
        });
        assert_eq!(args["method"], "GET");
        assert_eq!(args["url"], "https://example.com");
        assert!(args["headers"].is_object());
        assert_eq!(args["body"], "request body");
    }

    #[test]
    fn test_storage_put_params() {
        let args = json!({
            "key": "my-key",
            "value": base64_encode(b"value bytes")
        });
        assert_eq!(args["key"], "my-key");
        assert!(args["value"].as_str().is_some());
    }

    #[test]
    fn test_storage_get_params() {
        let args = json!({ "key": "lookup-key" });
        assert_eq!(args["key"], "lookup-key");
    }

    #[test]
    fn test_crypto_sign_params() {
        let args = json!({
            "data": base64_encode(b"data to sign")
        });
        assert!(args["data"].as_str().is_some());
    }

    #[test]
    fn test_crypto_verify_params() {
        let args = json!({
            "data": base64_encode(b"data"),
            "signature": base64_encode(b"sig"),
            "public_key": base64_encode(b"pubkey")
        });
        assert!(args["data"].as_str().is_some());
        assert!(args["signature"].as_str().is_some());
        assert!(args["public_key"].as_str().is_some());
    }

    #[test]
    fn test_crypto_hash_params() {
        let args = json!({
            "data": base64_encode(b"data"),
            "algorithm": "sha256"
        });
        assert_eq!(args["algorithm"], "sha256");
    }

    #[test]
    fn test_compute_execute_params() {
        let args = json!({
            "task": "inference",
            "params": {"model": "test"}
        });
        assert_eq!(args["task"], "inference");
        assert!(args["params"].is_object());
    }

    #[test]
    fn test_health_check_params() {
        let args = json!({ "primal": "beardog" });
        assert_eq!(args["primal"], "beardog");
    }

    #[tokio::test]
    async fn test_capability_client_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/path/12345.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.crypto_sign(b"test").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Failed")
                || err.contains("connect")
                || err.contains("timeout")
                || err.contains("Connection"),
            "expected connection error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_storage_put_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/456.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.storage_put("key", b"value").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_storage_get_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/789.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.storage_get("key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_storage_exists_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/exists.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.storage_exists("key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_http_request_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/http.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client
            .http_request("GET", "https://example.com", None, None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_execute_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/compute.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.compute_execute("task", json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discover_capability_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/discover.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.discover_capability("crypto").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_translations_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/list.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.list_translations().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_check_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/health.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.health_check("beardog").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_crypto_verify_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/verify.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.crypto_verify(b"data", b"sig", b"pubkey").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_crypto_hash_connection_refused() {
        let client = CapabilityClient::new("/nonexistent/socket/hash.sock")
            .with_timeout(Duration::from_millis(100));

        let result = client.crypto_hash(b"data", "sha256").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_neural_api_socket_invocation() {
        let result = resolve_neural_api_socket();
        match &result {
            Ok(p) => assert!(!p.as_os_str().is_empty()),
            Err(e) => {
                assert!(e.to_string().contains("not found") || e.to_string().contains("Neural"));
            }
        }
    }

    #[test]
    #[serial]
    fn test_resolve_neural_api_socket_from_env() {
        use biomeos_test_utils::TestEnvGuard;

        let tmp = tempfile::NamedTempFile::new().expect("temp file");
        let path = tmp.path().to_path_buf();
        let _guard = TestEnvGuard::set("NEURAL_API_SOCKET", path.to_string_lossy().as_ref());
        let got = resolve_neural_api_socket().expect("env path should resolve");
        assert_eq!(got, path);
    }

    async fn serve_one_jsonrpc_response(listener: UnixListener, reply: serde_json::Value) {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut read_half, mut write_half) = stream.into_split();
        let mut line = String::new();
        BufReader::new(&mut read_half)
            .read_line(&mut line)
            .await
            .expect("read line");
        let req: JsonRpcRequest = serde_json::from_str(line.trim()).expect("parse request");
        let id = req.id.clone().unwrap_or(serde_json::Value::Null);
        let resp = JsonRpcResponse {
            jsonrpc: biomeos_types::JSONRPC_VERSION.to_string(),
            result: Some(reply),
            error: None,
            id,
        };
        let body = serde_json::to_string(&resp).expect("serialize");
        write_half.write_all(body.as_bytes()).await.expect("write");
        write_half.write_all(b"\n").await.expect("newline");
        write_half.shutdown().await.ok();
    }

    #[tokio::test]
    async fn test_crypto_sign_success_via_mock_server() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("mock.sock");
        let sig = base64_encode(b"signed-payload");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            serve_one_jsonrpc_response(listener, serde_json::json!({ "signature": sig })).await;
        });

        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let out = client.crypto_sign(b"payload").await.expect("sign ok");
        assert_eq!(out.as_ref(), b"signed-payload");
        server.await.expect("server join");
    }

    #[tokio::test]
    async fn test_capability_call_rpc_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("err.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut read_half)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: JsonRpcRequest = serde_json::from_str(line.trim()).expect("parse");
            let id = req.id.clone().unwrap_or(serde_json::Value::Null);
            let resp = JsonRpcResponse {
                jsonrpc: biomeos_types::JSONRPC_VERSION.to_string(),
                result: None,
                error: Some(biomeos_types::JsonRpcError {
                    code: -32_000,
                    message: "boom".to_string(),
                    data: None,
                }),
                id,
            };
            let body = serde_json::to_string(&resp).expect("serialize");
            write_half.write_all(body.as_bytes()).await.unwrap();
            write_half.write_all(b"\n").await.unwrap();
            write_half.shutdown().await.ok();
        });

        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let err = client.storage_put("k", b"v").await.expect_err("rpc error");
        assert!(err.to_string().contains("boom") || err.to_string().contains("-32000"));
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_storage_get_null_value() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("null.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            serve_one_jsonrpc_response(listener, serde_json::json!({ "value": null })).await;
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let got = client.storage_get("k").await.expect("ok");
        assert!(got.is_none());
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_storage_get_string_value() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("get.sock");
        let b64 = base64_encode(b"hello");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            serve_one_jsonrpc_response(listener, serde_json::json!({ "value": b64 })).await;
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let got = client.storage_get("k").await.expect("get");
        assert_eq!(got.unwrap().as_ref(), b"hello");
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_crypto_verify_bool_result_alias() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("verify.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            serve_one_jsonrpc_response(listener, serde_json::json!({ "result": true })).await;
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let ok = client.crypto_verify(b"a", b"b", b"c").await.expect("v");
        assert!(ok);
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_crypto_hash_result_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("hash.sock");
        let h = base64_encode(b"hashbytes");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            serve_one_jsonrpc_response(listener, serde_json::json!({ "result": h })).await;
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let out = client.crypto_hash(b"x", "sha256").await.expect("hash");
        assert_eq!(out.as_ref(), b"hashbytes");
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_discover_capability_parses_primals() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("disc.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut read_half)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: JsonRpcRequest = serde_json::from_str(line.trim()).expect("parse");
            let id = req.id.clone().unwrap_or(serde_json::Value::Null);
            let resp = JsonRpcResponse {
                jsonrpc: biomeos_types::JSONRPC_VERSION.to_string(),
                result: Some(serde_json::json!({
                    "primals": [{"name": "beardog"}, {"name": "songbird"}]
                })),
                error: None,
                id,
            };
            let body = serde_json::to_string(&resp).expect("serialize");
            write_half.write_all(body.as_bytes()).await.unwrap();
            write_half.write_all(b"\n").await.unwrap();
            write_half.shutdown().await.ok();
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let names = client.discover_capability("crypto").await.expect("disc");
        assert_eq!(names, vec!["beardog", "songbird"]);
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_list_translations_success() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("list.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            serve_one_jsonrpc_response(listener, serde_json::json!({ "items": [] })).await;
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let v = client.list_translations().await.expect("list");
        assert!(v.get("items").is_some());
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_send_request_parse_error_short_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("bad.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut read_half)
                .read_line(&mut line)
                .await
                .expect("read");
            write_half.write_all(b"not-json").await.unwrap();
            write_half.shutdown().await.ok();
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        let err = client.health_check("x").await.expect_err("parse fail");
        assert!(err.to_string().contains("parse") || err.to_string().contains("JSON"));
        server.await.expect("join");
    }

    #[tokio::test]
    async fn test_storage_exists_result_alias() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("ex.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            serve_one_jsonrpc_response(listener, serde_json::json!({ "result": false })).await;
        });
        let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
        assert!(!client.storage_exists("k").await.expect("exists"));
        server.await.expect("join");
    }

    #[test]
    fn test_base64_encode_all_padding_cases() {
        assert_eq!(base64_encode(&[0xFF, 0xFF, 0xFF]).len(), 4);
    }
}
