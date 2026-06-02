// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Synchronous Neural API client for biomeOS — zero-tokio, std + `serde_json` + `thiserror`.
//!
//! Minimal synchronous JSON-RPC 2.0 client that talks to the biomeOS Neural API.
//! Uses `std`, `serde`/`serde_json`, and `thiserror` (no async runtime).
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

use biomeos_types::constants::runtime_paths::LINUX_RUNTIME_DIR_PREFIX;
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
#[derive(Debug, thiserror::Error)]
pub enum NeuralError {
    /// Neural API socket not found (biomeOS not running).
    #[error("Neural API not found: {0}")]
    NotFound(String),
    /// Connection failed.
    #[error("Connection error: {0}")]
    Connection(std::io::Error),
    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(String),
    /// JSON-RPC error response from the Neural API.
    #[error("RPC error {code}: {message}")]
    Rpc {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable error message.
        message: String,
    },
    /// Timeout waiting for response.
    #[error("Timeout")]
    Timeout,
}

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
        Self::discover_with_env(neural_api_socket, family_id, &SocketResolveEnv::default())
    }

    /// Like [`Self::discover_with`], with explicit runtime / temp overrides (no env mutation).
    #[must_use]
    pub fn discover_with_env(
        neural_api_socket: Option<&str>,
        family_id: Option<&str>,
        env: &SocketResolveEnv,
    ) -> Option<Self> {
        let path = resolve_socket_with_env(neural_api_socket, family_id, env)?;
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

/// Optional environment overrides for [`resolve_socket_with_env`] (tests and tooling).
#[derive(Debug, Clone, Default)]
pub struct SocketResolveEnv {
    /// When set, used instead of the process `XDG_RUNTIME_DIR` for tier 2.
    pub xdg_runtime_dir: Option<String>,
    /// When set, used as the parent for tier 4 instead of [`std::env::temp_dir`].
    pub tmpdir: Option<String>,
}

/// Resolve the Neural API socket path using the 5-tier discovery strategy.
///
/// Checks: `NEURAL_API_SOCKET` env var, `$XDG_RUNTIME_DIR/biomeos/`, `/run/user/{uid}/biomeos/`,
/// and platform temp dir, in order.
pub fn resolve_socket_with(
    neural_api_socket: Option<&str>,
    family_id_override: Option<&str>,
) -> Option<PathBuf> {
    resolve_socket_with_env(
        neural_api_socket,
        family_id_override,
        &SocketResolveEnv::default(),
    )
}

/// Like [`resolve_socket_with`], but supplies `XDG_RUNTIME_DIR` / `TMPDIR` equivalents explicitly.
#[must_use]
pub fn resolve_socket_with_env(
    neural_api_socket: Option<&str>,
    family_id_override: Option<&str>,
    env: &SocketResolveEnv,
) -> Option<PathBuf> {
    if let Some(path) = neural_api_socket {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let family_id = family_id_override
        .map(String::from)
        .or_else(|| std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY).ok())?;

    // Tier 2: XDG_RUNTIME_DIR
    let xdg = env
        .xdg_runtime_dir
        .clone()
        .or_else(|| std::env::var(biomeos_types::env_config::vars::XDG_RUNTIME_DIR).ok());
    if let Some(xdg) = xdg {
        let p = PathBuf::from(xdg)
            .join(biomeos_types::constants::runtime_paths::BIOMEOS_SUBDIR)
            .join(format!("neural-api-{family_id}.sock"));
        if p.exists() {
            return Some(p);
        }
    }

    // Tier 3: /run/user/{uid} — derive from XDG_RUNTIME_DIR or procfs
    let uid = uid_from_runtime_dir();
    let p = PathBuf::from(format!(
        "{LINUX_RUNTIME_DIR_PREFIX}/{uid}/biomeos/neural-api-{family_id}.sock"
    ));
    if p.exists() {
        return Some(p);
    }

    // Tier 4: platform temp-dir fallback (no hardcoded /tmp)
    let tmp_base = env
        .tmpdir
        .as_ref()
        .map_or_else(std::env::temp_dir, PathBuf::from);
    let p = tmp_base
        .join(biomeos_types::constants::runtime_paths::BIOMEOS_SUBDIR)
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
#[path = "lib_tests.rs"]
mod tests;
