// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! NUCLEUS high-level client: five-layer discovery and registration.

use std::sync::Arc;

use tracing::{info, warn};

use crate::{
    Error, Result, VerifiedPrimal,
    capability::{CapabilityLayer, CapabilityLayerImpl},
    discovery::{DiscoveryLayer, DiscoveryRequest, PhysicalDiscovery},
    identity::{IdentityLayer, IdentityLayerImpl},
    registry::Registry,
    trust::{TrustLayer, TrustLayerImpl, TrustLevel},
};

use super::family_seed;

/// NUCLEUS Client - Coordinates all 5 discovery layers
///
/// **Deep Debt Principles Applied**:
/// - No hardcoding: Discovers all primals at runtime
/// - No reimplementation: Delegates to `BearDog` and Songbird
/// - Fast AND safe: Zero unsafe code, async throughout
/// - Capability-based: Selects by what primals can do
pub struct NucleusClient {
    /// Layer 1: Physical discovery (Songbird)
    discovery: Arc<dyn PhysicalDiscovery>,
    /// Layer 2: Identity verification (`BearDog`)
    identity: Arc<dyn IdentityLayer>,
    /// Layer 3: Capability verification
    capability: Arc<dyn CapabilityLayer>,
    /// Layer 4: Trust evaluation (`BearDog`)
    trust: Arc<dyn TrustLayer>,
    /// Layer 5: Registry and tracking
    registry: Arc<Registry>,
}

impl NucleusClient {
    /// Create a new NUCLEUS client
    ///
    /// Initializes all 5 layers
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Discovery layer fails to initialize (Songbird socket discovery fails)
    /// - Identity layer fails to initialize (`BearDog` socket discovery fails)
    /// - Trust layer fails to initialize (system paths or credentials unavailable)
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Client (5-layer secure discovery)");

        let discovery = Arc::new(DiscoveryLayer::new().await?) as Arc<dyn PhysicalDiscovery>;
        let identity = Arc::new(IdentityLayerImpl::new().await?) as Arc<dyn IdentityLayer>;
        let capability = Arc::new(CapabilityLayerImpl::new()) as Arc<dyn CapabilityLayer>;
        let trust = Arc::new(TrustLayerImpl::new().await?) as Arc<dyn TrustLayer>;
        let registry = Arc::new(Registry::new());

        info!("✅ NUCLEUS Client initialized successfully");

        Ok(Self {
            discovery,
            identity,
            capability,
            trust,
            registry,
        })
    }

    /// Discover and verify primals
    ///
    /// Runs all 5 NUCLEUS layers:
    /// 1. Physical discovery (Songbird)
    /// 2. Identity verification (`BearDog`)
    /// 3. Capability verification
    /// 4. Trust evaluation (`BearDog`)
    /// 5. Registry and tracking
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Physical discovery fails (Songbird unreachable or returns error)
    /// - Identity verification fails (invalid signatures or `BearDog` unreachable)
    /// - Capability verification fails (primal doesn't match requested capability)
    /// - Trust evaluation fails (lineage verification or trust score below threshold)
    pub async fn discover(&self, request: DiscoveryRequest) -> Result<Vec<VerifiedPrimal>> {
        info!(
            capability = %request.capability,
            family = ?request.family,
            "Starting NUCLEUS 5-layer discovery"
        );

        // Layer 1: Physical Discovery (Songbird)
        let discovered = self.discovery.discover_by_capability(&request).await?;
        info!(
            count = discovered.len(),
            "Layer 1: Discovered {} primals",
            discovered.len()
        );

        let mut verified_primals = Vec::new();

        for primal in discovered {
            info!(primal = %primal.primal, "Verifying primal through remaining layers");

            // Layer 2: Identity Verification (BearDog)
            let identity = match self.identity.verify_identity(&primal).await {
                Ok(id) => {
                    info!(primal = %primal.primal, "Layer 2: Identity verified ✓");
                    id
                }
                Err(e) => {
                    warn!(primal = %primal.primal, error = %e, "Layer 2: Identity verification failed, skipping");
                    continue;
                }
            };

            // Layer 3: Capability Verification
            let _capability = match self
                .capability
                .verify_capabilities(&primal, &identity.proof)
                .await
            {
                Ok(cap) => {
                    info!(primal = %primal.primal, "Layer 3: Capabilities verified ✓");
                    cap
                }
                Err(e) => {
                    warn!(primal = %primal.primal, error = %e, "Layer 3: Capability verification failed, skipping");
                    continue;
                }
            };

            // Layer 4: Trust Evaluation (BearDog)
            // EVOLVED (Jan 27, 2026): Get family seed from secure storage
            // Priority: 1) Environment variable (for bootstrap)
            //           2) Default empty (graceful degradation - Known trust level)
            let family_seed = family_seed::load_family_seed_from_storage();
            let trust = match self
                .trust
                .evaluate_trust(&primal, &identity.proof, family_seed.as_ref())
                .await
            {
                Ok(trust) => {
                    info!(primal = %primal.primal, level = ?trust.level, "Layer 4: Trust evaluated ✓");
                    trust
                }
                Err(e) => {
                    warn!(primal = %primal.primal, error = %e, "Layer 4: Trust evaluation failed, using 'Known' level");
                    crate::trust::TrustEvaluation {
                        level: TrustLevel::Known,
                        relationship: None,
                        lineage_verified: false,
                        message: "Trust evaluation failed, defaulting to Known".to_string(),
                    }
                }
            };

            // Create verified primal
            let verified = VerifiedPrimal {
                name: primal.primal,
                node_id: primal.node_id,
                family_id: primal.family_id,
                capabilities: primal.capabilities,
                endpoint: primal.endpoints.first().cloned().ok_or_else(|| {
                    Error::invalid_response(&identity.proof.primal_name, "No endpoints")
                })?,
                trust_level: trust.level,
                version: identity.proof.version,
            };

            // Layer 5: Register
            self.registry.register(verified.clone()).await;
            info!(primal = %verified.name, "Layer 5: Registered ✓");

            verified_primals.push(verified);
        }

        info!(
            verified = verified_primals.len(),
            "NUCLEUS discovery complete: {} verified primals",
            verified_primals.len()
        );

        Ok(verified_primals)
    }

    /// Get registry for direct access
    #[must_use]
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

#[cfg(test)]
impl NucleusClient {
    /// Construct with injected layers (unit tests only).
    pub(crate) fn from_layers_for_test(
        discovery: Arc<dyn PhysicalDiscovery>,
        identity: Arc<dyn IdentityLayer>,
        capability: Arc<dyn CapabilityLayer>,
        trust: Arc<dyn TrustLayer>,
        registry: Arc<Registry>,
    ) -> Self {
        Self {
            discovery,
            identity,
            capability,
            trust,
            registry,
        }
    }
}

/// NUCLEUS Client Builder (for customization).
pub struct NucleusClientBuilder {
    _private: (),
}

impl NucleusClientBuilder {
    /// Create a new builder
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Build the client
    ///
    /// # Errors
    ///
    /// Returns an error if `NucleusClient::new()` fails. See [`NucleusClient::new`] for details.
    pub async fn build(self) -> Result<NucleusClient> {
        NucleusClient::new().await
    }
}

impl Default for NucleusClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
