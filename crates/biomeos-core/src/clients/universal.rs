// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Universal Primal Client - Dynamic API Adaptation
//!
//! This module provides a universal client that can interact with any primal
//! by discovering and adapting to its API schema at runtime.
//!
//! # Philosophy
//!
//! **Zero API Coupling**: biomeOS adapts to primal APIs, not the other way around.
//!
//! Instead of hardcoded client wrappers for each primal, the Universal Primal Client:
//! 1. Fetches the primal's API schema from `/api/schema`
//! 2. Parses the schema (OpenAPI, JSON Schema, etc.)
//! 3. Creates an appropriate adapter
//! 4. Dynamically calls any operation
//!
//! # Example
//!
//! ```no_run
//! use biomeos_core::clients::universal::UniversalPrimalClient;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Discover primal's API dynamically
//!     let client = UniversalPrimalClient::from_endpoint("http://primal:9000").await?;
//!     
//!     // Call any operation without hardcoded methods
//!     let result = client.call_operation(
//!         "createBucket",
//!         json!({"name": "my-bucket"})
//!     ).await?;
//!     
//!     println!("Result: {}", result);
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use biomeos_types::api_schema::{ApiSchemaResponse, ApiSchemaType};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use crate::clients::openapi_adapter::OpenApiAdapter;

/// Universal primal client that adapts to any API dynamically
#[derive(Debug, Clone)]
pub struct UniversalPrimalClient {
    /// Base endpoint URL
    endpoint: String,
    
    /// Schema type
    schema_type: ApiSchemaType,
    
    /// The adapter for this primal's API
    adapter: ApiAdapter,
    
    /// HTTP client for direct requests
    http_client: Client,
}

/// API adapter enum (supports multiple schema types)
#[derive(Debug, Clone)]
pub enum ApiAdapter {
    /// OpenAPI v3 adapter
    OpenApi(OpenApiAdapter),
    
    /// JSON Schema adapter (future)
    #[allow(dead_code)]
    JsonSchema,
    
    /// GraphQL adapter (future)
    #[allow(dead_code)]
    GraphQL,
}

impl UniversalPrimalClient {
    /// Create a new universal primal client from an endpoint
    ///
    /// This method:
    /// 1. Fetches the API schema from `{endpoint}/api/schema`
    /// 2. Parses the schema
    /// 3. Creates an appropriate adapter
    ///
    /// # Arguments
    /// * `endpoint` - Base endpoint URL (e.g., "http://primal:9000")
    ///
    /// # Errors
    /// Returns an error if:
    /// - Schema endpoint is not available
    /// - Schema format is unsupported
    /// - Schema cannot be parsed
    pub async fn from_endpoint(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();
        
        // Fetch schema from standard endpoint
        let schema_url = format!("{}/api/schema", endpoint);
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        let response = client
            .get(&schema_url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch schema from {}", schema_url))?;
        
        if !response.status().is_success() {
            anyhow::bail!(
                "Schema endpoint returned error: {} {}",
                response.status(),
                response.text().await?
            );
        }
        
        let schema_response: ApiSchemaResponse = response
            .json()
            .await
            .context("Failed to parse schema response")?;
        
        // Create appropriate adapter based on schema type
        let adapter = match schema_response.schema_type {
            ApiSchemaType::OpenAPI => {
                let openapi_adapter = OpenApiAdapter::from_spec(
                    endpoint.clone(),
                    schema_response.schema,
                )?;
                ApiAdapter::OpenApi(openapi_adapter)
            }
            ApiSchemaType::JSONSchema => {
                anyhow::bail!("JSON Schema adapter not yet implemented");
            }
            ApiSchemaType::GraphQL => {
                anyhow::bail!("GraphQL adapter not yet implemented");
            }
            ApiSchemaType::Custom => {
                anyhow::bail!("Custom schema format not supported");
            }
        };
        
        Ok(Self {
            endpoint,
            schema_type: schema_response.schema_type,
            adapter,
            http_client: client,
        })
    }
    
    /// Call an operation dynamically
    ///
    /// # Arguments
    /// * `operation_id` - Operation ID from the API schema
    /// * `params` - Parameters for the operation (JSON object)
    ///
    /// # Returns
    /// The response from the operation as a JSON value
    ///
    /// # Errors
    /// Returns an error if the operation fails or doesn't exist.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::universal::UniversalPrimalClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = UniversalPrimalClient::from_endpoint("http://primal:9000").await?;
    /// let result = client.call_operation("createBucket", json!({"name": "data"})).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_operation(&self, operation_id: &str, params: Value) -> Result<Value> {
        match &self.adapter {
            ApiAdapter::OpenApi(adapter) => {
                adapter.call_operation(operation_id, params).await
            }
            ApiAdapter::JsonSchema => {
                anyhow::bail!("JSON Schema adapter not yet implemented")
            }
            ApiAdapter::GraphQL => {
                anyhow::bail!("GraphQL adapter not yet implemented")
            }
        }
    }
    
    /// List available operations
    ///
    /// Returns the list of operation IDs that can be called via `call_operation`.
    pub fn list_operations(&self) -> Vec<String> {
        match &self.adapter {
            ApiAdapter::OpenApi(adapter) => adapter.list_operations(),
            ApiAdapter::JsonSchema => vec![],
            ApiAdapter::GraphQL => vec![],
        }
    }
    
    /// Get the endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    
    /// Get the schema type
    pub fn schema_type(&self) -> &ApiSchemaType {
        &self.schema_type
    }
    
    /// Get API metadata (title, version, etc.)
    pub fn api_metadata(&self) -> ApiMetadata {
        match &self.adapter {
            ApiAdapter::OpenApi(adapter) => ApiMetadata {
                title: adapter.api_title().to_string(),
                version: adapter.api_version().to_string(),
                schema_type: "OpenAPI".to_string(),
            },
            ApiAdapter::JsonSchema => ApiMetadata {
                title: "Unknown".to_string(),
                version: "Unknown".to_string(),
                schema_type: "JSONSchema".to_string(),
            },
            ApiAdapter::GraphQL => ApiMetadata {
                title: "Unknown".to_string(),
                version: "Unknown".to_string(),
                schema_type: "GraphQL".to_string(),
            },
        }
    }
}

/// API metadata
#[derive(Debug, Clone)]
pub struct ApiMetadata {
    /// API title
    pub title: String,
    
    /// API version
    pub version: String,
    
    /// Schema type
    pub schema_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_from_endpoint_openapi() {
        let mock_server = MockServer::start().await;
        
        // Mock the /api/schema endpoint
        Mock::given(method("GET"))
            .and(path("/api/schema"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "schema_type": "openapi",
                "schema_version": "3.1.0",
                "schema": {
                    "openapi": "3.1.0",
                    "info": {
                        "title": "Test API",
                        "version": "1.0.0"
                    },
                    "paths": {
                        "/api/v1/test": {
                            "get": {
                                "operationId": "testOperation",
                                "responses": {
                                    "200": {
                                        "description": "Success"
                                    }
                                }
                            }
                        }
                    }
                },
                "capabilities": ["test"]
            })))
            .mount(&mock_server)
            .await;
        
        let client = UniversalPrimalClient::from_endpoint(&mock_server.uri()).await.unwrap();
        
        assert_eq!(client.schema_type(), &ApiSchemaType::OpenAPI);
        assert_eq!(client.endpoint(), &mock_server.uri());
        
        let metadata = client.api_metadata();
        assert_eq!(metadata.title, "Test API");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_list_operations() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/api/schema"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "schema_type": "openapi",
                "schema_version": "3.1.0",
                "schema": {
                    "openapi": "3.1.0",
                    "info": {"title": "Test", "version": "1.0"},
                    "paths": {
                        "/test": {
                            "get": {"operationId": "getTest", "responses": {"200": {"description": "OK"}}},
                            "post": {"operationId": "createTest", "responses": {"200": {"description": "OK"}}}
                        }
                    }
                }
            })))
            .mount(&mock_server)
            .await;
        
        let client = UniversalPrimalClient::from_endpoint(&mock_server.uri()).await.unwrap();
        let operations = client.list_operations();
        
        assert!(operations.contains(&"getTest".to_string()));
        assert!(operations.contains(&"createTest".to_string()));
        assert_eq!(operations.len(), 2);
    }

    #[tokio::test]
    async fn test_call_operation() {
        let mock_server = MockServer::start().await;
        
        // Mock schema endpoint
        Mock::given(method("GET"))
            .and(path("/api/schema"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "schema_type": "openapi",
                "schema_version": "3.1.0",
                "schema": {
                    "openapi": "3.1.0",
                    "info": {"title": "Test", "version": "1.0"},
                    "paths": {
                        "/api/v1/test": {
                            "post": {
                                "operationId": "createTest",
                                "requestBody": {
                                    "content": {
                                        "application/json": {
                                            "schema": {"type": "object"}
                                        }
                                    }
                                },
                                "responses": {
                                    "200": {
                                        "description": "Success",
                                        "content": {
                                            "application/json": {
                                                "schema": {"type": "object"}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })))
            .mount(&mock_server)
            .await;
        
        // Mock the actual operation
        Mock::given(method("POST"))
            .and(path("/api/v1/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "test-123",
                "status": "created"
            })))
            .mount(&mock_server)
            .await;
        
        let client = UniversalPrimalClient::from_endpoint(&mock_server.uri()).await.unwrap();
        let result = client.call_operation("createTest", json!({"name": "test"})).await.unwrap();
        
        assert_eq!(result["id"], "test-123");
        assert_eq!(result["status"], "created");
    }
}

