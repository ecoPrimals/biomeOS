// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for `handlers/capability_heuristics.rs`.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::capability_heuristics::{
    build_operation_dependencies, capability_locality, estimate_operation_latency,
    operation_requires_gpu,
};

fn assert_dep_edge(dep: &serde_json::Value, from: &str, to: &str) {
    assert_eq!(
        dep.get("from").and_then(serde_json::Value::as_str).unwrap(),
        from
    );
    assert_eq!(
        dep.get("to").and_then(serde_json::Value::as_str).unwrap(),
        to
    );
}

fn assert_deps(deps: &[serde_json::Value], edges: &[(&str, &str)]) {
    assert_eq!(deps.len(), edges.len(), "unexpected dependency count");
    for (dep, (from, to)) in deps.iter().zip(edges) {
        assert_dep_edge(dep, from, to);
    }
}

#[test]
fn estimate_latency_compute_and_shader_fast_for_status_and_cancel() {
    for capability in ["compute", "shader"] {
        assert_eq!(estimate_operation_latency(capability, "status"), 5);
        assert_eq!(estimate_operation_latency(capability, "job.status"), 5);
        assert_eq!(estimate_operation_latency(capability, "cancel"), 5);
        assert_eq!(estimate_operation_latency(capability, "request.cancel"), 5);
    }
}

#[test]
fn estimate_latency_compute_and_shader_slow_for_other_operations() {
    for capability in ["compute", "shader"] {
        assert_eq!(estimate_operation_latency(capability, "dispatch"), 500);
        assert_eq!(estimate_operation_latency(capability, "compile"), 500);
    }
}

#[test]
fn estimate_latency_ai_and_ml() {
    for capability in ["ai", "ml"] {
        assert_eq!(estimate_operation_latency(capability, "infer"), 1000);
        assert_eq!(estimate_operation_latency(capability, "status"), 1000);
    }
}

#[test]
fn estimate_latency_storage_and_dag() {
    for capability in ["storage", "dag"] {
        assert_eq!(estimate_operation_latency(capability, "read"), 50);
    }
}

#[test]
fn estimate_latency_crypto_and_security() {
    for capability in ["crypto", "security"] {
        assert_eq!(estimate_operation_latency(capability, "sign"), 10);
    }
}

#[test]
fn estimate_latency_health() {
    assert_eq!(estimate_operation_latency("health", "check"), 5);
}

#[test]
fn estimate_latency_network_family() {
    for capability in ["network", "relay", "stun", "punch"] {
        assert_eq!(estimate_operation_latency(capability, "connect"), 100);
    }
}

#[test]
fn estimate_latency_unknown_capability_defaults_to_fifty() {
    assert_eq!(estimate_operation_latency("unknown", "anything"), 50);
    assert_eq!(estimate_operation_latency("", "noop"), 50);
}

#[test]
fn operation_requires_gpu_for_compute_domains() {
    for capability in ["compute", "shader", "ai", "ml"] {
        assert!(operation_requires_gpu(capability));
    }
}

#[test]
fn operation_does_not_require_gpu_for_other_domains() {
    for capability in [
        "storage", "dag", "crypto", "security", "health", "network", "relay", "unknown",
    ] {
        assert!(!operation_requires_gpu(capability));
    }
}

#[test]
fn capability_locality_mesh_for_cross_node_domains() {
    for capability in ["relay", "stun", "punch", "peer", "discovery"] {
        assert_eq!(capability_locality(capability), "mesh");
    }
}

#[test]
fn capability_locality_local_for_same_host_domains() {
    for capability in [
        "compute", "shader", "ai", "ml", "storage", "dag", "crypto", "security", "health",
        "network", "unknown",
    ] {
        assert_eq!(capability_locality(capability), "local");
    }
}

#[test]
fn build_operation_dependencies_compute_full_graph() {
    let operations = vec![
        "compute.compile".to_string(),
        "compute.dispatch".to_string(),
        "compute.status".to_string(),
    ];

    let deps = build_operation_dependencies("compute", &operations);

    assert_deps(&deps, &[("compile", "dispatch"), ("dispatch", "status")]);
}

#[test]
fn build_operation_dependencies_compute_partial_operations() {
    let only_compile = vec!["compute.compile".to_string()];
    assert!(build_operation_dependencies("compute", &only_compile).is_empty());

    let compile_and_dispatch = vec![
        "compute.compile".to_string(),
        "compute.dispatch".to_string(),
    ];
    let deps = build_operation_dependencies("compute", &compile_and_dispatch);
    assert_dep_edge(
        deps.first().expect("single dependency edge"),
        "compile",
        "dispatch",
    );
}

#[test]
fn build_operation_dependencies_dag_full_graph() {
    let operations = vec![
        "dag.session.create".to_string(),
        "dag.session.merge".to_string(),
        "dag.node.add".to_string(),
    ];

    let deps = build_operation_dependencies("dag", &operations);

    assert_deps(
        &deps,
        &[
            ("session.create", "session.merge"),
            ("session.create", "node.add"),
        ],
    );
}

#[test]
fn build_operation_dependencies_crypto_full_graph() {
    let operations = vec![
        "crypto.generate_key".to_string(),
        "crypto.sign".to_string(),
        "crypto.encrypt".to_string(),
    ];

    let deps = build_operation_dependencies("crypto", &operations);

    assert_deps(
        &deps,
        &[("generate_key", "sign"), ("generate_key", "encrypt")],
    );
}

#[test]
fn build_operation_dependencies_unknown_capability_has_no_rules() {
    let operations = vec![
        "foo.compile".to_string(),
        "foo.dispatch".to_string(),
        "foo.status".to_string(),
    ];

    assert!(build_operation_dependencies("storage", &operations).is_empty());
    assert!(build_operation_dependencies("unknown", &operations).is_empty());
}

#[test]
fn build_operation_dependencies_empty_operations() {
    assert!(build_operation_dependencies("compute", &[]).is_empty());
    assert!(build_operation_dependencies("dag", &[]).is_empty());
    assert!(build_operation_dependencies("crypto", &[]).is_empty());
}

#[test]
fn build_operation_dependencies_matches_operations_ending_with_rule_suffix() {
    let operations = vec!["gpu.compile".to_string(), "gpu.dispatch".to_string()];

    let deps = build_operation_dependencies("compute", &operations);
    assert_dep_edge(
        deps.first().expect("single dependency edge"),
        "compile",
        "dispatch",
    );
}

#[test]
fn build_operation_dependencies_does_not_match_embedded_suffix() {
    let operations = vec![
        "prefix.compile.suffix".to_string(),
        "other.dispatch".to_string(),
    ];

    assert!(build_operation_dependencies("compute", &operations).is_empty());
}
