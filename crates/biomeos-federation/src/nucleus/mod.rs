// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! NUCLEUS - Secure Primal Discovery Protocol
//!
//! Network-Universal Coordinated Lifecycle & Ecosystem Unification System
//!
//! This module provides secure, 5-layer primal discovery that delegates
//! cryptographic and communication responsibilities to the security and discovery primals.
//!
//! ## Core Principle: Delegate, Don't Reimplement!
//!
//! | Capability | Primal | What It Provides |
//! |------------|--------|------------------|
//! | **Crypto & Identity** | Security provider | Signatures, verification, trust evaluation |
//! | **Discovery & Comms** | Discovery primal | UDP multicast, registry, routing |
//! | **Coordination** | 🌱 biomeOS | Orchestrates protocol, no reimplementation |
//!
//! ## 5-Layer Protocol
//!
//! 1. **Physical Discovery** (Songbird) - UDP multicast, socket scanning
//! 2. **Identity Verification** (security provider) - Ed25519 challenge-response
//! 3. **Capability Verification** (biomeOS) - Query primal, validate capabilities
//! 4. **Trust Evaluation** (security provider) - Genetic lineage, trust level
//! 5. **Registration** (biomeOS) - Add to verified primal registry

mod discovery;
mod trust;
mod verification;

pub use trust::TrustLevel;
pub use verification::{IdentityProof, UNVERIFIED_SIGNATURE};

use crate::capability::{Capability, CapabilitySet};
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery, PrimalEndpoint};
use crate::security_client::SecurityProviderClient;
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

    /// Family ID (e.g., "`test_family`")
    pub family_id: Option<String>,

    /// Connection endpoints
    pub endpoints: Vec<PrimalEndpoint>,

    /// Verified capabilities (queried from primal, not inferred)
    pub capabilities: CapabilitySet,

    /// Identity proof from the security provider
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

    /// Security provider client (for Layer 2 & 4: Identity & Trust)
    security_client: Option<SecurityProviderClient>,

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
            security_client: None,
            verified_primals: HashMap::new(),
            family_id: None,
            _node_id: None,
        }
    }

    /// Create with discovery (Songbird) and security provider clients (delegated discovery)
    pub fn with_clients(
        songbird: Option<UnixSocketClient>,
        security_client: Option<SecurityProviderClient>,
    ) -> Self {
        info!("🧬 Initializing NUCLEUS with primal clients");
        Self {
            songbird,
            security_client,
            verified_primals: HashMap::new(),
            family_id: std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY).ok(),
            _node_id: std::env::var(biomeos_types::env_config::vars::NODE_ID_LEGACY).ok(),
        }
    }

    /// Discover primals using insecure basic discovery (for bootstrapping)
    ///
    /// This is used when discovery / security clients are not yet available.
    /// It falls back to basic socket scanning without verification.
    #[expect(clippy::expect_used, reason = "system clock before UNIX epoch")]
    pub async fn discover_insecure(&mut self) -> FederationResult<Vec<VerifiedPrimal>> {
        warn!("⚠️  Using insecure discovery (no discovery/security verification)");
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
    /// **Requires**: Discovery primal and/or security provider must be available
    ///
    /// ## Layers:
    /// 1. Physical Discovery (discovery primal)
    /// 2. Identity Verification (security provider)
    /// 3. Capability Verification (biomeOS)
    /// 4. Trust Evaluation (security provider)
    /// 5. Registration (biomeOS)
    pub async fn discover_secure(&mut self) -> FederationResult<Vec<VerifiedPrimal>> {
        info!("🔒 Starting secure 5-layer discovery");

        if self.songbird.is_none() && self.security_client.is_none() {
            return Err(FederationError::DiscoveryError(
                "Cannot perform secure discovery without discovery primal or security provider"
                    .to_string(),
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

        let identity_proof = if let Some(ref security_client) = self.security_client {
            layer2_identity_verification(security_client, &primal).await?
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

        let trust_level = if let Some(ref security_client) = self.security_client {
            layer4_trust_evaluation(security_client, &identity_proof, self.family_id.as_deref())
                .await?
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
    #[must_use]
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
    #[must_use]
    pub fn get_all(&self, name: &str) -> Vec<&VerifiedPrimal> {
        self.verified_primals
            .get(name)
            .map(|primals| primals.iter().collect())
            .unwrap_or_default()
    }

    /// Get all verified primals
    #[must_use]
    pub fn all(&self) -> Vec<VerifiedPrimal> {
        self.verified_primals
            .values()
            .flat_map(|primals| primals.iter().cloned())
            .collect()
    }

    /// Get primals with a specific capability
    #[must_use]
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
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn inject_primal_for_testing(mut self, primal: VerifiedPrimal) -> Self {
        self.verified_primals
            .entry(primal.name.clone())
            .or_default()
            .push(primal);
        self
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

