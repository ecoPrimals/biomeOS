// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Connection pool for [`TransportStream`] — reuses persistent connections to
//! avoid per-request connect/disconnect overhead on UDS dispatch paths.
//!
//! The server side (Neural API) already reads in a loop, so multiple requests
//! can flow over a single stream. This pool caches connected streams keyed by
//! endpoint and returns them for reuse.

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
/// # Usage
///
/// ```ignore
/// let pool = ConnectionPool::new();
/// let response = pool.send_jsonrpc(&endpoint, request).await?;
/// ```
#[derive(Clone)]
pub struct ConnectionPool {
    inner: Arc<DashMap<String, VecDeque<PooledStream>>>,
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
        }
    }

    /// Send a JSON-RPC request, reusing a pooled connection if available.
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

        if let Some(stream) = self.take(&key) {
            match Self::send_over(stream, &request_bytes).await {
                Ok((response, stream)) => {
                    self.put(&key, stream);
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
        self.put(&key, stream);
        Ok(response)
    }

    /// Number of idle connections across all endpoints.
    #[must_use]
    pub fn idle_count(&self) -> usize {
        self.inner
            .iter()
            .map(|entry| entry.value().len())
            .sum::<usize>()
    }

    async fn send_over(
        mut stream: TransportStream,
        request_bytes: &[u8],
    ) -> anyhow::Result<(biomeos_types::JsonRpcResponse, TransportStream)> {
        use tokio::io::AsyncReadExt;

        stream.write_all(request_bytes).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut buf = Vec::with_capacity(4096);
        let mut byte = [0u8; 1];

        loop {
            let n = stream.read(&mut byte).await?;
            if n == 0 {
                anyhow::bail!("Connection closed by peer");
            }
            if byte[0] == b'\n' {
                break;
            }
            buf.push(byte[0]);
        }

        let line = String::from_utf8(buf)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in response: {e}"))?;

        let response: biomeos_types::JsonRpcResponse = serde_json::from_str(line.trim())
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {e}"))?;

        Ok((response, stream))
    }

    fn take(&self, key: &str) -> Option<TransportStream> {
        let mut entry = self.inner.get_mut(key)?;
        let queue = entry.value_mut();

        while let Some(pooled) = queue.pop_front() {
            if pooled.last_used.elapsed() < IDLE_TIMEOUT {
                return Some(pooled.stream);
            }
        }
        None
    }

    fn put(&self, key: &str, stream: TransportStream) {
        let mut entry = self.inner.entry(key.to_string()).or_default();
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
    }

    #[test]
    fn pool_clone_shares_state() {
        let pool = ConnectionPool::new();
        let pool2 = pool.clone();
        assert_eq!(pool.idle_count(), pool2.idle_count());
    }
}
