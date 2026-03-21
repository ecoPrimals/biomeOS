// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Graph template management with NestGate integration
//!
//! This module provides functionality to save, load, and manage graph templates
//! using NestGate as the storage backend. Templates allow users to save
//! commonly-used graph structures and reuse them across deployments.
//!
//! Deep Debt Principles:
//! - Capability-based discovery (no hardcoded NestGate endpoints)
//! - Runtime primal discovery via Songbird
//! - Zero unsafe code
//! - Modern async Rust

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::graph::{GraphId, PrimalGraph};

/// Graph template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTemplate {
    /// Unique template ID
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what this template does
    pub description: String,

    /// Template version (semantic versioning)
    pub version: String,

    /// Author/creator
    pub author: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// The actual graph structure
    pub graph: PrimalGraph,

    /// Template parameters (for customization)
    pub parameters: HashMap<String, TemplateParameter>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last modified timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,

    /// Parameter description
    pub description: String,

    /// Parameter type
    pub param_type: ParameterType,

    /// Default value (if any)
    pub default: Option<serde_json::Value>,

    /// Whether this parameter is required
    pub required: bool,
}

/// Parameter type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// Template manager with NestGate integration
pub struct GraphTemplateManager {
    /// NestGate client (discovered at runtime)
    nestgate_client: Option<NestGateTemplateClient>,

    /// Local cache of templates
    cache: std::sync::Arc<tokio::sync::RwLock<HashMap<String, GraphTemplate>>>,
}

impl GraphTemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        Self {
            nestgate_client: None,
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Discover and connect to NestGate
    pub async fn discover_nestgate(&mut self) -> Result<()> {
        tracing::info!("Discovering NestGate for template storage...");

        // Capability-based discovery (no hardcoded endpoints!)
        match NestGateTemplateClient::discover().await {
            Ok(client) => {
                tracing::info!("✅ NestGate discovered for template storage");
                self.nestgate_client = Some(client);
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️  NestGate not available, templates will be memory-only: {}",
                    e
                );
                Ok(()) // Graceful degradation
            }
        }
    }

    /// Save a template
    pub async fn save_template(&self, template: GraphTemplate) -> Result<()> {
        // Update cache
        self.cache
            .write()
            .await
            .insert(template.id.clone(), template.clone());

        // Persist to NestGate if available
        if let Some(ref client) = self.nestgate_client {
            client.store_template(&template).await?;
            tracing::info!("✅ Template '{}' saved to NestGate", template.name);
        } else {
            tracing::warn!(
                "⚠️  Template '{}' saved to memory only (NestGate unavailable)",
                template.name
            );
        }

        Ok(())
    }

    /// Load a template by ID
    pub async fn load_template(&self, template_id: &str) -> Result<GraphTemplate> {
        // Check cache first
        if let Some(template) = self.cache.read().await.get(template_id) {
            return Ok(template.clone());
        }

        // Try NestGate
        if let Some(ref client) = self.nestgate_client {
            let template = client.retrieve_template(template_id).await?;

            // Update cache
            self.cache
                .write()
                .await
                .insert(template_id.to_string(), template.clone());

            return Ok(template);
        }

        anyhow::bail!("Template '{}' not found", template_id)
    }

    /// List all templates
    pub async fn list_templates(&self) -> Result<Vec<GraphTemplate>> {
        // Try NestGate first
        if let Some(ref client) = self.nestgate_client {
            let templates = client.list_templates().await?;

            // Update cache
            let mut cache = self.cache.write().await;
            for template in &templates {
                cache.insert(template.id.clone(), template.clone());
            }

            return Ok(templates);
        }

        // Fall back to cache
        Ok(self.cache.read().await.values().cloned().collect())
    }

    /// Delete a template
    pub async fn delete_template(&self, template_id: &str) -> Result<()> {
        // Remove from cache
        self.cache.write().await.remove(template_id);

        // Remove from NestGate if available
        if let Some(ref client) = self.nestgate_client {
            client.delete_template(template_id).await?;
            tracing::info!("✅ Template '{}' deleted from NestGate", template_id);
        }

        Ok(())
    }

    /// Instantiate a template with parameters
    pub async fn instantiate_template(
        &self,
        template_id: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<PrimalGraph> {
        let template = self.load_template(template_id).await?;

        // Validate parameters
        self.validate_parameters(&template, &parameters)?;

        // Clone the graph
        let mut graph = template.graph.clone();

        // Apply parameters (replace placeholders in graph)
        self.apply_parameters(&mut graph, &template.parameters, &parameters)?;

        // Generate new graph ID
        graph.id = GraphId::new(format!("{}_instance", template_id));

        Ok(graph)
    }

    /// Validate template parameters
    fn validate_parameters(
        &self,
        template: &GraphTemplate,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Check required parameters
        for (param_name, param_def) in &template.parameters {
            if param_def.required && !parameters.contains_key(param_name) {
                anyhow::bail!("Required parameter '{}' not provided", param_name);
            }
        }

        // Validate parameter types
        for (param_name, value) in parameters {
            if let Some(param_def) = template.parameters.get(param_name) {
                let is_valid = match param_def.param_type {
                    ParameterType::String => value.is_string(),
                    ParameterType::Number => value.is_number(),
                    ParameterType::Boolean => value.is_boolean(),
                    ParameterType::Array => value.is_array(),
                    ParameterType::Object => value.is_object(),
                };

                if !is_valid {
                    anyhow::bail!(
                        "Parameter '{}' has invalid type: expected {:?}, got {:?}",
                        param_name,
                        param_def.param_type,
                        value
                    );
                }
            }
        }

        Ok(())
    }

    /// Apply parameters to graph
    ///
    /// Substitutes `{{param_name}}` placeholders in operation params with actual values.
    fn apply_parameters(
        &self,
        graph: &mut PrimalGraph,
        param_defs: &HashMap<String, TemplateParameter>,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Build a map of parameter values (including defaults)
        let mut effective_params: HashMap<String, serde_json::Value> = HashMap::new();

        // Start with defaults
        for (name, def) in param_defs {
            if let Some(default) = &def.default {
                effective_params.insert(name.clone(), default.clone());
            }
        }

        // Override with provided parameters
        for (name, value) in parameters {
            effective_params.insert(name.clone(), value.clone());
        }

        // Apply substitutions to each node's operation params
        for node in &mut graph.nodes {
            let params_str = serde_json::to_string(&node.operation.params)?;

            // Replace {{param_name}} with actual values
            let mut substituted = params_str;
            for (param_name, value) in &effective_params {
                let placeholder = format!("{{{{{}}}}}", param_name);
                let value_str = match value {
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    _ => value.to_string(),
                };
                substituted = substituted.replace(&placeholder, &value_str);
            }

            // Parse back to JSON
            node.operation.params = serde_json::from_str(&substituted)?;
        }

        Ok(())
    }
}

impl Default for GraphTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// NestGate client for template storage via JSON-RPC
///
/// Uses capability-based discovery via SystemPaths to find NestGate socket.
/// All operations use standard JSON-RPC 2.0 calls.
struct NestGateTemplateClient {
    /// Family ID for storage isolation
    family_id: String,
    /// Socket path for NestGate connection
    socket_path: std::path::PathBuf,
}

impl NestGateTemplateClient {
    /// Discover NestGate via capability-based XDG-compliant discovery
    ///
    /// Locates NestGate socket using SystemPaths:
    /// 1. `$XDG_RUNTIME_DIR/biomeos/nestgate-{family_id}.sock`
    /// 2. `/tmp/biomeos-$USER/nestgate-{family_id}.sock`
    async fn discover() -> Result<Self> {
        let family_id =
            std::env::var("NESTGATE_FAMILY_ID").unwrap_or_else(|_| biomeos_core::family_discovery::get_family_id());

        // Use SystemPaths for XDG-compliant socket discovery
        let socket_path = if let Ok(paths) = biomeos_types::SystemPaths::new() {
            paths.primal_socket(&format!("nestgate-{}", family_id))
        } else {
            // Fallback path
            std::path::PathBuf::from(format!("/tmp/nestgate-{}.sock", family_id))
        };

        // Verify socket exists (NestGate must be running)
        if !socket_path.exists() {
            tracing::warn!(
                "NestGate socket not found at {} - template persistence unavailable",
                socket_path.display()
            );
            // Still return a client - calls will fail gracefully
        }

        Ok(Self {
            family_id,
            socket_path,
        })
    }

    /// Store a template in NestGate via JSON-RPC storage.store
    async fn store_template(&self, template: &GraphTemplate) -> Result<()> {
        let key = format!("template:{}", template.id);
        let value = serde_json::to_string(template)?;

        let params = serde_json::json!({
            "key": key,
            "value": value,
            "family_id": self.family_id,
        });

        tracing::debug!("Storing template '{}' in NestGate via JSON-RPC", template.id);

        // Call NestGate storage.store via JSON-RPC
        let _result: serde_json::Value = biomeos_nucleus::client::call_unix_socket_rpc(
            &self.socket_path,
            "storage.store",
            params,
        )
        .await
        .map_err(|e| anyhow::anyhow!("NestGate storage.store failed: {}", e))?;

        tracing::info!("✅ Template '{}' stored in NestGate", template.id);
        Ok(())
    }

    /// Retrieve a template from NestGate via JSON-RPC storage.retrieve
    async fn retrieve_template(&self, template_id: &str) -> Result<GraphTemplate> {
        let key = format!("template:{}", template_id);

        let params = serde_json::json!({
            "key": key,
            "family_id": self.family_id,
        });

        tracing::debug!("Retrieving template '{}' from NestGate via JSON-RPC", template_id);

        // Call NestGate storage.retrieve via JSON-RPC
        let result: serde_json::Value = biomeos_nucleus::client::call_unix_socket_rpc(
            &self.socket_path,
            "storage.retrieve",
            params,
        )
        .await
        .map_err(|e| anyhow::anyhow!("NestGate storage.retrieve failed: {}", e))?;

        // Extract value from result
        let value_str = result
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found in NestGate", template_id))?;

        // Deserialize template
        let template: GraphTemplate = serde_json::from_str(value_str)?;

        tracing::info!("✅ Template '{}' retrieved from NestGate", template_id);
        Ok(template)
    }

    /// List all templates from NestGate via JSON-RPC storage.list
    async fn list_templates(&self) -> Result<Vec<GraphTemplate>> {
        let params = serde_json::json!({
            "prefix": "template:",
            "family_id": self.family_id,
        });

        tracing::debug!("Listing templates from NestGate via JSON-RPC");

        // Call NestGate storage.list via JSON-RPC
        let result: serde_json::Value = biomeos_nucleus::client::call_unix_socket_rpc(
            &self.socket_path,
            "storage.list",
            params,
        )
        .await
        .map_err(|e| anyhow::anyhow!("NestGate storage.list failed: {}", e))?;

        // Extract items from result
        let items = result
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut templates = Vec::new();
        for item in items {
            if let Some(value_str) = item.get("value").and_then(|v| v.as_str()) {
                if let Ok(template) = serde_json::from_str::<GraphTemplate>(value_str) {
                    templates.push(template);
                }
            }
        }

        tracing::info!("✅ Listed {} templates from NestGate", templates.len());
        Ok(templates)
    }

    /// Delete a template from NestGate via JSON-RPC storage.delete
    async fn delete_template(&self, template_id: &str) -> Result<()> {
        let key = format!("template:{}", template_id);

        let params = serde_json::json!({
            "key": key,
            "family_id": self.family_id,
        });

        tracing::debug!("Deleting template '{}' from NestGate via JSON-RPC", template_id);

        // Call NestGate storage.delete via JSON-RPC
        let _result: serde_json::Value = biomeos_nucleus::client::call_unix_socket_rpc(
            &self.socket_path,
            "storage.delete",
            params,
        )
        .await
        .map_err(|e| anyhow::anyhow!("NestGate storage.delete failed: {}", e))?;

        tracing::info!("✅ Template '{}' deleted from NestGate", template_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{
        CoordinationPattern, GraphId, NodeConstraints, Operation, PrimalGraph, PrimalNode,
        PrimalSelector, RetryPolicy,
    };
    use chrono::Utc;

    fn create_test_template() -> GraphTemplate {
        // Create a simple graph
        let graph = PrimalGraph {
            id: GraphId::new("test_graph"),
            name: "test_graph".to_string(),
            description: "Test graph".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![PrimalNode {
                id: "node1".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "storage".to_string(),
                },
                operation: Operation {
                    name: "storage.store".to_string(),
                    params: serde_json::json!({"key": "test"}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
            }],
            edges: vec![],
            coordination: CoordinationPattern::Sequential,
        };

        GraphTemplate {
            id: "test_template".to_string(),
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            version: "1.0.0".to_string(),
            author: Some("test_user".to_string()),
            tags: vec!["test".to_string()],
            graph,
            parameters: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_template_manager_creation() {
        let manager = GraphTemplateManager::new();
        assert!(manager.nestgate_client.is_none());
    }

    #[tokio::test]
    async fn test_save_and_load_template() {
        let manager = GraphTemplateManager::new();
        let template = create_test_template();

        // Save template
        manager.save_template(template.clone()).await.unwrap();

        // Load template
        let loaded = manager.load_template(&template.id).await.unwrap();
        assert_eq!(loaded.id, template.id);
        assert_eq!(loaded.name, template.name);
    }

    #[tokio::test]
    async fn test_list_templates() {
        let manager = GraphTemplateManager::new();
        let template1 = create_test_template();
        let mut template2 = create_test_template();
        template2.id = "test_template_2".to_string();

        manager.save_template(template1).await.unwrap();
        manager.save_template(template2).await.unwrap();

        let templates = manager.list_templates().await.unwrap();
        assert_eq!(templates.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_template() {
        let manager = GraphTemplateManager::new();
        let template = create_test_template();

        manager.save_template(template.clone()).await.unwrap();
        manager.delete_template(&template.id).await.unwrap();

        // Should not be found
        assert!(manager.load_template(&template.id).await.is_err());
    }

    #[tokio::test]
    async fn test_instantiate_template() {
        let manager = GraphTemplateManager::new();
        let template = create_test_template();

        manager.save_template(template.clone()).await.unwrap();

        // Instantiate with no parameters
        let graph = manager
            .instantiate_template(&template.id, HashMap::new())
            .await
            .unwrap();

        // Should have a new ID
        assert_ne!(graph.id, template.graph.id);

        // Should have same structure
        assert_eq!(graph.nodes.len(), template.graph.nodes.len());
    }

    #[tokio::test]
    async fn test_template_with_parameters() {
        let manager = GraphTemplateManager::new();
        let mut template = create_test_template();

        // Add a required parameter
        template.parameters.insert(
            "storage_key".to_string(),
            TemplateParameter {
                name: "storage_key".to_string(),
                description: "Key for storage".to_string(),
                param_type: ParameterType::String,
                default: None,
                required: true,
            },
        );

        manager.save_template(template.clone()).await.unwrap();

        // Try to instantiate without required parameter (should fail)
        assert!(manager
            .instantiate_template(&template.id, HashMap::new())
            .await
            .is_err());

        // Instantiate with required parameter
        let mut params = HashMap::new();
        params.insert("storage_key".to_string(), serde_json::json!("my_key"));

        let graph = manager
            .instantiate_template(&template.id, params)
            .await
            .unwrap();
        assert!(graph.nodes.len() > 0);
    }

    #[test]
    fn test_parameter_type_serialization() {
        let param = TemplateParameter {
            name: "test".to_string(),
            description: "Test param".to_string(),
            param_type: ParameterType::String,
            default: Some(serde_json::json!("default_value")),
            required: false,
        };

        let json = serde_json::to_string(&param).unwrap();
        let deserialized: TemplateParameter = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, param.name);
    }
}
