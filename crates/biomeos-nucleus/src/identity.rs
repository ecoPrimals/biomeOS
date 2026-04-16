// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Layer 2: Identity Verification
//!
//! **Delegates to the security provider** (`BearDog` at runtime) — no reimplementation.
//!
//! The security primal handles:
//! - Ed25519 signature generation and verification
//! - Process identity validation
//! - Challenge-response authentication
//! - Family key management
//!
//! This layer coordinates those APIs over the `"encryption"` capability socket.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use biomeos_types::primal_names;

use crate::{Error, Result, discovery::DiscoveredPrimal};

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

/// Identity verification layer (delegates to the security provider)
pub trait IdentityLayer: Send + Sync {
    /// Request identity proof from a primal
    ///
    /// Delegates to the primal's `identity.get_proof` API
    fn request_proof(
        &self,
        endpoint: &str,
        challenge: &str,
    ) -> impl std::future::Future<Output = Result<IdentityProof>> + Send;

    /// Verify identity proof
    ///
    /// Delegates to `security.verify_primal_identity` on the security provider
    fn verify_proof(
        &self,
        proof: &IdentityProof,
    ) -> impl std::future::Future<Output = Result<IdentityVerification>> + Send;

    /// Full verification flow (request + verify)
    fn verify_identity(
        &self,
        discovered: &DiscoveredPrimal,
    ) -> impl std::future::Future<Output = Result<IdentityVerification>> + Send;
}

/// Identity layer implementation
pub struct IdentityLayerImpl {
    /// Security provider Unix socket (discovered at runtime via `"encryption"` capability)
    pub(crate) security_socket: Option<String>,
}

impl IdentityLayerImpl {
    /// Create a new identity layer
    ///
    /// **Deep Debt Principle**: Discovers the security provider at runtime, no hardcoding!
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Security provider socket cannot be discovered (service not running or socket not found)
    #[expect(
        clippy::unused_async,
        reason = "public API contract — callers already .await"
    )]
    pub async fn new() -> Result<Self> {
        Self::new_with_impl(None, None)
    }

    /// Create an identity layer with optional overrides (for tests and tooling).
    ///
    /// If `security_socket` is set, it is used as the security provider Unix socket path.
    /// Otherwise, if `runtime_dir` is set, socket discovery uses only that `XDG_RUNTIME_DIR`
    /// (no other env tiers). If both are `None`, full production discovery via process
    /// environment applies (same as [`Self::new`]).
    ///
    /// # Errors
    ///
    /// Returns an error if security provider socket discovery fails and no fallback is available.
    #[expect(
        clippy::unused_async,
        reason = "public API contract — callers already .await"
    )]
    pub async fn new_with(
        security_socket: Option<&str>,
        runtime_dir: Option<&str>,
    ) -> Result<Self> {
        Self::new_with_impl(security_socket, runtime_dir)
    }

    fn new_with_impl(security_socket: Option<&str>, runtime_dir: Option<&str>) -> Result<Self> {
        info!("Initializing NUCLEUS Identity Layer (security provider)");

        if let Some(path) = security_socket {
            return Ok(Self {
                security_socket: Some(path.to_string()),
            });
        }

        if let Some(rd) = runtime_dir {
            let xdg = rd.to_string();
            let env_fn = move |key: &str| -> Option<String> {
                if key == "XDG_RUNTIME_DIR" {
                    return Some(xdg.clone());
                }
                None
            };
            let sock = Self::discover_provider_socket("encryption", &env_fn)?;
            return Ok(Self {
                security_socket: Some(sock),
            });
        }

        let sock = Self::discover_provider_socket("encryption", &|k| {
            biomeos_types::capability_discovery::std_env(k)
        })?;
        Ok(Self {
            security_socket: Some(sock),
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

    /// Resolve a capability domain to a Unix socket via the 5-tier discovery protocol.
    ///
    /// Wraps [`biomeos_types::capability_discovery::discover_capability_socket`] with
    /// consistent [`Error`] mapping for nucleus callers.
    pub(crate) fn discover_provider_socket(
        capability: &str,
        env: &dyn Fn(&str) -> Option<String>,
    ) -> Result<String> {
        use biomeos_types::capability_discovery;

        debug!(
            capability = %capability,
            "Discovering provider socket (5-tier capability discovery)"
        );

        let not_found = if matches!(capability, "encryption" | "security") {
            "Could not discover security provider socket. Is the security primal running?"
        } else {
            "Could not discover provider socket for the requested capability."
        };

        capability_discovery::discover_capability_socket(capability, env)
            .ok_or_else(|| Error::discovery_failed(not_found, Some("identity".to_string())))
    }

    /// Security provider socket path (`encryption` capability)
    fn security_socket(&self) -> Result<&str> {
        self.security_socket.as_deref().ok_or_else(|| {
            Error::discovery_failed("Security provider socket not initialized", None)
        })
    }
}

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

        let security_socket = self.security_socket()?;

        let params = serde_json::json!({
            "identity_proof": proof,
            "family_id": proof.family_id,
        });

        let response: serde_json::Value = crate::client::call_unix_socket_rpc(
            security_socket,
            "security.verify_primal_identity",
            params,
        )
        .await?;

        // Parse verification result
        let verified = response
            .get("verified")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| {
                Error::invalid_response(primal_names::BEARDOG, "Missing 'verified' field")
            })?;

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

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveredPrimal;
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn sample_proof() -> IdentityProof {
        IdentityProof {
            primal_name: "beardog".to_string(),
            node_id: "n1".to_string(),
            family_id: "fam".to_string(),
            version: "1.0".to_string(),
            process_id: 1,
            socket_path: "/tmp/x.sock".to_string(),
            owner_uid: 1000,
            owner_gid: 1000,
            started_at: "2026-01-01".to_string(),
            challenge: "c".to_string(),
            signature: "sig".to_string(),
        }
    }

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
            "family_id": "1894e909e454",
            "version": "0.15.2",
            "process_id": 12345,
            "socket_path": "/tmp/beardog.sock",
            "owner_uid": 1000,
            "owner_gid": 1000,
            "started_at": "2026-01-09T00:00:00Z",
            "challenge": "abc123",
            "signature": "sig123"
        }"#;

        let proof: IdentityProof = serde_json::from_str(json).expect("parse proof");
        assert_eq!(proof.primal_name, "beardog");
        assert_eq!(proof.process_id, 12345);
        assert_eq!(proof.node_id, "node-alpha");
        assert_eq!(proof.family_id, "1894e909e454");
        assert_eq!(proof.socket_path, "/tmp/beardog.sock");
    }

    #[test]
    fn test_identity_proof_serialization() {
        let proof = IdentityProof {
            primal_name: "songbird".to_string(),
            node_id: "node-1".to_string(),
            family_id: "fam-1".to_string(),
            version: "0.1.0".to_string(),
            process_id: 9999,
            socket_path: "/tmp/songbird.sock".to_string(),
            owner_uid: 1000,
            owner_gid: 1000,
            started_at: "2026-01-01T00:00:00Z".to_string(),
            challenge: "challenge".to_string(),
            signature: "sig".to_string(),
        };
        let json = serde_json::to_string(&proof).expect("serialize");
        let loaded: IdentityProof = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(loaded.primal_name, proof.primal_name);
    }

    #[test]
    fn test_identity_verification_struct() {
        let proof = IdentityProof {
            primal_name: "test".to_string(),
            node_id: "n".to_string(),
            family_id: "f".to_string(),
            version: "0.1".to_string(),
            process_id: 1,
            socket_path: "/tmp/x.sock".to_string(),
            owner_uid: 0,
            owner_gid: 0,
            started_at: String::new(),
            challenge: "c".to_string(),
            signature: "s".to_string(),
        };
        let verification = IdentityVerification {
            verified: true,
            proof,
            message: "OK".to_string(),
        };
        assert!(verification.verified);
        assert_eq!(verification.message, "OK");
        assert_eq!(verification.proof.primal_name, "test");
    }

    #[tokio::test]
    async fn test_verify_identity_no_endpoints() {
        let layer = IdentityLayerImpl {
            security_socket: Some("/tmp/bd.sock".into()),
        };
        let discovered = DiscoveredPrimal {
            primal: "p".into(),
            node_id: "n".into(),
            family_id: "f".into(),
            capabilities: vec![],
            endpoints: vec![],
            signature: String::new(),
            timestamp: String::new(),
        };
        let r = layer.verify_identity(&discovered).await;
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(
            msg.contains("endpoint") || msg.contains("No endpoints"),
            "{msg}"
        );
    }

    #[tokio::test]
    async fn test_verify_proof_fails_when_not_verified() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("bd.sock");
        let (mut ready_tx, ready_rx) = ready_signal();
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        ready_tx.signal();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = stream.read(&mut buf).await;
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {"verified": false, "message": "bad"},
                    "id": 1
                });
                let _ = stream
                    .write_all(
                        format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
                    )
                    .await;
            }
        });
        ready_rx.wait().await.unwrap();

        let layer = IdentityLayerImpl {
            security_socket: Some(sock.to_string_lossy().into_owned()),
        };
        let r = layer.verify_proof(&sample_proof()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_verify_proof_success() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("bd2.sock");
        let (mut ready_tx, ready_rx) = ready_signal();
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        ready_tx.signal();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = stream.read(&mut buf).await;
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {"verified": true, "message": "ok"},
                    "id": 1
                });
                let _ = stream
                    .write_all(
                        format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
                    )
                    .await;
            }
        });
        ready_rx.wait().await.unwrap();

        let layer = IdentityLayerImpl {
            security_socket: Some(sock.to_string_lossy().into_owned()),
        };
        let r = layer.verify_proof(&sample_proof()).await;
        assert!(r.is_ok());
        assert!(r.unwrap().verified);
    }

    #[tokio::test]
    async fn test_verify_proof_missing_verified_field() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("bd3.sock");
        let (mut ready_tx, ready_rx) = ready_signal();
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        ready_tx.signal();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = stream.read(&mut buf).await;
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {"message": "oops"},
                    "id": 1
                });
                let _ = stream
                    .write_all(
                        format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
                    )
                    .await;
            }
        });
        ready_rx.wait().await.unwrap();

        let layer = IdentityLayerImpl {
            security_socket: Some(sock.to_string_lossy().into_owned()),
        };
        let r = layer.verify_proof(&sample_proof()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_request_proof_parses_identity_proof() {
        let temp = tempfile::tempdir().expect("tempdir");
        let primal_sock = temp.path().join("primal.sock");
        let proof = sample_proof();
        let proof_json = serde_json::to_value(&proof).unwrap();

        let (mut ready_tx, ready_rx) = ready_signal();
        let listener = tokio::net::UnixListener::bind(&primal_sock).expect("bind");
        ready_tx.signal();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = stream.read(&mut buf).await;
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": proof_json,
                    "id": 1
                });
                let _ = stream
                    .write_all(
                        format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
                    )
                    .await;
            }
        });
        ready_rx.wait().await.unwrap();

        let layer = IdentityLayerImpl {
            security_socket: Some("/unused".into()),
        };
        let r = layer
            .request_proof(primal_sock.to_str().unwrap(), "nonce")
            .await;
        assert!(r.is_ok());
        assert_eq!(r.unwrap().primal_name, "beardog");
    }

    #[tokio::test]
    async fn test_identity_layer_new_with_security_socket_env() {
        let layer =
            IdentityLayerImpl::new_with(Some("/tmp/nonexistent-but-env-ok.sock"), None).await;
        assert!(layer.is_ok());
        assert_eq!(
            layer.unwrap().security_socket.as_deref(),
            Some("/tmp/nonexistent-but-env-ok.sock")
        );
    }

    #[tokio::test]
    async fn test_identity_layer_new_discovers_via_xdg_runtime_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let biomeos_dir = tmp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).expect("biomeos dir");
        let sock_path = biomeos_dir.join("beardog.sock");
        std::fs::write(&sock_path, "").expect("write placeholder socket");

        let layer = IdentityLayerImpl::new_with(None, tmp.path().to_str())
            .await
            .expect("discover via XDG runtime");
        assert_eq!(
            layer.security_socket.as_deref(),
            Some(sock_path.to_str().expect("utf8"))
        );
    }
}
