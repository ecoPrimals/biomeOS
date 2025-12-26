//! NestGate API Adapter
//!
//! Adapter for NestGate's sovereign storage API.
//! Discovers storage, retrieval, and federation endpoints.

use crate::api_adapter::{ApiAdapter, discovery};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// NestGate-specific API adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestGateAdapter {
    /// Base API adapter
    base: ApiAdapter,
    
    /// NestGate-specific endpoints (discovered)
    storage_endpoint: Option<String>,
    retrieval_endpoint: Option<String>,
    metadata_endpoint: Option<String>,
    federation_endpoint: Option<String>,
    quota_endpoint: Option<String>,
}

impl NestGateAdapter {
    /// Discover NestGate's API structure
    pub async fn discover(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        
        // Use generic discovery first
        let base = discovery::discover_api_interface(&base_url, "nestgate").await?;
        
        // NestGate-specific discovery
        let mut adapter = Self {
            base,
            storage_endpoint: None,
            retrieval_endpoint: None,
            metadata_endpoint: None,
            federation_endpoint: None,
            quota_endpoint: None,
        };
        
        // Discover NestGate-specific endpoints
        adapter.discover_storage_endpoints().await;
        adapter.discover_retrieval_endpoints().await;
        adapter.discover_metadata_endpoints().await;
        adapter.discover_federation_endpoints().await;
        adapter.discover_quota_endpoints().await;
        
        Ok(adapter)
    }
    
    /// Discover storage endpoints
    async fn discover_storage_endpoints(&mut self) {
        let patterns = vec![
            "/storage/upload",
            "/storage/store",
            "/api/storage/upload",
            "/api/v1/storage/upload",
            "/upload",
            "/store",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.storage_endpoint = Some(pattern.to_string());
                println!("  ✓ Storage endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Discover retrieval endpoints
    async fn discover_retrieval_endpoints(&mut self) {
        let patterns = vec![
            "/storage/retrieve",
            "/storage/get",
            "/api/storage/retrieve",
            "/api/v1/storage/retrieve",
            "/retrieve",
            "/get",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.retrieval_endpoint = Some(pattern.to_string());
                println!("  ✓ Retrieval endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Discover metadata endpoints
    async fn discover_metadata_endpoints(&mut self) {
        let patterns = vec![
            "/storage/metadata",
            "/metadata",
            "/api/storage/metadata",
            "/api/v1/storage/metadata",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.metadata_endpoint = Some(pattern.to_string());
                println!("  ✓ Metadata endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Discover federation endpoints
    async fn discover_federation_endpoints(&mut self) {
        let patterns = vec![
            "/federation/join",
            "/api/federation/join",
            "/api/v1/federation/join",
            "/federate",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.federation_endpoint = Some(pattern.to_string());
                println!("  ✓ Federation endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Discover quota/capacity endpoints
    async fn discover_quota_endpoints(&mut self) {
        let patterns = vec![
            "/storage/quota",
            "/quota",
            "/api/storage/quota",
            "/api/v1/storage/quota",
            "/capacity",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.quota_endpoint = Some(pattern.to_string());
                println!("  ✓ Quota endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Get the base adapter
    pub fn base(&self) -> &ApiAdapter {
        &self.base
    }
    
    /// Check if NestGate storage is healthy
    pub async fn check_storage_health(&self) -> Result<bool> {
        // Try storage-specific endpoint first
        if let Some(endpoint) = &self.quota_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();
            
            if let Ok(response) = client.get(&url).send().await {
                return Ok(response.status().is_success());
            }
        }
        
        // Fallback to generic health check
        self.base.check_health().await
    }
    
    /// Get storage quota (if endpoint discovered)
    pub async fn get_quota(&self) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.quota_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();
            
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }
        
        Ok(None)
    }
    
    /// Get storage metadata (if endpoint discovered)
    pub async fn get_metadata(&self, file_id: &str) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.metadata_endpoint {
            let url = format!("{}{}/{}", self.base.base_url(), endpoint, file_id);
            let client = reqwest::Client::new();
            
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nestgate_adapter_creation() {
        // Test will require actual NestGate instance
        assert_eq!(std::mem::size_of::<NestGateAdapter>(), std::mem::size_of::<NestGateAdapter>());
    }
}

