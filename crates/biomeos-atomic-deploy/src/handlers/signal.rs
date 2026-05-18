// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Atomic signal dispatch handler.
//!
//! Decomposes atomic signals (e.g., `tower.publish`, `nest.store`,
//! `braid.partial_update`) into graph executions. Each signal maps to a
//! TOML graph in `graphs/signals/` that defines the primal-level capability
//! calls biomeOS orchestrates.
//!
//! This is the "composition collapse" layer: springs send one signal,
//! biomeOS executes a multi-node graph.

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Known atomic signal tiers. A capability.call with one of these as the
/// `capability` field is intercepted and dispatched as a signal graph.
///
/// - **tower**: security + mesh orchestration signals
/// - **node**: compute-level signals
/// - **nest**: storage + content signals
/// - **meta**: observability + composition signals
/// - **braid**: provenance braid lifecycle signals (partial updates, completion)
const SIGNAL_TIERS: &[&str] = &["tower", "node", "nest", "meta", "braid"];

/// Check whether a capability domain is a signal tier.
pub fn is_signal_tier(capability: &str) -> bool {
    SIGNAL_TIERS.contains(&capability)
}

/// Resolve a signal to its graph path.
///
/// `tower.publish` → `{graphs_dir}/signals/tower_publish.toml`
pub fn signal_graph_path(graphs_dir: &Path, tier: &str, signal: &str) -> PathBuf {
    graphs_dir.join(format!("signals/{}_{}.toml", tier, signal))
}

/// List all available signal graphs from the signals directory.
pub fn list_signal_graphs(graphs_dir: &Path) -> Vec<SignalInfo> {
    let signals_dir = graphs_dir.join("signals");
    let mut signals = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&signals_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some((tier, signal)) = stem.split_once('_') {
                        if SIGNAL_TIERS.contains(&tier) {
                            signals.push(SignalInfo {
                                name: format!("{tier}.{signal}"),
                                tier: tier.to_string(),
                                signal: signal.to_string(),
                                graph_path: path.display().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    signals.sort_by(|a, b| a.name.cmp(&b.name));
    signals
}

/// Load the signal tools schema (config/signal_tools.toml).
pub fn load_signal_schema(graphs_dir: &Path) -> Result<Value> {
    let schema_path = graphs_dir.join("../config/signal_tools.toml");
    if !schema_path.exists() {
        anyhow::bail!("signal_tools.toml not found at {}", schema_path.display());
    }
    let content =
        std::fs::read_to_string(&schema_path).context("Failed to read signal_tools.toml")?;
    let parsed: toml::Value =
        toml::from_str(&content).context("Failed to parse signal_tools.toml")?;
    Ok(serde_json::to_value(parsed)?)
}

/// Metadata about an available signal graph.
#[derive(Debug, Clone)]
pub struct SignalInfo {
    /// Signal graph file name (without extension).
    pub name: String,
    /// Atomic tier this signal belongs to (tower/node/nest/meta/braid).
    pub tier: String,
    /// Signal operation name within the tier.
    pub signal: String,
    /// Filesystem path to the signal graph TOML.
    pub graph_path: String,
}

/// Handle `signal.dispatch` — execute an atomic signal as a graph.
///
/// Params: `{ "tier": "tower", "signal": "publish", "params": {...} }`
/// or:     `{ "signal": "tower.publish", "params": {...} }`
pub async fn dispatch(
    graphs_dir: &Path,
    family_id: &str,
    graph_handler: &super::GraphHandler,
    params: &Option<Value>,
) -> Result<Value> {
    let params = params.as_ref().context("Missing parameters")?;

    let (tier, signal) = if let Some(signal_name) = params["signal"].as_str() {
        if let Some((t, s)) = signal_name.split_once('.') {
            (t.to_string(), s.to_string())
        } else {
            anyhow::bail!("signal must be 'tier.name' format, got: {signal_name}");
        }
    } else {
        let tier = params["tier"]
            .as_str()
            .context("Missing 'tier' or 'signal' field")?;
        let signal = params["operation"]
            .as_str()
            .or_else(|| params["signal_name"].as_str())
            .context("Missing 'operation' or 'signal_name' field")?;
        (tier.to_string(), signal.to_string())
    };

    if !SIGNAL_TIERS.contains(&tier.as_str()) {
        anyhow::bail!("Unknown signal tier '{tier}'. Valid: {SIGNAL_TIERS:?}");
    }

    let graph_path = signal_graph_path(graphs_dir, &tier, &signal);
    if !graph_path.exists() {
        anyhow::bail!(
            "Signal graph not found: {tier}.{signal} (expected at {})",
            graph_path.display()
        );
    }

    let graph_id = format!("signals/{}_{}", tier, signal);
    info!("⚡ Signal dispatch: {tier}.{signal} → {graph_id}");

    let signal_params = params.get("params").cloned().unwrap_or(json!({}));

    // The graph_id includes "signals/" prefix so graph_handler.execute()
    // resolves it from graphs/signals/. The signal_context metadata lets
    // MetricsCollector tag executions for PathwayLearner analysis under
    // a signal: namespace (e.g. "signal:tower.publish").
    let execute_params = json!({
        "graph_id": graph_id,
        "family_id": family_id,
        "signal_context": {
            "tier": tier,
            "signal": signal,
            "params": signal_params,
            "metrics_namespace": format!("signal:{tier}.{signal}"),
        }
    });

    let result = graph_handler.execute(&Some(execute_params)).await?;

    debug!("⚡ Signal {tier}.{signal} dispatched as graph {graph_id}");

    Ok(json!({
        "signal": format!("{tier}.{signal}"),
        "graph_id": graph_id,
        "execution": result,
    }))
}

/// Handle `signal.list` — enumerate available atomic signals.
pub async fn list(graphs_dir: &Path) -> Result<Value> {
    let signals = list_signal_graphs(graphs_dir);

    Ok(json!({
        "signals": signals.iter().map(|s| json!({
            "name": s.name,
            "tier": s.tier,
            "signal": s.signal,
            "graph": s.graph_path,
        })).collect::<Vec<_>>(),
        "count": signals.len(),
        "tiers": SIGNAL_TIERS,
    }))
}

/// Handle `signal.schema` — return the signal tools schema for Squirrel.
pub async fn schema(graphs_dir: &Path) -> Result<Value> {
    load_signal_schema(graphs_dir)
}
