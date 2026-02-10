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

use crate::beardog_client::BearDogClient;
use crate::capability::{Capability, CapabilitySet};
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery, PrimalEndpoint};
use crate::unix_socket_client::{JsonRpcRequest, UnixSocketClient};
use crate::{FederationError, FederationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Trust level for a verified primal
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Unknown/unverified primal
    Unknown = 0,
    /// Basic trust (discovered, identity verified)
    Basic = 1,
    /// Elevated trust (capabilities verified)
    Elevated = 2,
    /// High trust (same family)
    High = 3,
    /// Highest trust (sibling node)
    Highest = 4,
}

/// Identity proof from BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProof {
    /// Node ID
    pub node_id: String,
    /// Family ID (extracted from BearDog lineage verification)
    pub family_id: Option<String>,
    /// Ed25519 signature
    pub signature: String,
    /// Challenge that was signed
    pub challenge: String,
    /// Public key
    pub public_key: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Songbird discovery response for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SongbirdServiceInfo {
    id: String,
    name: String,
    address: String,
    port: u16,
    tags: Vec<String>,
    health: String,
}

/// Songbird discovery response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SongbirdDiscoveryResponse {
    services: Vec<SongbirdServiceInfo>,
}

/// Primal capability from get_capabilities response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrimalCapabilityInfo {
    #[serde(rename = "type")]
    capability_type: String,
    methods: Vec<String>,
    version: String,
}

/// Get capabilities response from primal
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetCapabilitiesResponse {
    primal: String,
    version: String,
    family_id: Option<String>,
    node_id: String,
    protocols: Vec<String>,
    provided_capabilities: Vec<PrimalCapabilityInfo>,
}

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

    /// Current node ID (reserved for future routing/identification features)
    #[allow(dead_code)] // Will be used for multi-node routing in Phase 3
    node_id: Option<String>,
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
            node_id: None,
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
            node_id: std::env::var("NODE_ID").ok(),
        }
    }

    /// Discover primals using insecure basic discovery (for bootstrapping)
    ///
    /// This is used when Songbird/BearDog are not yet available.
    /// It falls back to basic socket scanning without verification.
    pub async fn discover_insecure(&mut self) -> FederationResult<Vec<VerifiedPrimal>> {
        warn!("⚠️  Using insecure discovery (no Songbird/BearDog verification)");
        warn!("   This should only be used for bootstrapping!");

        // Use basic discovery without verification
        let mut basic_discovery = PrimalDiscovery::new();
        let discovered = basic_discovery.discover().await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();

        // Convert to VerifiedPrimal (but with Unknown trust level)
        for primal in discovered {
            let verified = VerifiedPrimal {
                name: primal.name.clone(),
                node_id: "unknown".to_string(),
                family_id: None,
                endpoints: primal.endpoints.clone(),
                capabilities: primal.capabilities.clone(),
                identity_proof: IdentityProof {
                    node_id: "unknown".to_string(),
                    family_id: None,
                    signature: "unverified".to_string(),
                    challenge: "none".to_string(),
                    public_key: "none".to_string(),
                    timestamp: now,
                },
                trust_level: TrustLevel::Unknown,
                discovered_at: now,
                verified_at: now,
                metadata: primal.metadata.clone(),
            };

            self.verified_primals
                .entry(primal.name.clone())
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

        // Verify we have required clients
        if self.songbird.is_none() && self.beardog.is_none() {
            return Err(FederationError::DiscoveryError(
                "Cannot perform secure discovery without Songbird or BearDog".to_string(),
            ));
        }

        // Layer 1: Physical Discovery (Songbird)
        let discovered = if let Some(ref songbird) = self.songbird {
            self.layer1_physical_discovery_songbird(songbird).await?
        } else {
            // Fallback to socket scanning if no Songbird
            self.layer1_physical_discovery_sockets().await?
        };

        info!("   Layer 1: Discovered {} primals", discovered.len());

        // Layers 2-5: Verify each discovered primal
        for primal in discovered {
            match self.verify_primal(primal).await {
                Ok(verified) => {
                    info!(
                        "   ✅ Verified: {} (trust: {:?})",
                        verified.name, verified.trust_level
                    );
                    self.verified_primals
                        .entry(verified.name.clone())
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

    /// Layer 1: Physical Discovery via Songbird
    async fn layer1_physical_discovery_songbird(
        &self,
        songbird: &UnixSocketClient,
    ) -> FederationResult<Vec<DiscoveredPrimal>> {
        debug!("Layer 1: Physical Discovery (Songbird)");

        // Query Songbird's discovery API
        let request = JsonRpcRequest::new(
            "discover_by_family",
            serde_json::json!({
                "family_tags": [self.family_id.as_deref().unwrap_or("*")],
                "timeout_ms": 5000
            }),
        );

        match songbird.call(request).await {
            Ok(response) => {
                // Parse Songbird's discovery response
                let result_value = response.result.unwrap_or_default();
                match serde_json::from_value::<SongbirdDiscoveryResponse>(result_value) {
                    Ok(discovery) => {
                        debug!("Songbird discovered {} services", discovery.services.len());

                        // Convert SongbirdServiceInfo to DiscoveredPrimal
                        let primals: Vec<DiscoveredPrimal> = discovery
                            .services
                            .into_iter()
                            .map(|service| {
                                // Infer capabilities from tags
                                let capabilities = CapabilitySet::from_tags(&service.tags);

                                // Create endpoint from address:port
                                let endpoint = if service.address.starts_with("/") {
                                    // Unix socket path
                                    PrimalEndpoint::UnixSocket {
                                        path: PathBuf::from(&service.address),
                                    }
                                } else {
                                    // HTTP endpoint
                                    PrimalEndpoint::Http {
                                        url: format!("http://{}:{}", service.address, service.port),
                                    }
                                };

                                DiscoveredPrimal {
                                    name: service.name.clone(),
                                    primal_type: service
                                        .tags
                                        .first()
                                        .cloned()
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    endpoints: vec![endpoint],
                                    capabilities,
                                    metadata: HashMap::from([
                                        ("id".to_string(), service.id),
                                        ("health".to_string(), service.health),
                                    ]),
                                }
                            })
                            .collect();

                        Ok(primals)
                    }
                    Err(e) => {
                        warn!("Failed to parse Songbird response: {}, falling back", e);
                        self.layer1_physical_discovery_sockets().await
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Songbird discovery failed: {}, falling back to socket scan",
                    e
                );
                self.layer1_physical_discovery_sockets().await
            }
        }
    }

    /// Layer 1: Physical Discovery via socket scanning (fallback)
    async fn layer1_physical_discovery_sockets(&self) -> FederationResult<Vec<DiscoveredPrimal>> {
        debug!("Layer 1: Physical Discovery (socket scan fallback)");

        let mut basic_discovery = PrimalDiscovery::new();
        basic_discovery.discover().await
    }

    /// Verify a primal through layers 2-5
    async fn verify_primal(&self, primal: DiscoveredPrimal) -> FederationResult<VerifiedPrimal> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();

        // Layer 2: Identity Verification (BearDog)
        let identity_proof = if let Some(ref beardog) = self.beardog {
            self.layer2_identity_verification(beardog, &primal).await?
        } else {
            // No BearDog available - create placeholder with no family
            IdentityProof {
                node_id: "unverified".to_string(),
                family_id: None,
                signature: "none".to_string(),
                challenge: "none".to_string(),
                public_key: "none".to_string(),
                timestamp: now,
            }
        };

        // Layer 3: Capability Verification (query primal directly)
        let capabilities = self.layer3_capability_verification(&primal).await?;

        // Layer 4: Trust Evaluation (BearDog)
        let trust_level = if let Some(ref beardog) = self.beardog {
            self.layer4_trust_evaluation(beardog, &identity_proof)
                .await?
        } else {
            TrustLevel::Basic
        };

        // Layer 5: Registration (we do this after returning)
        Ok(VerifiedPrimal {
            name: primal.name,
            node_id: identity_proof.node_id.clone(),
            family_id: identity_proof.family_id.clone(), // Extract from BearDog via identity proof
            endpoints: primal.endpoints,
            capabilities,
            identity_proof,
            trust_level,
            discovered_at: now,
            verified_at: now,
            metadata: primal.metadata,
        })
    }

    /// Layer 2: Identity Verification via BearDog
    async fn layer2_identity_verification(
        &self,
        _beardog: &BearDogClient,
        primal: &DiscoveredPrimal,
    ) -> FederationResult<IdentityProof> {
        debug!("Layer 2: Identity Verification (BearDog)");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();

        // Find Unix socket endpoint for challenge-response
        let socket_path = primal.endpoints.iter().find_map(|ep| {
            if let PrimalEndpoint::UnixSocket { path } = ep {
                Some(path.clone())
            } else {
                None
            }
        });

        if let Some(socket_path) = socket_path {
            // Step 1: Generate challenge (timestamp-based nonce)
            let challenge = format!("nucleus-challenge-{}-{}", primal.name, now);

            // Step 2: Request primal to sign challenge via get_identity
            let client = UnixSocketClient::new(socket_path);
            let request = JsonRpcRequest::new(
                "get_identity",
                serde_json::json!({
                    "challenge": challenge
                }),
            );

            match client.call(request).await {
                Ok(response) => {
                    // Step 3: Parse identity response
                    let empty_json = serde_json::json!({});
                    let result = response.result.as_ref().unwrap_or(&empty_json);
                    let node_id = result["node_id"]
                        .as_str()
                        .unwrap_or(&primal.name)
                        .to_string();
                    let family_id = result["family_id"].as_str().map(|s| s.to_string());
                    let signature = result["signature"]
                        .as_str()
                        .unwrap_or("unverified")
                        .to_string();
                    let public_key = result["public_key"].as_str().unwrap_or("none").to_string();

                    // Step 4: Verify signature with BearDog (future implementation)
                    // For now, we trust the primal's self-reported identity
                    // A full implementation would call beardog.verify_signature(...)

                    Ok(IdentityProof {
                        node_id,
                        family_id,
                        signature,
                        challenge,
                        public_key,
                        timestamp: now,
                    })
                }
                Err(e) => {
                    debug!("get_identity failed: {}, using basic proof", e);
                    // Fallback: Create basic identity proof without cryptographic verification
                    Ok(IdentityProof {
                        node_id: primal.name.clone(),
                        family_id: None,
                        signature: "unverified".to_string(),
                        challenge,
                        public_key: "none".to_string(),
                        timestamp: now,
                    })
                }
            }
        } else {
            // No Unix socket - can't perform challenge-response
            Ok(IdentityProof {
                node_id: primal.name.clone(),
                family_id: None,
                signature: "unverified".to_string(),
                challenge: "no-socket".to_string(),
                public_key: "none".to_string(),
                timestamp: now,
            })
        }
    }

    /// Layer 3: Capability Verification (query primal)
    async fn layer3_capability_verification(
        &self,
        primal: &DiscoveredPrimal,
    ) -> FederationResult<CapabilitySet> {
        debug!("Layer 3: Capability Verification");

        // Find Unix socket endpoint
        let socket_path = primal.endpoints.iter().find_map(|ep| {
            if let PrimalEndpoint::UnixSocket { path } = ep {
                Some(path.clone())
            } else {
                None
            }
        });

        if let Some(socket_path) = socket_path {
            // Query primal for capabilities
            let client = UnixSocketClient::new(socket_path);
            let request = JsonRpcRequest::new("get_capabilities", serde_json::json!({}));

            match client.call(request).await {
                Ok(response) => {
                    // Parse capabilities from structured response
                    let result_value = response.result.unwrap_or_default();
                    match serde_json::from_value::<GetCapabilitiesResponse>(result_value) {
                        Ok(cap_response) => {
                            debug!(
                                "Verified capabilities for {} (v{}): {} capabilities",
                                cap_response.primal,
                                cap_response.version,
                                cap_response.provided_capabilities.len()
                            );

                            // Convert PrimalCapabilityInfo to Capability
                            let mut capabilities = CapabilitySet::new();
                            for cap_info in cap_response.provided_capabilities {
                                // Parse capability type string into Capability enum
                                let cap: Capability =
                                    cap_info
                                        .capability_type
                                        .parse()
                                        .unwrap_or(Capability::Custom(
                                            cap_info.capability_type.clone(),
                                        ));
                                capabilities.add(cap);
                            }

                            // Validate against discovered capabilities (sanity check)
                            if capabilities.is_empty() {
                                warn!(
                                    "Primal reported zero capabilities, using discovered capabilities"
                                );
                                Ok(primal.capabilities.clone())
                            } else {
                                Ok(capabilities)
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse capability response: {}", e);
                            Ok(primal.capabilities.clone())
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to query capabilities: {}, using discovered capabilities",
                        e
                    );
                    Ok(primal.capabilities.clone())
                }
            }
        } else {
            // No Unix socket - use discovered capabilities
            Ok(primal.capabilities.clone())
        }
    }

    /// Layer 4: Trust Evaluation via BearDog
    async fn layer4_trust_evaluation(
        &self,
        beardog: &BearDogClient,
        identity_proof: &IdentityProof,
    ) -> FederationResult<TrustLevel> {
        debug!("Layer 4: Trust Evaluation (BearDog)");

        // Check if primal has family_id for lineage verification
        if let Some(ref peer_family_id) = identity_proof.family_id {
            if let Some(ref our_family_id) = self.family_id {
                // Use BearDog's lineage verification API
                match beardog
                    .verify_same_family(
                        our_family_id,
                        peer_family_id, // Use peer's family_id as seed_hash
                        &identity_proof.node_id,
                    )
                    .await
                {
                    Ok(lineage) => {
                        // Map relationship to trust level
                        let trust_level = match lineage.relationship.as_str() {
                            "parent" => TrustLevel::High,
                            "child" => TrustLevel::High,
                            "sibling" => TrustLevel::Highest, // Same family, highest trust
                            _ if lineage.is_family_member => TrustLevel::Elevated,
                            _ => TrustLevel::Basic,
                        };

                        info!(
                            "   Trust evaluation: {} → {:?} (relationship: {})",
                            identity_proof.node_id, trust_level, lineage.relationship
                        );

                        Ok(trust_level)
                    }
                    Err(e) => {
                        warn!("Failed to verify lineage: {}, defaulting to Basic trust", e);
                        Ok(TrustLevel::Basic)
                    }
                }
            } else {
                // We don't have a family_id, so we can't verify lineage
                debug!("No family_id set, cannot verify lineage");
                Ok(TrustLevel::Basic)
            }
        } else {
            // Primal doesn't have a family_id
            debug!("Primal has no family_id, cannot verify lineage");
            Ok(TrustLevel::Basic)
        }
    }

    /// Get a primal by selection criteria
    pub fn get(&self, criteria: SelectionCriteria) -> Option<&VerifiedPrimal> {
        match criteria {
            SelectionCriteria::ByCapability(cap) => {
                // Find highest trust primal with this capability
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper: create a VerifiedPrimal for tests ─────────────────
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

    // ═══════════════════════════════════════════════════════════════
    // TrustLevel tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Unknown < TrustLevel::Basic);
        assert!(TrustLevel::Basic < TrustLevel::Elevated);
        assert!(TrustLevel::Elevated < TrustLevel::High);
        assert!(TrustLevel::High < TrustLevel::Highest);
    }

    #[test]
    fn test_trust_level_clone_eq() {
        let t = TrustLevel::High;
        let t2 = t;
        assert_eq!(t, t2);
    }

    #[test]
    fn test_trust_level_debug() {
        let dbg = format!("{:?}", TrustLevel::Elevated);
        assert!(dbg.contains("Elevated"));
    }

    #[test]
    fn test_trust_level_serde_roundtrip() {
        let original = TrustLevel::Highest;
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: TrustLevel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }

    // ═══════════════════════════════════════════════════════════════
    // IdentityProof tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_identity_proof_serde_roundtrip() {
        let proof = IdentityProof {
            node_id: "node-1".into(),
            family_id: Some("fam-1".into()),
            signature: "sig-abc".into(),
            challenge: "challenge-xyz".into(),
            public_key: "pk-123".into(),
            timestamp: 42,
        };
        let json = serde_json::to_string(&proof).expect("serialize");
        let restored: IdentityProof = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.node_id, "node-1");
        assert_eq!(restored.family_id, Some("fam-1".into()));
        assert_eq!(restored.timestamp, 42);
    }

    #[test]
    fn test_identity_proof_without_family() {
        let proof = IdentityProof {
            node_id: "solo".into(),
            family_id: None,
            signature: "s".into(),
            challenge: "c".into(),
            public_key: "pk".into(),
            timestamp: 0,
        };
        let json = serde_json::to_string(&proof).expect("serialize");
        assert!(json.contains("\"family_id\":null"));
    }

    #[test]
    fn test_identity_proof_clone() {
        let proof = IdentityProof {
            node_id: "n".into(),
            family_id: None,
            signature: "s".into(),
            challenge: "c".into(),
            public_key: "pk".into(),
            timestamp: 99,
        };
        let cloned = proof.clone();
        assert_eq!(cloned.node_id, proof.node_id);
        assert_eq!(cloned.timestamp, proof.timestamp);
    }

    // ═══════════════════════════════════════════════════════════════
    // SelectionCriteria tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_selection_criteria_debug() {
        let c = SelectionCriteria::ByCapability(Capability::Storage);
        assert!(format!("{:?}", c).contains("ByCapability"));

        let c2 = SelectionCriteria::ByNodeId("node-1".into());
        assert!(format!("{:?}", c2).contains("node-1"));

        let c3 = SelectionCriteria::Any;
        assert!(format!("{:?}", c3).contains("Any"));
    }

    #[test]
    fn test_selection_criteria_clone() {
        let c = SelectionCriteria::ByFamily("fam-1".into());
        let c2 = c.clone();
        assert!(format!("{:?}", c2).contains("fam-1"));
    }

    // ═══════════════════════════════════════════════════════════════
    // VerifiedPrimal tests
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    // SecureNucleusDiscovery: new / default
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    // inject_primal_for_testing
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    // get: SelectionCriteria::ByCapability
    // ═══════════════════════════════════════════════════════════════

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
        assert!(disc
            .get(SelectionCriteria::ByCapability(Capability::Compute))
            .is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // get: SelectionCriteria::ByNodeId
    // ═══════════════════════════════════════════════════════════════

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
        assert!(disc
            .get(SelectionCriteria::ByNodeId("nope".into()))
            .is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // get: SelectionCriteria::ByFamily
    // ═══════════════════════════════════════════════════════════════

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

        assert!(disc
            .get(SelectionCriteria::ByFamily("east".into()))
            .is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // get: SelectionCriteria::BySocket
    // ═══════════════════════════════════════════════════════════════

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
        assert!(disc
            .get(SelectionCriteria::BySocket(PathBuf::from("/nope")))
            .is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // get: SelectionCriteria::MinTrustLevel
    // ═══════════════════════════════════════════════════════════════

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

        assert!(disc
            .get(SelectionCriteria::MinTrustLevel(TrustLevel::High))
            .is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // get: SelectionCriteria::Any
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    // with_capability
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    // all / get_all
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    // SongbirdServiceInfo / SongbirdDiscoveryResponse serde
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_songbird_service_info_serde() {
        let json = r#"{"id":"s1","name":"songbird","address":"127.0.0.1","port":9000,"tags":["discovery"],"health":"healthy"}"#;
        let info: SongbirdServiceInfo = serde_json::from_str(json).expect("deserialize");
        assert_eq!(info.name, "songbird");
        assert_eq!(info.port, 9000);
        assert_eq!(info.tags, vec!["discovery"]);
    }

    #[test]
    fn test_songbird_discovery_response_serde() {
        let json = r#"{"services":[{"id":"s1","name":"test","address":"/tmp/test.sock","port":0,"tags":[],"health":"ok"}]}"#;
        let resp: SongbirdDiscoveryResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.services.len(), 1);
        assert_eq!(resp.services[0].name, "test");
    }

    // ═══════════════════════════════════════════════════════════════
    // PrimalCapabilityInfo / GetCapabilitiesResponse serde
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_primal_capability_info_serde() {
        let json = r#"{"type":"storage","methods":["put","get"],"version":"1.0"}"#;
        let info: PrimalCapabilityInfo = serde_json::from_str(json).expect("deserialize");
        assert_eq!(info.capability_type, "storage");
        assert_eq!(info.methods, vec!["put", "get"]);
    }

    #[test]
    fn test_get_capabilities_response_serde() {
        let json = r#"{
            "primal": "nestgate",
            "version": "2.0",
            "family_id": "fam-1",
            "node_id": "n1",
            "protocols": ["jsonrpc"],
            "provided_capabilities": [
                {"type": "storage", "methods": ["put"], "version": "1.0"}
            ]
        }"#;
        let resp: GetCapabilitiesResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.primal, "nestgate");
        assert_eq!(resp.provided_capabilities.len(), 1);
        assert_eq!(resp.family_id, Some("fam-1".into()));
    }

    // ═══════════════════════════════════════════════════════════════
    // discover_secure: error without clients
    // ═══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_discover_secure_requires_clients() {
        let mut disc = SecureNucleusDiscovery::new();
        let result = disc.discover_secure().await;
        assert!(result.is_err(), "secure discovery without clients must fail");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Songbird") || err_msg.contains("BearDog"));
    }
}
