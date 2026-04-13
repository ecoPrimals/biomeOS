// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability Domain Mappings
//!
//! Two-tier resolution: runtime `CapabilityRegistry` loaded from
//! `config/capability_registry.toml`, with a compiled-in const fallback
//! (`CAPABILITY_DOMAINS`) for zero-config environments.
//!
//! # Design Principle
//!
//! biomeOS orchestrates, primals execute primitives.
//! The mapping is SEMANTIC → PROVIDER (not implementation-specific).

use biomeos_types::primal_names::{
    AIRSPRING, BARRACUDA, BEARDOG, CORALREEF, HEALTHSPRING, LOAMSPINE, LUDOSPRING, NESTGATE,
    NEURALSPRING, PETALTONGUE, RHIZOCRYPT, SONGBIRD, SQUIRREL, SWEETGRASS, TOADSTOOL, WETSPRING,
};
use std::collections::HashMap;
use std::path::Path;

/// Capability domain configuration
/// Loaded from `config/capability_registry.toml` in production
pub struct CapabilityDomain {
    pub provider: &'static str,
    pub capabilities: &'static [&'static str],
}

/// Default capability domains for fallback resolution
/// These align with `config/capability_registry.toml`
pub const CAPABILITY_DOMAINS: &[CapabilityDomain] = &[
    // Security domain (BearDog)
    CapabilityDomain {
        provider: BEARDOG,
        capabilities: &[
            "security",
            "crypto",
            "encryption",
            "genetic",
            "beacon",
            "tls",
            "jwt",
        ],
    },
    // Network domain (Songbird)
    CapabilityDomain {
        provider: SONGBIRD,
        capabilities: &[
            "discovery",
            "http",
            "orchestration",
            "federation",
            "network",
        ],
    },
    // Storage domain (NestGate)
    CapabilityDomain {
        provider: NESTGATE,
        capabilities: &["storage", "versioning", "persistence"],
    },
    // Compute domain (ToadStool)
    CapabilityDomain {
        provider: TOADSTOOL,
        capabilities: &["compute", "execution", "parsing", "hardware_learning"],
    },
    // GPU math/tensor/stats domain (barraCuda)
    CapabilityDomain {
        provider: BARRACUDA,
        capabilities: &["math", "tensor", "stats", "noise", "activation", "rng"],
    },
    // Shader compilation domain (coralReef)
    CapabilityDomain {
        provider: CORALREEF,
        capabilities: &["shader", "wgsl", "spirv"],
    },
    // AI domain (Squirrel)
    CapabilityDomain {
        provider: SQUIRREL,
        capabilities: &["ai", "mcp", "assistance", "ml"],
    },
    // Data domain (NestGate live providers)
    CapabilityDomain {
        provider: NESTGATE,
        capabilities: &[
            "data",
            "ncbi",
            "noaa",
            "iris",
            "weather_data",
            "seismic_data",
        ],
    },
    // Science domain (wetSpring) — compile-time bootstrap hints; runtime uses capability registry / discovery
    CapabilityDomain {
        provider: WETSPRING,
        capabilities: &[
            "science",
            "biodiversity",
            "spectral",
            "metagenomics",
            "kinetics",
            "monitoring",
        ],
    },
    // Neural science domain (neuralSpring)
    CapabilityDomain {
        provider: NEURALSPRING,
        capabilities: &[
            "spectral_analysis",
            "anderson_localization",
            "hessian_eigenanalysis",
            "agent_coordination",
            "training_trajectory",
        ],
    },
    // Ecology domain (airSpring)
    CapabilityDomain {
        provider: AIRSPRING,
        capabilities: &[
            "ecology",
            "et0",
            "irrigation",
            "water_balance",
            "yield",
            "agriculture",
            "soil_science",
            "drought",
            "statistics",
        ],
    },
    // Game science domain (ludoSpring)
    CapabilityDomain {
        provider: LUDOSPRING,
        capabilities: &[
            "game",
            "ludology",
            "interaction_design",
            "procedural_generation",
            "accessibility_scoring",
            "engagement_metrics",
        ],
    },
    // Visualization domain (petalTongue)
    CapabilityDomain {
        provider: PETALTONGUE,
        capabilities: &[
            "visualization",
            "ui",
            "interaction",
            "representation",
            "sensor_stream",
        ],
    },
    // XR / Immersive domain (petalTongue + ludoSpring)
    CapabilityDomain {
        provider: PETALTONGUE,
        capabilities: &["xr", "stereo", "vr", "ar", "tracking", "haptic", "mocap"],
    },
    // Medical / Surgical domain (healthSpring)
    CapabilityDomain {
        provider: HEALTHSPRING,
        capabilities: &[
            "medical",
            "surgical",
            "anatomy",
            "tissue",
            "biosignal",
            "pharmacokinetics",
        ],
    },
    // Ephemeral workspace domain (rhizoCrypt)
    CapabilityDomain {
        provider: RHIZOCRYPT,
        capabilities: &[
            "ephemeral_workspace",
            "dag",
            "session",
            "merkle",
            "dehydration",
            "slice",
            "vertex",
        ],
    },
    // Permanent history domain (LoamSpine)
    CapabilityDomain {
        provider: LOAMSPINE,
        capabilities: &[
            "permanent_storage",
            "linear_history",
            "spine",
            "certificate",
            "temporal_anchor",
            "commit",
        ],
    },
    // Attribution domain (sweetGrass)
    CapabilityDomain {
        provider: SWEETGRASS,
        capabilities: &[
            "attribution",
            "braid",
            "provenance",
            "contribution",
            "privacy",
            "prov_export",
        ],
    },
    // Composition health domain (biomeOS — cross-cutting aggregate)
    // Canonical namespace for composed system health: tower, webb, service,
    // nucleus, and spring-specific health aggregation.
    CapabilityDomain {
        provider: "biomeos",
        capabilities: &["composition"],
    },
];

/// Config-driven capability → provider registry.
///
/// Loads `[domains.*]` sections from `config/capability_registry.toml`.
/// Falls back to the compiled-in `CAPABILITY_DOMAINS` const for capabilities
/// not found in the config.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    /// capability name → provider primal name (from TOML config)
    config_map: HashMap<String, String>,
}

impl CapabilityRegistry {
    /// Load domain mappings from the capability registry TOML.
    ///
    /// Parses every `[domains.*]` section looking for `provider` and
    /// `capabilities` keys, building a reverse index.
    pub fn from_toml(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let table: toml::Table = content.parse()?;
        let mut config_map = HashMap::new();

        if let Some(domains) = table.get("domains").and_then(|v| v.as_table()) {
            for (_domain_name, domain_value) in domains {
                if let Some(domain_table) = domain_value.as_table() {
                    let provider = domain_table
                        .get("provider")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    if provider.is_empty() || provider == "*" {
                        continue;
                    }
                    if let Some(caps) = domain_table.get("capabilities").and_then(|v| v.as_array())
                    {
                        for cap in caps {
                            if let Some(cap_str) = cap.as_str() {
                                config_map.insert(cap_str.to_string(), provider.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { config_map })
    }

    /// Resolve a capability to its provider.
    ///
    /// 1. Exact match in config
    /// 2. Prefix match in config (e.g. `crypto.encrypt` → `crypto`)
    /// 3. Compiled-in fallback table
    pub fn resolve(&self, capability: &str) -> Option<String> {
        if let Some(provider) = self.config_map.get(capability) {
            return Some(provider.clone());
        }

        if let Some(prefix) = capability.split('.').next() {
            if let Some(provider) = self.config_map.get(prefix) {
                return Some(provider.clone());
            }
        }

        capability_to_provider_fallback(capability).map(str::to_string)
    }

    /// Number of config-loaded entries (excludes compiled-in fallback).
    #[cfg(test)]
    pub fn config_entry_count(&self) -> usize {
        self.config_map.len()
    }
}

/// Resolve capability to provider using the compiled-in domain table.
///
/// This is the lowest-priority fallback when neither the neural-api router
/// nor the TOML config have a match. Prefer `CapabilityRegistry::resolve`.
pub fn capability_to_provider_fallback(capability: &str) -> Option<&'static str> {
    for domain in CAPABILITY_DOMAINS {
        if domain.capabilities.contains(&capability) {
            return Some(domain.provider);
        }
    }

    if let Some(prefix) = capability.split('.').next() {
        for domain in CAPABILITY_DOMAINS {
            if domain.capabilities.contains(&prefix) {
                return Some(domain.provider);
            }
        }
    }

    None
}
