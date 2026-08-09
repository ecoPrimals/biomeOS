// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Connection pool for [`TransportStream`] — reuses persistent connections to
//! avoid per-request connect/disconnect overhead on UDS dispatch paths.
//!
//! The server side (Neural API) already reads in a loop, so multiple requests
//! can flow over a single stream. This pool caches connected streams keyed by
//! endpoint and returns them for reuse.
//!
//! ## riboCipher-aware framing
//!
//! NUCLEUS-deployed primals (sweetGrass, Neural API endpoints) require a 2-byte
//! riboCipher signal (`[0xEC, 0x01]`) at connection start. The pool tracks which
//! endpoints need this prefix and writes it once when establishing a fresh
//! connection. Pooled (reused) connections already had the signal written on
//! their first use, so subsequent requests skip the prefix.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::io::AsyncWriteExt;
use tracing::trace;

use super::TransportStream;
use crate::TransportEndpoint;

const MAX_IDLE_PER_ENDPOINT: usize = 4;
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

struct PooledStream {
    stream: TransportStream,
    last_used: Instant,
}

/// A connection pool that caches [`TransportStream`] instances by endpoint.
///
/// Supports optional riboCipher transport framing: callers that need the
/// `[0xEC, 0x01]` prefix use [`send_ribocipher_jsonrpc`] instead of
/// [`send_jsonrpc`]. The prefix is written once on fresh connections;
/// pooled streams retain their framing from initial handshake.
///
/// # Usage
///
/// ```ignore
/// let pool = ConnectionPool::new();
/// // Plain JSON-RPC (primals that accept raw framing):
/// let response = pool.send_jsonrpc(&endpoint, request).await?;
/// // riboCipher-framed JSON-RPC (NUCLEUS primals, Neural API endpoints):
/// let response = pool.send_ribocipher_jsonrpc(&endpoint, request).await?;
/// ```
#[derive(Clone)]
pub struct ConnectionPool {
    /// Pooled streams keyed by `"endpoint_string"` (plain JSON-RPC connections).
    inner: Arc<DashMap<String, VecDeque<PooledStream>>>,
    /// Pooled streams keyed by `"rc:endpoint_string"` (riboCipher-prefixed connections).
    /// Kept separate so a plain caller never accidentally reuses a riboCipher stream
    /// (the server consumed the prefix and expects raw JSON-RPC subsequently).
    inner_ribocipher: Arc<DashMap<String, VecDeque<PooledStream>>>,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionPool {
    /// Create a new empty connection pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
            inner_ribocipher: Arc::new(DashMap::new()),
        }
    }

    /// Send a JSON-RPC request over a **plain** (no riboCipher prefix) connection.
    ///
    /// On success, the connection is returned to the pool for future reuse.
    /// On failure, a fresh connection is attempted once before propagating the error.
    pub async fn send_jsonrpc(
        &self,
        endpoint: &TransportEndpoint,
        request: biomeos_types::JsonRpcRequest,
    ) -> anyhow::Result<biomeos_types::JsonRpcResponse> {
        let key = endpoint.to_string();
        let request_bytes = serde_json::to_vec(&request)?;

        if let Some(stream) = Self::take_from(&self.inner, &key) {
            match Self::send_over(stream, &request_bytes).await {
                Ok((response, stream)) => {
                    Self::put_into(&self.inner, &key, stream);
                    return Ok(response);
                }
                Err(_) => {
                    trace!("Pooled connection stale for {key}, reconnecting");
                }
            }
        }

        let stream = super::connect_transport(endpoint)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to {endpoint}: {e}"))?;

        let (response, stream) = Self::send_over(stream, &request_bytes).await?;
        Self::put_into(&self.inner, &key, stream);
        Ok(response)
    }

    /// Send a JSON-RPC request with riboCipher transport framing (`[0xEC, 0x01]` prefix).
    ///
    /// Fresh connections have the 2-byte signal written before the first payload.
    /// Pooled connections already received the prefix on initial use; the server
    /// consumes it once and then reads raw JSON-RPC for the connection lifetime.
    pub async fn send_ribocipher_jsonrpc(
        &self,
        endpoint: &TransportEndpoint,
        request: biomeos_types::JsonRpcRequest,
    ) -> anyhow::Result<biomeos_types::JsonRpcResponse> {
        let key = endpoint.to_string();
        let request_bytes = serde_json::to_vec(&request)?;

        if let Some(stream) = Self::take_from(&self.inner_ribocipher, &key) {
            match Self::send_over(stream, &request_bytes).await {
                Ok((response, stream)) => {
                    Self::put_into(&self.inner_ribocipher, &key, stream);
                    return Ok(response);
                }
                Err(_) => {
                    trace!("Pooled riboCipher connection stale for {key}, reconnecting");
                }
            }
        }

        let mut stream = super::connect_transport(endpoint)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to {endpoint}: {e}"))?;

        stream
            .write_all(&[
                biomeos_types::constants::ribocipher::SIGNAL_CLEAR,
                biomeos_types::constants::ribocipher::VERSION_1,
            ])
            .await?;

        let (response, stream) = Self::send_over(stream, &request_bytes).await?;
        Self::put_into(&self.inner_ribocipher, &key, stream);
        Ok(response)
    }

    /// Number of idle connections across all endpoints (both plain and riboCipher).
    #[must_use]
    pub fn idle_count(&self) -> usize {
        let plain: usize = self.inner.iter().map(|e| e.value().len()).sum();
        let rc: usize = self.inner_ribocipher.iter().map(|e| e.value().len()).sum();
        plain + rc
    }

    /// Send a JSON-RPC request with riboCipher **Tier 2** (mito-obfuscated) framing.
    ///
    /// Writes `[0xED, 0x01]` signal + 32-byte mito-tag on fresh connections.
    /// The mito-tag authenticates the client to the server (decoded by bearDog).
    /// Pooled connections already performed the handshake on initial use.
    ///
    /// Falls back to Tier 1 (`send_ribocipher_jsonrpc`) if `mito_tag` is None —
    /// callers should attempt to encode a tag via `crypto.encode_mito_tag` but
    /// gracefully degrade if bearDog is unavailable.
    pub async fn send_mito_jsonrpc(
        &self,
        endpoint: &TransportEndpoint,
        request: biomeos_types::JsonRpcRequest,
        mito_tag: Option<&[u8; biomeos_types::constants::ribocipher::MITO_TAG_LEN]>,
    ) -> anyhow::Result<biomeos_types::JsonRpcResponse> {
        let Some(tag) = mito_tag else {
            return self.send_ribocipher_jsonrpc(endpoint, request).await;
        };

        let key = format!("mito:{endpoint}");
        let request_bytes = serde_json::to_vec(&request)?;

        if let Some(stream) = Self::take_from(&self.inner_ribocipher, &key) {
            match Self::send_over(stream, &request_bytes).await {
                Ok((response, stream)) => {
                    Self::put_into(&self.inner_ribocipher, &key, stream);
                    return Ok(response);
                }
                Err(_) => {
                    trace!("Pooled mito connection stale for {key}, reconnecting");
                }
            }
        }

        let mut stream = super::connect_transport(endpoint)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to {endpoint}: {e}"))?;

        stream
            .write_all(&[
                biomeos_types::constants::ribocipher::SIGNAL_MITO,
                biomeos_types::constants::ribocipher::VERSION_1,
            ])
            .await?;
        stream.write_all(tag).await?;
        stream.flush().await?;

        let (response, stream) = Self::send_over(stream, &request_bytes).await?;
        Self::put_into(&self.inner_ribocipher, &key, stream);
        Ok(response)
    }

    /// Number of idle plain connections.
    #[must_use]
    pub fn idle_plain_count(&self) -> usize {
        self.inner.iter().map(|e| e.value().len()).sum()
    }

    /// Number of idle riboCipher connections.
    #[must_use]
    pub fn idle_ribocipher_count(&self) -> usize {
        self.inner_ribocipher.iter().map(|e| e.value().len()).sum()
    }

    async fn send_over(
        mut stream: TransportStream,
        request_bytes: &[u8],
    ) -> anyhow::Result<(biomeos_types::JsonRpcResponse, TransportStream)> {
        use tokio::io::AsyncBufReadExt;

        stream.write_all(request_bytes).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut reader = tokio::io::BufReader::new(stream);
        let mut line = String::with_capacity(4096);
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            anyhow::bail!("Connection closed by peer");
        }

        let response: biomeos_types::JsonRpcResponse = serde_json::from_str(line.trim())
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {e}"))?;

        let stream = reader.into_inner();
        Ok((response, stream))
    }

    fn take_from(
        map: &DashMap<String, VecDeque<PooledStream>>,
        key: &str,
    ) -> Option<TransportStream> {
        let mut entry = map.get_mut(key)?;
        let queue = entry.value_mut();

        while let Some(pooled) = queue.pop_front() {
            if pooled.last_used.elapsed() < IDLE_TIMEOUT {
                return Some(pooled.stream);
            }
        }
        None
    }

    fn put_into(map: &DashMap<String, VecDeque<PooledStream>>, key: &str, stream: TransportStream) {
        let mut entry = map.entry(key.to_owned()).or_default();
        let queue = entry.value_mut();

        if queue.len() >= MAX_IDLE_PER_ENDPOINT {
            queue.pop_front();
        }

        queue.push_back(PooledStream {
            stream,
            last_used: Instant::now(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_default_is_empty() {
        let pool = ConnectionPool::new();
        assert_eq!(pool.idle_count(), 0);
        assert_eq!(pool.idle_plain_count(), 0);
        assert_eq!(pool.idle_ribocipher_count(), 0);
    }

    #[test]
    fn pool_clone_shares_state() {
        let pool = ConnectionPool::new();
        let pool2 = pool.clone();
        assert_eq!(pool.idle_count(), pool2.idle_count());
    }
}
