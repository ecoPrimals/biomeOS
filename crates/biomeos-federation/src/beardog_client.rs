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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationRequest {
    pub parent_family: String,
    pub subfed_name: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationResponse {
    pub key_ref: String,
    pub algorithm: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptResponse {
    pub encrypted_data: String,
    pub nonce: String,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageVerificationRequest {
    pub family_id: String,
    pub seed_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageVerificationResponse {
    pub is_family_member: bool,
    pub parent_seed_hash: String,
    pub relationship: String,
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
            return Err(anyhow::anyhow!("Invalid endpoint format: {}", endpoint));
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
                            "BearDog reports unhealthy status: {}",
                            status
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
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_creation() {
        let client = BearDogClient::with_endpoint("http://localhost:9000".to_string()).unwrap();
        assert!(matches!(client.endpoint, BearDogEndpoint::Http(_)));

        let client = BearDogClient::with_endpoint("unix:///tmp/beardog.sock".to_string()).unwrap();
        assert!(matches!(client.endpoint, BearDogEndpoint::UnixSocket(_)));
    }

    #[test]
    fn test_invalid_endpoint() {
        let result = BearDogClient::with_endpoint("invalid://endpoint".to_string());
        assert!(result.is_err());
    }
}
