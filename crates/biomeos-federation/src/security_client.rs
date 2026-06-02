// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Security provider client for cryptographic operations
//!
//! This client discovers the security provider via runtime discovery and delegates
//! all cryptographic operations to the provider's HSM.

use anyhow::{Context, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

use crate::discovery::{PrimalDiscovery, PrimalEndpoint};
use crate::unix_socket_client::UnixSocketClient;

/// Request payload for deriving a sub-federation key via the security provider
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

/// Security provider client for cryptographic operations
#[derive(Debug)]
pub struct SecurityProviderClient {
    endpoint: SecurityEndpoint,
}

#[derive(Debug)]
enum SecurityEndpoint {
    UnixSocket(PathBuf),
}

impl SecurityProviderClient {
    /// Create a security provider client from runtime discovery
    pub async fn from_discovery() -> Result<Self> {
        let mut discovery = PrimalDiscovery::new();
        discovery
            .discover()
            .await
            .context("Failed to discover primals")?;

        let security_name = biomeos_types::env_config::security_provider()
            .or_else(|| {
                biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal("security")
                    .map(String::from)
            })
            .unwrap_or_else(|| biomeos_types::primal_names::BEARDOG.to_string());
        let security_provider = discovery
            .get(&security_name)
            .ok_or_else(|| anyhow::anyhow!("Security provider '{security_name}' not found via discovery"))?;

        if security_provider.endpoints.is_empty() {
            return Err(anyhow::anyhow!("Security provider has no endpoints"));
        }

        let endpoint = match &security_provider.endpoints[0] {
            PrimalEndpoint::UnixSocket { path } => SecurityEndpoint::UnixSocket(path.clone()),
            other => {
                return Err(anyhow::anyhow!(
                    "Security provider only supports Unix sockets, found: {other:?}"
                ));
            }
        };

        Ok(Self { endpoint })
    }

    /// Create a security provider client from an explicit endpoint string (no env reads).
    ///
    /// Alias for [`Self::with_endpoint`].
    pub fn from_endpoint(endpoint: &str) -> Result<Self> {
        Self::with_endpoint(endpoint)
    }

    /// Create a security provider client from an already-populated [`PrimalDiscovery`] (no env reads).
    pub fn from_primal_discovery(discovery: &PrimalDiscovery) -> Result<Self> {
        let security_name = biomeos_types::env_config::security_provider()
            .or_else(|| {
                biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal("security")
                    .map(String::from)
            })
            .unwrap_or_else(|| biomeos_types::primal_names::BEARDOG.to_string());
        let security_provider = discovery
            .get(&security_name)
            .ok_or_else(|| anyhow::anyhow!("Security provider '{security_name}' not found via discovery"))?;

        if security_provider.endpoints.is_empty() {
            return Err(anyhow::anyhow!("Security provider has no endpoints"));
        }

        let endpoint = match &security_provider.endpoints[0] {
            PrimalEndpoint::UnixSocket { path } => SecurityEndpoint::UnixSocket(path.clone()),
            other => {
                return Err(anyhow::anyhow!(
                    "Security provider only supports Unix sockets, found: {other:?}"
                ));
            }
        };

        Ok(Self { endpoint })
    }

    /// Create a security provider client with explicit endpoint
    pub fn with_endpoint(endpoint: impl AsRef<str>) -> Result<Self> {
        let endpoint = endpoint.as_ref();
        let path = if let Some(stripped) = endpoint.strip_prefix("unix://") {
            stripped
        } else if endpoint.starts_with('/') {
            endpoint
        } else {
            return Err(anyhow::anyhow!(
                "Security provider only supports Unix sockets (unix:// or absolute path), got: {endpoint}"
            ));
        };

        Ok(Self {
            endpoint: SecurityEndpoint::UnixSocket(PathBuf::from(path)),
        })
    }

    /// Check if the security provider is available
    pub fn is_available(&self) -> bool {
        let SecurityEndpoint::UnixSocket(path) = &self.endpoint;
        path.exists()
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        let SecurityEndpoint::UnixSocket(path) = &self.endpoint;
        let client = UnixSocketClient::new(path);

        if !client.is_available() {
            return Err(anyhow::anyhow!(
                "Security provider Unix socket not found: {}",
                path.display()
            ));
        }

        let result = client
            .call_method("health.check", serde_json::json!({}))
            .await
            .context("Unix socket health check failed")?;

        if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
            if status == "healthy" || status == "ok" {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Security provider reports unhealthy status: {status}"
                ))
            }
        } else {
            Ok(())
        }
    }

    /// Verify if a seed is part of a family (security provider v0.15.2+)
    pub async fn verify_same_family(
        &self,
        family_id: &str,
        seed_hash: &str,
        node_id: &str,
    ) -> Result<LineageVerificationResponse> {
        let SecurityEndpoint::UnixSocket(path) = &self.endpoint;
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

    /// Derive a sub-federation encryption key
    pub async fn derive_subfed_key(
        &self,
        request: KeyDerivationRequest,
    ) -> Result<KeyDerivationResponse> {
        let SecurityEndpoint::UnixSocket(path) = &self.endpoint;
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

    /// Encrypt data using the security provider's HSM
    pub async fn encrypt_data(&self, data: &[u8], key_ref: &str) -> Result<EncryptResponse> {
        let SecurityEndpoint::UnixSocket(path) = &self.endpoint;
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

    /// Decrypt data using the security provider's HSM
    pub async fn decrypt_data(
        &self,
        encrypted_data: &str,
        nonce: &str,
        tag: &str,
        key_ref: &str,
    ) -> Result<Bytes> {
        let SecurityEndpoint::UnixSocket(path) = &self.endpoint;
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
        let data = engine
            .decode(data_b64)
            .context("Failed to decode decrypted data")?;
        Ok(Bytes::from(data))
    }
}

#[cfg(test)]
#[path = "security_client_tests.rs"]
mod tests;
