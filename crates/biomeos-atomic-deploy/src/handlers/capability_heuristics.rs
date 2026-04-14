// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability heuristics for cost estimation, dependency graphs, and metadata.
//!
//! These are pure functions used by the `CapabilityHandler` to provide
//! operational metadata about capabilities. They are extracted here to
//! keep the handler module focused on request routing.

use serde_json::{Value, json};

/// Heuristic latency estimate based on capability domain.
///
/// Returns estimated milliseconds. These are conservative defaults —
/// primals can override via `capability.register` metadata.
pub(crate) fn estimate_operation_latency(capability: &str, operation: &str) -> u64 {
    match capability {
        "compute" | "shader" => {
            if operation.contains("status") || operation.contains("cancel") {
                5
            } else {
                500
            }
        }
        "ai" | "ml" => 1000,
        "storage" | "dag" => 50,
        "crypto" | "security" => 10,
        "health" => 5,
        "network" | "relay" | "stun" | "punch" => 100,
        _ => 50,
    }
}

/// Whether a capability domain typically requires GPU resources.
pub(crate) fn operation_requires_gpu(capability: &str) -> bool {
    matches!(capability, "compute" | "shader" | "ai" | "ml")
}

/// Capability locality: "local" for same-host IPC, "mesh" for cross-node.
pub(crate) fn capability_locality(capability: &str) -> &'static str {
    match capability {
        "relay" | "stun" | "punch" | "peer" | "discovery" => "mesh",
        _ => "local",
    }
}

/// Build operation dependency DAG edges for a capability domain.
///
/// Returns `[{"from": "op_a", "to": "op_b"}]` meaning `op_a` must
/// complete before `op_b` can run.
pub(crate) fn build_operation_dependencies(capability: &str, operations: &[String]) -> Vec<Value> {
    let mut deps = Vec::new();

    let dependency_rules: &[(&str, &str)] = match capability {
        "compute" => &[("compile", "dispatch"), ("dispatch", "status")],
        "dag" => &[
            ("session.create", "session.merge"),
            ("session.create", "node.add"),
        ],
        "crypto" => &[("generate_key", "sign"), ("generate_key", "encrypt")],
        _ => &[],
    };

    for (from, to) in dependency_rules {
        let has_from = operations.iter().any(|o| o.ends_with(from));
        let has_to = operations.iter().any(|o| o.ends_with(to));
        if has_from && has_to {
            deps.push(json!({"from": from, "to": to}));
        }
    }

    deps
}
