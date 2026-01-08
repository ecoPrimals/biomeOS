//! BearDog client for cryptographic operations
//!
//! This client discovers BearDog via runtime discovery and delegates
//! all cryptographic operations to BearDog's HSM.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::discovery::{PrimalDiscovery, PrimalEndpoint};
use crate::FederationResult;

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
        discovery.discover().await
            .context("Failed to discover primals")?;
        
        let beardog = discovery
            .get("beardog")
            .ok_or_else(|| anyhow::anyhow!("BearDog not found via discovery"))?;
        
        if beardog.endpoints.is_empty() {
            return Err(anyhow::anyhow!("BearDog has no endpoints"));
        }
        
        let endpoint = match &beardog.endpoints[0] {
            PrimalEndpoint::UnixSocket { path } => {
                BearDogEndpoint::UnixSocket(path.clone())
            }
            PrimalEndpoint::Http { url } => {
                BearDogEndpoint::Http(url.clone())
            }
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
            BearDogEndpoint::UnixSocket(_path) => {
                // TODO: Implement Unix socket health check
                // For now, just check if socket exists
                Ok(())
            }
            BearDogEndpoint::Http(url) => {
                let client = reqwest::Client::new();
                let response = client
                    .get(format!("{}/health", url))
                    .send()
                    .await
                    .context("Failed to connect to BearDog")?;
                
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("BearDog health check failed: {}", response.status()))
                }
            }
        }
    }
    
    /// Verify if a seed is part of a family
    pub async fn verify_same_family(
        &self,
        family_id: &str,
        seed_hash: &str,
    ) -> Result<bool> {
        let request = LineageVerificationRequest {
            family_id: family_id.to_string(),
            seed_hash: seed_hash.to_string(),
        };
        
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(_path) => {
                // TODO: Implement Unix socket client
                // For now, return an error
                Err(anyhow::anyhow!("Unix socket lineage verification not yet implemented"))
            }
            BearDogEndpoint::Http(url) => {
                let client = reqwest::Client::new();
                let response: LineageVerificationResponse = client
                    .post(format!("{}/api/v1/lineage/verify_family", url))
                    .json(&request)
                    .send()
                    .await
                    .context("Failed to send lineage verification request")?
                    .json()
                    .await
                    .context("Failed to parse lineage verification response")?;
                
                Ok(response.is_family_member)
            }
        }
    }
    
    /// Derive a sub-federation encryption key
    pub async fn derive_subfed_key(
        &self,
        request: KeyDerivationRequest,
    ) -> Result<KeyDerivationResponse> {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(_path) => {
                // TODO: Implement Unix socket client
                Err(anyhow::anyhow!("Unix socket key derivation not yet implemented"))
            }
            BearDogEndpoint::Http(url) => {
                let client = reqwest::Client::new();
                let response: KeyDerivationResponse = client
                    .post(format!("{}/api/v1/keys/derive_subfed_key", url))
                    .json(&request)
                    .send()
                    .await
                    .context("Failed to send key derivation request")?
                    .json()
                    .await
                    .context("Failed to parse key derivation response")?;
                
                Ok(response)
            }
        }
    }
    
    /// Encrypt data using BearDog's HSM
    pub async fn encrypt_data(
        &self,
        data: &[u8],
        key_ref: &str,
    ) -> Result<Vec<u8>> {
        match &self.endpoint {
            BearDogEndpoint::UnixSocket(_path) => {
                Err(anyhow::anyhow!("Unix socket encryption not yet implemented"))
            }
            BearDogEndpoint::Http(url) => {
                #[derive(Serialize)]
                struct EncryptRequest {
                    data: String,
                    key_ref: String,
                }
                
                #[derive(Deserialize)]
                struct EncryptResponse {
                    encrypted_data: String,
                }
                
                let request = EncryptRequest {
                    data: base64::encode(data),
                    key_ref: key_ref.to_string(),
                };
                
                let client = reqwest::Client::new();
                let response: EncryptResponse = client
                    .post(format!("{}/api/v1/encrypt", url))
                    .json(&request)
                    .send()
                    .await
                    .context("Failed to send encryption request")?
                    .json()
                    .await
                    .context("Failed to parse encryption response")?;
                
                base64::decode(&response.encrypted_data)
                    .context("Failed to decode encrypted data")
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

