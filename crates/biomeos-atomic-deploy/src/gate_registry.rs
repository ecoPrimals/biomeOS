// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Gate registry for cross-gate deployment.
//!
//! Maps gate names (e.g., "gate2", "pixel") to remote biomeOS Neural API
//! transport endpoints. Used by the graph executor to forward nodes marked
//! with `gate = "gate2"` to the correct remote biomeOS instance.

use biomeos_core::TransportEndpoint;
use std::collections::HashMap;

/// Registry mapping gate names to their biomeOS Neural API endpoints.
#[derive(Debug, Clone, Default)]
pub struct GateRegistry {
    gates: HashMap<String, TransportEndpoint>,
}

impl GateRegistry {
    /// Create an empty gate registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a gate with its biomeOS Neural API endpoint.
    pub fn register(&mut self, name: impl Into<String>, endpoint: TransportEndpoint) {
        self.gates.insert(name.into(), endpoint);
    }

    /// Resolve a gate name to its transport endpoint.
    ///
    /// Returns `None` for unknown gates and for `"local"` (which means
    /// execute on the current biomeOS instance).
    pub fn resolve(&self, gate: &str) -> Option<&TransportEndpoint> {
        if gate == "local" {
            return None;
        }
        self.gates.get(gate)
    }

    /// Whether a gate name refers to a remote biomeOS instance.
    pub fn is_remote(&self, gate: &str) -> bool {
        gate != "local" && self.gates.contains_key(gate)
    }

    /// Number of registered gates (excluding "local").
    pub fn len(&self) -> usize {
        self.gates.len()
    }

    /// Whether the registry has no gates.
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }

    /// Build a gate registry from a graph's `[graph.env]` section.
    ///
    /// Convention: env keys whose values parse as transport endpoints are
    /// treated as gate definitions. This keeps graphs self-contained — the
    /// same TOML file declares both the node topology and the gate endpoints.
    ///
    /// ```toml
    /// [graph.env]
    /// gate2 = "tcp://192.168.1.132:9001"
    /// pixel = "@biomeos-pixel"
    /// ```
    pub fn from_graph_env(env: &HashMap<String, String>) -> Self {
        let mut registry = Self::new();
        for (key, value) in env {
            if let Some(endpoint) = TransportEndpoint::parse(value) {
                tracing::info!(
                    "🌉 Gate registered from graph env: {} → {}",
                    key,
                    endpoint.display_string()
                );
                registry.register(key.clone(), endpoint);
            }
        }
        registry
    }

    /// Iterate over all registered gates.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &TransportEndpoint)> {
        self.gates.iter().map(|(k, v)| (k.as_str(), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let reg = GateRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(reg.resolve("gate2").is_none());
        assert!(!reg.is_remote("gate2"));
    }

    #[test]
    fn test_register_and_resolve() {
        let mut reg = GateRegistry::new();
        let ep = TransportEndpoint::parse("tcp://192.168.1.132:9001").unwrap();
        reg.register("gate2", ep);

        assert_eq!(reg.len(), 1);
        assert!(reg.is_remote("gate2"));

        let resolved = reg.resolve("gate2").unwrap();
        assert_eq!(resolved.display_string(), "tcp://192.168.1.132:9001");
    }

    #[test]
    fn test_local_always_resolves_to_none() {
        let mut reg = GateRegistry::new();
        let ep = TransportEndpoint::parse("tcp://127.0.0.1:9001").unwrap();
        reg.register("local", ep);

        assert!(
            reg.resolve("local").is_none(),
            "\"local\" should never resolve to remote"
        );
        assert!(!reg.is_remote("local"));
    }

    #[test]
    fn test_unknown_gate() {
        let reg = GateRegistry::new();
        assert!(reg.resolve("nonexistent").is_none());
        assert!(!reg.is_remote("nonexistent"));
    }

    #[test]
    fn test_from_graph_env() {
        let mut env = HashMap::new();
        env.insert("gate2".into(), "tcp://192.168.1.132:9001".into());
        env.insert("pixel".into(), "@biomeos-pixel".into());
        env.insert("RUST_LOG".into(), "info".into()); // not a transport — should be skipped

        let reg = GateRegistry::from_graph_env(&env);
        assert_eq!(reg.len(), 2);
        assert!(reg.is_remote("gate2"));
        assert!(reg.is_remote("pixel"));
        assert!(!reg.is_remote("RUST_LOG"));
    }

    #[test]
    fn test_from_graph_env_http() {
        let mut env = HashMap::new();
        env.insert("tower".into(), "http://10.0.0.1:8080".into());

        let reg = GateRegistry::from_graph_env(&env);
        assert_eq!(reg.len(), 1);
        let ep = reg.resolve("tower").unwrap();
        assert_eq!(ep.display_string(), "http://10.0.0.1:8080/jsonrpc");
    }

    #[test]
    fn test_iter() {
        let mut reg = GateRegistry::new();
        reg.register(
            "g1",
            TransportEndpoint::parse("tcp://10.0.0.1:9001").unwrap(),
        );
        reg.register(
            "g2",
            TransportEndpoint::parse("tcp://10.0.0.2:9001").unwrap(),
        );

        let names: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"g1"));
        assert!(names.contains(&"g2"));
    }
}
