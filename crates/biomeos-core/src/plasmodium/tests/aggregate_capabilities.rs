// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::Plasmodium;
use super::super::types::*;

#[test]
fn test_aggregate_capabilities_sorted() {
    let gates = vec![
        GateInfo {
            gate_id: "z-gate".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![PrimalStatus {
                name: "beardog".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "a-gate".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![PrimalStatus {
                name: "songbird".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
    ];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert!(!caps.capabilities.is_empty());
    let mut sorted = caps.capabilities.clone();
    sorted.sort();
    assert_eq!(caps.capabilities, sorted, "capabilities should be sorted");
}

#[test]
fn test_aggregate_capabilities_unhealthy_primal_excluded() {
    let gates = vec![GateInfo {
        gate_id: "gate".to_string(),
        address: "local".to_string(),
        is_local: true,
        primals: vec![PrimalStatus {
            name: "beardog".to_string(),
            healthy: false,
            version: None,
        }],
        compute: ComputeInfo::default(),
        models: vec![],
        load: 0.0,
        reachable: true,
        bond_type: BondType::Covalent,
    }];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert!(
        caps.capabilities.is_empty(),
        "unhealthy primals don't contribute"
    );
}

#[test]
fn test_aggregate_capabilities_model_availability() {
    let gates = vec![GateInfo {
        gate_id: "gate1".to_string(),
        address: "local".to_string(),
        is_local: true,
        primals: vec![],
        compute: ComputeInfo::default(),
        models: vec!["model-a".to_string(), "model-b".to_string()],
        load: 0.0,
        reachable: true,
        bond_type: BondType::Covalent,
    }];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert_eq!(caps.total_models, 2);
    assert_eq!(caps.models.len(), 2);
}

#[test]
fn test_aggregate_capabilities_same_model_multiple_gates() {
    let gates = vec![
        GateInfo {
            gate_id: "gate1".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![],
            compute: ComputeInfo::default(),
            models: vec!["shared-model".to_string()],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "gate2".to_string(),
            address: "remote".to_string(),
            is_local: false,
            primals: vec![],
            compute: ComputeInfo::default(),
            models: vec!["shared-model".to_string()],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
    ];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert_eq!(caps.total_models, 1);
    assert_eq!(caps.models[0].gates.len(), 2);
}

#[test]
fn test_aggregate_capabilities_multiple_gates_same_primal_name() {
    let gates = vec![
        GateInfo {
            gate_id: "g1".to_string(),
            address: "l".to_string(),
            is_local: true,
            primals: vec![PrimalStatus {
                name: "beardog".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "g2".to_string(),
            address: "r".to_string(),
            is_local: false,
            primals: vec![PrimalStatus {
                name: "beardog".to_string(),
                healthy: true,
                version: Some("2".to_string()),
            }],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
    ];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert!(
        caps.capabilities.contains(&"crypto".to_string()),
        "beardog should map to at least crypto: {:?}",
        caps.capabilities
    );
}

#[test]
fn test_aggregate_capabilities_models_merge_duplicate_ids() {
    let gates = vec![
        GateInfo {
            gate_id: "a".to_string(),
            address: "l".to_string(),
            is_local: true,
            primals: vec![],
            compute: ComputeInfo::default(),
            models: vec!["m1".to_string()],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "b".to_string(),
            address: "r".to_string(),
            is_local: false,
            primals: vec![],
            compute: ComputeInfo::default(),
            models: vec!["m1".to_string()],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
    ];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert_eq!(caps.total_models, 1);
    assert_eq!(caps.models[0].gates.len(), 2);
}

#[test]
fn test_aggregate_capabilities_duplicate_capability_names_sorted() {
    let gates = vec![
        GateInfo {
            gate_id: "g1".to_string(),
            address: "l".to_string(),
            is_local: true,
            primals: vec![PrimalStatus {
                name: "beardog".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "g2".to_string(),
            address: "r".to_string(),
            is_local: false,
            primals: vec![PrimalStatus {
                name: "beardog".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Weak,
        },
    ];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    let mut sorted = caps.capabilities.clone();
    sorted.sort();
    assert_eq!(caps.capabilities, sorted);
}
