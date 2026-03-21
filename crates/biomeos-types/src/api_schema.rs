// SPDX-License-Identifier: AGPL-3.0-only
//
// Copyright 2025-2026 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
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

const fn default_timeout() -> u64 {
    30
}
const fn default_cache() -> bool {
    true
}
const fn default_cache_ttl() -> u64 {
    3600
} // 1 hour
const fn default_fallback() -> bool {
    true
}

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
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    #[test]
    fn test_api_schema_type_all_variants() {
        for schema_type in [
            ApiSchemaType::OpenAPI,
            ApiSchemaType::JSONSchema,
            ApiSchemaType::GraphQL,
            ApiSchemaType::Custom,
        ] {
            let json = serde_json::to_string(&schema_type).unwrap();
            let back: ApiSchemaType = serde_json::from_str(&json).unwrap();
            assert_eq!(schema_type, back);
        }
    }

    #[test]
    fn test_api_schema_response_serde_roundtrip() {
        let response = ApiSchemaResponse {
            schema_type: ApiSchemaType::OpenAPI,
            schema_version: "3.1.0".to_string(),
            schema: serde_json::json!({"openapi": "3.1.0", "info": {"title": "Test"}}),
            capabilities: vec!["storage".to_string()],
            primal_info: Some(PrimalInfo {
                name: "nestgate".to_string(),
                version: "1.0.0".to_string(),
                metadata: HashMap::new(),
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        let back: ApiSchemaResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.schema_type, back.schema_type);
        assert_eq!(response.capabilities, back.capabilities);
    }

    #[test]
    fn test_primal_info_serde_roundtrip() {
        let info = PrimalInfo {
            name: "test-primal".to_string(),
            version: "0.1.0".to_string(),
            metadata: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: PrimalInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.name, back.name);
        assert_eq!(info.metadata.get("key"), back.metadata.get("key"));
    }

    #[test]
    fn test_parameter_metadata_serde_roundtrip() {
        let param = ParameterMetadata {
            name: "bucket_id".to_string(),
            location: "path".to_string(),
            required: true,
            schema: Some(serde_json::json!({"type": "string"})),
        };
        let json = serde_json::to_string(&param).unwrap();
        let back: ParameterMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(param.name, back.name);
        assert_eq!(param.required, back.required);
    }

    #[test]
    fn test_schema_discovery_config_serde_roundtrip() {
        let config = SchemaDiscoveryConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: SchemaDiscoveryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.timeout_secs, back.timeout_secs);
        assert_eq!(config.cache_schemas, back.cache_schemas);
    }
}
