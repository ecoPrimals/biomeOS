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

use crate::clients::base::PrimalHttpClient;
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// NestGate storage and persistence client
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::nestgate::NestGateClient;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let nestgate = NestGateClient::new("http://localhost:8002");
///
///     // Store data
///     let _stored = nestgate.store("my-key", &json!({"data": "value"})).await?;
///     println!("Data stored successfully");
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct NestGateClient {
    http: PrimalHttpClient,
    endpoint: String,
}

impl NestGateClient {
    /// Create a new NestGate client
    ///
    /// # Arguments
    /// * `endpoint` - NestGate endpoint URL (discovered via capability query for "storage")
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }

    /// Store data with a key
    ///
    /// # Arguments
    /// * `key` - Storage key
    /// * `data` - Data to store
    ///
    /// # Returns
    /// Storage confirmation with metadata
    ///
    /// # Errors
    /// Returns an error if the storage operation fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::nestgate::NestGateClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let nestgate = NestGateClient::new("http://localhost:8002");
    /// let result = nestgate.store("my-key", &json!({"value": 42})).await?;
    /// println!("Stored at: {}", result.key);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store(&self, key: &str, data: &Value) -> Result<StorageResult> {
        let body = serde_json::json!({
            "key": key,
            "data": data
        });

        let response = self.http.post("/api/v1/storage/store", body).await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse storage result: {}", e))
    }

    /// Retrieve data by key
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Returns
    /// Retrieved data
    ///
    /// # Errors
    /// Returns an error if the key is not found or retrieval fails.
    pub async fn retrieve(&self, key: &str) -> Result<Value> {
        let response = self
            .http
            .get(&format!("/api/v1/storage/retrieve/{}", key))
            .await?;

        Ok(response["data"].clone())
    }

    /// Delete data by key
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.http
            .post(
                &format!("/api/v1/storage/delete/{}", key),
                serde_json::json!({}),
            )
            .await?;

        Ok(())
    }

    /// List all stored keys
    ///
    /// # Arguments
    /// * `prefix` - Optional key prefix filter
    ///
    /// # Errors
    /// Returns an error if listing fails.
    pub async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let path = if let Some(prefix) = prefix {
            format!("/api/v1/storage/list?prefix={}", prefix)
        } else {
            "/api/v1/storage/list".to_string()
        };

        let response = self.http.get(&path).await?;

        serde_json::from_value(response["keys"].clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse key list: {}", e))
    }

    /// Get storage statistics
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let response = self.http.get("/api/v1/storage/stats").await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse storage stats: {}", e))
    }

    /// Store a blob (binary data)
    ///
    /// # Arguments
    /// * `key` - Storage key
    /// * `blob` - Binary data
    ///
    /// # Errors
    /// Returns an error if the storage operation fails.
    pub async fn store_blob(&self, key: &str, blob: &[u8]) -> Result<StorageResult> {
        use base64::Engine;
        let body = serde_json::json!({
            "key": key,
            "blob": base64::engine::general_purpose::STANDARD.encode(blob)
        });

        let response = self.http.post("/api/v1/storage/blob", body).await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse storage result: {}", e))
    }

    /// Retrieve a blob (binary data)
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Returns
    /// Binary data
    ///
    /// # Errors
    /// Returns an error if the key is not found or retrieval fails.
    pub async fn retrieve_blob(&self, key: &str) -> Result<Vec<u8>> {
        use base64::Engine;
        let response = self
            .http
            .get(&format!("/api/v1/storage/blob/{}", key))
            .await?;

        let base64_data = response["blob"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No blob data in response"))?;

        base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| anyhow::anyhow!("Failed to decode blob: {}", e))
    }
}

#[async_trait]
impl PrimalClient for NestGateClient {
    fn name(&self) -> &str {
        "nestgate"
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            details: Some(response),
        })
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
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

    #[test]
    fn test_nestgate_client_creation() {
        let client = NestGateClient::new("http://localhost:8002");
        assert_eq!(client.name(), "nestgate");
        assert_eq!(client.endpoint(), "http://localhost:8002");
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
