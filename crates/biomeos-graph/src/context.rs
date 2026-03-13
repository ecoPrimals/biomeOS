// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// =============================================================================
// Execution Context - Runtime State Management
// =============================================================================
//
// Modern idiomatic Rust context:
// - Thread-safe (Arc + RwLock)
// - No global state
// - Clear ownership
//
// =============================================================================

use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Context for graph execution
///
/// Holds runtime state like:
/// - Output variables from previous nodes
/// - Discovered primals (capability-based)
/// - Execution metrics
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    inner: Arc<RwLock<ExecutionContextInner>>,
}

#[derive(Debug)]
struct ExecutionContextInner {
    /// Output variables from nodes
    outputs: HashMap<String, Value>,

    /// Discovered primals by capability
    primals_by_capability: HashMap<String, Vec<String>>,

    /// Primal info cache
    primal_info: HashMap<String, PrimalInfo>,
}

#[derive(Debug, Clone)]
pub struct PrimalInfo {
    pub id: String,
    pub capabilities: Vec<String>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExecutionContextInner {
                outputs: HashMap::new(),
                primals_by_capability: HashMap::new(),
                primal_info: HashMap::new(),
            })),
        }
    }

    /// Store output from a node
    pub fn set_output(&self, key: String, value: Value) {
        let mut inner = self
            .inner
            .write()
            .expect("execution context lock poisoned");
        inner.outputs.insert(key, value);
    }

    /// Get output from a previous node
    pub fn get_output(&self, key: &str) -> Option<Value> {
        let inner = self.inner.read().expect("execution context lock poisoned");
        inner.outputs.get(key).cloned()
    }

    /// Get all outputs
    pub fn get_all_outputs(&self) -> HashMap<String, Value> {
        let inner = self.inner.read().expect("execution context lock poisoned");
        inner.outputs.clone()
    }

    /// Register a discovered primal
    pub fn register_primal(&self, id: String, capabilities: Vec<String>) {
        let mut inner = self
            .inner
            .write()
            .expect("execution context lock poisoned");

        // Store primal info
        inner.primal_info.insert(
            id.clone(),
            PrimalInfo {
                id: id.clone(),
                capabilities: capabilities.clone(),
            },
        );

        // Index by capabilities
        for cap in capabilities {
            inner
                .primals_by_capability
                .entry(cap)
                .or_default()
                .push(id.clone());
        }
    }

    /// Find primals by capability (runtime discovery!)
    pub fn find_primal_by_capability(&self, capability: &str) -> Option<String> {
        let inner = self.inner.read().expect("execution context lock poisoned");
        inner
            .primals_by_capability
            .get(capability)
            .and_then(|primals| primals.first())
            .cloned()
    }

    /// Find primals by multiple capabilities (all required)
    pub fn find_primal_by_capabilities(&self, capabilities: &[String]) -> Option<String> {
        let inner = self.inner.read().expect("execution context lock poisoned");

        // Find primals that have ALL capabilities
        for (primal_id, info) in &inner.primal_info {
            let has_all = capabilities
                .iter()
                .all(|req_cap| info.capabilities.contains(req_cap));

            if has_all {
                return Some(primal_id.clone());
            }
        }

        None
    }

    /// Get primal info
    pub fn get_primal_info(&self, id: &str) -> Option<PrimalInfo> {
        let inner = self.inner.read().expect("execution context lock poisoned");
        inner.primal_info.get(id).cloned()
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_output() {
        let ctx = ExecutionContext::new();
        ctx.set_output("key1".to_string(), Value::String("value1".to_string()));

        let value = ctx.get_output("key1");
        assert_eq!(value, Some(Value::String("value1".to_string())));
    }

    #[test]
    fn test_register_and_find_primal() {
        let ctx = ExecutionContext::new();

        ctx.register_primal(
            "songbird-1".to_string(),
            vec!["discovery".to_string(), "tunneling".to_string()],
        );

        let found = ctx.find_primal_by_capability("discovery");
        assert_eq!(found, Some("songbird-1".to_string()));
    }

    #[test]
    fn test_find_by_multiple_capabilities() {
        let ctx = ExecutionContext::new();

        ctx.register_primal("primal-1".to_string(), vec!["discovery".to_string()]);

        ctx.register_primal(
            "primal-2".to_string(),
            vec!["discovery".to_string(), "encryption".to_string()],
        );

        let found =
            ctx.find_primal_by_capabilities(&["discovery".to_string(), "encryption".to_string()]);

        assert_eq!(found, Some("primal-2".to_string()));
    }
}
