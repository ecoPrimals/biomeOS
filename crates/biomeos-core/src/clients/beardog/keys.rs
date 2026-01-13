//! BearDog Key Management Client
//!
//! Provides key generation, rotation, and lifecycle management.
//!
//! **NOTE**: This is a stub module created to allow compilation.
//! Full implementation pending BTSP Wave 2B completion.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Key management client for BearDog
///
/// **STUB**: Will be fully implemented in BTSP Wave 2B
pub struct KeysClient {
    // Implementation pending
    _placeholder: (),
}

impl KeysClient {
    /// Create a new keys client
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for KeysClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Key information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Key ID
    pub key_id: String,
    /// Key type (e.g., "rsa", "ed25519")
    pub key_type: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp (if any)
    pub expires_at: Option<u64>,
}
