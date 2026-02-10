//! Plasmodium mode - Over-NUCLEUS collective coordination
//!
//! Provides a unified view of all covalently bonded NUCLEUS instances.
//! Named after the slime mold *Physarum polycephalum*.
//!
//! See `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md` for the full specification.

use anyhow::Result;
use biomeos_core::plasmodium::Plasmodium;

use crate::PlasmodiumCommand;

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

    let reachable_gates: Vec<_> = state.gates.iter().filter(|g| g.reachable).collect();

    println!();
    println!("  Plasmodium Status - Family: {}", state.family_id);
    println!("  =========================================");
    println!();
    println!(
        "  Collective: {} gate{} bonded (covalent)",
        reachable_gates.len(),
        if reachable_gates.len() != 1 { "s" } else { "" }
    );
    println!();

    // Table header
    println!(
        "  {:<18} {:<12} {:>5} {:>8} {:>8} {:>7}",
        "GATE", "PRIMALS", "GPUs", "RAM", "LOAD", "MODELS"
    );
    println!("  {}", "-".repeat(65));

    for gate in &state.gates {
        let primal_status = if gate.reachable {
            let healthy = gate.primals.iter().filter(|p| p.healthy).count();
            let total = gate.primals.len();
            format!("{}/{}", healthy, total)
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

        println!(
            "  {:<18} {:<12} {:>5} {:>8} {:>8} {:>7}",
            gate_name, primal_status, gpu_count, ram, load, models
        );
    }

    // Totals
    println!("  {}", "-".repeat(65));
    println!(
        "  {:<18} {:<12} {:>5} {:>5} GB {:>8} {:>4} unique",
        "TOTAL",
        format!(
            "{}",
            state
                .gates
                .iter()
                .flat_map(|g| &g.primals)
                .filter(|p| p.healthy)
                .count()
        ),
        state.collective.total_gpus,
        state.collective.total_ram_gb,
        "",
        state.collective.total_models,
    );

    // Capabilities
    if !state.collective.capabilities.is_empty() {
        println!();
        println!(
            "  Capabilities: {}",
            state.collective.capabilities.join(", ")
        );
    }

    // Bond info
    let bond_types: Vec<String> = state
        .gates
        .iter()
        .filter(|g| !g.is_local && g.reachable)
        .map(|g| g.bond_type.to_string())
        .collect();
    if !bond_types.is_empty() {
        println!(
            "  Bond: {} (shared family seed, genetic trust)",
            bond_types.first().unwrap_or(&"none".to_string())
        );
    }

    // Snapshot time
    println!();
    println!("  Snapshot: {}", state.snapshot_at);
    println!();

    Ok(())
}

/// Show detailed per-gate information
async fn gates() -> Result<()> {
    let plasmodium = Plasmodium::new();
    let state = plasmodium.query_collective().await?;

    println!();
    println!("  Plasmodium Gates - Family: {}", state.family_id);
    println!("  =========================================");

    for gate in &state.gates {
        println!();
        let label = if gate.is_local {
            format!("Gate: {} (local)", gate.gate_id)
        } else {
            format!("Gate: {}", gate.gate_id)
        };
        println!("  {}", label);

        println!("    Address:  {}", gate.address);
        println!(
            "    Status:   {}",
            if gate.reachable { "online" } else { "offline" }
        );
        println!("    Bond:     {}", gate.bond_type);

        // Primals
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
            println!("    Primals:  {}", primal_names.join(", "));
        }

        // GPUs
        if !gate.compute.gpus.is_empty() {
            let gpu_strs: Vec<String> = gate.compute.gpus.iter().map(|g| g.to_string()).collect();
            println!("    GPUs:     {}", gpu_strs.join(", "));
        }

        // RAM
        if gate.compute.ram_gb > 0 {
            println!("    RAM:      {} GB", gate.compute.ram_gb);
        }

        // CPU
        if gate.compute.cpu_cores > 0 {
            println!("    CPU:      {} cores", gate.compute.cpu_cores);
        }

        // Load
        if gate.reachable {
            println!("    Load:     {:.0}%", gate.load * 100.0);
        }

        // Models
        if !gate.models.is_empty() {
            println!("    Models:");
            for model in &gate.models {
                println!("      - {}", model);
            }
        }
    }

    println!();
    Ok(())
}

/// Show aggregate model view across all gates
async fn models() -> Result<()> {
    let plasmodium = Plasmodium::new();
    let state = plasmodium.query_collective().await?;

    println!();
    println!("  Plasmodium Models - Family: {}", state.family_id);
    println!("  =========================================");
    println!();

    if state.collective.models.is_empty() {
        // Fall back to showing local models from each gate
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

    // Build a unified model view from gate data
    let mut model_gates: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for gate in &state.gates {
        for model in &gate.models {
            model_gates
                .entry(model.clone())
                .or_default()
                .push(gate.gate_id.clone());
        }
    }

    // Also include from collective aggregation
    for model in &state.collective.models {
        model_gates.entry(model.model_id.clone()).or_default();
        for gate_id in &model.gates {
            let entry = model_gates.entry(model.model_id.clone()).or_default();
            if !entry.contains(gate_id) {
                entry.push(gate_id.clone());
            }
        }
    }

    if model_gates.is_empty() {
        println!("  No models found across the collective.");
        println!();
        return Ok(());
    }

    // Display
    println!("  {:<45} {:<20}", "MODEL", "GATES");
    println!("  {}", "-".repeat(70));

    let mut sorted_models: Vec<_> = model_gates.iter().collect();
    sorted_models.sort_by_key(|(k, _)| (*k).clone());

    for (model_id, gates) in &sorted_models {
        println!("  {:<45} {:<20}", model_id, gates.join(", "),);
    }

    println!("  {}", "-".repeat(70));
    println!(
        "  Total: {} unique model{} across {} gate{}",
        sorted_models.len(),
        if sorted_models.len() != 1 { "s" } else { "" },
        state.gates.iter().filter(|g| g.reachable).count(),
        if state.gates.iter().filter(|g| g.reachable).count() != 1 {
            "s"
        } else {
            ""
        }
    );
    println!();

    Ok(())
}
