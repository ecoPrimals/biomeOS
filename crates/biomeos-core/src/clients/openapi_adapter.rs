// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! OpenAPI v3 Adapter for Dynamic API Interaction
//!
//! This adapter enables biomeOS to interact with any primal that provides an
//! OpenAPI v3 specification, without requiring hardcoded client wrappers.
//!
//! # Architecture
//!
//! ```text
//! Primal (/api/schema) → OpenAPI Spec → OpenApiAdapter → Dynamic Requests
//! ```
//!
//! # Example
//!
//! ```no_run
//! use biomeos_core::clients::openapi_adapter::OpenApiAdapter;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Fetch OpenAPI spec from primal
//!     let spec = reqwest::get("http://primal:9000/api/schema")
//!         .await?
//!         .json::<serde_json::Value>()
//!         .await?;
//!     
//!     // Create adapter
//!     let adapter = OpenApiAdapter::from_spec(
//!         "http://primal:9000",
//!         spec["schema"].clone()
//!     )?;
//!     
//!     // Call any operation dynamically
//!     let result = adapter.call_operation(
//!         "createBucket",
//!         json!({"name": "my-bucket"})
//!     ).await?;
//!     
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use reqwest::{Client, Method, Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// OpenAPI v3 adapter for dynamic API interaction
#[derive(Debug, Clone)]
pub struct OpenApiAdapter {
    /// Base URL for the API
    base_url: String,
    
    /// Parsed OpenAPI specification
    spec: OpenApiSpec,
    
    /// HTTP client for making requests
    http_client: Client,
    
    /// Operation lookup table
    operations: HashMap<String, Operation>,
}

/// Simplified OpenAPI v3 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    /// OpenAPI version
    pub openapi: String,
    
    /// API information
    pub info: ApiInfo,
    
    /// API paths
    #[serde(default)]
    pub paths: HashMap<String, PathItem>,
    
    /// Component schemas
    #[serde(default)]
    pub components: Option<Components>,
}

/// API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    /// API title
    pub title: String,
    
    /// API version
    pub version: String,
    
    /// API description
    #[serde(default)]
    pub description: Option<String>,
}

/// Path item (collection of operations)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathItem {
    #[serde(default)]
    pub get: Option<OperationObject>,
    
    #[serde(default)]
    pub post: Option<OperationObject>,
    
    #[serde(default)]
    pub put: Option<OperationObject>,
    
    #[serde(default)]
    pub delete: Option<OperationObject>,
    
    #[serde(default)]
    pub patch: Option<OperationObject>,
}

/// Operation object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationObject {
    /// Operation ID (required for dynamic invocation)
    pub operation_id: Option<String>,
    
    /// Operation summary
    #[serde(default)]
    pub summary: Option<String>,
    
    /// Request body
    #[serde(default)]
    pub request_body: Option<RequestBody>,
    
    /// Responses
    #[serde(default)]
    pub responses: HashMap<String, Response>,
    
    /// Parameters
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

/// Request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Content types
    pub content: HashMap<String, MediaType>,
    
    /// Whether required
    #[serde(default)]
    pub required: bool,
}

/// Media type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    /// Schema
    #[serde(default)]
    pub schema: Option<Value>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Description
    pub description: String,
    
    /// Content
    #[serde(default)]
    pub content: Option<HashMap<String, MediaType>>,
}

/// Parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter location
    #[serde(rename = "in")]
    pub location: String,
    
    /// Whether required
    #[serde(default)]
    pub required: bool,
    
    /// Schema
    #[serde(default)]
    pub schema: Option<Value>,
}

/// Components (schemas, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Schema definitions
    #[serde(default)]
    pub schemas: HashMap<String, Value>,
}

/// Internal operation metadata
#[derive(Debug, Clone)]
struct Operation {
    /// Operation ID
    id: String,
    
    /// HTTP method
    method: Method,
    
    /// URL path
    path: String,
    
    /// Request body schema
    request_schema: Option<Value>,
    
    /// Response schema
    response_schema: Option<Value>,
    
    /// Parameters
    parameters: Vec<Parameter>,
}

impl OpenApiAdapter {
    /// Create a new OpenAPI adapter from a specification
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the API (e.g., "http://primal:9000")
    /// * `spec_value` - OpenAPI specification as JSON value
    ///
    /// # Errors
    /// Returns an error if the specification is invalid or cannot be parsed.
    pub fn from_spec(base_url: impl Into<String>, spec_value: Value) -> Result<Self> {
        let spec: OpenApiSpec = serde_json::from_value(spec_value)
            .context("Failed to parse OpenAPI specification")?;
        
        // Build operation lookup table
        let mut operations = HashMap::new();
        for (path, path_item) in &spec.paths {
            Self::register_operation(&mut operations, path, Method::GET, &path_item.get);
            Self::register_operation(&mut operations, path, Method::POST, &path_item.post);
            Self::register_operation(&mut operations, path, Method::PUT, &path_item.put);
            Self::register_operation(&mut operations, path, Method::DELETE, &path_item.delete);
            Self::register_operation(&mut operations, path, Method::PATCH, &path_item.patch);
        }
        
        Ok(Self {
            base_url: base_url.into(),
            spec,
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            operations,
        })
    }
    
    /// Register an operation in the lookup table
    fn register_operation(
        operations: &mut HashMap<String, Operation>,
        path: &str,
        method: Method,
        operation_obj: &Option<OperationObject>,
    ) {
        if let Some(op) = operation_obj {
            if let Some(operation_id) = &op.operation_id {
                let request_schema = op.request_body.as_ref()
                    .and_then(|rb| rb.content.get("application/json"))
                    .and_then(|mt| mt.schema.clone());
                
                let response_schema = op.responses.get("200")
                    .or_else(|| op.responses.get("201"))
                    .and_then(|r| r.content.as_ref())
                    .and_then(|c| c.get("application/json"))
                    .and_then(|mt| mt.schema.clone());
                
                operations.insert(
                    operation_id.clone(),
                    Operation {
                        id: operation_id.clone(),
                        method,
                        path: path.to_string(),
                        request_schema,
                        response_schema,
                        parameters: op.parameters.clone(),
                    },
                );
            }
        }
    }
    
    /// Call an operation dynamically
    ///
    /// # Arguments
    /// * `operation_id` - Operation ID from OpenAPI spec (e.g., "createBucket")
    /// * `params` - Parameters for the operation (JSON object)
    ///
    /// # Returns
    /// The response from the API as a JSON value
    ///
    /// # Errors
    /// Returns an error if the operation doesn't exist, request fails, or response is invalid.
    pub async fn call_operation(&self, operation_id: &str, params: Value) -> Result<Value> {
        let operation = self.operations.get(operation_id)
            .ok_or_else(|| anyhow::anyhow!("Operation '{}' not found in API spec", operation_id))?;
        
        // Build request
        let request = self.build_request(operation, params)?;
        
        // Execute request
        let response = self.http_client.execute(request).await
            .with_context(|| format!("Failed to execute operation '{}'", operation_id))?;
        
        // Parse response
        let status = response.status();
        let body = response.text().await?;
        
        if !status.is_success() {
            anyhow::bail!("Operation '{}' failed with status {}: {}", operation_id, status, body);
        }
        
        serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse response for operation '{}'", operation_id))
    }
    
    /// Build an HTTP request from operation metadata
    fn build_request(&self, operation: &Operation, params: Value) -> Result<Request> {
        let mut url = format!("{}{}", self.base_url, operation.path);
        let mut body = None;
        
        // Handle path parameters
        if let Some(obj) = params.as_object() {
            for param in &operation.parameters {
                if param.location == "path" {
                    if let Some(value) = obj.get(&param.name) {
                        let value_str = if value.is_string() {
                            value.as_str().unwrap().to_string()
                        } else {
                            value.to_string()
                        };
                        url = url.replace(&format!("{{{}}}", param.name), &value_str);
                    } else if param.required {
                        anyhow::bail!("Required path parameter '{}' not provided", param.name);
                    }
                }
            }
            
            // For POST/PUT/PATCH, use params as request body
            if matches!(operation.method, Method::POST | Method::PUT | Method::PATCH) {
                body = Some(params.clone());
            }
        }
        
        // Build request
        let mut request_builder = self.http_client
            .request(operation.method.clone(), &url);
        
        if let Some(body_value) = body {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .json(&body_value);
        }
        
        request_builder.build()
            .context("Failed to build HTTP request")
    }
    
    /// Get the list of available operations
    pub fn list_operations(&self) -> Vec<String> {
        self.operations.keys().cloned().collect()
    }
    
    /// Get operation metadata
    pub fn get_operation(&self, operation_id: &str) -> Option<&Operation> {
        self.operations.get(operation_id)
    }
    
    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Get the API title
    pub fn api_title(&self) -> &str {
        &self.spec.info.title
    }
    
    /// Get the API version
    pub fn api_version(&self) -> &str {
        &self.spec.info.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_openapi_spec() -> Value {
        json!({
            "openapi": "3.1.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0",
                "description": "A test API"
            },
            "paths": {
                "/api/v1/buckets": {
                    "post": {
                        "operationId": "createBucket",
                        "summary": "Create a new bucket",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "required": ["name"],
                                        "properties": {
                                            "name": {"type": "string"}
                                        }
                                    }
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "description": "Bucket created",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "object",
                                            "properties": {
                                                "id": {"type": "string"},
                                                "name": {"type": "string"}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/api/v1/buckets/{bucket_id}": {
                    "get": {
                        "operationId": "getBucket",
                        "parameters": [
                            {
                                "name": "bucket_id",
                                "in": "path",
                                "required": true,
                                "schema": {"type": "string"}
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "Bucket details"
                            }
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn test_parse_openapi_spec() {
        let spec = sample_openapi_spec();
        let adapter = OpenApiAdapter::from_spec("http://localhost:9000", spec).unwrap();
        
        assert_eq!(adapter.api_title(), "Test API");
        assert_eq!(adapter.api_version(), "1.0.0");
        assert_eq!(adapter.base_url(), "http://localhost:9000");
    }

    #[test]
    fn test_list_operations() {
        let spec = sample_openapi_spec();
        let adapter = OpenApiAdapter::from_spec("http://localhost:9000", spec).unwrap();
        
        let operations = adapter.list_operations();
        assert!(operations.contains(&"createBucket".to_string()));
        assert!(operations.contains(&"getBucket".to_string()));
        assert_eq!(operations.len(), 2);
    }

    #[test]
    fn test_get_operation() {
        let spec = sample_openapi_spec();
        let adapter = OpenApiAdapter::from_spec("http://localhost:9000", spec).unwrap();
        
        let op = adapter.get_operation("createBucket").unwrap();
        assert_eq!(op.id, "createBucket");
        assert_eq!(op.method, Method::POST);
        assert_eq!(op.path, "/api/v1/buckets");
        assert!(op.request_schema.is_some());
    }

    #[test]
    fn test_build_request_with_body() {
        let spec = sample_openapi_spec();
        let adapter = OpenApiAdapter::from_spec("http://localhost:9000", spec).unwrap();
        
        let operation = adapter.get_operation("createBucket").unwrap();
        let params = json!({"name": "test-bucket"});
        
        let request = adapter.build_request(operation, params).unwrap();
        assert_eq!(request.method(), Method::POST);
        assert_eq!(request.url().as_str(), "http://localhost:9000/api/v1/buckets");
    }

    #[test]
    fn test_build_request_with_path_params() {
        let spec = sample_openapi_spec();
        let adapter = OpenApiAdapter::from_spec("http://localhost:9000", spec).unwrap();
        
        let operation = adapter.get_operation("getBucket").unwrap();
        let params = json!({"bucket_id": "bucket-123"});
        
        let request = adapter.build_request(operation, params).unwrap();
        assert_eq!(request.method(), Method::GET);
        assert_eq!(request.url().as_str(), "http://localhost:9000/api/v1/buckets/bucket-123");
    }
}

