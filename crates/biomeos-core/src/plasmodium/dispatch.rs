// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Workload dispatch — route compute tasks to the best gate in the collective.
//!
//! Given a set of requirements (GPU VRAM, RAM, capabilities), the dispatcher
//! scores each reachable gate and returns a ranked list of candidates. The
//! graph executor uses this to auto-assign `gate` when nodes declare compute
//! requirements but no explicit target.

use super::types::{GateInfo, PlasmodiumState};

/// Requirements for a workload that needs scheduling across the collective.
#[derive(Debug, Clone, Default)]
pub struct WorkloadRequirements {
    /// Minimum GPU VRAM in megabytes (0 = no GPU required).
    pub min_vram_mb: u64,
    /// Minimum system RAM in gigabytes (0 = any).
    pub min_ram_gb: u64,
    /// Minimum CPU cores (0 = any).
    pub min_cpu_cores: usize,
    /// Required capability domain (e.g. "compute", "storage", "inference").
    pub capability: Option<String>,
    /// Prefer local execution when score is tied.
    pub prefer_local: bool,
    /// Maximum acceptable load (0.0-1.0). Gates above this are excluded.
    pub max_load: f64,
}

/// A scored candidate gate for workload placement.
#[derive(Debug, Clone)]
pub struct DispatchCandidate<'a> {
    /// Reference to the gate info.
    pub gate: &'a GateInfo,
    /// Scheduling score (lower = better). 0 is optimal.
    pub score: u32,
    /// Reason this gate was selected (for observability).
    pub reason: String,
}

/// Select the best gates for a workload from the collective state.
///
/// Returns candidates sorted by score (best first). Empty if no gate meets
/// the requirements. Unreachable gates are always excluded.
#[must_use]
pub fn select_gates<'a>(
    state: &'a PlasmodiumState,
    requirements: &WorkloadRequirements,
) -> Vec<DispatchCandidate<'a>> {
    let mut candidates: Vec<DispatchCandidate<'a>> = state
        .gates
        .iter()
        .filter(|g| g.reachable)
        .filter(|g| {
            requirements.max_load <= 0.0
                || requirements.max_load >= 1.0
                || g.load <= requirements.max_load
        })
        .filter(|g| g.compute.ram_gb >= requirements.min_ram_gb)
        .filter(|g| g.compute.cpu_cores >= requirements.min_cpu_cores)
        .filter(|g| {
            if requirements.min_vram_mb == 0 {
                return true;
            }
            g.compute
                .gpus
                .iter()
                .any(|gpu| gpu.vram_mb >= requirements.min_vram_mb)
        })
        .filter(|g| {
            if let Some(ref cap) = requirements.capability {
                g.primals
                    .iter()
                    .filter(|p| p.healthy)
                    .any(|p| {
                        biomeos_types::capability_taxonomy::capabilities_for_primal(&p.name)
                            .contains(cap)
                    })
            } else {
                true
            }
        })
        .map(|gate| {
            let score = compute_score(gate, requirements);
            let reason = format_reason(gate, requirements);
            DispatchCandidate {
                gate,
                score,
                reason,
            }
        })
        .collect();

    candidates.sort_by_key(|c| c.score);
    candidates
}

/// Compute a scheduling score for a gate (lower = better).
fn compute_score(gate: &GateInfo, req: &WorkloadRequirements) -> u32 {
    let mut score = 0u32;

    // Locality bonus
    if gate.is_local && req.prefer_local {
        // Local gets a significant bonus
    } else if !gate.is_local {
        score += 5;
    }

    // VRAM scoring: more VRAM = better for GPU workloads
    if req.min_vram_mb > 0 {
        let max_vram = gate.compute.gpus.iter().map(|g| g.vram_mb).max().unwrap_or(0);
        score += match max_vram {
            v if v >= 24_000 => 0,
            v if v >= 16_000 => 2,
            v if v >= 8_000 => 5,
            _ => 10,
        };
    }

    // Load scoring: lower load = better
    #[expect(clippy::cast_possible_truncation)]
    let load_penalty = (gate.load * 10.0) as u32;
    score += load_penalty;

    // RAM headroom bonus
    if req.min_ram_gb > 0 && gate.compute.ram_gb >= req.min_ram_gb * 2 {
        score = score.saturating_sub(1);
    }

    score
}

fn format_reason(gate: &GateInfo, req: &WorkloadRequirements) -> String {
    let mut parts = Vec::new();

    if gate.is_local {
        parts.push("local".to_string());
    }

    if req.min_vram_mb > 0 {
        let max_vram = gate.compute.gpus.iter().map(|g| g.vram_mb).max().unwrap_or(0);
        parts.push(format!("vram={max_vram}MB"));
    }

    if gate.load > 0.0 {
        parts.push(format!("load={:.0}%", gate.load * 100.0));
    }

    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plasmodium::types::*;

    fn make_gate(id: &str, is_local: bool, vram_mb: u64, ram_gb: u64, load: f64) -> GateInfo {
        GateInfo {
            gate_id: id.to_string(),
            address: if is_local {
                "local".to_string()
            } else {
                format!("10.0.0.{}", id.len())
            },
            is_local,
            primals: vec![PrimalStatus {
                name: "toadstool".to_string(),
                healthy: true,
                version: None,
            }],
            compute: ComputeInfo {
                gpus: if vram_mb > 0 {
                    vec![GpuInfo {
                        name: "GPU".to_string(),
                        vram_mb,
                        gate_id: id.to_string(),
                    }]
                } else {
                    vec![]
                },
                ram_gb,
                cpu_cores: 8,
            },
            models: vec![],
            load,
            reachable: true,
            bond_type: BondType::Covalent,
        }
    }

    fn make_state(gates: Vec<GateInfo>) -> PlasmodiumState {
        PlasmodiumState {
            gates,
            snapshot_at: "2026-07-27T00:00:00Z".to_string(),
            family_id: "test-family".to_string(),
            collective: CollectiveCapabilities::default(),
        }
    }

    #[test]
    fn prefers_higher_vram_for_gpu_workloads() {
        let state = make_state(vec![
            make_gate("east", true, 12_000, 64, 0.3),
            make_gate("iron", false, 24_000, 256, 0.1),
        ]);

        let req = WorkloadRequirements {
            min_vram_mb: 8_000,
            ..Default::default()
        };

        let candidates = select_gates(&state, &req);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].gate.gate_id, "iron");
    }

    #[test]
    fn prefers_local_when_scores_tied() {
        let state = make_state(vec![
            make_gate("east", true, 24_000, 64, 0.2),
            make_gate("iron", false, 24_000, 256, 0.2),
        ]);

        let req = WorkloadRequirements {
            min_vram_mb: 8_000,
            prefer_local: true,
            ..Default::default()
        };

        let candidates = select_gates(&state, &req);
        assert_eq!(candidates[0].gate.gate_id, "east");
    }

    #[test]
    fn excludes_overloaded_gates() {
        let state = make_state(vec![
            make_gate("east", true, 12_000, 64, 0.9),
            make_gate("iron", false, 24_000, 256, 0.2),
        ]);

        let req = WorkloadRequirements {
            min_vram_mb: 8_000,
            max_load: 0.8,
            ..Default::default()
        };

        let candidates = select_gates(&state, &req);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].gate.gate_id, "iron");
    }

    #[test]
    fn excludes_insufficient_ram() {
        let state = make_state(vec![
            make_gate("east", true, 12_000, 16, 0.1),
            make_gate("iron", false, 24_000, 256, 0.1),
        ]);

        let req = WorkloadRequirements {
            min_ram_gb: 128,
            ..Default::default()
        };

        let candidates = select_gates(&state, &req);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].gate.gate_id, "iron");
    }

    #[test]
    fn excludes_unreachable_gates() {
        let mut unreachable = make_gate("strand", false, 24_000, 256, 0.0);
        unreachable.reachable = false;

        let state = make_state(vec![
            make_gate("east", true, 12_000, 64, 0.1),
            unreachable,
        ]);

        let req = WorkloadRequirements {
            min_vram_mb: 8_000,
            ..Default::default()
        };

        let candidates = select_gates(&state, &req);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].gate.gate_id, "east");
    }

    #[test]
    fn empty_when_no_gate_meets_requirements() {
        let state = make_state(vec![make_gate("east", true, 4_000, 16, 0.1)]);

        let req = WorkloadRequirements {
            min_vram_mb: 24_000,
            min_ram_gb: 256,
            ..Default::default()
        };

        let candidates = select_gates(&state, &req);
        assert!(candidates.is_empty());
    }

    #[test]
    fn no_requirements_returns_all_reachable() {
        let state = make_state(vec![
            make_gate("east", true, 0, 64, 0.1),
            make_gate("iron", false, 24_000, 256, 0.2),
            make_gate("north", false, 0, 32, 0.5),
        ]);

        let req = WorkloadRequirements::default();
        let candidates = select_gates(&state, &req);
        assert_eq!(candidates.len(), 3);
    }

    #[test]
    fn lower_load_ranks_better() {
        let state = make_state(vec![
            make_gate("iron", false, 24_000, 256, 0.8),
            make_gate("strand", false, 24_000, 256, 0.1),
        ]);

        let req = WorkloadRequirements {
            min_vram_mb: 8_000,
            ..Default::default()
        };

        let candidates = select_gates(&state, &req);
        assert_eq!(candidates[0].gate.gate_id, "strand");
    }
}
