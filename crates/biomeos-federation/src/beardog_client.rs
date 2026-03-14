// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BearDog client for cryptographic operations
//!
//! This client discovers BearDog via runtime discovery and delegates
//! all cryptographic operations to BearDog's HSM.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

use crate::discovery::{PrimalDiscovery, PrimalEndpoint};
use crate::unix_socket_client::UnixSocketClient;

/// Request payload for deriving a sub-federation key via BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationRequest {
    /// Parent family identifier
    pub parent_family: String,
    /// Sub-federation name
    pub subfed_name: String,
    /// Key purpose (e.g. "encryption", "signing")
    pub purpose: String,
}

/// Response from a key derivation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationResponse {
    /// Opaque key reference (never raw material)
    pub key_ref: String,
    /// Cryptographic algorithm used
    pub algorithm: String,
    /// ISO-8601 creation timestamp
    pub created_at: String,
}

/// Response from an encryption operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptResponse {
    /// Base64-encoded ciphertext
    pub encrypted_data: String,
    /// Base64-encoded nonce / IV
    pub nonce: String,
    /// Base64-encoded authentication tag
    pub tag: String,
}

/// Request payload for verifying genetic lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageVerificationRequest {
    /// Family identifier to verify
    pub family_id: String,
    /// SHA-256 hash of the family seed
    pub seed_hash: String,
}

/// Response from a lineage verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageVerificationResponse {
    /// Whether the requester is a member of the family
    pub is_family_member: bool,
    /// Hash of the parent family seed for chain verification
    pub parent_seed_hash: String,
    /// Relationship descriptor (e.g. "child", "sibling")
    pub relationship: String,
}

impl std::fmt::Display for LineageVerificationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LineageVerification(member={}, relationship={}, parent_hash={})",
            self.is_family_member, self.relationship, self.parent_seed_hash
        )
    }
}

/// BearDog client for cryptographic operations
pub struct BearDogClient {
    endpoint: BearDogEndpoint,
}

enum BearDogEndpoint {
    UnixSocket(PathBuf),
    Http(String),
}

impl BearDogClient {
    /// Create a BearDog client from runtime discovery
    pub async fn from_discovery() -> Result<Self> {
        let mut discovery = PrimalDiscovery::new();
        discovery
            .discover()
            .await
            .context("Failed to discover primals")?;

        let beardog = discovery
            .get("beardog")
            .ok_or_else(|| anyhow::anyhow!("BearDog not found via discovery"))?;

        if beardog.endpoints.is_empty() {
            return Err(anyhow::anyhow!("BearDog has no endpoints"));
        }

        let endpoint = match &beardog.endpoints[0] {
            PrimalEndpoint::UnixSocket { path } => BearDogEndpoint::UnixSocket(path.clone()),
            PrimalEndpoint::Http { url } => BearDogEndpoint::Http(url.clone()),
            PrimalEndpoint::Udp { .. } => {
                return Err(anyhow::anyhow!("BearDog UDP endpoint not supported yet"));
            }
        };

        Ok(Self { endpoint })
    }

    /// Create a BearDog client with explicit endpoint
    pub fn with_endpoint(endpoint: String) -> Result<Self> {
        let endpoint = if endpoint.starts_with("unix://") {
            let path = endpoint.strip_prefix("unix://").unwrap_or(&endpoint);
            BearDogEndpoint::UnixSocket(PathBuf::from(path))
        } else if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            BearDogEndpoint::Http(endpoint)
        } else {
            return Err(anyhow::anyhow!("Invalid endpoint format: {endpoint}"));
        };

        Ok(Self { endpoint })
    }

    /// Check if BearDog is available
    pub async fn is_available(&self) -> bool {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(path) => path.exists(),
            BearDogEndpoint::Http(_) => {
                // Try a health check
                self.health_check().await.is_ok()
            }
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(path) => {
                // Complete implementation: JSON-RPC health check over Unix socket
                let client = UnixSocketClient::new(path);

                // First check if socket exists
                if !client.is_available() {
                    return Err(anyhow::anyhow!(
                        "BearDog Unix socket not found: {}",
                        path.display()
                    ));
                }

                // Call health.check method
                let result = client
                    .call_method("health.check", serde_json::json!({}))
                    .await
                    .context("Unix socket health check failed")?;

                // Check if response indicates healthy status
                if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
                    if status == "healthy" || status == "ok" {
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!(
                            "BearDog reports unhealthy status: {status}"
                        ))
                    }
                } else {
                    // If no status field, successful response means healthy
                    Ok(())
                }
            }
            BearDogEndpoint::Http(_url) => {
                // DEPRECATED: BearDog only uses Unix sockets (no HTTP)
                // HTTP has been moved to Songbird (Concentrated Gap strategy)
                Err(anyhow::anyhow!(
                    "HTTP endpoint deprecated - BearDog uses Unix sockets only"
                ))
            }
        }
    }

    /// Verify if a seed is part of a family (BearDog v0.15.2+)
    pub async fn verify_same_family(
        &self,
        family_id: &str,
        seed_hash: &str,
        node_id: &str,
    ) -> Result<LineageVerificationResponse> {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(path) => {
                let client = UnixSocketClient::new(path);

                let params = json!({
                    "family_id": family_id,
                    "seed_hash": seed_hash,
                    "node_id": node_id,
                });

                let result = client
                    .call_method("federation.verify_family_member", params)
                    .await
                    .context("Failed to call federation.verify_family_member")?;

                Ok(LineageVerificationResponse {
                    is_family_member: result["is_family_member"].as_bool().unwrap_or(false),
                    parent_seed_hash: result["parent_seed_hash"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    relationship: result["relationship"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string(),
                })
            }
            BearDogEndpoint::Http(_url) => {
                // DEPRECATED: BearDog only uses Unix sockets (no HTTP)
                Err(anyhow::anyhow!(
                    "HTTP endpoint deprecated - BearDog uses Unix sockets only"
                ))
            }
        }
    }

    /// Derive a sub-federation encryption key
    pub async fn derive_subfed_key(
        &self,
        request: KeyDerivationRequest,
    ) -> Result<KeyDerivationResponse> {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(path) => {
                let client = UnixSocketClient::new(path);

                let params = json!({
                    "parent_family": request.parent_family,
                    "subfed_name": request.subfed_name,
                    "purpose": request.purpose,
                    "derivation_info": format!("{}-{}", request.subfed_name, chrono::Utc::now().format("%Y-%m-%d")),
                });

                let result = client
                    .call_method("federation.derive_subfed_key", params)
                    .await
                    .context("Failed to call federation.derive_subfed_key")?;

                Ok(KeyDerivationResponse {
                    key_ref: result["key_ref"].as_str().unwrap_or("").to_string(),
                    algorithm: result["algorithm"]
                        .as_str()
                        .unwrap_or("AES-256-GCM")
                        .to_string(),
                    created_at: result["created_at"].as_str().unwrap_or("").to_string(),
                })
            }
            BearDogEndpoint::Http(_url) => {
                // DEPRECATED: BearDog only uses Unix sockets (no HTTP)
                Err(anyhow::anyhow!(
                    "HTTP endpoint deprecated - BearDog uses Unix sockets only"
                ))
            }
        }
    }

    /// Encrypt data using BearDog's HSM
    pub async fn encrypt_data(&self, data: &[u8], key_ref: &str) -> Result<EncryptResponse> {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(path) => {
                let client = UnixSocketClient::new(path);

                use base64::Engine;
                let engine = base64::engine::general_purpose::STANDARD;
                let data_b64 = engine.encode(data);

                let params = json!({
                    "data": data_b64,
                    "key_ref": key_ref,
                    "algorithm": "AES-256-GCM",
                });

                let result = client
                    .call_method("encryption.encrypt", params)
                    .await
                    .context("Failed to call encryption.encrypt")?;

                Ok(EncryptResponse {
                    encrypted_data: result["encrypted_data"].as_str().unwrap_or("").to_string(),
                    nonce: result["nonce"].as_str().unwrap_or("").to_string(),
                    tag: result["tag"].as_str().unwrap_or("").to_string(),
                })
            }
            BearDogEndpoint::Http(_url) => {
                // DEPRECATED: BearDog only uses Unix sockets (no HTTP)
                Err(anyhow::anyhow!(
                    "HTTP endpoint deprecated - BearDog uses Unix sockets only"
                ))
            }
        }
    }

    /// Decrypt data using BearDog's HSM
    pub async fn decrypt_data(
        &self,
        encrypted_data: &str,
        nonce: &str,
        tag: &str,
        key_ref: &str,
    ) -> Result<Vec<u8>> {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(path) => {
                let client = UnixSocketClient::new(path);

                let params = json!({
                    "encrypted_data": encrypted_data,
                    "nonce": nonce,
                    "tag": tag,
                    "key_ref": key_ref,
                });

                let result = client
                    .call_method("encryption.decrypt", params)
                    .await
                    .context("Failed to call encryption.decrypt")?;

                use base64::Engine;
                let engine = base64::engine::general_purpose::STANDARD;
                let data_b64 = result["data"].as_str().unwrap_or("");
                engine
                    .decode(data_b64)
                    .context("Failed to decode decrypted data")
            }
            BearDogEndpoint::Http(_url) => {
                // DEPRECATED: BearDog only uses Unix sockets (no HTTP)
                Err(anyhow::anyhow!(
                    "HTTP endpoint deprecated - BearDog uses Unix sockets only"
                ))
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_creation() {
        let client = BearDogClient::with_endpoint("http://localhost:9000".to_string())
            .expect("http endpoint should parse");
        assert!(matches!(client.endpoint, BearDogEndpoint::Http(_)));

        let client = BearDogClient::with_endpoint("unix:///tmp/beardog.sock".to_string())
            .expect("unix endpoint should parse");
        assert!(matches!(client.endpoint, BearDogEndpoint::UnixSocket(_)));

        let client = BearDogClient::with_endpoint("https://localhost:9000".to_string())
            .expect("https endpoint should parse");
        assert!(matches!(client.endpoint, BearDogEndpoint::Http(_)));
    }

    #[test]
    fn test_invalid_endpoint() {
        let result = BearDogClient::with_endpoint("invalid://endpoint".to_string());
        assert!(result.is_err(), "invalid scheme should fail");
    }

    #[test]
    fn test_key_derivation_request_serialization() {
        let req = KeyDerivationRequest {
            parent_family: "family-1".to_string(),
            subfed_name: "gaming".to_string(),
            purpose: "encryption".to_string(),
        };
        let json = serde_json::to_string(&req).expect("serialize KeyDerivationRequest");
        let restored: KeyDerivationRequest =
            serde_json::from_str(&json).expect("deserialize KeyDerivationRequest");
        assert_eq!(restored.parent_family, req.parent_family);
        assert_eq!(restored.subfed_name, req.subfed_name);
        assert_eq!(restored.purpose, req.purpose);
    }

    #[test]
    fn test_key_derivation_response_serialization() {
        let resp = KeyDerivationResponse {
            key_ref: "key-ref-123".to_string(),
            algorithm: "AES-256-GCM".to_string(),
            created_at: "2026-01-15T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize KeyDerivationResponse");
        let restored: KeyDerivationResponse =
            serde_json::from_str(&json).expect("deserialize KeyDerivationResponse");
        assert_eq!(restored.key_ref, resp.key_ref);
        assert_eq!(restored.algorithm, resp.algorithm);
    }

    #[test]
    fn test_encrypt_response_serialization() {
        let resp = EncryptResponse {
            encrypted_data: "base64data".to_string(),
            nonce: "base64nonce".to_string(),
            tag: "base64tag".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize EncryptResponse");
        let restored: EncryptResponse =
            serde_json::from_str(&json).expect("deserialize EncryptResponse");
        assert_eq!(restored.encrypted_data, resp.encrypted_data);
        assert_eq!(restored.nonce, resp.nonce);
        assert_eq!(restored.tag, resp.tag);
    }

    #[test]
    fn test_lineage_verification_request_serialization() {
        let req = LineageVerificationRequest {
            family_id: "family-1".to_string(),
            seed_hash: "sha256hash".to_string(),
        };
        let json = serde_json::to_string(&req).expect("serialize LineageVerificationRequest");
        let restored: LineageVerificationRequest =
            serde_json::from_str(&json).expect("deserialize LineageVerificationRequest");
        assert_eq!(restored.family_id, req.family_id);
        assert_eq!(restored.seed_hash, req.seed_hash);
    }

    #[test]
    fn test_lineage_verification_response_serialization_and_display() {
        let resp = LineageVerificationResponse {
            is_family_member: true,
            parent_seed_hash: "parent-hash".to_string(),
            relationship: "child".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize LineageVerificationResponse");
        let restored: LineageVerificationResponse =
            serde_json::from_str(&json).expect("deserialize LineageVerificationResponse");
        assert_eq!(restored.is_family_member, resp.is_family_member);
        assert_eq!(restored.relationship, resp.relationship);

        let display = resp.to_string();
        assert!(display.contains("member=true"));
        assert!(display.contains("relationship=child"));
        assert!(display.contains("parent_hash=parent-hash"));
    }

    #[test]
    fn test_with_endpoint_unix_path() {
        let client =
            BearDogClient::with_endpoint("unix:///run/user/1000/biomeos/beardog.sock".to_string())
                .unwrap();
        assert!(matches!(client.endpoint, BearDogEndpoint::UnixSocket(_)));
    }

    #[test]
    fn test_invalid_endpoint_ftp() {
        let result = BearDogClient::with_endpoint("ftp://localhost/path".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_endpoint_empty() {
        let result = BearDogClient::with_endpoint("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation_request_clone() {
        let req = KeyDerivationRequest {
            parent_family: "fam".to_string(),
            subfed_name: "sub".to_string(),
            purpose: "encryption".to_string(),
        };
        let cloned = req.clone();
        assert_eq!(cloned.parent_family, req.parent_family);
    }

    #[test]
    fn test_lineage_verification_response_not_member() {
        let resp = LineageVerificationResponse {
            is_family_member: false,
            parent_seed_hash: "".to_string(),
            relationship: "unknown".to_string(),
        };
        let display = resp.to_string();
        assert!(display.contains("member=false"));
        assert!(display.contains("unknown"));
    }
}
