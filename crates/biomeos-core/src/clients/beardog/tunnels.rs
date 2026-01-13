//! BearDog BTSP Tunnel Operations Client
//!
//! Provides low-level BTSP tunnel operations.
//!
//! **NOTE**: This is a stub module created to allow compilation.
//! Full implementation pending BTSP Wave 2B completion.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// BTSP tunnel operations client for BearDog
///
/// **STUB**: Will be fully implemented in BTSP Wave 2B
pub struct TunnelsClient {
    // Implementation pending
    _placeholder: (),
}

impl TunnelsClient {
    /// Create a new tunnels client
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for TunnelsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Tunnel information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    /// Tunnel ID
    pub tunnel_id: String,
    /// Local endpoint
    pub local_endpoint: String,
    /// Remote endpoint
    pub remote_endpoint: String,
    /// Tunnel status
    pub status: TunnelStatus,
}

/// Tunnel status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelStatus {
    /// Tunnel is active
    Active,
    /// Tunnel is establishing
    Establishing,
    /// Tunnel is closed
    Closed,
    /// Tunnel encountered an error
    Error,
}
