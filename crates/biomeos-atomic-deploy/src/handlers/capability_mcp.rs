// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! MCP tool aggregation — extracted from `capability.rs` for file-size governance.

use super::capability::CapabilityHandler;
use anyhow::Result;
use serde_json::{Value, json};

impl CapabilityHandler {
    /// Aggregate MCP tool definitions from all registered capabilities.
    ///
    /// JSON-RPC method: `mcp.tools.list`
    ///
    /// biomeOS aggregates tool manifests from all known capability providers
    /// into a single MCP-compliant response that Squirrel's AI gateway can
    /// consume. Each tool references its provider and the JSON-RPC method
    /// to invoke, enabling AI-driven capability routing.
    pub async fn mcp_tools_list(&self) -> Result<Value> {
        use biomeos_types::mcp::{McpToolDefinition, SchemaBuilder};

        let registry = self.translation_registry.read().await;
        let all_translations = registry.list_all();

        let tools: Vec<Value> = all_translations
            .iter()
            .map(|t| {
                let tool = McpToolDefinition {
                    name: t.semantic.clone(),
                    description: format!(
                        "Invoke {}.{} via {} (auto-discovered)",
                        t.provider, t.actual_method, t.provider
                    ),
                    input_schema: SchemaBuilder::object()
                        .optional_property(
                            "args",
                            SchemaBuilder::object()
                                .description("Method-specific arguments")
                                .build(),
                        )
                        .build(),
                    provider: Some(t.provider.clone()),
                    rpc_method: Some(t.actual_method.clone()),
                };
                serde_json::to_value(&tool).unwrap_or_default()
            })
            .collect();

        let providers: Vec<&str> = {
            let mut seen = std::collections::HashSet::new();
            all_translations
                .iter()
                .filter_map(|t| {
                    if seen.insert(t.provider.as_str()) {
                        Some(t.provider.as_str())
                    } else {
                        None
                    }
                })
                .collect()
        };

        Ok(json!({
            "tools": tools,
            "tool_count": tools.len(),
            "providers": providers,
            "provider_count": providers.len(),
        }))
    }
}
