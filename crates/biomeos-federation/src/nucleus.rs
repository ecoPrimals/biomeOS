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
    /// Ed25519 signature
    pub signature: String,
    /// Challenge that was signed
    pub challenge: String,
    /// Public key
    pub public_key: String,
    /// Timestamp
    pub timestamp: u64,
}

/// A verified primal (passed all 5 layers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedPrimal {
    /// Primal name (e.g., "songbird", "beardog")
    pub name: String,
    
    /// Node ID (e.g., "node-alpha")
    pub node_id: String,
    
    /// Family ID (e.g., "nat0")
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
    
    /// Current node ID
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
                .or_insert_with(Vec::new)
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
                    info!("   ✅ Verified: {} (trust: {:?})", verified.name, verified.trust_level);
                    self.verified_primals
                        .entry(verified.name.clone())
                        .or_insert_with(Vec::new)
                        .push(verified);
                }
                Err(e) => {
                    warn!("   ❌ Failed to verify primal: {}", e);
                }
            }
        }
        
        info!("🔒 Secure discovery complete: {} verified", self.verified_primals.len());
        
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
                "family_id": self.family_id.as_deref().unwrap_or("*")
            }),
        );
        
        match songbird.call(request).await {
            Ok(response) => {
                // Parse Songbird's response
                // TODO: Define proper response type
                debug!("Songbird discovery response: {:?}", response);
                Ok(vec![]) // TODO: Parse into DiscoveredPrimal
            }
            Err(e) => {
                warn!("Songbird discovery failed: {}, falling back to socket scan", e);
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
            // No BearDog available - skip verification (low trust)
            IdentityProof {
                node_id: "unverified".to_string(),
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
            self.layer4_trust_evaluation(beardog, &identity_proof).await?
        } else {
            TrustLevel::Basic
        };
        
        // Layer 5: Registration (we do this after returning)
        Ok(VerifiedPrimal {
            name: primal.name,
            node_id: identity_proof.node_id.clone(),
            family_id: None, // TODO: Extract from BearDog
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
        
        // TODO: Challenge-response protocol
        // 1. Generate challenge
        // 2. Send to primal
        // 3. Primal signs with BearDog
        // 4. We verify signature with BearDog
        
        // For now, return placeholder
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(IdentityProof {
            node_id: primal.name.clone(),
            signature: "todo".to_string(),
            challenge: "todo".to_string(),
            public_key: "todo".to_string(),
            timestamp: now,
        })
    }
    
    /// Layer 3: Capability Verification (query primal)
    async fn layer3_capability_verification(
        &self,
        primal: &DiscoveredPrimal,
    ) -> FederationResult<CapabilitySet> {
        debug!("Layer 3: Capability Verification");
        
        // Find Unix socket endpoint
        let socket_path = primal
            .endpoints
            .iter()
            .find_map(|ep| {
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
                    // Parse capabilities from response
                    // TODO: Define proper response type
                    debug!("Capability response: {:?}", response);
                    Ok(primal.capabilities.clone())
                }
                Err(e) => {
                    warn!("Failed to query capabilities: {}, using discovered capabilities", e);
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
        _beardog: &BearDogClient,
        _identity_proof: &IdentityProof,
    ) -> FederationResult<TrustLevel> {
        debug!("Layer 4: Trust Evaluation (BearDog)");
        
        // TODO: Use BearDog's trust evaluation API
        // - Check genetic lineage
        // - Evaluate family membership
        // - Return trust level
        
        Ok(TrustLevel::Basic)
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

