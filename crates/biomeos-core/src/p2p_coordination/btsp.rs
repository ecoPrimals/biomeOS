// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP (`BearDog` Transport Security Protocol) Coordination
//!
//! `BiomeOS` coordinates BTSP tunnel creation between any security primal
//! and any discovery primal in pure Rust.
//!
//! # Agnostic Design
//!
//! This coordinator works with **any** primals that implement:
//! - `SecurityProvider` - Provides BTSP tunnel creation
//! - `DiscoveryProvider` - Provides transport registration
//!
//! It doesn't care if the security provider is `BearDog` or something else!

use super::{
    DiscoveryProvider, LineageProof, OverallHealth, SecurityProvider, TransportEndpoint,
    TransportHealth, TunnelHealth, TunnelInfo, TunnelStatus,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Latency above this threshold (ms) indicates transport degradation.
const DEGRADED_LATENCY_MS: u32 = 200;

/// Packet loss above this percentage indicates transport degradation.
const DEGRADED_PACKET_LOSS: f32 = 1.0;

/// Key rotation older than this is considered stale.
const KEY_ROTATION_STALE: Duration = Duration::from_secs(86_400);

/// Per-tunnel transport routing state used during degradation recovery.
#[derive(Debug, Default)]
struct TunnelPathState {
    endpoints: Vec<TransportEndpoint>,
    preferred_index: usize,
    path_degraded: bool,
}

/// BTSP tunnel coordinator
///
/// Coordinates secure tunnel creation between:
/// - Any security primal (provides encryption)
/// - Any discovery primal (provides transport registration)
pub struct BtspCoordinator<S: SecurityProvider, D: DiscoveryProvider> {
    /// Security provider (agnostic - works with any security primal)
    security: Arc<S>,

    /// Discovery provider (agnostic - works with any discovery primal)
    discovery: Arc<D>,

    /// Active and fallback transport paths per tunnel.
    path_states: Mutex<HashMap<String, TunnelPathState>>,
}

impl<S: SecurityProvider, D: DiscoveryProvider> BtspCoordinator<S, D> {
    /// Create a new BTSP coordinator
    ///
    /// # Arguments
    ///
    /// * `security` - Any primal providing security capabilities
    /// * `discovery` - Any primal providing discovery capabilities
    ///
    /// # Philosophy
    ///
    /// This constructor is **agnostic** - it accepts any primal that implements
    /// the required traits, regardless of what it's called.
    pub fn new(security: Arc<S>, discovery: Arc<D>) -> Self {
        Self {
            security,
            discovery,
            path_states: Mutex::new(HashMap::new()),
        }
    }

    /// Create a BTSP tunnel between two nodes
    ///
    /// # Coordination Flow
    ///
    /// 1. Request tunnel from security provider (e.g., `BearDog`)
    /// 2. Register endpoints with discovery provider (e.g., Songbird)
    /// 3. Verify tunnel is operational
    /// 4. Return tunnel information
    ///
    /// This flow is **primal-agnostic** - it works with any combination of
    /// security and discovery primals.
    pub async fn create_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        lineage_proof: LineageProof,
    ) -> Result<TunnelInfo> {
        // Step 1: Request tunnel from security provider
        let tunnel_request = self
            .security
            .request_tunnel(node_a, node_b, &lineage_proof)
            .await
            .context("Security provider failed to create tunnel")?;

        // Step 2: Register endpoint A with discovery provider
        self.discovery
            .register_transport(&tunnel_request.endpoint_a)
            .await
            .context("Failed to register endpoint A with discovery provider")?;

        // Step 3: Register endpoint B with discovery provider
        self.discovery
            .register_transport(&tunnel_request.endpoint_b)
            .await
            .context("Failed to register endpoint B with discovery provider")?;

        // Step 4: Verify tunnel is operational
        let health = self
            .monitor_tunnel(&tunnel_request.id)
            .await
            .context("Failed to verify tunnel health")?;

        if health.security_health.status != super::HealthStatus::Healthy {
            anyhow::bail!("Tunnel created but security health check failed");
        }

        // Step 5: Return tunnel information
        let endpoints = vec![
            tunnel_request.endpoint_a.clone(),
            tunnel_request.endpoint_b.clone(),
        ];
        self.record_tunnel_endpoints(&tunnel_request.id, &endpoints);

        Ok(TunnelInfo {
            tunnel_id: tunnel_request.id,
            status: TunnelStatus::Active,
            endpoints,
            established_at: SystemTime::now(),
        })
    }

    /// Monitor tunnel health
    ///
    /// Checks both:
    /// - Security provider (encryption status, key rotation)
    /// - Discovery provider (connection status, latency)
    ///
    /// Returns combined health status.
    pub async fn monitor_tunnel(&self, tunnel_id: &str) -> Result<super::OverallHealth> {
        // Check security component
        let security_health = self
            .security
            .check_tunnel_health(tunnel_id)
            .await
            .context("Security provider health check failed")?;

        // Check transport component
        let transport_health = self
            .discovery
            .check_transport_health(tunnel_id)
            .await
            .context("Discovery provider health check failed")?;

        // Compute overall status
        let status = compute_overall_status(&security_health, &transport_health);

        Ok(super::OverallHealth {
            tunnel_id: tunnel_id.to_string(),
            security_health,
            transport_health,
            status,
        })
    }

    /// Recover a degraded tunnel
    ///
    /// Attempts to recover by:
    /// 1. Checking if security provider can rotate keys
    /// 2. Checking if discovery provider can re-establish transport
    /// 3. Coordinating recovery between both
    pub async fn recover_tunnel(&self, tunnel_id: &str) -> Result<TunnelInfo> {
        // Get current health
        let health = self.monitor_tunnel(tunnel_id).await?;

        // Determine recovery strategy based on which component is unhealthy
        match health.status {
            super::HealthStatus::Healthy => {
                // Already healthy, nothing to do
                Ok(TunnelInfo {
                    tunnel_id: tunnel_id.to_string(),
                    status: TunnelStatus::Active,
                    endpoints: vec![],
                    established_at: SystemTime::now(),
                })
            }
            super::HealthStatus::Degraded => {
                // Graceful recovery: Diagnose and repair
                Ok(self.recover_degraded_tunnel(tunnel_id, &health).await?)
            }
            super::HealthStatus::Unhealthy => {
                // Need full tunnel recreation
                anyhow::bail!("Tunnel unhealthy - requires recreation");
            }
        }
    }

    /// Recover a degraded tunnel through diagnosis and repair
    async fn recover_degraded_tunnel(
        &self,
        tunnel_id: &str,
        health: &OverallHealth,
    ) -> Result<TunnelInfo> {
        tracing::info!("Attempting graceful recovery for tunnel: {}", tunnel_id);

        // Diagnose the issue from live provider health metrics.
        let degradation_cause =
            classify_degradation(&health.security_health, &health.transport_health);
        tracing::info!(
            tunnel_id,
            ?degradation_cause,
            "Diagnosed tunnel degradation cause"
        );

        // Apply appropriate recovery strategy
        match degradation_cause {
            DegradationCause::TransportLatency
            | DegradationCause::TransportPacketLoss
            | DegradationCause::TransportConnectivity => {
                tracing::info!("Recovery: Optimizing transport path");
                self.optimize_transport_path(tunnel_id).await?;
            }
            DegradationCause::AuthFailure | DegradationCause::KeyRotation => {
                tracing::info!(
                    "Recovery: Security component degraded ({degradation_cause:?}); awaiting provider recovery"
                );
            }
            DegradationCause::Unknown => {
                tracing::warn!(
                    "Recovery: Unable to classify degradation cause for tunnel {tunnel_id}"
                );
            }
        }

        // Verify recovery via security provider (which has check_tunnel_health)
        let health = self.security.check_tunnel_health(tunnel_id).await?;
        if health.status == super::HealthStatus::Healthy {
            tracing::info!("✅ Tunnel recovered successfully: {}", tunnel_id);
            Ok(TunnelInfo {
                tunnel_id: tunnel_id.to_string(),
                status: TunnelStatus::Active,
                endpoints: vec![],
                established_at: SystemTime::now(),
            })
        } else {
            anyhow::bail!("Recovery failed - tunnel still degraded");
        }
    }

    fn record_tunnel_endpoints(&self, tunnel_id: &str, endpoints: &[TransportEndpoint]) {
        if let Ok(mut states) = self.path_states.lock() {
            states.entry(tunnel_id.to_string()).or_default().endpoints = endpoints.to_vec();
        }
    }

    /// Optimize the transport path by preferring a TCP fallback when local transport is degraded.
    async fn optimize_transport_path(&self, tunnel_id: &str) -> Result<()> {
        let fallback = {
            let mut states = self
                .path_states
                .lock()
                .map_err(|_| anyhow::anyhow!("transport path state lock poisoned"))?;
            let state = states.entry(tunnel_id.to_string()).or_default();
            state.path_degraded = true;

            state
                .endpoints
                .iter()
                .find_map(build_tcp_fallback)
                .map(|endpoint| (state.endpoints.len(), endpoint))
        };

        let Some((endpoint_count, fallback)) = fallback else {
            tracing::debug!(
                tunnel_id,
                "No alternative transport endpoint available; current path marked degraded"
            );
            return Ok(());
        };

        self.discovery
            .register_transport(&fallback)
            .await
            .context("Failed to register fallback transport endpoint")?;

        {
            let mut states = self
                .path_states
                .lock()
                .map_err(|_| anyhow::anyhow!("transport path state lock poisoned"))?;
            if let Some(state) = states.get_mut(tunnel_id) {
                state.endpoints.push(fallback.clone());
                state.preferred_index = endpoint_count;
            }
        }

        tracing::info!(
            tunnel_id,
            node_id = %fallback.node_id,
            address = %fallback.address,
            port = fallback.port,
            protocol = %fallback.protocol,
            "Registered TCP fallback transport and updated routing preference"
        );
        Ok(())
    }
}

const fn compute_overall_status(
    security: &TunnelHealth,
    transport: &TransportHealth,
) -> super::HealthStatus {
    use super::HealthStatus;

    match (security.status, transport.status) {
        (HealthStatus::Healthy, HealthStatus::Healthy) => HealthStatus::Healthy,
        (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
        _ => HealthStatus::Degraded,
    }
}

/// Classify degradation cause from security and transport health snapshots.
fn classify_degradation(security: &TunnelHealth, transport: &TransportHealth) -> DegradationCause {
    use super::HealthStatus;

    if let Some(loss) = transport.packet_loss {
        if loss >= DEGRADED_PACKET_LOSS {
            return DegradationCause::TransportPacketLoss;
        }
    }

    if let Some(latency) = transport.latency_ms {
        if latency >= DEGRADED_LATENCY_MS {
            return DegradationCause::TransportLatency;
        }
    }

    if transport.connection_status != HealthStatus::Healthy {
        return DegradationCause::TransportConnectivity;
    }

    if security.encryption_status != HealthStatus::Healthy {
        return DegradationCause::AuthFailure;
    }

    if key_rotation_stale(security) {
        return DegradationCause::KeyRotation;
    }

    if transport.status == HealthStatus::Degraded {
        return DegradationCause::TransportConnectivity;
    }

    if security.status == HealthStatus::Degraded {
        return DegradationCause::Unknown;
    }

    DegradationCause::Unknown
}

fn key_rotation_stale(security: &TunnelHealth) -> bool {
    use super::HealthStatus;

    if security.status != HealthStatus::Degraded
        || security.encryption_status != HealthStatus::Healthy
    {
        return false;
    }

    match security.last_key_rotation {
        None => false,
        Some(rotated_at) => rotated_at
            .elapsed()
            .map_or(true, |elapsed| elapsed >= KEY_ROTATION_STALE),
    }
}

fn is_local_transport(endpoint: &TransportEndpoint) -> bool {
    matches!(
        endpoint.protocol.as_str(),
        "uds" | "unix" | "abstract" | "unix-stream"
    ) || std::path::Path::new(&endpoint.address)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
}

fn build_tcp_fallback(endpoint: &TransportEndpoint) -> Option<TransportEndpoint> {
    let (address, port) = resolve_tcp_fallback_env(&endpoint.node_id)?;
    build_tcp_fallback_endpoint(endpoint, address, port)
}

fn build_tcp_fallback_endpoint(
    endpoint: &TransportEndpoint,
    address: String,
    port: u16,
) -> Option<TransportEndpoint> {
    if !is_local_transport(endpoint) {
        return None;
    }

    Some(TransportEndpoint {
        node_id: endpoint.node_id.clone(),
        address,
        port,
        protocol: "tcp".to_string(),
        secure: endpoint.secure,
    })
}

fn resolve_tcp_fallback_env(node_id: &str) -> Option<(String, u16)> {
    let prefix = node_id.to_uppercase().replace('-', "_");
    let tcp_env = std::env::var(format!("{prefix}_TCP")).ok()?;
    parse_tcp_fallback_value(&tcp_env)
}

fn parse_tcp_fallback_value(tcp_env: &str) -> Option<(String, u16)> {
    if let Some((host, port_str)) = tcp_env.split_once(':') {
        let port = port_str.parse().ok()?;
        return Some((host.to_string(), port));
    }

    let port = tcp_env.parse().ok()?;
    Some((
        biomeos_types::constants::endpoints::DEFAULT_LOCALHOST.to_string(),
        port,
    ))
}

/// Reasons why a tunnel might be degraded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DegradationCause {
    /// Transport experiencing high latency
    TransportLatency,
    /// Transport experiencing elevated packet loss
    TransportPacketLoss,
    /// Transport connectivity is unstable
    TransportConnectivity,
    /// Authentication or encryption handshake failures
    AuthFailure,
    /// Security keys require rotation
    KeyRotation,
    /// Cause could not be determined from available metrics
    Unknown,
}

#[cfg(test)]
#[path = "btsp_tests/mod.rs"]
mod btsp_tests;
