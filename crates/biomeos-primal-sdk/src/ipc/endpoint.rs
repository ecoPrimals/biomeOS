// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Transport endpoint descriptor for SDK IPC clients.

use std::path::PathBuf;

/// Transport endpoint for connecting to a primal or the Neural API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportEndpoint {
    /// Unix domain socket (Tier 1 — Linux, macOS). On Windows, falls back to TCP via port-file.
    UnixSocket {
        /// Path to the socket file.
        path: PathBuf,
    },
}

impl std::fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnixSocket { path } => write!(f, "unix://{}", path.display()),
        }
    }
}
