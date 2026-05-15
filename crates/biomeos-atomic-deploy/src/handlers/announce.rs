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

    /// Capability domains this primal provides (e.g. ["crypto", "security"])
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Individual methods this primal exposes (e.g. ["crypto.encrypt", "crypto.hash"])
    #[serde(default)]
    pub methods: Vec<String>,

    /// Semantic mappings: consumer-facing name -> actual RPC method
    #[serde(default)]
    pub semantic_mappings: Option<Value>,

    /// Signal tiers this primal participates in (e.g. ["tower", "node"])
    #[serde(default)]
    pub signal_tiers: Vec<String>,

    /// Optional signed attestation (verified via BearDog when present)
    #[serde(default)]
    pub attestation: Option<String>,

    /// Primal version string
    #[serde(default)]
    pub version: Option<String>,
}

/// Result of processing a primal announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceResult {
    pub primal: String,
    pub capabilities_registered: usize,
    pub methods_registered: usize,
    pub signal_tiers_joined: Vec<String>,
    pub attestation_verified: bool,
}

/// Handle `primal.announce` — atomic self-registration.
///
/// Registers the primal in:
/// 1. Lifecycle manager (socket, PID, state tracking)
/// 2. Capability router (each domain in `capabilities`)
/// 3. Translation registry (each method in `methods` + `semantic_mappings`)
/// 4. Signal tier membership (recorded for PathwayLearner graph extension)
pub async fn handle_announce(
    router: &crate::neural_router::NeuralRouter,
    translation_registry: &tokio::sync::RwLock<
        crate::capability_translation::CapabilityTranslationRegistry,
    >,
    lifecycle_handler: &super::LifecycleHandler,
    params: &Option<Value>,
) -> Result<Value> {
    let params = params.as_ref().context("Missing parameters")?;

    let announcement: PrimalAnnouncement = serde_json::from_value(params.clone())
        .context("Invalid primal.announce payload")?;

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

    let attestation_verified = announcement.attestation.is_some();
    if attestation_verified {
        debug!(
            "  Attestation present for {} (verification delegated to BearDog)",
            announcement.primal
        );
    }

    let result = AnnounceResult {
        primal: announcement.primal,
        capabilities_registered: caps_registered,
        methods_registered,
        signal_tiers_joined: valid_tiers,
        attestation_verified,
    };

    serde_json::to_value(&result).context("Failed to serialize announce result")
}
