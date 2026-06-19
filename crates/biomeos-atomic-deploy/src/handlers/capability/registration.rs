// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability, gate, route, and method registration handlers.

use super::CapabilityHandler;
use anyhow::{Context, Result};
use biomeos_core::TransportEndpoint;
use serde_json::{Value, json};
use std::path::PathBuf;
use tracing::{debug, info};

impl CapabilityHandler {
    /// Register a capability for a primal.
    ///
    /// JSON-RPC method: `capability.register`
    ///
    /// # Parameters
    /// - `capability`: Capability name
    /// - `primal`: Primal name
    /// - `socket`: Unix socket path
    /// - `source`: Registration source (optional)
    /// - `semantic_mappings`: Optional semantic operation mappings
    pub async fn register(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        let primal_name = params["primal"]
            .as_str()
            .context("Missing 'primal' field")?;
        let socket_path = params["socket"]
            .as_str()
            .context("Missing 'socket' field")?;
        let source = params["source"].as_str().unwrap_or("manual");

        info!(
            "📝 Registering: {} → {} (from {})",
            capability, primal_name, source
        );

        // Register via transport-aware endpoint (parse or default to Unix socket)
        let endpoint = TransportEndpoint::parse(socket_path).unwrap_or_else(|| {
            TransportEndpoint::UnixSocket {
                path: PathBuf::from(socket_path),
            }
        });

        self.router
            .register_capability(capability, primal_name, endpoint, source)
            .await?;

        // Register semantic mappings if provided
        if let Some(semantic_mappings) = params.get("semantic_mappings") {
            self.register_semantic_mappings(
                capability,
                primal_name,
                semantic_mappings,
                socket_path,
            )
            .await?;
        }

        Ok(json!({
            "success": true,
            "capability": capability,
            "primal": primal_name,
            "socket": socket_path
        }))
    }

    /// Batch-register all capabilities for a remote primal in one call.
    ///
    /// JSON-RPC method: `route.register`
    ///
    /// # Parameters
    /// - `primal`: Primal name (e.g., "security-provider")
    /// - `transport`: Transport endpoint string (e.g., "<tcp://192.0.2.100:9001>")
    /// - `capabilities`: Array of capability names to register
    /// - `gate`: Gate label (optional, stored as source metadata)
    /// - `source`: Registration source (optional, defaults to "route.register")
    pub async fn register_route(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let primal_name = params["primal"]
            .as_str()
            .context("Missing 'primal' field")?;
        let transport_str = params["transport"]
            .as_str()
            .context("Missing 'transport' field")?;
        let capabilities = params["capabilities"]
            .as_array()
            .context("Missing or invalid 'capabilities' array")?;

        if capabilities.is_empty() {
            anyhow::bail!("'capabilities' array must not be empty");
        }

        let gate = params.get("gate").and_then(|v| v.as_str());
        let source = params
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("route.register");

        let endpoint = TransportEndpoint::parse(transport_str)
            .with_context(|| format!("Failed to parse transport endpoint: {transport_str}"))?;

        if let Some(gate_name) = gate {
            let mut reg = self.gate_registry.write().await;
            if reg.resolve(gate_name).is_none() {
                info!("🌉 gate.register (via route.register): {gate_name} → {transport_str}");
                reg.register(gate_name, endpoint.clone());
            }
        }

        let source_tag = match gate {
            Some(g) => format!("{source}@{g}"),
            None => source.to_owned(),
        };

        info!(
            "📝 route.register: {} capabilities for {} @ {}{}",
            capabilities.len(),
            primal_name,
            transport_str,
            gate.map(|g| format!(" (gate: {g})")).unwrap_or_default()
        );

        let mut registered = Vec::with_capacity(capabilities.len());
        for cap_value in capabilities {
            let cap = cap_value
                .as_str()
                .with_context(|| format!("Each capability must be a string, got: {cap_value}"))?;

            self.router
                .register_capability(cap, primal_name, endpoint.clone(), &source_tag)
                .await?;

            registered.push(cap);
        }

        Ok(json!({
            "registered": registered.len(),
            "primal": primal_name,
            "gate": gate,
            "endpoint": transport_str,
            "capabilities": registered
        }))
    }

    /// Register a remote gate endpoint for cross-gate capability.call forwarding.
    ///
    /// JSON-RPC method: `gate.register`
    ///
    /// # Parameters
    /// - `gate`: Gate label (e.g., "eastGate", "gate2")
    /// - `endpoint`: Transport endpoint string (e.g., "tcp://192.168.4.100:9001")
    pub async fn register_gate(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let gate_name = params["gate"].as_str().context("Missing 'gate' field")?;
        let transport_str = params["endpoint"]
            .as_str()
            .context("Missing 'endpoint' field")?;

        if gate_name == "local" {
            anyhow::bail!("Cannot register 'local' as a remote gate");
        }

        let endpoint = TransportEndpoint::parse(transport_str)
            .with_context(|| format!("Failed to parse endpoint: {transport_str}"))?;

        let mut reg = self.gate_registry.write().await;
        let was_new = reg.resolve(gate_name).is_none();
        reg.register(gate_name, endpoint);

        info!(
            "🌉 gate.register: {gate_name} → {transport_str} ({})",
            if was_new { "new" } else { "updated" }
        );

        Ok(json!({
            "gate": gate_name,
            "endpoint": transport_str,
            "status": if was_new { "registered" } else { "updated" },
            "total_gates": reg.len()
        }))
    }

    /// List all registered gates.
    ///
    /// JSON-RPC method: `gate.list`
    pub async fn list_gates(&self) -> Result<Value> {
        let reg = self.gate_registry.read().await;
        let gates: Vec<Value> = reg
            .iter()
            .map(|(name, ep)| {
                json!({
                    "gate": name,
                    "endpoint": ep.display_string()
                })
            })
            .collect();

        Ok(json!({
            "gates": gates,
            "total": gates.len()
        }))
    }

    /// Register spring-originated IPC methods into the semantic routing layer.
    ///
    /// JSON-RPC method: `method.register` (GAP-09)
    ///
    /// Springs call this to register their methods by name. The domain prefix
    /// (e.g. `game` from `game.start`) becomes the capability. Each unique
    /// domain is registered as a capability for the given transport endpoint,
    /// and each method is registered as a semantic mapping.
    ///
    /// # Parameters
    /// - `primal`: Primal/spring name
    /// - `transport`: Transport endpoint string (socket path or `tcp://host:port`)
    /// - `methods`: Array of method name strings (e.g. `["game.start", "game.join"]`)
    /// - `source`: Registration source (optional, defaults to `"method.register"`)
    pub async fn register_methods(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let primal_name = params["primal"]
            .as_str()
            .context("Missing 'primal' field")?;
        let transport_str = params["transport"]
            .as_str()
            .context("Missing 'transport' field")?;
        let methods = params["methods"]
            .as_array()
            .context("Missing or invalid 'methods' array")?;

        if methods.is_empty() {
            anyhow::bail!("'methods' array must not be empty");
        }

        let source = params
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("method.register");

        let endpoint = TransportEndpoint::parse(transport_str)
            .with_context(|| format!("Failed to parse transport endpoint: {transport_str}"))?;

        let mut domains: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for method_value in methods {
            let method = method_value
                .as_str()
                .with_context(|| format!("Each method must be a string, got: {method_value}"))?;

            if let Some((domain, operation)) = method.split_once('.') {
                if !domain.is_empty() && !operation.is_empty() {
                    domains
                        .entry(domain.to_owned())
                        .or_default()
                        .push(operation.to_owned());
                }
            }
        }

        if domains.is_empty() {
            anyhow::bail!("No valid 'domain.operation' methods found in array");
        }

        let mut registered_count = 0usize;
        let mut registered_domains = Vec::new();

        for (domain, operations) in &domains {
            self.router
                .register_capability(domain, primal_name, endpoint.clone(), source)
                .await?;

            let mut registry = self.translation_registry.write().await;
            for op in operations {
                let semantic_name = format!("{domain}.{op}");
                let actual_method = format!("{domain}.{op}");
                registry.register_translation(
                    &semantic_name,
                    primal_name,
                    &actual_method,
                    transport_str,
                    None,
                );
                registered_count += 1;
            }
            registered_domains.push(domain.as_str());
        }

        info!(
            "📝 method.register: {registered_count} methods across {} domains for {primal_name} @ {transport_str}",
            domains.len()
        );

        Ok(json!({
            "registered": registered_count,
            "domains": registered_domains,
            "primal": primal_name,
            "endpoint": transport_str,
        }))
    }

    /// Register semantic mappings for a capability.
    async fn register_semantic_mappings(
        &self,
        capability: &str,
        primal_name: &str,
        semantic_mappings: &Value,
        socket_path: &str,
    ) -> Result<()> {
        if let Some(mappings_obj) = semantic_mappings.as_object() {
            debug!(
                "   Registering {} semantic mappings for {}",
                mappings_obj.len(),
                capability
            );

            let mut registry = self.translation_registry.write().await;
            for (semantic_op, value) in mappings_obj {
                if let Some(actual_method) = value.as_str() {
                    let semantic_name = format!("{capability}.{semantic_op}");
                    registry.register_translation(
                        &semantic_name,
                        primal_name,
                        actual_method,
                        socket_path,
                        None,
                    );
                }
            }
        }
        Ok(())
    }
}
