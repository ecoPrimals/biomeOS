// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal self-announcement handler (`primal.announce`).
//!
//! Provides a single atomic registration path for primals joining the
//! ecosystem. Instead of requiring separate `lifecycle.register`,
//! `capability.register`, and `method.register` calls, a primal sends
//! one `primal.announce` with its complete capability surface.
//!
//! This enables new primals (e.g. a new crypto or protocol primal) to
//! self-register and immediately participate in capability routing,
//! signal tier membership, and PathwayLearner observation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Structured announcement from a primal declaring its capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalAnnouncement {
    /// Primal name (e.g. "beardog", "songbird")
    pub primal: String,

    /// Transport endpoint (Unix socket path or TCP address)
    pub socket: String,

    /// Process ID (optional, for lifecycle tracking)
    #[serde(default)]
    pub pid: Option<u32>,

    /// Capability domains this primal provides (e.g. `["crypto", "security"]`).
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Individual methods this primal exposes (e.g. `["crypto.encrypt", "crypto.hash"]`).
    #[serde(default)]
    pub methods: Vec<String>,

    /// Semantic mappings: consumer-facing name -> actual RPC method.
    #[serde(default)]
    pub semantic_mappings: Option<Value>,

    /// Signal tiers this primal participates in (e.g. `["tower", "node"]`).
    #[serde(default)]
    pub signal_tiers: Vec<String>,

    /// Optional signed attestation (verified via BearDog when present)
    #[serde(default)]
    pub attestation: Option<String>,

    /// Primal version string
    #[serde(default)]
    pub version: Option<String>,

    /// Cost hints per capability — primals self-report expected dispatch cost
    /// (arbitrary units, lower = cheaper). Used by the routing weight system
    /// to prefer cheaper providers when quality is similar.
    ///
    /// Example: `{ "compute": 100.0, "storage": 10.0 }`
    #[serde(default)]
    pub cost_hints: Option<Value>,

    /// Latency estimates per capability (ms) — primals self-report expected
    /// response latency. Used to seed routing weights before operational data
    /// accumulates.
    ///
    /// Example: `{ "crypto": 5, "compute": 200 }`
    #[serde(default)]
    pub latency_estimates: Option<Value>,
}

/// Result of processing a primal announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceResult {
    /// Primal name that announced.
    pub primal: String,
    /// Number of capability domains registered.
    pub capabilities_registered: usize,
    /// Number of individual methods registered.
    pub methods_registered: usize,
    /// Signal tiers the primal joined (filtered to known tiers).
    pub signal_tiers_joined: Vec<String>,
    /// Whether the signed attestation was verified.
    pub attestation_verified: bool,
}

/// Handle `primal.announce` — atomic self-registration.
///
/// Registers the primal in:
/// 1. Lifecycle manager (socket, PID, state tracking)
/// 2. Capability router (each domain in `capabilities`)
/// 3. Translation registry (each method in `methods` + `semantic_mappings`)
/// 4. Signal tier membership (recorded for PathwayLearner graph extension)
/// 5. Attestation verification (Ed25519 via BearDog when available)
pub async fn handle_announce(
    router: &crate::neural_router::NeuralRouter,
    translation_registry: &tokio::sync::RwLock<
        crate::capability_translation::CapabilityTranslationRegistry,
    >,
    lifecycle_handler: &super::LifecycleHandler,
    beardog_verifier: &Option<biomeos_core::BearDogVerifier>,
    params: &Option<Value>,
) -> Result<Value> {
    let params = params.as_ref().context("Missing parameters")?;

    let announcement: PrimalAnnouncement =
        serde_json::from_value(params.clone()).context("Invalid primal.announce payload")?;

    info!(
        "Primal announcement: {} at {} ({} capabilities, {} methods, {} signal tiers)",
        announcement.primal,
        announcement.socket,
        announcement.capabilities.len(),
        announcement.methods.len(),
        announcement.signal_tiers.len(),
    );

    let endpoint =
        biomeos_core::TransportEndpoint::parse(&announcement.socket).unwrap_or_else(|| {
            biomeos_core::TransportEndpoint::UnixSocket {
                path: PathBuf::from(&announcement.socket),
            }
        });

    // 1. Lifecycle registration
    if let Some(pid) = announcement.pid {
        let lifecycle_params = json!({
            "name": announcement.primal,
            "socket_path": announcement.socket,
            "pid": pid,
        });
        if let Err(e) = lifecycle_handler.register(&Some(lifecycle_params)).await {
            warn!(
                "Lifecycle registration failed for {}: {e} (continuing with capability registration)",
                announcement.primal
            );
        }
    }

    // 2. Capability domain registration
    let mut caps_registered = 0;
    for capability in &announcement.capabilities {
        match router
            .register_capability(
                capability,
                &announcement.primal,
                endpoint.clone(),
                "primal.announce",
            )
            .await
        {
            Ok(_) => {
                caps_registered += 1;
                debug!(
                    "  Registered capability: {} -> {}",
                    capability, announcement.primal
                );
            }
            Err(e) => warn!(
                "  Failed to register capability {} for {}: {e}",
                capability, announcement.primal
            ),
        }
    }

    // 3. Method + translation registration
    let mut methods_registered = 0;
    {
        let mut registry = translation_registry.write().await;

        for method in &announcement.methods {
            if let Some((domain, _operation)) = method.split_once('.') {
                // Ensure the domain is registered on the router
                if !announcement.capabilities.contains(&domain.to_string()) {
                    let _ = router
                        .register_capability(
                            domain,
                            &announcement.primal,
                            endpoint.clone(),
                            "primal.announce",
                        )
                        .await;
                }

                registry.register_translation(
                    method,
                    &announcement.primal,
                    method,
                    &announcement.socket,
                    None,
                );
                methods_registered += 1;
            }
        }

        // Semantic mappings (consumer name -> actual method)
        if let Some(ref mappings) = announcement.semantic_mappings {
            if let Some(map) = mappings.as_object() {
                for (semantic, actual) in map {
                    if let Some(actual_str) = actual.as_str() {
                        registry.register_translation(
                            semantic,
                            &announcement.primal,
                            actual_str,
                            &announcement.socket,
                            None,
                        );
                        methods_registered += 1;
                    }
                }
            }
        }
    }

    // 4. Signal tier membership
    let valid_tiers: Vec<String> = announcement
        .signal_tiers
        .iter()
        .filter(|t| super::signal::is_signal_tier(t))
        .cloned()
        .collect();

    if !valid_tiers.is_empty() {
        info!(
            "  {} joins signal tiers: {:?}",
            announcement.primal, valid_tiers
        );
    }

    // 5. Routing weight seeding from self-reported hints (Layer 4 evolution)
    if let Some(ref cost_hints) = announcement.cost_hints {
        if let Some(map) = cost_hints.as_object() {
            for (capability, cost) in map {
                if let Some(cost_f) = cost.as_f64() {
                    router
                        .set_provider_cost_hint(capability, &announcement.primal, cost_f)
                        .await;
                    debug!(
                        "  Routing weight: {}.{} cost_hint={}",
                        announcement.primal, capability, cost_f
                    );
                }
            }
        }
    }

    if let Some(ref latency_estimates) = announcement.latency_estimates {
        if let Some(map) = latency_estimates.as_object() {
            for (capability, latency) in map {
                if let Some(latency_ms) = latency.as_f64() {
                    // Seed the weight with the primal's self-reported latency.
                    // Higher affinity for primals that self-report (they're cooperating).
                    router
                        .set_provider_affinity(capability, &announcement.primal, 0.6)
                        .await;
                    debug!(
                        "  Routing weight: {}.{} latency_estimate={}ms",
                        announcement.primal, capability, latency_ms
                    );
                }
            }
        }
    }

    // 6. Attestation verification via BearDog
    let attestation_verified = if let Some(ref attestation) = announcement.attestation {
        if let Some(verifier) = beardog_verifier {
            let verified = verifier.verify_async(attestation).await.is_some();
            if verified {
                info!(
                    "  Attestation verified for {} via BearDog",
                    announcement.primal
                );
            } else {
                warn!(
                    "  Attestation FAILED verification for {} — accepting registration but marking unverified",
                    announcement.primal
                );
            }
            verified
        } else {
            debug!(
                "  Attestation present for {} but BearDog unavailable — skipping verification",
                announcement.primal
            );
            false
        }
    } else {
        false
    };

    let result = AnnounceResult {
        primal: announcement.primal,
        capabilities_registered: caps_registered,
        methods_registered,
        signal_tiers_joined: valid_tiers,
        attestation_verified,
    };

    serde_json::to_value(&result).context("Failed to serialize announce result")
}
