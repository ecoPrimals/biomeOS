//! Layer 2: Identity Verification
//!
//! **Delegates to `BearDog`** - No reimplementation!
//!
//! `BearDog` handles:
//! - Ed25519 signature generation and verification
//! - Process identity validation
//! - Challenge-response authentication
//! - Family key management
//!
//! This layer just coordinates `BearDog`'s existing APIs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{discovery::DiscoveredPrimal, Error, Result};

/// Identity proof (from primal, signed by `BearDog`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProof {
    /// Primal name
    pub primal_name: String,
    /// Node ID
    pub node_id: String,
    /// Family ID
    pub family_id: String,
    /// Version
    pub version: String,
    /// Process ID
    pub process_id: u32,
    /// Socket path
    pub socket_path: String,
    /// Owner UID
    pub owner_uid: u32,
    /// Owner GID
    pub owner_gid: u32,
    /// Started at timestamp
    pub started_at: String,
    /// Challenge nonce (for freshness)
    pub challenge: String,
    /// Ed25519 signature (signed by `BearDog`)
    pub signature: String,
}

/// Identity verification result
#[derive(Debug, Clone)]
pub struct IdentityVerification {
    /// Whether verification succeeded
    pub verified: bool,
    /// Identity proof
    pub proof: IdentityProof,
    /// Verification message
    pub message: String,
}

/// Identity verification layer (delegates to `BearDog`)
#[async_trait]
pub trait IdentityLayer: Send + Sync {
    /// Request identity proof from a primal
    ///
    /// Delegates to the primal's `identity.get_proof` API
    async fn request_proof(&self, endpoint: &str, challenge: &str) -> Result<IdentityProof>;

    /// Verify identity proof
    ///
    /// Delegates to `BearDog`'s `security.verify_primal_identity` API
    async fn verify_proof(&self, proof: &IdentityProof) -> Result<IdentityVerification>;

    /// Full verification flow (request + verify)
    async fn verify_identity(&self, discovered: &DiscoveredPrimal) -> Result<IdentityVerification>;
}

/// Identity layer implementation
pub struct IdentityLayerImpl {
    /// `BearDog` socket (discovered at runtime)
    pub(crate) beardog_socket: Option<String>,
}

impl IdentityLayerImpl {
    /// Create a new identity layer
    ///
    /// **Deep Debt Principle**: Discovers `BearDog` at runtime, no hardcoding!
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `BearDog` socket cannot be discovered (`BearDog` not running or socket not found)
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Identity Layer (delegating to BearDog)");

        // Discover BearDog socket (no hardcoded paths!)
        let beardog_socket = Self::discover_beardog_socket().await?;

        Ok(Self {
            beardog_socket: Some(beardog_socket),
        })
    }

    /// Generate a challenge nonce
    #[must_use]
    pub fn generate_challenge() -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(uuid::Uuid::new_v4().as_bytes());
        hasher.update(chrono::Utc::now().timestamp().to_le_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Discover `BearDog`'s Unix socket
    ///
    /// **Deep Debt Principle**: Runtime discovery, not hardcoded!
    async fn discover_beardog_socket() -> Result<String> {
        debug!("Discovering BearDog socket (no hardcoded paths)");

        // 1. Check environment variable
        if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
            debug!(
                "Found BearDog socket via BEARDOG_SOCKET env var: {}",
                socket
            );
            return Ok(socket);
        }

        // 2. Check XDG runtime directory (standard location)
        if let Ok(uid) = std::env::var("UID") {
            let runtime_path = format!("/run/user/{uid}/biomeos/beardog.sock");
            if tokio::fs::metadata(&runtime_path).await.is_ok() {
                debug!(
                    "Found BearDog socket in XDG runtime directory: {}",
                    runtime_path
                );
                return Ok(runtime_path);
            }
        }

        // 3. Check tmp directory
        let mut read_dir = tokio::fs::read_dir("/tmp")
            .await
            .map_err(|e| Error::discovery_failed(format!("Failed to read /tmp: {e}"), None))?;

        while let Some(entry) = read_dir.next_entry().await.map_err(|e| {
            Error::discovery_failed(format!("Failed to read directory entry: {e}"), None)
        })? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("beardog-")
                    && std::path::Path::new(filename)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                {
                    debug!("Found BearDog socket: {}", path.display());
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }

        Err(Error::discovery_failed(
            "Could not discover BearDog socket. Is BearDog running?",
            Some("identity".to_string()),
        ))
    }

    /// Get `BearDog` socket path
    fn beardog_socket(&self) -> Result<&str> {
        self.beardog_socket
            .as_deref()
            .ok_or_else(|| Error::discovery_failed("BearDog socket not initialized", None))
    }
}

#[async_trait]
impl IdentityLayer for IdentityLayerImpl {
    async fn verify_identity(&self, discovered: &DiscoveredPrimal) -> Result<IdentityVerification> {
        // Generate challenge
        let challenge = Self::generate_challenge();

        // Get primary endpoint
        let endpoint = discovered
            .endpoints
            .first()
            .ok_or_else(|| Error::invalid_response(&discovered.primal, "No endpoints available"))?;

        // Request proof
        let proof = self.request_proof(&endpoint.address, &challenge).await?;

        // Verify proof
        self.verify_proof(&proof).await
    }

    async fn request_proof(&self, endpoint: &str, challenge: &str) -> Result<IdentityProof> {
        debug!(
            endpoint = %endpoint,
            challenge = %challenge,
            "Requesting identity proof from primal"
        );

        let params = serde_json::json!({
            "challenge": challenge,
        });

        let response: serde_json::Value =
            crate::client::call_unix_socket_rpc(endpoint, "identity.get_proof", params).await?;

        // Parse proof
        let proof: IdentityProof = serde_json::from_value(response)?;

        debug!(primal = %proof.primal_name, "Received identity proof");
        Ok(proof)
    }

    async fn verify_proof(&self, proof: &IdentityProof) -> Result<IdentityVerification> {
        info!(
            primal = %proof.primal_name,
            family = %proof.family_id,
            "Verifying identity proof (via BearDog)"
        );

        let beardog_socket = self.beardog_socket()?;

        let params = serde_json::json!({
            "identity_proof": proof,
            "family_id": proof.family_id,
        });

        let response: serde_json::Value = crate::client::call_unix_socket_rpc(
            beardog_socket,
            "security.verify_primal_identity",
            params,
        )
        .await?;

        // Parse verification result
        let verified = response
            .get("verified")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| Error::invalid_response("beardog", "Missing 'verified' field"))?;

        let message = response
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message")
            .to_string();

        if !verified {
            return Err(Error::identity_verification_failed(
                &proof.primal_name,
                &message,
            ));
        }

        info!(primal = %proof.primal_name, "Identity verification successful");
        Ok(IdentityVerification {
            verified,
            proof: proof.clone(),
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_challenge() {
        let challenge1 = IdentityLayerImpl::generate_challenge();
        let challenge2 = IdentityLayerImpl::generate_challenge();

        // Challenges should be different
        assert_ne!(challenge1, challenge2);

        // Challenges should be hex strings
        assert!(challenge1.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(challenge1.len(), 64); // SHA256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_identity_proof_parsing() {
        let json = r#"{
            "primal_name": "beardog",
            "node_id": "node-alpha",
            "family_id": "nat0",
            "version": "0.15.2",
            "process_id": 12345,
            "socket_path": "/tmp/beardog.sock",
            "owner_uid": 1000,
            "owner_gid": 1000,
            "started_at": "2026-01-09T00:00:00Z",
            "challenge": "abc123",
            "signature": "sig123"
        }"#;

        let proof: IdentityProof = serde_json::from_str(json).unwrap();
        assert_eq!(proof.primal_name, "beardog");
        assert_eq!(proof.process_id, 12345);
    }
}
