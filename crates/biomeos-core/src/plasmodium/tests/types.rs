// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::types::*;

#[test]
fn test_bond_type_display() {
    assert_eq!(BondType::Covalent.to_string(), "covalent");
    assert_eq!(BondType::Ionic.to_string(), "ionic");
    assert_eq!(BondType::Metallic.to_string(), "metallic");
    assert_eq!(BondType::Weak.to_string(), "weak");
}

#[test]
fn test_collective_capabilities_default_empty() {
    let c = CollectiveCapabilities {
        total_gpus: 0,
        gpus: vec![],
        total_ram_gb: 0,
        total_models: 0,
        models: vec![],
        capabilities: vec![],
    };
    assert!(c.capabilities.is_empty());
}

#[test]
fn test_gate_info_reachable_field() {
    let g = GateInfo {
        gate_id: "g".to_string(),
        address: "a".to_string(),
        is_local: false,
        primals: vec![],
        compute: ComputeInfo::default(),
        models: vec![],
        load: 0.0,
        reachable: false,
        bond_type: BondType::Covalent,
    };
    assert!(!g.reachable);
}

#[test]
fn test_primal_status_version_some() {
    let p = PrimalStatus {
        name: "x".to_string(),
        healthy: true,
        version: Some("2.0".to_string()),
    };
    assert_eq!(p.version.as_deref(), Some("2.0"));
}

#[test]
fn test_compute_info_default_ram() {
    let c = ComputeInfo::default();
    assert_eq!(c.ram_gb, 0);
    assert!(c.gpus.is_empty());
}

#[test]
fn test_model_availability_struct() {
    let m = ModelAvailability {
        model_id: "m".to_string(),
        size_bytes: 0,
        format: String::new(),
        gates: vec!["a".to_string()],
    };
    assert_eq!(m.gates.len(), 1);
}

#[test]
fn test_plasmodium_state_fields() {
    let s = PlasmodiumState {
        gates: vec![],
        snapshot_at: "t".to_string(),
        family_id: "fam".to_string(),
        collective: CollectiveCapabilities {
            total_gpus: 0,
            gpus: vec![],
            total_ram_gb: 0,
            total_models: 0,
            models: vec![],
            capabilities: vec![],
        },
    };
    assert_eq!(s.family_id, "fam");
}

#[test]
fn test_plasmodium_state_snapshot_rfc3339() {
    let s = PlasmodiumState {
        gates: vec![],
        snapshot_at: "2025-01-01T00:00:00+00:00".to_string(),
        family_id: "f".to_string(),
        collective: CollectiveCapabilities::default(),
    };
    assert!(s.snapshot_at.contains('T'));
}
