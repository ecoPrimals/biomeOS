// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! MCP (Model Context Protocol) tool definitions for Squirrel AI discovery.
//!
//! Absorbed from healthSpring (23 schemas), airSpring (10 schemas), and
//! wetSpring (8 schemas). Provides the shared wire types that primals use
//! to advertise capabilities to Squirrel's MCP gateway.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON Schema definition for a tool parameter or return type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonSchema {
    /// JSON Schema `type` (e.g. "object", "string", "number", "array").
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Property definitions for object types.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Self>,
    /// Required property names for object types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Items schema for array types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Self>>,
    /// Enum of allowed values.
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<serde_json::Value>,
    /// Default value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

/// MCP tool definition advertised to Squirrel for AI tool discovery.
///
/// Follows the MCP `tools/list` response schema so Squirrel can
/// route AI requests to the correct primal method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    /// Unique tool name (e.g. "`ecology.calculate_et0`").
    pub name: String,
    /// Human-readable description for AI context.
    pub description: String,
    /// Input parameters schema.
    #[serde(rename = "inputSchema")]
    pub input_schema: JsonSchema,
    /// Which primal provides this tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// The JSON-RPC method name to invoke (may differ from `name`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_method: Option<String>,
}

/// Collection of MCP tool definitions from a single primal.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpToolManifest {
    /// Primal that provides these tools.
    pub primal: String,
    /// Version of this manifest.
    pub version: String,
    /// Tool definitions.
    pub tools: Vec<McpToolDefinition>,
}

impl McpToolManifest {
    /// Create a new manifest for a primal.
    pub fn new(primal: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            primal: primal.into(),
            version: version.into(),
            tools: Vec::new(),
        }
    }

    /// Add a tool definition.
    pub fn add_tool(&mut self, tool: McpToolDefinition) -> &mut Self {
        self.tools.push(tool);
        self
    }

    /// Number of tools in this manifest.
    #[must_use]
    pub const fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Look up a tool by name.
    #[must_use]
    pub fn find_tool(&self, name: &str) -> Option<&McpToolDefinition> {
        self.tools.iter().find(|t| t.name == name)
    }
}

/// Builder for constructing `JsonSchema` objects ergonomically.
#[derive(Debug, Default)]
pub struct SchemaBuilder {
    schema: JsonSchema,
}

impl SchemaBuilder {
    /// Create an object schema.
    #[must_use]
    pub fn object() -> Self {
        Self {
            schema: JsonSchema {
                schema_type: "object".to_owned(),
                ..Default::default()
            },
        }
    }

    /// Create a string schema.
    #[must_use]
    pub fn string() -> Self {
        Self {
            schema: JsonSchema {
                schema_type: "string".to_owned(),
                ..Default::default()
            },
        }
    }

    /// Create a number schema.
    #[must_use]
    pub fn number() -> Self {
        Self {
            schema: JsonSchema {
                schema_type: "number".to_owned(),
                ..Default::default()
            },
        }
    }

    /// Add a description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.schema.description = Some(desc.into());
        self
    }

    /// Add a required property.
    #[must_use]
    pub fn property(mut self, name: impl Into<String>, schema: JsonSchema) -> Self {
        let name = name.into();
        self.schema.properties.insert(name.clone(), schema);
        self.schema.required.push(name);
        self
    }

    /// Add an optional property.
    #[must_use]
    pub fn optional_property(mut self, name: impl Into<String>, schema: JsonSchema) -> Self {
        self.schema.properties.insert(name.into(), schema);
        self
    }

    /// Build the final schema.
    #[must_use]
    pub fn build(self) -> JsonSchema {
        self.schema
    }
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_new_and_add() {
        let mut manifest = McpToolManifest::new("airspring", "0.10.0");
        manifest.add_tool(McpToolDefinition {
            name: "ecology.calculate_et0".to_owned(),
            description: "Calculate reference evapotranspiration".to_owned(),
            input_schema: SchemaBuilder::object()
                .property(
                    "temperature",
                    SchemaBuilder::number().description("Celsius").build(),
                )
                .build(),
            provider: Some("airspring".to_owned()),
            rpc_method: Some("ecology.calculate_et0".to_owned()),
        });
        assert_eq!(manifest.tool_count(), 1);
        assert!(manifest.find_tool("ecology.calculate_et0").is_some());
        assert!(manifest.find_tool("nonexistent").is_none());
    }

    #[test]
    fn schema_builder_object() {
        let schema = SchemaBuilder::object()
            .property("name", SchemaBuilder::string().build())
            .optional_property(
                "description",
                SchemaBuilder::string().description("optional desc").build(),
            )
            .build();
        assert_eq!(schema.schema_type, "object");
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.required.len(), 1);
        assert_eq!(schema.required[0], "name");
    }

    #[test]
    fn tool_definition_serde_roundtrip() {
        let tool = McpToolDefinition {
            name: "test.tool".to_owned(),
            description: "A test tool".to_owned(),
            input_schema: SchemaBuilder::object().build(),
            provider: Some("testprimal".to_owned()),
            rpc_method: None,
        };
        let json = serde_json::to_string(&tool).expect("serialize");
        let parsed: McpToolDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.name, "test.tool");
        assert_eq!(parsed.provider.as_deref(), Some("testprimal"));
    }

    #[test]
    fn manifest_serde_roundtrip() {
        let manifest = McpToolManifest::new("wetspring", "0.1.0");
        let json = serde_json::to_string(&manifest).expect("serialize");
        let parsed: McpToolManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.primal, "wetspring");
        assert_eq!(parsed.tool_count(), 0);
    }
}
