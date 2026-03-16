// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Synchronous Neural API client for biomeOS — zero-tokio, std + serde_json only.
//!
//! Minimal synchronous JSON-RPC 2.0 client that talks to the biomeOS Neural API.
//! Zero external dependencies beyond `std` + `serde_json`.
//!
//! # Discovery
//!
//! The Neural API socket is discovered using biomeOS's 5-tier resolution:
//!
//! 1. `NEURAL_API_SOCKET` env var
//! 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family_id}.sock`
//! 3. `/run/user/{uid}/biomeos/neural-api-{family_id}.sock`
//! 4. `{temp_dir}/biomeos/neural-api-{family_id}.sock` (platform temp dir)
//!
//! # Usage
//!
//! ```no_run
//! use neural_api_client_sync::NeuralBridge;
//!
//! let bridge = NeuralBridge::discover().unwrap();
//! let result = bridge.capability_call("ecology", "et0_pm", &serde_json::json!({
//!     "tmin": 12.3, "tmax": 21.5
//! }));
//! ```

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Connection to the biomeOS Neural API.
pub struct NeuralBridge {
    socket_path: PathBuf,
    timeout: Duration,
}

/// Result of a `capability.call` invocation.
#[derive(Debug)]
pub struct CallResult {
    /// The JSON value returned by the capability call.
    pub value: serde_json::Value,
}

/// Error from Neural API communication.
#[derive(Debug)]
pub enum NeuralError {
    /// Neural API socket not found (biomeOS not running).
    NotFound(String),
    /// Connection failed.
    Connection(std::io::Error),
    /// JSON serialization/deserialization error.
    Json(String),
    /// JSON-RPC error response from the Neural API.
    Rpc {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable error message.
        message: String,
    },
    /// Timeout waiting for response.
    Timeout,
}

impl std::fmt::Display for NeuralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Neural API not found: {msg}"),
            Self::Connection(e) => write!(f, "Connection error: {e}"),
            Self::Json(msg) => write!(f, "JSON error: {msg}"),
            Self::Rpc { code, message } => write!(f, "RPC error {code}: {message}"),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

impl std::error::Error for NeuralError {}

impl NeuralBridge {
    /// Discover the Neural API socket using biomeOS 5-tier resolution.
    ///
    /// Returns `None` if biomeOS is not running (no socket found).
    #[must_use]
    pub fn discover() -> Option<Self> {
        Self::discover_with(None, None)
    }

    /// Discover the Neural API socket with explicit overrides.
    ///
    /// Accepts optional socket path and family ID for environments where
    /// automatic discovery is insufficient.
    #[must_use]
    pub fn discover_with(neural_api_socket: Option<&str>, family_id: Option<&str>) -> Option<Self> {
        let path = resolve_socket_with(neural_api_socket, family_id)?;
        Some(Self {
            socket_path: path,
            timeout: Duration::from_secs(30),
        })
    }

    /// Set the timeout for requests.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a `capability.call` request to the Neural API.
    ///
    /// The Neural API routes this to the appropriate primal based on
    /// the capability translation registry.
    ///
    /// # Errors
    ///
    /// Returns `NeuralError` if the socket connection fails, the request
    /// is malformed, or the remote primal returns an RPC error.
    pub fn capability_call(
        &self,
        capability: &str,
        operation: &str,
        args: &serde_json::Value,
    ) -> Result<CallResult, NeuralError> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.call",
            "params": {
                "capability": capability,
                "operation": operation,
                "args": args,
            },
            "id": id,
        });

        let response = self.send_request(&request)?;
        parse_response(&response)
    }

    /// Discover capabilities available in the ecosystem.
    ///
    /// # Errors
    ///
    /// Returns `NeuralError` on connection or protocol failure.
    pub fn discover_capability(&self, capability: &str) -> Result<serde_json::Value, NeuralError> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.discover",
            "params": { "capability": capability },
            "id": id,
        });
        let response = self.send_request(&request)?;
        parse_response(&response).map(|r| r.value)
    }

    /// Check if the Neural API is reachable.
    ///
    /// # Errors
    ///
    /// Returns `NeuralError` on connection or protocol failure.
    pub fn health_check(&self) -> Result<serde_json::Value, NeuralError> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "lifecycle.status",
            "params": {},
            "id": id,
        });
        let response = self.send_request(&request)?;
        parse_response(&response).map(|r| r.value)
    }

    /// The socket path we're connected to.
    #[must_use]
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }

    fn send_request(&self, request: &serde_json::Value) -> Result<serde_json::Value, NeuralError> {
        let mut stream = UnixStream::connect(&self.socket_path).map_err(NeuralError::Connection)?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(NeuralError::Connection)?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(NeuralError::Connection)?;

        let mut payload =
            serde_json::to_string(request).map_err(|e| NeuralError::Json(e.to_string()))?;
        payload.push('\n');
        stream
            .write_all(payload.as_bytes())
            .map_err(NeuralError::Connection)?;
        stream.flush().map_err(NeuralError::Connection)?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(NeuralError::Connection)?;

        serde_json::from_str(&line).map_err(|e| NeuralError::Json(e.to_string()))
    }
}

/// Resolve the Neural API socket path using the 5-tier discovery strategy.
///
/// Checks: `NEURAL_API_SOCKET` env var, `$XDG_RUNTIME_DIR/biomeos/`, `/run/user/{uid}/biomeos/`,
/// and platform temp dir, in order.
pub fn resolve_socket_with(
    neural_api_socket: Option<&str>,
    family_id_override: Option<&str>,
) -> Option<PathBuf> {
    if let Some(path) = neural_api_socket {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let family_id = family_id_override
        .map(String::from)
        .or_else(|| std::env::var("FAMILY_ID").ok())?;

    // Tier 2: XDG_RUNTIME_DIR
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(xdg)
            .join("biomeos")
            .join(format!("neural-api-{family_id}.sock"));
        if p.exists() {
            return Some(p);
        }
    }

    // Tier 3: /run/user/{uid} — derive from XDG_RUNTIME_DIR or procfs
    let uid = uid_from_runtime_dir();
    let p = PathBuf::from(format!(
        "/run/user/{uid}/biomeos/neural-api-{family_id}.sock"
    ));
    if p.exists() {
        return Some(p);
    }

    // Tier 4: platform temp-dir fallback (no hardcoded /tmp)
    let p = std::env::temp_dir()
        .join("biomeos")
        .join(format!("neural-api-{family_id}.sock"));
    if p.exists() {
        return Some(p);
    }

    None
}

const PROC_STATUS_PATH: &str = "/proc/self/status";

/// Extract real UID from `/proc/self/status` (safe, no libc).
///
/// Falls back to `nobody` (65534) rather than assuming a specific user.
/// A hardcoded UID like 1000 is fragile — different distros assign different
/// first-user UIDs.  65534 is the POSIX `nobody` sentinel and will fail
/// visibly rather than silently resolve to the wrong user's runtime dir.
///
/// Public so biomeos-types and other crates can use it for runtime dir resolution.
#[must_use]
pub fn uid_from_runtime_dir() -> u32 {
    const NOBODY_UID: u32 = 65534;
    std::fs::read_to_string(PROC_STATUS_PATH)
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("Uid:")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|s| s.parse::<u32>().ok())
            })
        })
        .unwrap_or(NOBODY_UID)
}

fn parse_response(response: &serde_json::Value) -> Result<CallResult, NeuralError> {
    if let Some(error) = response.get("error") {
        let code = error["code"].as_i64().unwrap_or(-1);
        let message = error["message"]
            .as_str()
            .unwrap_or("unknown error")
            .to_string();
        return Err(NeuralError::Rpc { code, message });
    }
    Ok(CallResult {
        value: response
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn no_socket_returns_none() {
        assert!(NeuralBridge::discover_with(
            Some("/nonexistent/neural-api.sock"),
            Some("test-family")
        )
        .is_none());
    }

    #[test]
    fn parse_success_response() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": { "et0": 3.88 },
            "id": 1
        });
        let result = parse_response(&resp).unwrap();
        let et0 = result.value["et0"].as_f64().unwrap();
        assert!((et0 - 3.88).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_error_response() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32601, "message": "Method not found" },
            "id": 1
        });
        let err = parse_response(&resp).unwrap_err();
        assert!(matches!(err, NeuralError::Rpc { code: -32601, .. }));
    }

    #[test]
    fn with_timeout_builder() {
        let bridge = NeuralBridge {
            socket_path: PathBuf::from("/tmp/test.sock"),
            timeout: Duration::from_secs(30),
        };
        let bridge = bridge.with_timeout(Duration::from_secs(5));
        assert_eq!(bridge.timeout, Duration::from_secs(5));
    }

    #[test]
    fn socket_path_getter() {
        let path = PathBuf::from("/tmp/neural-api.sock");
        let bridge = NeuralBridge {
            socket_path: path.clone(),
            timeout: Duration::from_secs(30),
        };
        assert_eq!(bridge.socket_path(), path.as_path());
    }

    #[test]
    fn neural_error_display() {
        let err = NeuralError::NotFound("socket not found".to_string());
        assert!(err.to_string().contains("Neural API not found"));
        assert!(err.to_string().contains("socket not found"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = NeuralError::Connection(io_err);
        assert!(err.to_string().contains("Connection error"));

        let err = NeuralError::Json("parse error".to_string());
        assert!(err.to_string().contains("JSON error"));

        let err = NeuralError::Rpc {
            code: -32601,
            message: "Method not found".to_string(),
        };
        assert!(err.to_string().contains("RPC error"));
        assert!(err.to_string().contains("-32601"));

        let err = NeuralError::Timeout;
        assert_eq!(err.to_string(), "Timeout");
    }

    #[test]
    fn parse_response_null_result() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": null,
            "id": 1
        });
        let result = parse_response(&resp).unwrap();
        assert!(result.value.is_null());
    }

    #[test]
    fn parse_response_missing_result() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1
        });
        let result = parse_response(&resp).unwrap();
        assert!(result.value.is_null());
    }

    #[test]
    fn resolve_socket_via_env_var() {
        let temp = tempfile::tempdir().expect("temp dir");
        let sock_path = temp.path().join("neural-api.sock");
        std::fs::write(&sock_path, "").expect("create socket file");

        let bridge = NeuralBridge::discover_with(
            Some(sock_path.to_string_lossy().as_ref()),
            Some("test-family"),
        );
        assert!(bridge.is_some());
        assert_eq!(bridge.unwrap().socket_path(), sock_path.as_path());
    }

    #[test]
    fn capability_call_fails_on_nonexistent_socket() {
        let bridge = NeuralBridge {
            socket_path: PathBuf::from("/nonexistent/path/neural-api-xyz.sock"),
            timeout: Duration::from_millis(100),
        };
        let result = bridge.capability_call("ecology", "et0_pm", &serde_json::json!({}));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, NeuralError::Connection(_)));
    }

    #[test]
    fn discover_capability_fails_on_nonexistent_socket() {
        let bridge = NeuralBridge {
            socket_path: PathBuf::from("/nonexistent/path/neural-api-xyz.sock"),
            timeout: Duration::from_millis(100),
        };
        let result = bridge.discover_capability("ecology");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NeuralError::Connection(_)));
    }

    #[test]
    fn health_check_fails_on_nonexistent_socket() {
        let bridge = NeuralBridge {
            socket_path: PathBuf::from("/nonexistent/path/neural-api-xyz.sock"),
            timeout: Duration::from_millis(100),
        };
        let result = bridge.health_check();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NeuralError::Connection(_)));
    }

    #[test]
    fn uid_from_runtime_dir_returns_valid_uid_on_linux() {
        #[cfg(target_os = "linux")]
        {
            let uid = uid_from_runtime_dir();
            // On Linux, when running as a real user, uid should not be 65534 (nobody)
            // unless the test is actually running as nobody. In CI or normal dev,
            // we expect a real UID. 65534 indicates failure to read /proc/self/status.
            assert_ne!(
                uid, 65534,
                "uid_from_runtime_dir should return real UID on Linux, not nobody (65534)"
            );
        }
        #[cfg(not(target_os = "linux"))]
        {
            // On non-Linux, we just verify it returns something (no /proc)
            let uid = uid_from_runtime_dir();
            assert_eq!(uid, 65534, "non-Linux falls back to nobody");
        }
    }
}
