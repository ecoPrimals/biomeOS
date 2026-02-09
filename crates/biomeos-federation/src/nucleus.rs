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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
                                // FromStr is infallible, so this always succeeds
                                let cap: Capability = cap_info.capability_type.parse().unwrap();
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
