// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Fusion configuration for chimeras
//!
//! Defines how primal components are fused together into a unified chimera.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Complete fusion configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fusion {
    /// Named fusion bindings (how components connect)
    #[serde(default)]
    pub bindings: HashMap<String, FusionBinding>,

    /// Unified API surface
    #[serde(default)]
    pub api: FusionApi,
}

/// A binding between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionBinding {
    /// Provider component.module (e.g., "security.btsp")
    #[serde(default)]
    pub provider: Option<String>,

    /// Consumer component.modules (e.g., `["network.mesh", "network.birdsong"]`)
    #[serde(default)]
    pub consumers: Vec<String>,

    /// Binding-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Unified API specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FusionApi {
    /// API endpoints exposed by the chimera
    #[serde(default)]
    pub endpoints: Vec<FusionEndpoint>,
}

/// An endpoint in the unified API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionEndpoint {
    /// Endpoint name (e.g., "connect")
    pub name: String,

    /// Description of what this endpoint does
    #[serde(default)]
    pub description: String,

    /// Parameter names
    #[serde(default)]
    pub params: Vec<String>,

    /// Return type description
    #[serde(default)]
    pub returns: String,

    /// JSON-RPC capability to forward to (e.g., "network.connect")
    ///
    /// When set, codegen emits IPC forwarding via `capability.call` instead of
    /// a stub error. This is the primal-native pattern: chimeras route to
    /// capabilities discovered at runtime rather than containing business logic.
    #[serde(default)]
    pub capability: Option<String>,
}

impl Fusion {
    /// Create an empty fusion configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a binding
    #[must_use]
    pub fn with_binding(mut self, name: impl Into<String>, binding: FusionBinding) -> Self {
        self.bindings.insert(name.into(), binding);
        self
    }

    /// Add an API endpoint
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: FusionEndpoint) -> Self {
        self.api.endpoints.push(endpoint);
        self
    }

    /// Get all provider references
    #[must_use]
    pub fn providers(&self) -> Vec<&str> {
        self.bindings
            .values()
            .filter_map(|b| b.provider.as_deref())
            .collect()
    }

    /// Get all consumer references
    #[must_use]
    pub fn consumers(&self) -> Vec<&str> {
        self.bindings
            .values()
            .flat_map(|b| b.consumers.iter().map(String::as_str))
            .collect()
    }

    /// Validate that all references exist
    ///
    /// # Errors
    ///
    /// Returns an error if any binding references a primal that is not in the `available_primals` list
    pub fn validate_references(&self, available_primals: &[&str]) -> Result<(), String> {
        for (name, binding) in &self.bindings {
            // Check provider
            if let Some(ref provider) = binding.provider {
                let primal = provider.split('.').next().unwrap_or("");
                let primal = primal.trim_end_matches("[]");
                if !available_primals.contains(&primal) {
                    return Err(format!(
                        "Binding '{name}' references unknown primal '{primal}' in provider"
                    ));
                }
            }

            // Check consumers
            for consumer in &binding.consumers {
                let primal = consumer.split('.').next().unwrap_or("");
                let primal = primal.trim_end_matches("[]");
                if !available_primals.contains(&primal) {
                    return Err(format!(
                        "Binding '{name}' references unknown primal '{primal}' in consumer"
                    ));
                }
            }
        }
        Ok(())
    }
}

impl FusionBinding {
    /// Create a new binding
    #[must_use]
    pub fn new() -> Self {
        Self {
            provider: None,
            consumers: Vec::new(),
            config: HashMap::new(),
        }
    }

    /// Set the provider
    #[must_use]
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Add consumers
    #[must_use]
    pub fn with_consumers(mut self, consumers: Vec<String>) -> Self {
        self.consumers = consumers;
        self
    }

    /// Add configuration
    #[must_use]
    pub fn with_config(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }
}

impl Default for FusionBinding {
    fn default() -> Self {
        Self::new()
    }
}

impl FusionEndpoint {
    /// Create a new endpoint
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            params: Vec::new(),
            returns: String::new(),
            capability: None,
        }
    }

    /// Add description
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add parameters
    #[must_use]
    pub fn with_params(mut self, params: Vec<String>) -> Self {
        self.params = params;
        self
    }

    /// Set return type
    #[must_use]
    pub fn with_returns(mut self, returns: impl Into<String>) -> Self {
        self.returns = returns.into();
        self
    }

    /// Map to a JSON-RPC capability for IPC forwarding
    #[must_use]
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capability = Some(capability.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_builder() {
        let fusion = Fusion::new()
            .with_binding(
                "encryption_layer",
                FusionBinding::new()
                    .with_provider("security.btsp")
                    .with_consumers(vec!["network.mesh".into()]),
            )
            .with_endpoint(
                FusionEndpoint::new("connect")
                    .with_description("Connect to peer")
                    .with_params(vec!["peer_did".into()])
                    .with_returns("SecureChannel"),
            );

        assert_eq!(fusion.providers(), vec!["security.btsp"]);
        assert_eq!(fusion.api.endpoints.len(), 1);
    }

    #[test]
    fn test_validate_references() {
        let fusion =
            Fusion::new().with_binding("test", FusionBinding::new().with_provider("security.btsp"));

        assert!(fusion.validate_references(&["security", "network"]).is_ok());
        assert!(fusion.validate_references(&["network"]).is_err());
    }

    #[test]
    fn test_fusion_default() {
        let fusion = Fusion::default();
        assert!(fusion.bindings.is_empty());
        assert!(fusion.api.endpoints.is_empty());
        assert!(fusion.providers().is_empty());
        assert!(fusion.consumers().is_empty());
    }

    #[test]
    fn test_fusion_validate_consumer_reference() {
        let fusion = Fusion::new().with_binding(
            "test",
            FusionBinding::new()
                .with_provider("security.btsp")
                .with_consumers(vec!["unknown.mesh".into()]),
        );
        assert!(fusion.validate_references(&["security"]).is_err());
        assert!(fusion.validate_references(&["security", "unknown"]).is_ok());
    }

    #[test]
    fn test_fusion_validate_array_consumer() {
        let fusion = Fusion::new().with_binding(
            "test",
            FusionBinding::new()
                .with_provider("security.btsp")
                .with_consumers(vec!["network[].mesh".into()]),
        );
        assert!(fusion.validate_references(&["security", "network"]).is_ok());
    }

    #[test]
    fn test_fusion_binding_default() {
        let binding = FusionBinding::default();
        assert!(binding.provider.is_none());
        assert!(binding.consumers.is_empty());
        assert!(binding.config.is_empty());
    }

    #[test]
    fn test_fusion_binding_with_config() {
        let binding = FusionBinding::new()
            .with_provider("security.btsp")
            .with_config("timeout", serde_json::json!(30));
        assert_eq!(binding.config.get("timeout"), Some(&serde_json::json!(30)));
    }

    #[test]
    fn test_fusion_endpoint_builder() {
        let endpoint = FusionEndpoint::new("get_status")
            .with_description("Get component status")
            .with_params(vec!["component_id".into()])
            .with_returns("Status");
        assert_eq!(endpoint.name, "get_status");
        assert_eq!(endpoint.description, "Get component status");
        assert_eq!(endpoint.params, vec!["component_id"]);
        assert_eq!(endpoint.returns, "Status");
    }

    #[test]
    fn test_fusion_consumers() {
        let fusion = Fusion::new()
            .with_binding(
                "b1",
                FusionBinding::new().with_consumers(vec!["a.mod".into(), "b.mod".into()]),
            )
            .with_binding(
                "b2",
                FusionBinding::new().with_consumers(vec!["c.mod".into()]),
            );
        let consumers = fusion.consumers();
        assert_eq!(consumers.len(), 3);
        assert!(consumers.contains(&"a.mod"));
        assert!(consumers.contains(&"b.mod"));
        assert!(consumers.contains(&"c.mod"));
    }

    #[test]
    fn test_validate_references_empty_binding() {
        let fusion = Fusion::new().with_binding(
            "empty",
            FusionBinding::new(), // no provider, no consumers
        );
        assert!(fusion.validate_references(&["security"]).is_ok());
    }
}
