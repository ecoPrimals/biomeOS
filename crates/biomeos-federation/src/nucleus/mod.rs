// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! NUCLEUS - Secure Primal Discovery Protocol
//!
//! Network-Universal Coordinated Lifecycle & Ecosystem Unification System
//!
//! This module provides secure, 5-layer primal discovery that delegates
//! cryptographic and communication responsibilities to BearDog and Songbird.
//!
//! ## Core Principle: Delegate, Don't Reimplement!
//!
//! | Capability | Primal | What It Provides |
//! |------------|--------|------------------|
//! | **Crypto & Identity** | 🐻 BearDog | Signatures, verification, trust evaluation |
//! | **Discovery & Comms** | 🐦 Songbird | UDP multicast, registry, routing |
//! | **Coordination** | 🌱 biomeOS | Orchestrates protocol, no reimplementation |
//!
//! ## 5-Layer Protocol
//!
//! 1. **Physical Discovery** (Songbird) - UDP multicast, socket scanning
//! 2. **Identity Verification** (BearDog) - Ed25519 challenge-response
//! 3. **Capability Verification** (biomeOS) - Query primal, validate capabilities
//! 4. **Trust Evaluation** (BearDog) - Genetic lineage, trust level
//! 5. **Registration** (biomeOS) - Add to verified primal registry

mod discovery;
mod trust;
mod verification;

pub use trust::TrustLevel;
pub use verification::{IdentityProof, UNVERIFIED_SIGNATURE};

use crate::beardog_client::BearDogClient;
use crate::capability::{Capability, CapabilitySet};
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery, PrimalEndpoint};
use crate::unix_socket_client::UnixSocketClient;
use crate::{FederationError, FederationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use self::discovery::{layer1_physical_discovery_sockets, layer1_physical_discovery_songbird};
use self::trust::layer4_trust_evaluation;
use self::verification::{layer2_identity_verification, layer3_capability_verification};

/// A verified primal (passed all 5 layers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedPrimal {
    /// Primal name (e.g., "songbird", "beardog")
    pub name: String,

    /// Node ID (e.g., "node-alpha")
    pub node_id: String,

    /// Family ID (e.g., "test_family")
    pub family_id: Option<String>,

    /// Connection endpoints
    pub endpoints: Vec<PrimalEndpoint>,

    /// Verified capabilities (queried from primal, not inferred)
    pub capabilities: CapabilitySet,

    /// Identity proof from BearDog
    pub identity_proof: IdentityProof,

    /// Trust level
    pub trust_level: TrustLevel,

    /// When it was discovered
    pub discovered_at: u64,

    /// When it was verified
    pub verified_at: u64,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Selection criteria for primal lookup
#[derive(Debug, Clone)]
pub enum SelectionCriteria {
    /// Any primal with this capability (highest trust level wins)
    ByCapability(Capability),

    /// Specific node ID
    ByNodeId(String),

    /// Specific family
    ByFamily(String),

    /// Specific socket path
    BySocket(PathBuf),

    /// Minimum trust level required
    MinTrustLevel(TrustLevel),

    /// Any (first available, for testing only)
    Any,
}

/// Secure primal discovery using 5-layer verification
pub struct SecureNucleusDiscovery {
    /// Songbird client (for Layer 1: Physical Discovery)
    songbird: Option<UnixSocketClient>,

    /// BearDog client (for Layer 2 & 4: Identity & Trust)
    beardog: Option<BearDogClient>,

    /// Verified primals (multiple instances per name possible)
    verified_primals: HashMap<String, Vec<VerifiedPrimal>>,

    /// Current family ID
    family_id: Option<String>,

    /// Planned: wire up for multi-node routing in Phase 3.
    _node_id: Option<String>,
}

impl SecureNucleusDiscovery {
    /// Create a new secure nucleus discovery
    pub fn new() -> Self {
        info!("🧬 Initializing NUCLEUS (Secure Discovery Protocol)");
        Self {
            songbird: None,
            beardog: None,
            verified_primals: HashMap::new(),
            family_id: None,
            _node_id: None,
        }
    }

    /// Create with Songbird and BearDog clients (delegated discovery)
    pub fn with_clients(
        songbird: Option<UnixSocketClient>,
        beardog: Option<BearDogClient>,
    ) -> Self {
        info!("🧬 Initializing NUCLEUS with primal clients");
        Self {
            songbird,
            beardog,
            verified_primals: HashMap::new(),
            family_id: std::env::var("FAMILY_ID").ok(),
            _node_id: std::env::var("NODE_ID").ok(),
        }
    }

    /// Discover primals using insecure basic discovery (for bootstrapping)
    ///
    /// This is used when Songbird/BearDog are not yet available.
    /// It falls back to basic socket scanning without verification.
    #[expect(clippy::expect_used, reason = "system clock before UNIX epoch")]
    pub async fn discover_insecure(&mut self) -> FederationResult<Vec<VerifiedPrimal>> {
        warn!("⚠️  Using insecure discovery (no Songbird/BearDog verification)");
        warn!("   This should only be used for bootstrapping!");

        let mut basic_discovery = PrimalDiscovery::new();
        let discovered = basic_discovery.discover().await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();

        for primal in discovered {
            let name = primal.name;
            let verified = VerifiedPrimal {
                name: name.clone(),
                node_id: "unknown".to_string(),
                family_id: None,
                endpoints: primal.endpoints,
                capabilities: primal.capabilities,
                identity_proof: IdentityProof {
                    node_id: "unknown".to_string(),
                    family_id: None,
                    signature: verification::UNVERIFIED_SIGNATURE.to_string(),
                    challenge: "none".to_string(),
                    public_key: "none".to_string(),
                    timestamp: now,
                },
                trust_level: TrustLevel::Unknown,
                discovered_at: now,
                verified_at: now,
                metadata: primal.metadata,
            };

            self.verified_primals
                .entry(name)
                .or_default()
                .push(verified);
        }

        Ok(self.all())
    }

    /// Discover primals using secure 5-layer protocol
    ///
    /// **Requires**: Songbird and BearDog must be available
    ///
    /// ## Layers:
    /// 1. Physical Discovery (Songbird)
    /// 2. Identity Verification (BearDog)
    /// 3. Capability Verification (biomeOS)
    /// 4. Trust Evaluation (BearDog)
    /// 5. Registration (biomeOS)
    pub async fn discover_secure(&mut self) -> FederationResult<Vec<VerifiedPrimal>> {
        info!("🔒 Starting secure 5-layer discovery");

        if self.songbird.is_none() && self.beardog.is_none() {
            return Err(FederationError::DiscoveryError(
                "Cannot perform secure discovery without Songbird or BearDog".to_string(),
            ));
        }

        let discovered = if let Some(ref songbird) = self.songbird {
            layer1_physical_discovery_songbird(songbird, self.family_id.as_deref()).await?
        } else {
            layer1_physical_discovery_sockets().await?
        };

        info!("   Layer 1: Discovered {} primals", discovered.len());

        for primal in discovered {
            match self.verify_primal(primal).await {
                Ok(verified) => {
                    info!(
                        "   ✅ Verified: {} (trust: {:?})",
                        verified.name, verified.trust_level
                    );
                    let name = verified.name.clone();
                    self.verified_primals
                        .entry(name)
                        .or_default()
                        .push(verified);
                }
                Err(e) => {
                    warn!("   ❌ Failed to verify primal: {}", e);
                }
            }
        }

        info!(
            "🔒 Secure discovery complete: {} verified",
            self.verified_primals.len()
        );

        Ok(self.all())
    }

    /// Verify a primal through layers 2-5
    #[expect(clippy::expect_used, reason = "system clock before UNIX epoch")]
    async fn verify_primal(&self, primal: DiscoveredPrimal) -> FederationResult<VerifiedPrimal> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();

        let identity_proof = if let Some(ref beardog) = self.beardog {
            layer2_identity_verification(beardog, &primal).await?
        } else {
            IdentityProof {
                node_id: primal.name.clone(),
                family_id: None,
                signature: verification::UNVERIFIED_SIGNATURE.to_string(),
                challenge: "none".to_string(),
                public_key: "none".to_string(),
                timestamp: now,
            }
        };

        let capabilities = layer3_capability_verification(&primal).await?;

        let trust_level = if let Some(ref beardog) = self.beardog {
            layer4_trust_evaluation(beardog, &identity_proof, self.family_id.as_deref()).await?
        } else {
            TrustLevel::Basic
        };

        Ok(VerifiedPrimal {
            name: primal.name,
            node_id: identity_proof.node_id.clone(),
            family_id: identity_proof.family_id.clone(),
            endpoints: primal.endpoints,
            capabilities,
            identity_proof,
            trust_level,
            discovered_at: now,
            verified_at: now,
            metadata: primal.metadata,
        })
    }

    /// Get a primal by selection criteria
    pub fn get(&self, criteria: SelectionCriteria) -> Option<&VerifiedPrimal> {
        match criteria {
            SelectionCriteria::ByCapability(cap) => {
                self.verified_primals
                    .values()
                    .flat_map(|primals| primals.iter())
                    .filter(|p| p.capabilities.has(&cap))
                    .max_by_key(|p| p.trust_level)
            }
            SelectionCriteria::ByNodeId(node_id) => {
                self.verified_primals
                    .values()
                    .flat_map(|primals| primals.iter())
                    .find(|p| p.node_id == node_id)
            }
            SelectionCriteria::ByFamily(family_id) => {
                self.verified_primals
                    .values()
                    .flat_map(|primals| primals.iter())
                    .find(|p| p.family_id.as_ref() == Some(&family_id))
            }
            SelectionCriteria::BySocket(socket_path) => {
                self.verified_primals
                    .values()
                    .flat_map(|primals| primals.iter())
                    .find(|p| {
                        p.endpoints.iter().any(|ep| {
                            matches!(ep, PrimalEndpoint::UnixSocket { path } if path == &socket_path)
                        })
                    })
            }
            SelectionCriteria::MinTrustLevel(min_trust) => {
                self.verified_primals
                    .values()
                    .flat_map(|primals| primals.iter())
                    .filter(|p| p.trust_level >= min_trust)
                    .max_by_key(|p| p.trust_level)
            }
            SelectionCriteria::Any => {
                self.verified_primals
                    .values()
                    .flat_map(|primals| primals.iter())
                    .next()
            }
        }
    }

    /// Get all instances of a primal by name
    pub fn get_all(&self, name: &str) -> Vec<&VerifiedPrimal> {
        self.verified_primals
            .get(name)
            .map(|primals| primals.iter().collect())
            .unwrap_or_default()
    }

    /// Get all verified primals
    pub fn all(&self) -> Vec<VerifiedPrimal> {
        self.verified_primals
            .values()
            .flat_map(|primals| primals.iter().cloned())
            .collect()
    }

    /// Get primals with a specific capability
    pub fn with_capability(&self, cap: &Capability) -> Vec<&VerifiedPrimal> {
        self.verified_primals
            .values()
            .flat_map(|primals| primals.iter())
            .filter(|p| p.capabilities.has(cap))
            .collect()
    }
}

impl Default for SecureNucleusDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureNucleusDiscovery {
    /// Test-only method to inject a verified primal
    ///
    /// This method is only available in test builds and allows
    /// injecting verified primals directly into the registry for testing.
    #[doc(hidden)]
    pub fn inject_primal_for_testing(mut self, primal: VerifiedPrimal) -> Self {
        self.verified_primals
            .entry(primal.name.clone())
            .or_default()
            .push(primal);
        self
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    fn make_verified(
        name: &str,
        node_id: &str,
        family_id: Option<&str>,
        trust: TrustLevel,
        caps: Vec<Capability>,
        endpoints: Vec<PrimalEndpoint>,
    ) -> VerifiedPrimal {
        let mut cap_set = CapabilitySet::new();
        for c in caps {
            cap_set.add(c);
        }
        VerifiedPrimal {
            name: name.into(),
            node_id: node_id.into(),
            family_id: family_id.map(String::from),
            endpoints,
            capabilities: cap_set,
            identity_proof: IdentityProof {
                node_id: node_id.into(),
                family_id: family_id.map(String::from),
                signature: "test-sig".into(),
                challenge: "test-challenge".into(),
                public_key: "test-pubkey".into(),
                timestamp: 1000,
            },
            trust_level: trust,
            discovered_at: 1000,
            verified_at: 1001,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_selection_criteria_debug() {
        let c = SelectionCriteria::ByCapability(Capability::Storage);
        assert!(format!("{c:?}").contains("ByCapability"));

        let c2 = SelectionCriteria::ByNodeId("node-1".into());
        assert!(format!("{c2:?}").contains("node-1"));

        let c3 = SelectionCriteria::Any;
        assert!(format!("{c3:?}").contains("Any"));
    }

    #[test]
    fn test_selection_criteria_clone() {
        let c = SelectionCriteria::ByFamily("fam-1".into());
        let c2 = c;
        assert!(format!("{c2:?}").contains("fam-1"));
    }

    #[test]
    fn test_verified_primal_serde_roundtrip() {
        let vp = make_verified(
            "beardog",
            "node-bd",
            Some("fam-x"),
            TrustLevel::High,
            vec![Capability::Storage, Capability::Compute],
            vec![PrimalEndpoint::UnixSocket {
                path: PathBuf::from("/tmp/beardog.sock"),
            }],
        );
        let json = serde_json::to_string(&vp).expect("serialize");
        let restored: VerifiedPrimal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.name, "beardog");
        assert_eq!(restored.node_id, "node-bd");
        assert_eq!(restored.trust_level, TrustLevel::High);
        assert!(restored.capabilities.has(&Capability::Storage));
    }

    #[test]
    fn test_verified_primal_clone() {
        let vp = make_verified("test", "n", None, TrustLevel::Basic, vec![], vec![]);
        let c = vp.clone();
        assert_eq!(c.name, vp.name);
    }

    #[test]
    fn test_new_creates_empty() {
        let disc = SecureNucleusDiscovery::new();
        assert!(disc.verified_primals.is_empty());
        assert!(disc.songbird.is_none());
        assert!(disc.beardog.is_none());
        assert!(disc.family_id.is_none());
    }

    #[test]
    fn test_default_same_as_new() {
        let disc = SecureNucleusDiscovery::default();
        assert!(disc.verified_primals.is_empty());
    }

    #[test]
    fn test_inject_single_primal() {
        let vp = make_verified("songbird", "s1", None, TrustLevel::Basic, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

        assert_eq!(disc.all().len(), 1);
        assert_eq!(disc.get_all("songbird").len(), 1);
    }

    #[test]
    fn test_inject_multiple_instances_same_name() {
        let vp1 = make_verified("beardog", "bd-1", None, TrustLevel::Basic, vec![], vec![]);
        let vp2 = make_verified("beardog", "bd-2", None, TrustLevel::High, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new()
            .inject_primal_for_testing(vp1)
            .inject_primal_for_testing(vp2);

        assert_eq!(disc.get_all("beardog").len(), 2);
    }

    #[test]
    fn test_get_by_capability_returns_highest_trust() {
        let low = make_verified(
            "a",
            "a1",
            None,
            TrustLevel::Basic,
            vec![Capability::Storage],
            vec![],
        );
        let high = make_verified(
            "b",
            "b1",
            None,
            TrustLevel::High,
            vec![Capability::Storage],
            vec![],
        );
        let disc = SecureNucleusDiscovery::new()
            .inject_primal_for_testing(low)
            .inject_primal_for_testing(high);

        let result = disc.get(SelectionCriteria::ByCapability(Capability::Storage));
        assert!(result.is_some());
        assert_eq!(result.expect("should find").trust_level, TrustLevel::High);
    }

    #[test]
    fn test_get_by_capability_none_when_missing() {
        let disc = SecureNucleusDiscovery::new();
        assert!(
            disc.get(SelectionCriteria::ByCapability(Capability::Compute))
                .is_none()
        );
    }

    #[test]
    fn test_get_by_node_id() {
        let vp = make_verified("x", "node-alpha", None, TrustLevel::Basic, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

        let found = disc.get(SelectionCriteria::ByNodeId("node-alpha".into()));
        assert!(found.is_some());
        assert_eq!(found.expect("should find").node_id, "node-alpha");
    }

    #[test]
    fn test_get_by_node_id_not_found() {
        let disc = SecureNucleusDiscovery::new();
        assert!(
            disc.get(SelectionCriteria::ByNodeId("nope".into()))
                .is_none()
        );
    }

    #[test]
    fn test_get_by_family() {
        let vp = make_verified(
            "svc",
            "n1",
            Some("family-east"),
            TrustLevel::High,
            vec![],
            vec![],
        );
        let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

        let found = disc.get(SelectionCriteria::ByFamily("family-east".into()));
        assert!(found.is_some());
    }

    #[test]
    fn test_get_by_family_no_match() {
        let vp = make_verified("svc", "n1", Some("west"), TrustLevel::Basic, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

        assert!(
            disc.get(SelectionCriteria::ByFamily("east".into()))
                .is_none()
        );
    }

    #[test]
    fn test_get_by_socket() {
        let sock_path = PathBuf::from("/run/biomeos/beardog.sock");
        let vp = make_verified(
            "beardog",
            "bd",
            None,
            TrustLevel::Basic,
            vec![],
            vec![PrimalEndpoint::UnixSocket {
                path: sock_path.clone(),
            }],
        );
        let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

        let found = disc.get(SelectionCriteria::BySocket(sock_path));
        assert!(found.is_some());
        assert_eq!(found.expect("should find").name, "beardog");
    }

    #[test]
    fn test_get_by_socket_no_match() {
        let disc = SecureNucleusDiscovery::new();
        assert!(
            disc.get(SelectionCriteria::BySocket(PathBuf::from("/nope")))
                .is_none()
        );
    }

    #[test]
    fn test_get_min_trust_level() {
        let low = make_verified("a", "a1", None, TrustLevel::Basic, vec![], vec![]);
        let high = make_verified("b", "b1", None, TrustLevel::Highest, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new()
            .inject_primal_for_testing(low)
            .inject_primal_for_testing(high);

        let found = disc.get(SelectionCriteria::MinTrustLevel(TrustLevel::High));
        assert!(found.is_some());
        assert_eq!(found.expect("should find").trust_level, TrustLevel::Highest);
    }

    #[test]
    fn test_get_min_trust_level_none_below() {
        let low = make_verified("a", "a1", None, TrustLevel::Basic, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(low);

        assert!(
            disc.get(SelectionCriteria::MinTrustLevel(TrustLevel::High))
                .is_none()
        );
    }

    #[test]
    fn test_get_any() {
        let vp = make_verified("svc", "n", None, TrustLevel::Basic, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

        assert!(disc.get(SelectionCriteria::Any).is_some());
    }

    #[test]
    fn test_get_any_empty() {
        let disc = SecureNucleusDiscovery::new();
        assert!(disc.get(SelectionCriteria::Any).is_none());
    }

    #[test]
    fn test_with_capability() {
        let vp1 = make_verified(
            "store",
            "s1",
            None,
            TrustLevel::Basic,
            vec![Capability::Storage],
            vec![],
        );
        let vp2 = make_verified(
            "compute",
            "c1",
            None,
            TrustLevel::Basic,
            vec![Capability::Compute],
            vec![],
        );
        let disc = SecureNucleusDiscovery::new()
            .inject_primal_for_testing(vp1)
            .inject_primal_for_testing(vp2);

        let storage_providers = disc.with_capability(&Capability::Storage);
        assert_eq!(storage_providers.len(), 1);
        assert_eq!(storage_providers[0].name, "store");

        let compute_providers = disc.with_capability(&Capability::Compute);
        assert_eq!(compute_providers.len(), 1);

        let discovery_providers = disc.with_capability(&Capability::Discovery);
        assert!(discovery_providers.is_empty());
    }

    #[test]
    fn test_all_empty() {
        let disc = SecureNucleusDiscovery::new();
        assert!(disc.all().is_empty());
    }

    #[test]
    fn test_all_returns_all() {
        let vp1 = make_verified("a", "a1", None, TrustLevel::Basic, vec![], vec![]);
        let vp2 = make_verified("b", "b1", None, TrustLevel::High, vec![], vec![]);
        let disc = SecureNucleusDiscovery::new()
            .inject_primal_for_testing(vp1)
            .inject_primal_for_testing(vp2);

        assert_eq!(disc.all().len(), 2);
    }

    #[test]
    fn test_get_all_unknown_name() {
        let disc = SecureNucleusDiscovery::new();
        assert!(disc.get_all("nonexistent").is_empty());
    }

    #[tokio::test]
    async fn test_discover_secure_requires_clients() {
        let mut disc = SecureNucleusDiscovery::new();
        let result = disc.discover_secure().await;
        assert!(
            result.is_err(),
            "secure discovery without clients must fail"
        );
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Songbird") || err_msg.contains("BearDog"));
    }
}
