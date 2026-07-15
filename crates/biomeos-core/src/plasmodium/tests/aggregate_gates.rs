// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::Plasmodium;
use super::super::types::*;

#[test]
fn test_aggregate_empty() {
    let caps = Plasmodium::aggregate_capabilities(&[]);
    assert_eq!(caps.total_gpus, 0);
    assert_eq!(caps.total_ram_gb, 0);
    assert!(caps.models.is_empty());
}

#[test]
fn test_aggregate_with_gates() {
    let gates = vec![
        GateInfo {
            gate_id: "tower".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![
                PrimalStatus {
                    name: "beardog".to_string(),
                    healthy: true,
                    version: None,
                },
                PrimalStatus {
                    name: "songbird".to_string(),
                    healthy: true,
                    version: None,
                },
            ],
            compute: ComputeInfo {
                gpus: vec![GpuInfo {
                    name: "RTX 4070".to_string(),
                    vram_mb: 12288,
                    gate_id: "tower".to_string(),
                }],
                ram_gb: 32,
                cpu_cores: 16,
            },
            models: vec!["TinyLlama/1.1B".to_string()],
            load: 0.1,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "gate2".to_string(),
            address: "192.0.2.132".to_string(),
            is_local: false,
            primals: vec![PrimalStatus {
                name: "toadstool".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo {
                gpus: vec![GpuInfo {
                    name: "RTX 3090".to_string(),
                    vram_mb: 24576,
                    gate_id: "gate2".to_string(),
                }],
                ram_gb: 256,
                cpu_cores: 64,
            },
            models: vec!["TinyLlama/1.1B".to_string(), "Mistral-7B".to_string()],
            load: 0.05,
            reachable: true,
            bond_type: BondType::Covalent,
        },
    ];

    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert_eq!(caps.total_gpus, 2);
    assert_eq!(caps.total_ram_gb, 32 + 256);
    assert_eq!(caps.total_models, 2); // unique
    assert!(caps.capabilities.contains(&"crypto".to_string()));
    assert!(caps.capabilities.contains(&"compute".to_string()));
}

#[test]
fn test_aggregate_unreachable_gates_excluded() {
    let gates = vec![
        GateInfo {
            gate_id: "reachable".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![PrimalStatus {
                name: "beardog".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo {
                gpus: vec![],
                ram_gb: 16,
                cpu_cores: 8,
            },
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "unreachable".to_string(),
            address: "192.0.2.99".to_string(),
            is_local: false,
            primals: vec![],
            compute: ComputeInfo {
                gpus: vec![GpuInfo {
                    name: "RTX 4090".to_string(),
                    vram_mb: 24576,
                    gate_id: "unreachable".to_string(),
                }],
                ram_gb: 64,
                cpu_cores: 32,
            },
            models: vec!["BigModel".to_string()],
            load: 0.0,
            reachable: false,
            bond_type: BondType::Covalent,
        },
    ];

    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert_eq!(caps.total_gpus, 0, "unreachable gate GPUs excluded");
    assert_eq!(caps.total_ram_gb, 16, "only reachable gate RAM");
    assert_eq!(caps.models.len(), 0, "unreachable models excluded");
}

#[test]
fn test_aggregate_ionic_bond_type() {
    let gates = vec![GateInfo {
        gate_id: "ionic".to_string(),
        address: "local".to_string(),
        is_local: true,
        primals: vec![],
        compute: ComputeInfo::default(),
        models: vec![],
        load: 0.0,
        reachable: true,
        bond_type: BondType::Ionic,
    }];

    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert!(caps.capabilities.is_empty());
}

#[test]
fn test_aggregate_gpu_dedup_by_gate() {
    let gates = vec![
        GateInfo {
            gate_id: "g1".to_string(),
            address: "l".to_string(),
            is_local: true,
            primals: vec![],
            compute: ComputeInfo {
                gpus: vec![GpuInfo {
                    name: "GPU".to_string(),
                    vram_mb: 1000,
                    gate_id: "g1".to_string(),
                }],
                ram_gb: 8,
                cpu_cores: 4,
            },
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
        GateInfo {
            gate_id: "g2".to_string(),
            address: "r".to_string(),
            is_local: false,
            primals: vec![],
            compute: ComputeInfo {
                gpus: vec![GpuInfo {
                    name: "GPU2".to_string(),
                    vram_mb: 2000,
                    gate_id: "g2".to_string(),
                }],
                ram_gb: 16,
                cpu_cores: 8,
            },
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        },
    ];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert_eq!(caps.total_gpus, 2);
    assert_eq!(caps.gpus.len(), 2);
}

#[test]
fn test_aggregate_capabilities_zero_ram_multiple_gpus() {
    let gates = vec![GateInfo {
        gate_id: "gpu-only".to_string(),
        address: "l".to_string(),
        is_local: true,
        primals: vec![],
        compute: ComputeInfo {
            gpus: vec![
                GpuInfo {
                    name: "A".to_string(),
                    vram_mb: 1000,
                    gate_id: "gpu-only".to_string(),
                },
                GpuInfo {
                    name: "B".to_string(),
                    vram_mb: 2000,
                    gate_id: "gpu-only".to_string(),
                },
            ],
            ram_gb: 0,
            cpu_cores: 0,
        },
        models: vec![],
        load: 0.0,
        reachable: true,
        bond_type: BondType::Metallic,
    }];
    let caps = Plasmodium::aggregate_capabilities(&gates);
    assert_eq!(caps.total_gpus, 2);
    assert_eq!(caps.total_ram_gb, 0);
}
