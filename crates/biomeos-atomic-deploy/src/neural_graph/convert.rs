// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Conversion from `DeploymentGraph` node format to the Neural API `GraphNode` schema.

use anyhow::Context;
use std::collections::HashMap;

use super::{Constraints, Graph, GraphNode, Operation, PrimalSelector};

impl Graph {
    /// Convert a `[[graph.nodes]]` (`DeploymentGraph`) node into the `neural_graph` `GraphNode` schema.
    ///
    /// Accepts both biomeOS-native fields (`id`, `capability`, `config.primal`) and
    /// primalSpring cell-graph fields (`name`, `binary`, `by_capability`, `order`,
    /// `security_model`), mapping the latter into the canonical format.
    pub(super) fn convert_deployment_node(node_value: &toml::Value) -> anyhow::Result<GraphNode> {
        let table = node_value.as_table().context("Node must be a TOML table")?;

        let id = table
            .get("id")
            .or_else(|| table.get("name"))
            .and_then(|v| v.as_str())
            .context("Node missing 'id' (or 'name')")?
            .to_string();

        let capability = table
            .get("capability")
            .or_else(|| table.get("by_capability"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let depends_on: Vec<String> = table
            .get("depends_on")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let budget_ms: Option<u64> = table.get("budget_ms").and_then(|v| {
            let f = v.as_float().or_else(|| {
                v.as_integer().map(|i| {
                    // budget_ms values are small (1-16 ms), no precision loss
                    i as f64
                })
            })?;
            Some(f as u64)
        });

        let name = table
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let feedback_to = table
            .get("feedback_to")
            .and_then(|v| v.as_str())
            .map(String::from);

        let fallback = table
            .get("fallback")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Extract params from [graph.nodes.params]
        let params: HashMap<String, serde_json::Value> = table
            .get("params")
            .and_then(|v| v.as_table())
            .map(|t| {
                t.iter()
                    .filter_map(|(k, v)| toml_value_to_json(v).map(|jv| (k.clone(), jv)))
                    .collect()
            })
            .unwrap_or_default();

        // Extract primal hint from [graph.nodes.config] or top-level `binary`
        let primal_name = table
            .get("config")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("primal"))
            .and_then(|v| v.as_str())
            .or_else(|| table.get("binary").and_then(|v| v.as_str()))
            .map(String::from);

        let primal_selector = if table
            .get("by_capability")
            .and_then(|v| v.as_str())
            .is_some()
        {
            Some(PrimalSelector {
                by_capability: Some(capability.clone()),
                by_name: primal_name.clone(),
            })
        } else if primal_name.is_some() {
            Some(PrimalSelector {
                by_capability: None,
                by_name: primal_name.clone(),
            })
        } else {
            None
        };

        let operation = if capability.is_empty() {
            None
        } else {
            Some(Operation {
                name: "capability_call".to_string(),
                target: None,
                params: {
                    let mut p = HashMap::new();
                    p.insert(
                        "capability".to_string(),
                        serde_json::Value::String(capability.clone()),
                    );
                    for (k, v) in &params {
                        p.insert(k.clone(), v.clone());
                    }
                    p
                },
                environment: None,
            })
        };

        let constraints = budget_ms.map(|ms| Constraints {
            timeout_ms: Some(ms),
            retry: None,
        });

        let mut config = HashMap::new();
        if let Some(ft) = feedback_to {
            config.insert("feedback_to".to_string(), serde_json::Value::String(ft));
        }
        if let Some(ref pn) = primal_name {
            config.insert("primal".to_string(), serde_json::Value::String(pn.clone()));
        }
        if !name.is_empty() {
            config.insert("name".to_string(), serde_json::Value::String(name));
        }
        if !capability.is_empty() {
            config.insert(
                "capability".to_string(),
                serde_json::Value::String(capability.clone()),
            );
        }
        if !params.is_empty() {
            config.insert("params".to_string(), serde_json::json!(params));
        }
        if let Some(security_model) = table.get("security_model").and_then(|v| v.as_str()) {
            config.insert(
                "security_model".to_string(),
                serde_json::Value::String(security_model.to_string()),
            );
        }

        let mut capabilities = if capability.is_empty() {
            vec![]
        } else {
            vec![capability]
        };

        // Merge capabilities from node-level array (primalSpring v2.0 format)
        if let Some(caps_array) = table.get("capabilities").and_then(|v| v.as_array()) {
            for cap_val in caps_array {
                if let Some(cap_str) = cap_val.as_str() {
                    let s = cap_str.to_string();
                    if !capabilities.contains(&s) {
                        capabilities.push(s);
                    }
                }
            }
        }

        let cost_estimate_ms = table
            .get("cost_estimate_ms")
            .and_then(|v| v.as_integer())
            .and_then(|v| u64::try_from(v).ok());

        let operation_dependencies: Vec<String> = table
            .get("operation_dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let gate = table.get("gate").and_then(|v| v.as_str()).map(String::from);

        Ok(GraphNode {
            id,
            primal: primal_selector,
            output: None,
            operation,
            constraints,
            depends_on,
            capabilities,
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config,
            outputs: vec![],
            fallback,
            cost_estimate_ms,
            operation_dependencies,
            gate,
        })
    }
}

/// Convert a TOML value to a `serde_json` Value.
fn toml_value_to_json(v: &toml::Value) -> Option<serde_json::Value> {
    match v {
        toml::Value::String(s) => Some(serde_json::Value::String(s.clone())),
        toml::Value::Integer(i) => Some(serde_json::json!(i)),
        toml::Value::Float(f) => Some(serde_json::json!(f)),
        toml::Value::Boolean(b) => Some(serde_json::Value::Bool(*b)),
        toml::Value::Array(arr) => {
            let items: Vec<_> = arr.iter().filter_map(toml_value_to_json).collect();
            Some(serde_json::Value::Array(items))
        }
        toml::Value::Table(t) => {
            let map: serde_json::Map<String, serde_json::Value> = t
                .iter()
                .filter_map(|(k, v)| toml_value_to_json(v).map(|jv| (k.clone(), jv)))
                .collect();
            Some(serde_json::Value::Object(map))
        }
        toml::Value::Datetime(dt) => Some(serde_json::Value::String(dt.to_string())),
    }
}
