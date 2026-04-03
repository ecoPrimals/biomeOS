// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Configurable backoff for Neural API connection attempts.
//!
//! Used by [`crate::connection::json_rpc_call`] and [`crate::NeuralApiClient`].

use tokio::time::Duration;

/// Backoff and retry limits for connecting to the Neural API socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeuralApiRetryConfig {
    /// Maximum connection attempts (including the first try). When `1`, no backoff sleep runs.
    pub max_connect_attempts: u32,
    /// Delay before each retry after the first failed attempt.
    pub initial_backoff: Duration,
}

impl NeuralApiRetryConfig {
    /// No retries: single connection attempt (default production behavior).
    #[must_use]
    pub const fn no_retry() -> Self {
        Self {
            max_connect_attempts: 1,
            initial_backoff: Duration::from_millis(50),
        }
    }
}

impl Default for NeuralApiRetryConfig {
    fn default() -> Self {
        Self::no_retry()
    }
}
