//! API schema parsing and validation
//!
//! Schema types are defined for future schema-driven API calls.
//!
//! ## Future Extensions
//! - OpenAPI 3.x parser (using `openapi` or `openapiv3` crate)
//! - JSON Schema parser (using `jsonschema` crate)
//! - Custom Primal Manifest format
//!
//! Currently, the client uses convention-based API patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::primal_client::error::Result;

/// Parsed API schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSchema {
    /// Schema version
    pub version: String,

    /// Schema format
    pub format: SchemaFormat,

    /// Available operations
    pub operations: HashMap<String, Operation>,

    /// Type definitions
    pub types: HashMap<String, TypeDefinition>,
}

/// Schema format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaFormat {
    /// OpenAPI 3.x
    OpenApi3,

    /// JSON Schema
    JsonSchema,

    /// Custom format
    Custom(String),
}

/// Operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Path template
    pub path: String,

    /// Request type (optional)
    pub request_type: Option<String>,

    /// Response type
    pub response_type: String,

    /// Error type (optional)
    pub error_type: Option<String>,
}

/// Type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// Type name
    pub name: String,

    /// Fields/properties
    pub fields: HashMap<String, FieldDefinition>,
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Field type
    pub field_type: String,

    /// Is required
    pub required: bool,
}

/// Trait for schema parsers
pub trait SchemaParser: Send + Sync {
    /// Parse schema from bytes
    fn parse(&self, schema_bytes: &[u8]) -> Result<ApiSchema>;

    /// Get operation information
    fn get_operation<'a>(&self, schema: &'a ApiSchema, operation: &str) -> Result<&'a Operation>;
}

// Placeholder implementations for future schema-driven features

/// OpenAPI schema parser (not yet implemented)
///
/// Future implementation would use the `openapiv3` crate to parse
/// OpenAPI 3.x specifications and enable schema-driven API calls.
pub struct OpenApiSchemaParser;

impl SchemaParser for OpenApiSchemaParser {
    fn parse(&self, _schema_bytes: &[u8]) -> Result<ApiSchema> {
        Err(crate::primal_client::error::ApiError::Other {
            message: "OpenAPI parser not yet implemented".to_string(),
        })
    }

    fn get_operation<'a>(&self, schema: &'a ApiSchema, operation: &str) -> Result<&'a Operation> {
        schema.operations.get(operation).ok_or_else(|| {
            crate::primal_client::error::ApiError::NotFound {
                resource: format!("Operation: {}", operation),
            }
        })
    }
}
