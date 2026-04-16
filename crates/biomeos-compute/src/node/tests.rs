// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::*;

#[test]
fn node_topology_serde_roundtrip() {
    for topo in [
        NodeTopology::Leaf,
        NodeTopology::BinaryTree,
        NodeTopology::NAryTree {
            branching_factor: 4,
        },
        NodeTopology::QuadTree,
        NodeTopology::Hybrid,
    ] {
        let json = serde_json::to_string(&topo).unwrap();
        let restored: NodeTopology = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{topo:?}"), format!("{restored:?}"));
    }
}

#[test]
fn workload_id_new_and_display() {
    let id = WorkloadId::new();
    assert!(!id.0.is_nil());
    let s = id.to_string();
    assert!(!s.is_empty());
    assert!(s.len() >= 32);
}

#[test]
fn workload_id_default() {
    let id = WorkloadId::default();
    assert!(!id.0.is_nil());
}

#[test]
fn resource_requirements_default() {
    let req = ResourceRequirements::default();
    assert_eq!(req.cpu_cores, Some(1));
    assert_eq!(req.memory_mb, Some(256));
    assert!(req.gpu_memory_mb.is_none());
}

#[test]
fn resource_info_aggregate() {
    let mut a = ResourceInfo {
        cpu_cores: 4,
        memory_mb: 1024,
        gpu_count: 0,
        gpu_memory_mb: 0,
        disk_mb: 100,
    };
    let b = ResourceInfo {
        cpu_cores: 2,
        memory_mb: 512,
        gpu_count: 1,
        gpu_memory_mb: 4096,
        disk_mb: 50,
    };
    a.aggregate(&b);
    assert_eq!(a.cpu_cores, 6);
    assert_eq!(a.memory_mb, 1536);
    assert_eq!(a.gpu_count, 1);
    assert_eq!(a.gpu_memory_mb, 4096);
    assert_eq!(a.disk_mb, 150);
}

#[test]
fn workload_priority_ordering() {
    assert!(WorkloadPriority::Critical > WorkloadPriority::High);
    assert!(WorkloadPriority::High > WorkloadPriority::Normal);
    assert!(WorkloadPriority::Normal > WorkloadPriority::Low);
}

#[test]
fn workload_status_serde_roundtrip() {
    for status in [
        WorkloadStatus::Queued,
        WorkloadStatus::Running,
        WorkloadStatus::Completed,
        WorkloadStatus::Failed {
            error: "test".to_string(),
        },
        WorkloadStatus::Cancelled,
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let restored: WorkloadStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{status:?}"), format!("{restored:?}"));
    }
}

#[test]
fn workload_new() {
    let w = Workload::new("test", Runtime::Wasm);
    assert_eq!(w.name, "test");
    assert_eq!(w.runtime, Runtime::Wasm);
    assert!(w.code.is_empty());
    assert!(!w.parallelizable);
    assert_eq!(w.priority, WorkloadPriority::Normal);
}

#[test]
fn runtime_serde_roundtrip() {
    for r in [
        Runtime::Native,
        Runtime::Wasm,
        Runtime::Container,
        Runtime::Python,
        Runtime::Gpu,
    ] {
        let json = serde_json::to_string(&r).unwrap();
        let restored: Runtime = serde_json::from_str(&json).unwrap();
        assert_eq!(r, restored);
    }
}

#[test]
fn resource_type_variants() {
    for t in [
        ResourceType::Cpu,
        ResourceType::Gpu,
        ResourceType::Memory,
        ResourceType::Hybrid,
    ] {
        let _ = format!("{t:?}");
    }
}

#[test]
fn health_status_serde_roundtrip() {
    for status in [
        HealthStatus::Healthy,
        HealthStatus::Degraded {
            reason: "load".to_string(),
        },
        HealthStatus::Unhealthy {
            error: "crash".to_string(),
        },
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let restored: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{status:?}"), format!("{restored:?}"));
    }
}

#[test]
fn workload_builder_fluent() {
    let w = Workload::builder("test", Runtime::Native)
        .cpu_cores(4)
        .memory_mb(512)
        .priority(WorkloadPriority::High)
        .parallelizable(true)
        .build();
    assert_eq!(w.name, "test");
    assert_eq!(w.runtime, Runtime::Native);
    assert_eq!(w.resource_requirements.cpu_cores, Some(4));
    assert_eq!(w.resource_requirements.memory_mb, Some(512));
    assert_eq!(w.priority, WorkloadPriority::High);
    assert!(w.parallelizable);
}
