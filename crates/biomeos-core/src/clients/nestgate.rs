// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! NestGate client for storage and persistence
//!
//! NestGate is the storage and persistence primal. It provides:
//! - Data storage and retrieval
//! - Object storage
//! - Key-value storage
//! - Blob storage
//!
//! # Transport Evolution
//!
//! **NEW**: Auto-discovery via Unix socket (JSON-RPC 2.0)
//! - **PRIMARY**: JSON-RPC over Unix socket (100x faster, secure)
//! - **FALLBACK**: HTTP REST API (deprecated, legacy only)

use crate::clients::transport::{TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// NestGate storage and persistence client
#[derive(Debug, Clone)]
pub struct NestGateClient {
    transport: TransportClient,
    family_id: String,
}

impl NestGateClient {
    /// Auto-discover NestGate via Unix socket
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = TransportClient::discover_with_preference(
            "nestgate",
            family_id,
            TransportPreference::JsonRpcUnixSocket,
        ).await
            .context("Failed to discover NestGate. Is it running?")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    #[deprecated(note = "Use NestGateClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let _endpoint = endpoint.into();
        let transport = TransportClient::discover_with_preference(
            "nestgate",
            family_id,
            TransportPreference::Http
        ).await
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    #[deprecated(note = "Use NestGateClient::discover() instead")]
    pub fn new(_endpoint: impl Into<String>) -> Self {
        panic!("NestGateClient::new() is deprecated. Use NestGateClient::discover() instead.");
    }

    /// Store data with a key (JSON-RPC: storage.store)
    pub async fn store(&self, key: &str, data: &Value) -> Result<StorageResult> {
        let response = self.transport.call(
            "storage.store",
            Some(serde_json::json!({
                "key": key,
                "data": data,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call storage.store")?;

        serde_json::from_value(response)
            .context("Failed to parse storage result from response")
    }

    /// Retrieve data by key (JSON-RPC: storage.retrieve)
    pub async fn retrieve(&self, key: &str) -> Result<Value> {
        let response = self.transport.call(
            "storage.retrieve",
            Some(serde_json::json!({
                "key": key,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call storage.retrieve")?;

        Ok(response["data"].clone())
    }

    /// Delete data by key (JSON-RPC: storage.delete)
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.transport.call(
            "storage.delete",
            Some(serde_json::json!({
                "key": key,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call storage.delete")?;

        Ok(())
    }

    /// List all stored keys (JSON-RPC: storage.list)
    pub async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let mut params = serde_json::json!({
            "family_id": self.family_id
        });
        
        if let Some(prefix) = prefix {
            params["prefix"] = serde_json::json!(prefix);
        }

        let response = self.transport.call("storage.list", Some(params)).await
            .context("Failed to call storage.list")?;

        serde_json::from_value(response["keys"].clone())
            .context("Failed to parse key list from response")
    }

    /// Get storage statistics (JSON-RPC: storage.stats)
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let response = self.transport.call(
            "storage.stats",
            Some(serde_json::json!({
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call storage.stats")?;

        serde_json::from_value(response)
            .context("Failed to parse storage stats from response")
    }

    /// Store a blob (binary data) (JSON-RPC: storage.store_blob)
    pub async fn store_blob(&self, key: &str, blob: &[u8]) -> Result<StorageResult> {
        use base64::Engine;
        let response = self.transport.call(
            "storage.store_blob",
            Some(serde_json::json!({
                "key": key,
                "blob": base64::engine::general_purpose::STANDARD.encode(blob),
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call storage.store_blob")?;

        serde_json::from_value(response)
            .context("Failed to parse storage result from response")
    }

    /// Retrieve a blob (binary data) (JSON-RPC: storage.retrieve_blob)
    pub async fn retrieve_blob(&self, key: &str) -> Result<Vec<u8>> {
        use base64::Engine;
        let response = self.transport.call(
            "storage.retrieve_blob",
            Some(serde_json::json!({
                "key": key,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call storage.retrieve_blob")?;

        let base64_data = response["blob"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No blob data in response"))?;

        base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .context("Failed to decode blob from base64")
    }
}

#[async_trait]
impl PrimalClient for NestGateClient {
    fn name(&self) -> &str {
        "nestgate"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        self.transport.call(method, body).await
    }
}

/// Storage operation result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageResult {
    /// Storage key
    pub key: String,

    /// Storage status
    pub status: String,

    /// Data size in bytes
    pub size_bytes: u64,

    /// Storage timestamp
    pub timestamp: String,
}

/// Storage statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageStats {
    /// Total number of stored keys
    pub total_keys: u64,

    /// Total storage used in bytes
    pub total_bytes: u64,

    /// Available storage in bytes
    pub available_bytes: u64,

    /// Storage utilization percentage
    pub utilization_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nestgate_client_creation() {
        let client = NestGateClient::discover("nat0").await.unwrap();
        assert_eq!(client.name(), "nestgate");
    }

    #[test]
    fn test_storage_result_deserialization() {
        let json = serde_json::json!({
            "key": "test-key",
            "status": "stored",
            "size_bytes": 1024,
            "timestamp": "2025-12-24T12:00:00Z"
        });

        let result: StorageResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.key, "test-key");
        assert_eq!(result.size_bytes, 1024);
    }

    #[test]
    fn test_storage_stats_deserialization() {
        let json = serde_json::json!({
            "total_keys": 100,
            "total_bytes": 1048576,
            "available_bytes": 9437184,
            "utilization_percent": 10.0
        });

        let stats: StorageStats = serde_json::from_value(json).unwrap();
        assert_eq!(stats.total_keys, 100);
        assert_eq!(stats.utilization_percent, 10.0);
    }
}
