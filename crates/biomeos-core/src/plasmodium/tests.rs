// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::Plasmodium;
use super::PlasmodiumEnvOverrides;
use super::system;
use super::types::*;

#[test]
fn test_bond_type_display() {
    assert_eq!(BondType::Covalent.to_string(), "covalent");
    assert_eq!(BondType::Ionic.to_string(), "ionic");
    assert_eq!(BondType::Metallic.to_string(), "metallic");
    assert_eq!(BondType::Weak.to_string(), "weak");
}

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
fn test_system_ram() {
    // Just verify it doesn't panic
    let ram = system::get_system_ram_gb();
    assert!(ram > 0);
}

#[test]
fn test_num_cpus() {
    assert!(system::num_cpus() > 0);
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
fn test_plasmodium_default() {
    let _p = Plasmodium::default();
    // Just verify construction succeeds
}

#[test]
fn test_plasmodium_new() {
    let _p = Plasmodium::new();
    // Just verify construction succeeds
}

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

#[tokio::test]
async fn test_query_collective_no_peers() {
    let p = Plasmodium::new();
    let result = p.query_collective().await;
    assert!(result.is_ok());
    let state = result.unwrap();
    assert!(!state.gates.is_empty(), "at least local gate");
    assert!(!state.family_id.is_empty());
    assert!(!state.snapshot_at.is_empty());
}

#[tokio::test]
async fn test_query_collective_merges_plasmodium_peers_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("peer-a@127.0.0.1:59997,peer-b@host-only".to_string()),
        ..Default::default()
    });
    let result = p.query_collective().await;
    assert!(result.is_ok());
    let state = result.unwrap();
    assert!(
        state
            .gates
            .iter()
            .any(|g| g.gate_id == "peer-a" || g.gate_id == "peer-b"),
        "expected env-listed peers to appear in collective state: {:?}",
        state.gates.iter().map(|g| &g.gate_id).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_plasmodium_peers_bare_hostname_branch() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("bare-hostname-only-token-unique-8821".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert!(
        state
            .gates
            .iter()
            .any(|g| g.gate_id == "bare-hostname-only-token-unique-8821"),
        "bare entry should use same token for id and address"
    );
}

#[tokio::test]
async fn test_discover_peers_dedupes_duplicate_node_ids_in_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("dup@127.0.0.1:1,dup@127.0.0.1:2".to_string()),
        ..Default::default()
    });
    let peers = p.discover_peers().await;
    assert_eq!(peers.iter().filter(|x| x.node_id == "dup").count(), 1);
    assert_eq!(
        peers.iter().find(|x| x.node_id == "dup").unwrap().address,
        "127.0.0.1:1"
    );
}

#[tokio::test]
async fn test_discover_peers_splitn_preserves_at_in_address() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("node@ssh:user@remote.host".to_string()),
        ..Default::default()
    });
    let peers = p.discover_peers().await;
    let peer = peers.iter().find(|x| x.node_id == "node").expect("peer");
    assert_eq!(peer.address, "ssh:user@remote.host");
}

#[tokio::test]
async fn test_plasmodium_peers_skips_empty_segments() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some(" ,  dup@127.0.0.1:1 , dup@127.0.0.1:2 ".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert!(
        state.gates.iter().filter(|g| g.gate_id == "dup").count() <= 1,
        "duplicate node ids from env should be deduped"
    );
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

#[tokio::test]
async fn test_query_collective_family_id_from_family_id_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        family_id: Some("plasmo-env-family-42".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert_eq!(state.family_id, "plasmo-env-family-42");
}

#[tokio::test]
async fn test_query_collective_node_family_id_fallback_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        node_family_id: Some("node-fam-99".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert_eq!(state.family_id, "node-fam-99");
}

#[tokio::test]
async fn test_query_collective_gate_id_from_gate_id_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        gate_id: Some("gate-env-unique-771".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    let local = state.gates.iter().find(|g| g.is_local).expect("local gate");
    assert_eq!(local.gate_id, "gate-env-unique-771");
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
