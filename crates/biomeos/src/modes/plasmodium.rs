// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Plasmodium mode - Over-NUCLEUS collective coordination
//!
//! Provides a unified view of all covalently bonded NUCLEUS instances.
//! Named after the slime mold *Physarum polycephalum*.
//!
//! See `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md` for the full specification.

use anyhow::Result;
use biomeos_core::plasmodium::{Plasmodium, PlasmodiumState};
use std::collections::HashMap;

use crate::PlasmodiumCommand;

/// Build unified model-to-gates map from collective state (pure, testable)
pub(crate) fn build_model_gates_map(state: &PlasmodiumState) -> HashMap<String, Vec<String>> {
    let mut model_gates: HashMap<String, Vec<String>> = HashMap::new();

    for gate in &state.gates {
        for model in &gate.models {
            model_gates
                .entry(model.clone())
                .or_default()
                .push(gate.gate_id.clone());
        }
    }

    for model in &state.collective.models {
        model_gates.entry(model.model_id.clone()).or_default();
        for gate_id in &model.gates {
            let entry = model_gates.entry(model.model_id.clone()).or_default();
            if !entry.contains(gate_id) {
                entry.push(gate_id.clone());
            }
        }
    }

    model_gates
}

/// Pluralize suffix for counts (e.g. "gate" -> "gates" when count != 1)
pub(crate) fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// Format status table lines (pure, testable).
pub(crate) fn format_status_table(state: &PlasmodiumState) -> Vec<String> {
    let mut lines = Vec::new();
    let reachable_gates: Vec<_> = state.gates.iter().filter(|g| g.reachable).collect();

    lines.push(String::new());
    lines.push(format!("  Plasmodium Status - Family: {}", state.family_id));
    lines.push("  =========================================".to_string());
    lines.push(String::new());
    lines.push(format!(
        "  Collective: {} gate{} bonded (covalent)",
        reachable_gates.len(),
        plural_suffix(reachable_gates.len())
    ));
    lines.push(String::new());
    lines.push(format!(
        "  {:<18} {:<12} {:>5} {:>8} {:>8} {:>7}",
        "GATE", "PRIMALS", "GPUs", "RAM", "LOAD", "MODELS"
    ));
    lines.push(format!("  {}", "-".repeat(65)));

    for gate in &state.gates {
        let primal_status = if gate.reachable {
            let healthy = gate.primals.iter().filter(|p| p.healthy).count();
            let total = gate.primals.len();
            format!("{healthy}/{total}")
        } else {
            "offline".to_string()
        };

        let gate_name = if gate.is_local {
            format!("{} (local)", gate.gate_id)
        } else {
            gate.gate_id.clone()
        };

        let gpu_count = gate.compute.gpus.len();
        let ram = if gate.compute.ram_gb > 0 {
            format!("{} GB", gate.compute.ram_gb)
        } else {
            "-".to_string()
        };
        let load = if gate.reachable {
            format!("{:.0}%", gate.load * 100.0)
        } else {
            "-".to_string()
        };
        let models = gate.models.len().to_string();

        lines.push(format!(
            "  {gate_name:<18} {primal_status:<12} {gpu_count:>5} {ram:>8} {load:>8} {models:>7}"
        ));
    }

    lines.push(format!("  {}", "-".repeat(65)));
    lines.push(format!(
        "  {:<18} {:<12} {:>5} {:>5} GB {:>8} {:>4} unique",
        "TOTAL",
        state
            .gates
            .iter()
            .flat_map(|g| &g.primals)
            .filter(|p| p.healthy)
            .count(),
        state.collective.total_gpus,
        state.collective.total_ram_gb,
        "",
        state.collective.total_models,
    ));

    if !state.collective.capabilities.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "  Capabilities: {}",
            state.collective.capabilities.join(", ")
        ));
    }

    let bond_types: Vec<String> = state
        .gates
        .iter()
        .filter(|g| !g.is_local && g.reachable)
        .map(|g| g.bond_type.to_string())
        .collect();
    if !bond_types.is_empty() {
        lines.push(format!(
            "  Bond: {} (shared family seed, genetic trust)",
            bond_types.first().map_or("none", |s| s.as_str())
        ));
    }

    lines.push(String::new());
    lines.push(format!("  Snapshot: {}", state.snapshot_at));
    lines.push(String::new());
    lines
}

/// Format gates detail lines (pure, testable).
pub(crate) fn format_gates_detail(state: &PlasmodiumState) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(format!("  Plasmodium Gates - Family: {}", state.family_id));
    lines.push("  =========================================".to_string());

    for gate in &state.gates {
        lines.push(String::new());
        let label = if gate.is_local {
            format!("Gate: {} (local)", gate.gate_id)
        } else {
            format!("Gate: {}", gate.gate_id)
        };
        lines.push(format!("  {label}"));

        lines.push(format!("    Address:  {}", gate.address));
        lines.push(format!(
            "    Status:   {}",
            if gate.reachable { "online" } else { "offline" }
        ));
        lines.push(format!("    Bond:     {}", gate.bond_type));

        if !gate.primals.is_empty() {
            let primal_names: Vec<String> = gate
                .primals
                .iter()
                .map(|p| {
                    if p.healthy {
                        p.name.clone()
                    } else {
                        format!("{} (down)", p.name)
                    }
                })
                .collect();
            lines.push(format!("    Primals:  {}", primal_names.join(", ")));
        }

        if !gate.compute.gpus.is_empty() {
            let gpu_strs: Vec<String> = gate
                .compute
                .gpus
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            lines.push(format!("    GPUs:     {}", gpu_strs.join(", ")));
        }

        if gate.compute.ram_gb > 0 {
            lines.push(format!("    RAM:      {} GB", gate.compute.ram_gb));
        }

        if gate.compute.cpu_cores > 0 {
            lines.push(format!("    CPU:      {} cores", gate.compute.cpu_cores));
        }

        if gate.reachable {
            lines.push(format!("    Load:     {:.0}%", gate.load * 100.0));
        }

        if !gate.models.is_empty() {
            lines.push("    Models:".to_string());
            for model in &gate.models {
                lines.push(format!("      - {model}"));
            }
        }
    }

    lines.push(String::new());
    lines
}

/// Format models table lines (pure, testable).
pub(crate) fn format_models_table(
    model_gates: &HashMap<String, Vec<String>>,
    state: &PlasmodiumState,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(format!("  Plasmodium Models - Family: {}", state.family_id));
    lines.push("  =========================================".to_string());
    lines.push(String::new());

    if model_gates.is_empty() {
        return lines;
    }

    lines.push(format!("  {:<45} {:<20}", "MODEL", "GATES"));
    lines.push(format!("  {}", "-".repeat(70)));

    let mut sorted_models: Vec<_> = model_gates.iter().collect();
    sorted_models.sort_by_key(|(k, _)| (*k).clone());

    for (model_id, gates) in &sorted_models {
        lines.push(format!("  {:<45} {:<20}", model_id, gates.join(", ")));
    }

    lines.push(format!("  {}", "-".repeat(70)));
    let reachable_count = state.gates.iter().filter(|g| g.reachable).count();
    lines.push(format!(
        "  Total: {} unique model{} across {} gate{}",
        sorted_models.len(),
        plural_suffix(sorted_models.len()),
        reachable_count,
        plural_suffix(reachable_count)
    ));
    lines.push(String::new());
    lines
}

/// Run plasmodium command
pub async fn run(command: PlasmodiumCommand) -> Result<()> {
    match command {
        PlasmodiumCommand::Status => status().await,
        PlasmodiumCommand::Gates => gates().await,
        PlasmodiumCommand::Models => models().await,
    }
}

/// Show the collective status of all bonded gates
async fn status() -> Result<()> {
    let plasmodium = Plasmodium::new();
    let state = plasmodium.query_collective().await?;
    for line in format_status_table(&state) {
        println!("{line}");
    }
    Ok(())
}

/// Show detailed per-gate information
async fn gates() -> Result<()> {
    let plasmodium = Plasmodium::new();
    let state = plasmodium.query_collective().await?;
    for line in format_gates_detail(&state) {
        println!("{line}");
    }
    Ok(())
}

/// Show aggregate model view across all gates
async fn models() -> Result<()> {
    let plasmodium = Plasmodium::new();
    let state = plasmodium.query_collective().await?;

    if state.collective.models.is_empty() {
        let mut has_models = false;
        for gate in &state.gates {
            if !gate.models.is_empty() {
                has_models = true;
            }
        }
        if !has_models {
            println!("  No models found across the collective.");
            println!("  Run 'biomeos model-cache import-hf' on each gate to register models.");
            println!();
            return Ok(());
        }
    }

    let model_gates = build_model_gates_map(&state);
    if model_gates.is_empty() {
        println!("  No models found across the collective.");
        println!();
        return Ok(());
    }

    for line in format_models_table(&model_gates, &state) {
        println!("{line}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use biomeos_core::plasmodium::{
        BondType, CollectiveCapabilities, ComputeInfo, GateInfo, ModelAvailability,
        PlasmodiumState, PrimalStatus,
    };
    use biomeos_types::primal_names;

    fn make_gate(gate_id: &str, models: Vec<&str>) -> GateInfo {
        GateInfo {
            gate_id: gate_id.to_string(),
            address: format!("{gate_id}:{}", biomeos_types::constants::ports::HTTP_BRIDGE),
            is_local: gate_id == "local",
            primals: vec![PrimalStatus {
                name: primal_names::BEARDOG.to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo::default(),
            models: models.into_iter().map(String::from).collect(),
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        }
    }

    #[test]
    fn test_build_model_gates_map_empty() {
        let state = PlasmodiumState {
            gates: vec![],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let map = build_model_gates_map(&state);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_model_gates_map_from_gates() {
        let state = PlasmodiumState {
            gates: vec![
                make_gate("gate-a", vec!["model/1", "model/2"]),
                make_gate("gate-b", vec!["model/1", "model/3"]),
            ],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let map = build_model_gates_map(&state);
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("model/1").expect("model/1"), &["gate-a", "gate-b"]);
        assert_eq!(map.get("model/2").expect("model/2"), &["gate-a"]);
        assert_eq!(map.get("model/3").expect("model/3"), &["gate-b"]);
    }

    #[test]
    fn test_build_model_gates_map_merges_collective() {
        let state = PlasmodiumState {
            gates: vec![make_gate("gate-a", vec!["model/1"])],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities {
                models: vec![ModelAvailability {
                    model_id: "model/1".to_string(),
                    size_bytes: 0,
                    format: "gguf".to_string(),
                    gates: vec!["gate-a".to_string(), "gate-b".to_string()],
                }],
                ..Default::default()
            },
        };
        let map = build_model_gates_map(&state);
        let gates = map.get("model/1").expect("model/1");
        assert!(gates.contains(&"gate-a".to_string()));
        assert!(gates.contains(&"gate-b".to_string()));
    }

    #[test]
    fn test_plural_suffix() {
        assert_eq!(plural_suffix(0), "s");
        assert_eq!(plural_suffix(1), "");
        assert_eq!(plural_suffix(2), "s");
        assert_eq!(plural_suffix(42), "s");
    }

    #[test]
    fn test_format_status_table_empty() {
        let state = PlasmodiumState {
            gates: vec![],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let lines = format_status_table(&state);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("Plasmodium Status")));
        assert!(lines.iter().any(|l| l.contains("test")));
    }

    #[test]
    fn test_format_gates_detail_empty() {
        let state = PlasmodiumState {
            gates: vec![],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "fam".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let lines = format_gates_detail(&state);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("Plasmodium Gates")));
    }

    #[test]
    fn test_format_models_table() {
        let mut model_gates = HashMap::new();
        model_gates.insert("model/a".to_string(), vec!["gate-1".to_string()]);
        model_gates.insert(
            "model/b".to_string(),
            vec!["gate-1".to_string(), "gate-2".to_string()],
        );
        let state = PlasmodiumState {
            gates: vec![make_gate("gate-1", vec!["model/a"])],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let lines = format_models_table(&model_gates, &state);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("MODEL")));
        assert!(lines.iter().any(|l| l.contains("model/a")));
    }

    #[test]
    fn test_plural_suffix_edge_cases() {
        assert_eq!(plural_suffix(usize::MAX), "s");
        assert_eq!(plural_suffix(1000), "s");
    }

    #[test]
    fn test_build_model_gates_map_collective_only_model() {
        // Model in collective but not in any gate
        let state = PlasmodiumState {
            gates: vec![],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities {
                models: vec![ModelAvailability {
                    model_id: "orphan-model".to_string(),
                    size_bytes: 0,
                    format: "gguf".to_string(),
                    gates: vec!["gate-x".to_string()],
                }],
                ..Default::default()
            },
        };
        let map = build_model_gates_map(&state);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("orphan-model").unwrap(), &["gate-x"]);
    }

    #[test]
    fn test_build_model_gates_map_deduplicates_gates_from_collective() {
        // Collective has gate-a, gate-b for model/1; gates also has gate-a with model/1
        // Should not duplicate gate-a in the result
        let state = PlasmodiumState {
            gates: vec![make_gate("gate-a", vec!["model/1"])],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities {
                models: vec![ModelAvailability {
                    model_id: "model/1".to_string(),
                    size_bytes: 0,
                    format: "gguf".to_string(),
                    gates: vec!["gate-a".to_string(), "gate-b".to_string()],
                }],
                ..Default::default()
            },
        };
        let map = build_model_gates_map(&state);
        let gates = map.get("model/1").unwrap();
        assert!(gates.contains(&"gate-a".to_string()));
        assert!(gates.contains(&"gate-b".to_string()));
        assert_eq!(gates.iter().filter(|g| *g == "gate-a").count(), 1);
    }

    #[test]
    fn test_build_model_gates_map_gate_with_empty_models() {
        let mut gate = make_gate("gate-empty", vec![]);
        gate.models = vec![];
        let state = PlasmodiumState {
            gates: vec![gate],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let map = build_model_gates_map(&state);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_model_gates_map_single_gate_single_model() {
        let state = PlasmodiumState {
            gates: vec![make_gate("solo", vec!["model/only"])],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "test".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let map = build_model_gates_map(&state);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("model/only").unwrap(), &["solo"]);
    }

    #[tokio::test]
    async fn test_run_dispatches_all_commands() {
        for cmd in [
            crate::PlasmodiumCommand::Status,
            crate::PlasmodiumCommand::Gates,
            crate::PlasmodiumCommand::Models,
        ] {
            let _ = run(cmd).await;
        }
    }

    fn make_rich_gate(gate_id: &str, local: bool, reachable: bool) -> GateInfo {
        GateInfo {
            gate_id: gate_id.to_string(),
            address: format!("{gate_id}:9100"),
            is_local: local,
            primals: vec![
                PrimalStatus {
                    name: primal_names::BEARDOG.to_string(),
                    healthy: true,
                    version: Some("1.0.0".to_string()),
                },
                PrimalStatus {
                    name: primal_names::SONGBIRD.to_string(),
                    healthy: false,
                    version: None,
                },
            ],
            compute: ComputeInfo {
                gpus: vec![biomeos_core::plasmodium::GpuInfo {
                    name: "RTX 4090".to_string(),
                    vram_mb: 24576,
                    gate_id: gate_id.to_string(),
                }],
                ram_gb: 64,
                cpu_cores: 16,
            },
            models: vec!["llama3".to_string(), "mistral".to_string()],
            load: 0.42,
            reachable,
            bond_type: BondType::Covalent,
        }
    }

    #[test]
    fn test_format_status_table_rich_gates() {
        let state = PlasmodiumState {
            gates: vec![
                make_rich_gate("local", true, true),
                make_rich_gate("remote-1", false, true),
                make_rich_gate("offline-1", false, false),
            ],
            snapshot_at: "2026-01-15T00:00:00Z".to_string(),
            family_id: "rich-test".to_string(),
            collective: CollectiveCapabilities {
                total_gpus: 2,
                total_ram_gb: 128,
                total_models: 3,
                capabilities: vec!["ai.inference".to_string(), "storage".to_string()],
                ..Default::default()
            },
        };
        let lines = format_status_table(&state);
        let joined = lines.join("\n");
        assert!(joined.contains("local (local)"));
        assert!(joined.contains("remote-1"));
        assert!(joined.contains("offline"));
        assert!(joined.contains("64 GB"));
        assert!(joined.contains("42%"));
        assert!(joined.contains("Capabilities:"));
        assert!(joined.contains("ai.inference"));
        assert!(joined.contains("Bond:"));
    }

    #[test]
    fn test_format_gates_detail_rich() {
        let state = PlasmodiumState {
            gates: vec![
                make_rich_gate("local", true, true),
                make_rich_gate("remote-2", false, false),
            ],
            snapshot_at: "2026-01-15T00:00:00Z".to_string(),
            family_id: "detail-test".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let lines = format_gates_detail(&state);
        let joined = lines.join("\n");
        assert!(joined.contains("Gate: local (local)"));
        assert!(joined.contains("Gate: remote-2"));
        assert!(joined.contains("online"));
        assert!(joined.contains("offline"));
        assert!(joined.contains("RTX 4090"));
        assert!(joined.contains("64 GB"));
        assert!(joined.contains("16 cores"));
        assert!(joined.contains("42%"));
        assert!(joined.contains("llama3"));
        assert!(joined.contains("songbird (down)"));
    }

    #[test]
    fn test_format_models_table_empty_map() {
        let state = PlasmodiumState {
            gates: vec![],
            snapshot_at: "2026-01-15T00:00:00Z".to_string(),
            family_id: "empty".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let map: HashMap<String, Vec<String>> = HashMap::new();
        let lines = format_models_table(&map, &state);
        assert!(lines.iter().any(|l| l.contains("Plasmodium Models")));
        assert!(!lines.iter().any(|l| l.contains("MODEL")));
    }
}
