// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Composition intelligence — tier classification and named patterns.
//!
//! Evolved from primalSpring's exploratory `neural_routing` module into
//! biomeOS runtime. primalSpring studied the 452-method surface and
//! discovered the tier/pattern structure; biomeOS now owns it as the
//! runtime substrate for dispatch decisions.
//!
//! # Tiers
//!
//! Every capability domain maps to a composition tier — which atomic
//! deployment pattern it participates in. This informs:
//! - Signal graph selection (`tier.signal` → `graphs/signals/`)
//! - Provider grouping for `discover_tower_atomic`, `discover_nest_atomic`
//! - Graph pre-staging and parallelism decisions
//! - Self-announcing primal tier membership validation
//!
//! # Patterns
//!
//! Named composition patterns describe method sequences that form
//! emergent systems. They can be executed as graphs and are the
//! building blocks for adaptive dispatch optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Composition tier — which atomic deployment a capability participates in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionTier {
    /// Tower: security + discovery + defense (bearDog, songbird, skunkBat)
    Tower,
    /// Node: Tower + compute (+ toadStool, barraCuda, coralReef)
    Node,
    /// Nest: Tower + storage + provenance (+ nestgate, rhizoCrypt, loamSpine, sweetGrass)
    Nest,
    /// NUCLEUS: all primals composed
    Nucleus,
    /// Meta: observability + AI (petalTongue, squirrel)
    Meta,
    /// Orchestration: biomeOS infrastructure (lifecycle, topology, federation)
    Orchestration,
    /// Standalone: method works with a single primal, no tier composition
    Standalone,
}

impl CompositionTier {
    /// Classify a capability domain + provider into its composition tier.
    pub fn classify(domain: &str, provider: &str) -> Self {
        match domain {
            "crypto" | "security" | "auth" | "btsp" | "fido2" | "genetic"
            | "beacon" | "lineage" | "tls" | "birdsong" | "identity"
            | "encryption" | "jwt" => Self::Tower,
            "discovery" | "network" | "stun" | "onion" | "tor" | "mesh"
            | "http" | "relay" | "dns" | "turn" | "ipc" => Self::Tower,
            "defense" | "recon" | "threat" | "audit" => Self::Tower,
            "compute" | "dispatch" | "toadstool" | "sovereign"
            | "execution" | "parsing" | "hardware_learning" | "workload" => Self::Node,
            "tensor" | "math" | "ode" | "ml" | "nautilus" | "rng"
            | "stats" | "linalg" | "spectral" | "noise" | "shader"
            | "activation" | "wgsl" | "spirv" | "fhe" => Self::Node,
            "storage" | "content" | "secrets" | "versioning"
            | "persistence" | "publishing" => Self::Nest,
            "dag" | "spine" | "event" | "entry" | "session"
            | "certificate" | "permanence" | "proof" | "anchor"
            | "merkle" | "vertex" | "dehydration" | "slice" => Self::Nest,
            "braid" | "anchoring" | "provenance" | "attribution"
            | "contribution" | "ledger" | "commit" => Self::Nest,
            "visualization" | "render" | "viz" | "interaction"
            | "proprioception" => Self::Meta,
            "ai" | "inference" | "squirrel" | "context" | "science" => Self::Meta,
            "orchestration" | "federation" | "biomeos" | "primal"
            | "signal" | "topology" | "route" | "system"
            | "neural_api" => Self::Orchestration,
            "health" | "capabilities" | "lifecycle" | "mcp"
            | "tool" | "tools" | "rpc" | "coordination"
            | "graph" | "nucleus" | "membrane" | "cell" => Self::Orchestration,
            _ => Self::from_provider(provider),
        }
    }

    fn from_provider(provider: &str) -> Self {
        match provider {
            "beardog" | "songbird" | "skunkbat" => Self::Tower,
            "toadstool" | "barracuda" | "coralreef" => Self::Node,
            "nestgate" | "rhizocrypt" | "loamspine" | "sweetgrass" => Self::Nest,
            "petaltongue" | "squirrel" => Self::Meta,
            "biomeos" => Self::Orchestration,
            _ => Self::Standalone,
        }
    }

    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Tower => "tower",
            Self::Node => "node",
            Self::Nest => "nest",
            Self::Nucleus => "nucleus",
            Self::Meta => "meta",
            Self::Orchestration => "orchestration",
            Self::Standalone => "standalone",
        }
    }

    /// Whether this tier corresponds to a signal tier (graphs/signals/).
    pub const fn is_signal_tier(&self) -> bool {
        matches!(self, Self::Tower | Self::Node | Self::Nest | Self::Meta)
    }
}

/// A named composition pattern — a method sequence that forms an emergent system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPattern {
    /// Pattern name (e.g., "rootpulse_commit").
    pub name: Arc<str>,
    /// Ordered method sequence (topological dependency order).
    pub methods: Vec<Arc<str>>,
    /// Primals involved.
    pub primals: Vec<Arc<str>>,
    /// Which tier this pattern belongs to.
    pub tier: CompositionTier,
    /// Optional graph file path (if backed by a signal graph).
    pub graph_file: Option<String>,
}

/// Registry of known composition patterns.
///
/// Patterns are registered at startup from deploy graphs and signal
/// definitions. primal.announce can extend this at runtime as new
/// primals declare their participation in patterns.
#[derive(Debug, Default)]
pub struct CompositionPatternRegistry {
    patterns: HashMap<Arc<str>, CompositionPattern>,
}

impl CompositionPatternRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry seeded with canonical patterns.
    pub fn with_canonical_patterns() -> Self {
        let mut registry = Self::new();

        registry.register(CompositionPattern {
            name: Arc::from("rootpulse_commit"),
            methods: vec![
                Arc::from("crypto.sign"),
                Arc::from("dag.event.append"),
                Arc::from("braid.anchor"),
                Arc::from("spine.commit"),
            ],
            primals: vec![
                Arc::from("beardog"),
                Arc::from("rhizocrypt"),
                Arc::from("sweetgrass"),
                Arc::from("loamspine"),
            ],
            tier: CompositionTier::Nest,
            graph_file: None,
        });

        registry.register(CompositionPattern {
            name: Arc::from("tower_atomic_bootstrap"),
            methods: vec![
                Arc::from("crypto.sign_ed25519"),
                Arc::from("discovery.announce"),
                Arc::from("security.audit_event"),
            ],
            primals: vec![
                Arc::from("beardog"),
                Arc::from("songbird"),
                Arc::from("skunkbat"),
            ],
            tier: CompositionTier::Tower,
            graph_file: Some("graphs/tower_atomic_bootstrap.toml".to_owned()),
        });

        registry.register(CompositionPattern {
            name: Arc::from("nest_store"),
            methods: vec![
                Arc::from("content.put"),
                Arc::from("dag.event.append"),
                Arc::from("spine.seal"),
                Arc::from("braid.create"),
            ],
            primals: vec![
                Arc::from("nestgate"),
                Arc::from("rhizocrypt"),
                Arc::from("loamspine"),
                Arc::from("sweetgrass"),
            ],
            tier: CompositionTier::Nest,
            graph_file: Some("graphs/signals/nest_store.toml".to_owned()),
        });

        registry.register(CompositionPattern {
            name: Arc::from("tower_publish"),
            methods: vec![
                Arc::from("crypto.sign"),
                Arc::from("discovery.announce"),
                Arc::from("security.audit_event"),
            ],
            primals: vec![
                Arc::from("beardog"),
                Arc::from("songbird"),
                Arc::from("skunkbat"),
            ],
            tier: CompositionTier::Tower,
            graph_file: Some("graphs/signals/tower_publish.toml".to_owned()),
        });

        registry.register(CompositionPattern {
            name: Arc::from("meta_observe"),
            methods: vec![
                Arc::from("visualization.session.create"),
                Arc::from("context.push"),
                Arc::from("graph.list"),
            ],
            primals: vec![
                Arc::from("petaltongue"),
                Arc::from("squirrel"),
                Arc::from("biomeos"),
            ],
            tier: CompositionTier::Meta,
            graph_file: Some("graphs/signals/meta_observe.toml".to_owned()),
        });

        registry.register(CompositionPattern {
            name: Arc::from("ionic_bond_lifecycle"),
            methods: vec![
                Arc::from("bonding.propose"),
                Arc::from("crypto.ionic_bond.verify_proposal"),
                Arc::from("bonding.accept"),
                Arc::from("bonding.status"),
                Arc::from("bonding.terminate"),
            ],
            primals: vec![
                Arc::from("primalspring"),
                Arc::from("beardog"),
            ],
            tier: CompositionTier::Standalone,
            graph_file: None,
        });

        registry
    }

    /// Register a pattern.
    pub fn register(&mut self, pattern: CompositionPattern) {
        self.patterns.insert(pattern.name.clone(), pattern);
    }

    /// Look up a pattern by name.
    pub fn get(&self, name: &str) -> Option<&CompositionPattern> {
        self.patterns.get(name)
    }

    /// All registered patterns.
    pub fn all(&self) -> Vec<&CompositionPattern> {
        self.patterns.values().collect()
    }

    /// Patterns involving a specific primal.
    pub fn involving_primal(&self, primal: &str) -> Vec<&CompositionPattern> {
        self.patterns
            .values()
            .filter(|p| p.primals.iter().any(|pr| pr.as_ref() == primal))
            .collect()
    }

    /// Patterns in a specific tier.
    pub fn in_tier(&self, tier: CompositionTier) -> Vec<&CompositionPattern> {
        self.patterns
            .values()
            .filter(|p| p.tier == tier)
            .collect()
    }

    /// Number of registered patterns.
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// JSON snapshot for RPC responses.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "patterns": self.patterns.values().collect::<Vec<_>>(),
            "count": self.patterns.len(),
        })
    }
}

/// Tier composition summary — what primals and domains are needed
/// to deploy a specific tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCompositionPlan {
    /// Which tier.
    pub tier: CompositionTier,
    /// Primals that must be deployed.
    pub required_primals: Vec<Arc<str>>,
    /// Capability domains covered.
    pub domains: Vec<Arc<str>>,
    /// Patterns available in this tier.
    pub available_patterns: Vec<Arc<str>>,
}

/// Generate a composition plan for a tier from a pattern registry.
pub fn plan_tier(
    tier: CompositionTier,
    registry: &CompositionPatternRegistry,
) -> TierCompositionPlan {
    let patterns = registry.in_tier(tier);
    let mut primals: Vec<Arc<str>> = patterns
        .iter()
        .flat_map(|p| p.primals.iter().cloned())
        .collect();
    primals.sort();
    primals.dedup();

    let pattern_names: Vec<Arc<str>> = patterns
        .iter()
        .map(|p| p.name.clone())
        .collect();

    let domains = match tier {
        CompositionTier::Tower => vec![
            Arc::from("security"), Arc::from("crypto"), Arc::from("discovery"),
            Arc::from("network"), Arc::from("defense"),
        ],
        CompositionTier::Node => vec![
            Arc::from("compute"), Arc::from("tensor"), Arc::from("shader"),
        ],
        CompositionTier::Nest => vec![
            Arc::from("storage"), Arc::from("content"), Arc::from("dag"),
            Arc::from("provenance"), Arc::from("braid"),
        ],
        CompositionTier::Meta => vec![
            Arc::from("visualization"), Arc::from("ai"), Arc::from("science"),
        ],
        CompositionTier::Orchestration => vec![
            Arc::from("orchestration"), Arc::from("lifecycle"), Arc::from("topology"),
        ],
        CompositionTier::Nucleus | CompositionTier::Standalone => vec![],
    };

    TierCompositionPlan {
        tier,
        required_primals: primals,
        domains,
        available_patterns: pattern_names,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_security_domains() {
        assert_eq!(CompositionTier::classify("crypto", "beardog"), CompositionTier::Tower);
        assert_eq!(CompositionTier::classify("security", "beardog"), CompositionTier::Tower);
        assert_eq!(CompositionTier::classify("genetic", "beardog"), CompositionTier::Tower);
    }

    #[test]
    fn classify_compute_domains() {
        assert_eq!(CompositionTier::classify("compute", "toadstool"), CompositionTier::Node);
        assert_eq!(CompositionTier::classify("tensor", "barracuda"), CompositionTier::Node);
        assert_eq!(CompositionTier::classify("shader", "coralreef"), CompositionTier::Node);
    }

    #[test]
    fn classify_storage_domains() {
        assert_eq!(CompositionTier::classify("storage", "nestgate"), CompositionTier::Nest);
        assert_eq!(CompositionTier::classify("dag", "rhizocrypt"), CompositionTier::Nest);
        assert_eq!(CompositionTier::classify("braid", "sweetgrass"), CompositionTier::Nest);
    }

    #[test]
    fn classify_meta_domains() {
        assert_eq!(CompositionTier::classify("ai", "squirrel"), CompositionTier::Meta);
        assert_eq!(CompositionTier::classify("visualization", "petaltongue"), CompositionTier::Meta);
        assert_eq!(CompositionTier::classify("science", "neuralspring"), CompositionTier::Meta);
    }

    #[test]
    fn classify_orchestration() {
        assert_eq!(CompositionTier::classify("orchestration", "biomeos"), CompositionTier::Orchestration);
        assert_eq!(CompositionTier::classify("neural_api", "biomeos"), CompositionTier::Orchestration);
        assert_eq!(CompositionTier::classify("lifecycle", "biomeos"), CompositionTier::Orchestration);
    }

    #[test]
    fn classify_unknown_falls_to_provider() {
        assert_eq!(CompositionTier::classify("unknown_domain", "beardog"), CompositionTier::Tower);
        assert_eq!(CompositionTier::classify("unknown_domain", "nestgate"), CompositionTier::Nest);
        assert_eq!(CompositionTier::classify("unknown_domain", "unknown"), CompositionTier::Standalone);
    }

    #[test]
    fn signal_tiers() {
        assert!(CompositionTier::Tower.is_signal_tier());
        assert!(CompositionTier::Node.is_signal_tier());
        assert!(CompositionTier::Nest.is_signal_tier());
        assert!(CompositionTier::Meta.is_signal_tier());
        assert!(!CompositionTier::Orchestration.is_signal_tier());
        assert!(!CompositionTier::Standalone.is_signal_tier());
    }

    #[test]
    fn canonical_patterns_loaded() {
        let reg = CompositionPatternRegistry::with_canonical_patterns();
        assert!(reg.len() >= 6);
        assert!(reg.get("rootpulse_commit").is_some());
        assert!(reg.get("tower_atomic_bootstrap").is_some());
        assert!(reg.get("nest_store").is_some());
    }

    #[test]
    fn patterns_involving_beardog() {
        let reg = CompositionPatternRegistry::with_canonical_patterns();
        let involving = reg.involving_primal("beardog");
        assert!(involving.len() >= 3, "beardog participates in 3+ patterns");
    }

    #[test]
    fn patterns_in_nest_tier() {
        let reg = CompositionPatternRegistry::with_canonical_patterns();
        let nest = reg.in_tier(CompositionTier::Nest);
        assert!(nest.len() >= 2, "Nest tier has 2+ patterns");
    }

    #[test]
    fn plan_tower_tier() {
        let reg = CompositionPatternRegistry::with_canonical_patterns();
        let plan = plan_tier(CompositionTier::Tower, &reg);
        assert!(plan.required_primals.contains(&Arc::from("beardog")));
        assert!(plan.required_primals.contains(&Arc::from("songbird")));
        assert!(plan.domains.contains(&Arc::from("security")));
    }

    #[test]
    fn plan_nest_tier() {
        let reg = CompositionPatternRegistry::with_canonical_patterns();
        let plan = plan_tier(CompositionTier::Nest, &reg);
        assert!(plan.required_primals.contains(&Arc::from("nestgate")));
        assert!(plan.required_primals.contains(&Arc::from("loamspine")));
    }

    #[test]
    fn pattern_json_snapshot() {
        let reg = CompositionPatternRegistry::with_canonical_patterns();
        let json = reg.to_json();
        assert!(json["count"].as_u64().unwrap() >= 6);
    }
}
