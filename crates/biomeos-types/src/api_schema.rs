// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Dynamic API Schema Types
//!
//! Types for discovering and adapting to primal API structures at runtime.
//! Enables biomeOS to work with any primal API without hardcoded client wrappers.
//!
//! # Philosophy
//!
//! Primals advertise their API structures just like they advertise capabilities.
//! biomeOS discovers these schemas and adapts dynamically, eliminating API coupling.
//!
//! # Example
//!
//! ```no_run
//! use biomeos_types::api_schema::{ApiSchemaResponse, ApiSchemaType};
//!
//! // Primal advertises its schema
//! let schema_response = ApiSchemaResponse {
//!     schema_type: ApiSchemaType::OpenAPI,
//!     schema_version: "3.1.0".to_string(),
//!     schema: serde_json::json!({
//!         "openapi": "3.1.0",
//!         "info": {"title": "My Primal API", "version": "1.0.0"},
//!         "paths": { /* ... */ }
//!     }),
//!     capabilities: vec!["storage".to_string()],
//!     primal_info: None,
//! };
//!
//! // biomeOS discovers and adapts
//! // (implementation in biomeos-core/src/clients/universal.rs)
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API schema type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiSchemaType {
    /// OpenAPI specification (v3.x)
    #[serde(rename = "openapi")]
    OpenAPI,
    /// JSON Schema
    #[serde(rename = "json-schema")]
    JSONSchema,
    /// GraphQL schema
    #[serde(rename = "graphql")]
    GraphQL,
    /// Custom schema format
    #[serde(rename = "custom")]
    Custom,
}

/// API schema response from GET /api/schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSchemaResponse {
    /// Schema type identifier
    pub schema_type: ApiSchemaType,
    
    /// Schema version (e.g., "3.1.0" for OpenAPI)
    pub schema_version: String,
    
    /// The actual schema document
    pub schema: serde_json::Value,
    
    /// Capabilities provided by this primal
    #[serde(default)]
    pub capabilities: Vec<String>,
    
    /// Optional primal metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primal_info: Option<PrimalInfo>,
}

/// Primal information metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name (e.g., "nestgate", "songbird")
    pub name: String,
    
    /// Primal version
    pub version: String,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// OpenAPI operation metadata (extracted from spec)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    /// Operation ID (e.g., "createBucket")
    pub operation_id: String,
    
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    
    /// URL path (e.g., "/api/v1/buckets")
    pub path: String,
    
    /// Request body schema (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_schema: Option<serde_json::Value>,
    
    /// Response schema
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<serde_json::Value>,
    
    /// Parameters (path, query, header)
    #[serde(default)]
    pub parameters: Vec<ParameterMetadata>,
}

/// Parameter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterMetadata {
    /// Parameter name
    pub name: String,
    
    /// Parameter location (path, query, header, cookie)
    #[serde(rename = "in")]
    pub location: String,
    
    /// Whether parameter is required
    #[serde(default)]
    pub required: bool,
    
    /// Parameter schema
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// Schema discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDiscoveryConfig {
    /// Timeout for schema fetching (seconds)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    
    /// Whether to cache discovered schemas
    #[serde(default = "default_cache")]
    pub cache_schemas: bool,
    
    /// Schema cache TTL (seconds)
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
    
    /// Fallback to static clients if schema discovery fails
    #[serde(default = "default_fallback")]
    pub fallback_to_static: bool,
}

fn default_timeout() -> u64 { 30 }
fn default_cache() -> bool { true }
fn default_cache_ttl() -> u64 { 3600 } // 1 hour
fn default_fallback() -> bool { true }

impl Default for SchemaDiscoveryConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            cache_schemas: default_cache(),
            cache_ttl_secs: default_cache_ttl(),
            fallback_to_static: default_fallback(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ApiSchemaType::OpenAPI).unwrap(),
            "\"openapi\""
        );
        assert_eq!(
            serde_json::to_string(&ApiSchemaType::JSONSchema).unwrap(),
            "\"json-schema\""
        );
    }

    #[test]
    fn test_schema_response_deserialization() {
        let json = serde_json::json!({
            "schema_type": "openapi",
            "schema_version": "3.1.0",
            "schema": {
                "openapi": "3.1.0",
                "info": {"title": "Test API", "version": "1.0.0"}
            },
            "capabilities": ["storage", "federation"]
        });

        let response: ApiSchemaResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.schema_type, ApiSchemaType::OpenAPI);
        assert_eq!(response.schema_version, "3.1.0");
        assert_eq!(response.capabilities, vec!["storage", "federation"]);
    }

    #[test]
    fn test_config_defaults() {
        let config = SchemaDiscoveryConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert!(config.cache_schemas);
        assert_eq!(config.cache_ttl_secs, 3600);
        assert!(config.fallback_to_static);
    }

    #[test]
    fn test_operation_metadata_serialization() {
        let op = OperationMetadata {
            operation_id: "createBucket".to_string(),
            method: "POST".to_string(),
            path: "/api/v1/buckets".to_string(),
            request_schema: Some(serde_json::json!({"type": "object"})),
            response_schema: Some(serde_json::json!({"type": "object"})),
            parameters: vec![],
        };

        let json = serde_json::to_value(&op).unwrap();
        assert_eq!(json["operation_id"], "createBucket");
        assert_eq!(json["method"], "POST");
    }
}

