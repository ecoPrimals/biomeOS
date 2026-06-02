// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Routing weight table — persistent store for provider weights.
//!
//! Maps `(capability, provider)` → `ProviderWeight`. Optionally backed by
//! redb for persistence across process restarts.

use redb::{Database, ReadableTable, TableDefinition};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, warn};

use super::scoring::{EXPLORATION_BONUS, ProviderWeight};

/// redb table definition for persistent routing weights.
const WEIGHTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("routing_weights");

/// The routing weight table — adaptive dispatch intelligence.
///
/// Maps `(capability, provider)` → `ProviderWeight`. Used by `capability.call`
/// to select the best provider when multiple can serve a request.
///
/// When backed by a redb database (via `open` or `with_db`), weights are
/// persisted after every mutation so they survive process restarts. Without
/// a database, the table operates purely in-memory.
#[derive(Debug, Default)]
pub struct RoutingWeightTable {
    /// (capability, provider) → weight
    weights: HashMap<(Arc<str>, Arc<str>), ProviderWeight>,
    /// Optional redb database for persistence.
    db: Option<Arc<Database>>,
}

impl RoutingWeightTable {
    /// Create an empty in-memory weight table (no persistence).
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a persistent weight table backed by a redb database.
    ///
    /// If the database file exists, weights are loaded from it. If it doesn't
    /// exist, a new database is created. Falls back to in-memory on I/O error.
    pub fn open(path: &Path) -> Self {
        match Database::create(path) {
            Ok(db) => {
                let db = Arc::new(db);
                let weights = load_weights_from_db(&db);
                debug!(
                    "Loaded {} routing weights from {}",
                    weights.len(),
                    path.display()
                );
                Self {
                    weights,
                    db: Some(db),
                }
            }
            Err(e) => {
                warn!(
                    "Failed to open routing weights DB at {}: {} — falling back to in-memory",
                    path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    /// Attach a redb database for persistence and flush current state.
    pub fn with_db(mut self, db: Arc<Database>) -> Self {
        self.db = Some(db);
        self.flush();
        self
    }

    /// Whether this table is backed by persistent storage.
    pub fn is_persistent(&self) -> bool {
        self.db.is_some()
    }

    /// Flush all weights to the backing database (no-op if in-memory).
    pub fn flush(&self) {
        let Some(db) = &self.db else { return };
        if let Err(e) = flush_weights_to_db(db, &self.weights) {
            warn!("Failed to flush routing weights to disk: {e}");
        }
    }

    /// Get or create a weight entry for a provider+capability pair.
    pub fn get_or_create(&mut self, capability: &str, provider: &str) -> &mut ProviderWeight {
        let key = (Arc::from(capability), Arc::from(provider));
        self.weights
            .entry(key)
            .or_insert_with(|| ProviderWeight::new(provider, capability))
    }

    /// Get a weight entry (read-only).
    pub fn get(&self, capability: &str, provider: &str) -> Option<&ProviderWeight> {
        let key = (Arc::from(capability), Arc::from(provider));
        self.weights.get(&key)
    }

    /// Select the best provider for a capability from a list of candidates.
    ///
    /// Returns `None` if no candidates are available (all circuit-broken).
    pub fn select_best<'a>(
        &self,
        capability: &str,
        candidates: &'a [Arc<str>],
    ) -> Option<&'a Arc<str>> {
        let mut best: Option<(&Arc<str>, f64)> = None;

        for candidate in candidates {
            let key = (Arc::from(capability), candidate.clone());
            let score = self
                .weights
                .get(&key)
                .map(|w| {
                    if w.is_available() {
                        w.score()
                    } else {
                        0.0
                    }
                })
                // Unknown providers get a default score (encourages discovery).
                .unwrap_or(0.5 + EXPLORATION_BONUS);

            if score > 0.0 {
                match best {
                    Some((_, best_score)) if score > best_score => {
                        best = Some((candidate, score));
                    }
                    None => {
                        best = Some((candidate, score));
                    }
                    _ => {}
                }
            }
        }

        best.map(|(provider, _)| provider)
    }

    /// Look up the current score for a `(capability, provider)` key.
    pub fn score_for(&self, key: &(Arc<str>, Arc<str>)) -> Option<f64> {
        self.weights.get(key).map(|w| w.score())
    }

    /// Record a dispatch outcome, updating the provider's weight.
    pub fn record_outcome(
        &mut self,
        capability: &str,
        provider: &str,
        success: bool,
        latency_ms: u64,
    ) {
        let weight = self.get_or_create(capability, provider);
        if success {
            weight.record_success(latency_ms);
        } else {
            weight.record_failure();
        }
        self.persist_entry(capability, provider);
    }

    /// Set affinity hint for a provider (from primal.announce or config).
    pub fn set_affinity(&mut self, capability: &str, provider: &str, affinity: f64) {
        let weight = self.get_or_create(capability, provider);
        weight.affinity = affinity.clamp(0.0, 1.0);
        self.persist_entry(capability, provider);
    }

    /// Set cost hint for a provider (from primal.announce).
    pub fn set_cost_hint(&mut self, capability: &str, provider: &str, cost: f64) {
        let weight = self.get_or_create(capability, provider);
        weight.cost_hint = Some(cost);
        self.persist_entry(capability, provider);
    }

    /// Set topology affinity for a provider (inferred from transport endpoint).
    pub fn set_topology_affinity(
        &mut self,
        capability: &str,
        provider: &str,
        topology_affinity: f64,
    ) {
        let weight = self.get_or_create(capability, provider);
        weight.topology_affinity = topology_affinity.clamp(0.0, 1.0);
        self.persist_entry(capability, provider);
    }

    /// Persist a single weight entry to the backing database.
    fn persist_entry(&self, capability: &str, provider: &str) {
        let Some(db) = &self.db else { return };
        let key = (Arc::from(capability), Arc::from(provider));
        let Some(weight) = self.weights.get(&key) else {
            return;
        };
        let db_key = format!("{capability}\0{provider}");
        let Ok(bytes) = serde_json::to_vec(weight) else {
            return;
        };
        if let Err(e) = persist_single_weight(db, &db_key, &bytes) {
            warn!("Failed to persist weight {db_key}: {e}");
        }
    }

    /// All weights for a capability domain.
    pub fn weights_for_capability(&self, capability: &str) -> Vec<&ProviderWeight> {
        self.weights
            .iter()
            .filter(|((cap, _), _)| cap.as_ref() == capability)
            .map(|(_, w)| w)
            .collect()
    }

    /// Snapshot of all weights (for serialization / RPC response).
    pub fn snapshot(&self) -> Vec<ProviderWeight> {
        self.weights.values().cloned().collect()
    }

    /// Number of tracked provider+capability pairs.
    pub fn len(&self) -> usize {
        self.weights.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }

    /// Summary statistics for the weight table.
    pub fn summary(&self) -> WeightTableSummary {
        let total_dispatches: u64 = self.weights.values().map(|w| w.total_dispatches()).sum();
        let circuit_open = self.weights.values().filter(|w| w.circuit_open).count();
        let providers: Vec<Arc<str>> = {
            let mut v: Vec<_> = self.weights.values().map(|w| w.provider.clone()).collect();
            v.sort();
            v.dedup();
            v
        };
        let capabilities: Vec<Arc<str>> = {
            let mut v: Vec<_> = self
                .weights
                .values()
                .map(|w| w.capability.clone())
                .collect();
            v.sort();
            v.dedup();
            v
        };

        WeightTableSummary {
            entries: self.weights.len(),
            total_dispatches,
            circuit_open,
            unique_providers: providers.len(),
            unique_capabilities: capabilities.len(),
        }
    }
}

/// Summary of the routing weight table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeightTableSummary {
    /// Total (capability, provider) pairs tracked.
    pub entries: usize,
    /// Sum of all dispatches across all providers.
    pub total_dispatches: u64,
    /// Number of providers with open circuit breakers.
    pub circuit_open: usize,
    /// Distinct primal providers in the table.
    pub unique_providers: usize,
    /// Distinct capability domains in the table.
    pub unique_capabilities: usize,
}

// ── redb I/O helpers ─────────────────────────────────────────────────

fn load_weights_from_db(db: &Database) -> HashMap<(Arc<str>, Arc<str>), ProviderWeight> {
    let mut map = HashMap::new();
    let Ok(txn) = db.begin_read() else {
        return map;
    };
    let Ok(table) = txn.open_table(WEIGHTS_TABLE) else {
        return map;
    };
    let Ok(iter) = table.iter() else {
        return map;
    };
    for entry in iter.flatten() {
        let key_str = entry.0.value();
        let bytes = entry.1.value();
        if let Some((cap, prov)) = key_str.split_once('\0') {
            if let Ok(weight) = serde_json::from_slice::<ProviderWeight>(bytes) {
                map.insert((Arc::from(cap), Arc::from(prov)), weight);
            }
        }
    }
    map
}

fn flush_weights_to_db(
    db: &Database,
    weights: &HashMap<(Arc<str>, Arc<str>), ProviderWeight>,
) -> Result<(), anyhow::Error> {
    let txn = db.begin_write()?;
    {
        let mut table = txn.open_table(WEIGHTS_TABLE)?;
        for ((cap, prov), weight) in weights {
            let key = format!("{cap}\0{prov}");
            if let Ok(bytes) = serde_json::to_vec(weight) {
                table.insert(key.as_str(), bytes.as_slice())?;
            }
        }
    }
    txn.commit()?;
    Ok(())
}

fn persist_single_weight(db: &Database, key: &str, bytes: &[u8]) -> Result<(), anyhow::Error> {
    let txn = db.begin_write()?;
    {
        let mut table = txn.open_table(WEIGHTS_TABLE)?;
        table.insert(key, bytes)?;
    }
    txn.commit()?;
    Ok(())
}
