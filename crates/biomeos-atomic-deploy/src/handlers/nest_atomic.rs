// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Nest Atomic composition handlers — per-domain health probes.
//!
//! Modeled on hotSpring's `composition.rs` thin-layer pattern: each Nest
//! domain (security, discovery, storage, dag, ledger, attribution) is probed
//! independently. The aggregate `nest_health` is derived from individual
//! domain statuses, not from a monolithic check.
//!
//! # Nest Atomic domains (6)
//!
//! | Domain       | Provider    | Primal names / aliases           |
//! |-------------|-------------|----------------------------------|
//! | security    | bearDog     | `beardog`, `crypto`, `security`  |
//! | discovery   | songBird    | `songbird`, `network`            |
//! | storage     | nestGate    | `nestgate`, `permanence`         |
//! | dag         | rhizoCrypt  | `rhizocrypt`, `dag`              |
//! | ledger      | loamSpine   | `loamspine`, `ledger`            |
//! | attribution | sweetGrass  | `sweetgrass`, `attribution`      |
//!
//! # Neural API methods
//!
//! - `nest.health` / `composition.nest_health` — per-domain health report
//! - `nest.capabilities` — list all available nest capabilities + translations

use anyhow::Result;
use biomeos_types::primal_names::{
    BEARDOG, LOAMSPINE, NESTGATE, RHIZOCRYPT, SONGBIRD, SWEETGRASS,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::LifecycleHandler;
use crate::capability_translation::CapabilityTranslationRegistry;

/// Nest domain: maps a logical domain to the primal names/aliases that
/// serve it. Primals register under canonical names or capability aliases
/// — we match both.
struct NestDomain {
    name: &'static str,
    primal_names: &'static [&'static str],
    capability_prefixes: &'static [&'static str],
}

const NEST_DOMAINS: &[NestDomain] = &[
    NestDomain {
        name: "security",
        primal_names: &[BEARDOG, "crypto", "security", "beacon", "ed25519", "x25519"],
        capability_prefixes: &["crypto.", "security.", "beacon."],
    },
    NestDomain {
        name: "discovery",
        primal_names: &[SONGBIRD, "network", "mesh"],
        capability_prefixes: &["discovery.", "relay.", "network."],
    },
    NestDomain {
        name: "storage",
        primal_names: &[NESTGATE, "permanence", "storage"],
        capability_prefixes: &["storage.", "content."],
    },
    NestDomain {
        name: "dag",
        primal_names: &[RHIZOCRYPT, "dag"],
        capability_prefixes: &["dag."],
    },
    NestDomain {
        name: "ledger",
        primal_names: &[LOAMSPINE, "ledger"],
        capability_prefixes: &["spine.", "anchor.", "ledger."],
    },
    NestDomain {
        name: "attribution",
        primal_names: &[SWEETGRASS, "attribution"],
        capability_prefixes: &["braid.", "provenance.", "attribution."],
    },
];

impl LifecycleHandler {
    /// Handle `nest.health` / `composition.nest_health`.
    ///
    /// Probes each Nest domain independently by checking which primals are
    /// alive and match that domain (by name or capability alias). Returns
    /// per-domain status following hotSpring's atomic health pattern.
    pub async fn nest_atomic_health(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let mut domain_results = serde_json::Map::new();
        let mut all_healthy = true;
        let mut primals_alive = 0u32;
        let mut domains_ok = 0u32;

        for domain in NEST_DOMAINS {
            let mut providers: Vec<Value> = Vec::new();
            let mut domain_healthy = false;

            for (name, state) in &status {
                let name_lower = name.to_lowercase();
                let in_domain = domain.primal_names.iter().any(|p| *p == name_lower);
                if !in_domain {
                    continue;
                }

                let is_active = matches!(
                    state,
                    crate::lifecycle_manager::LifecycleState::Active { .. }
                );
                let status_str = if is_active {
                    primals_alive += 1;
                    domain_healthy = true;
                    "healthy"
                } else {
                    "unavailable"
                };
                providers.push(json!({
                    "primal": name,
                    "status": status_str,
                }));
            }

            if providers.is_empty() {
                domain_results.insert(
                    domain.name.to_string(),
                    json!({
                        "status": "unavailable",
                        "providers": [],
                    }),
                );
                all_healthy = false;
            } else {
                let status = if domain_healthy { "ok" } else { "degraded" };
                if domain_healthy {
                    domains_ok += 1;
                } else {
                    all_healthy = false;
                }
                domain_results.insert(
                    domain.name.to_string(),
                    json!({
                        "status": status,
                        "providers": providers,
                    }),
                );
            }
        }

        let pipeline_ready = domains_ok == NEST_DOMAINS.len() as u32;

        Ok(json!({
            "atomic": "nest",
            "healthy": all_healthy,
            "pipeline_ready": pipeline_ready,
            "domains": domain_results,
            "domains_ok": domains_ok,
            "domains_total": NEST_DOMAINS.len(),
            "primals_alive": primals_alive,
        }))
    }

    /// Handle `nest.capabilities` — list all Nest translation routes + ribocipher flags.
    pub async fn nest_atomic_capabilities(
        &self,
        translation_registry: &Arc<RwLock<CapabilityTranslationRegistry>>,
    ) -> Result<Value> {
        let registry = translation_registry.read().await;

        let mut domain_caps: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut total = 0u32;

        for domain in NEST_DOMAINS {
            let translations: Vec<Value> = domain
                .capability_prefixes
                .iter()
                .flat_map(|prefix| {
                    registry
                        .translations_with_prefix(prefix)
                        .into_iter()
                        .map(|t| {
                            json!({
                                "semantic": t.semantic,
                                "provider": t.provider,
                                "method": t.actual_method,
                                "ribocipher": t.ribocipher,
                            })
                        })
                })
                .collect();
            total += translations.len() as u32;
            domain_caps.insert(domain.name.to_string(), json!(translations));
        }

        Ok(json!({
            "atomic": "nest",
            "domains": domain_caps,
            "total_translations": total,
        }))
    }
}
