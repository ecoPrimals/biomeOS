// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::Capability;
use biomeos_types::PrimalId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a registered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal ID
    pub id: PrimalId,

    /// Capabilities this primal provides
    pub provides: Vec<Capability>,

    /// Capabilities this primal requires
    pub requires: Vec<Capability>,

    /// Unix socket path for IPC
    pub socket_path: Option<String>,

    /// HTTP endpoint (if any)
    pub http_endpoint: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,

    /// Last heartbeat timestamp
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Registry request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum RegistryRequest {
    /// Register a primal
    Register {
        /// Primal identifier
        id: String,
        /// Correlation request identifier
        request_id: String,
        /// Registration parameters
        params: RegisterParams,
    },

    /// Query for capability provider
    GetProvider {
        /// Correlation request identifier
        request_id: String,
        /// Capability to look up
        capability: Capability,
    },

    /// List all registered primals
    ListPrimals {
        /// Correlation request identifier
        request_id: String,
    },

    /// Heartbeat
    Heartbeat {
        /// Correlation request identifier
        request_id: String,
        /// Primal sending the heartbeat
        primal_id: String,
    },

    /// Unregister a primal
    Unregister {
        /// Correlation request identifier
        request_id: String,
        /// Primal to unregister
        primal_id: String,
    },
}

/// Registration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    /// Capabilities this primal provides
    pub provides: Vec<Capability>,
    /// Capabilities this primal requires from others
    pub requires: Vec<Capability>,
    /// Unix socket path for JSON-RPC
    pub socket_path: Option<String>,
    /// HTTP endpoint URL (temporary bridge)
    pub http_endpoint: Option<String>,
    /// Arbitrary key-value metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Registry response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryResponse {
    /// Correlation request identifier
    pub request_id: String,
    /// Response status
    pub status: ResponseStatus,
    /// Payload (if any)
    pub data: Option<serde_json::Value>,
    /// Error message (if any)
    pub error: Option<String>,
}

/// Response status codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Operation succeeded
    Success,
    /// Operation failed
    Error,
    /// Requested resource not found
    NotFound,
}
