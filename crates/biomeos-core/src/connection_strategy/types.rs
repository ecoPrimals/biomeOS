// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Data types for multi-tier connection strategy (tiers, NAT classification, rendezvous payloads).

use serde::{Deserialize, Serialize};

/// Result of a connection strategy decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    /// Which tier succeeded
    pub tier: ConnectionTier,
    /// The connection endpoint (socket path, address, relay session ID)
    pub endpoint: String,
    /// How long the strategy took (ms)
    pub elapsed_ms: u64,
    /// Tiers that were attempted before success
    pub tiers_attempted: Vec<ConnectionTier>,
}

/// Connection tier in priority order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionTier {
    /// Tier 1: Direct LAN connection (no NAT traversal needed)
    LanDirect,
    /// Tier 2: Direct UDP hole punch (both peers have predictable NAT)
    DirectPunch,
    /// Tier 3: Relay-assisted coordinated punch (symmetric NAT, port prediction)
    CoordinatedPunch,
    /// Tier 4: Pure relay (always works, higher latency)
    PureRelay,
}

impl std::fmt::Display for ConnectionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LanDirect => write!(f, "LAN Direct"),
            Self::DirectPunch => write!(f, "Direct Punch"),
            Self::CoordinatedPunch => write!(f, "Coordinated Punch"),
            Self::PureRelay => write!(f, "Pure Relay"),
        }
    }
}

/// NAT type as detected by STUN
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatType {
    /// No NAT (public IP)
    None,
    /// Full cone NAT (most permissive)
    FullCone,
    /// Address-restricted cone NAT
    AddressRestricted,
    /// Port-restricted cone NAT
    PortRestricted,
    /// Symmetric NAT (hardest to traverse)
    Symmetric,
    /// Could not determine
    Unknown,
}

impl NatType {
    /// Whether this NAT type is symmetric (requiring relay-assisted punch)
    #[must_use] 
    pub const fn is_symmetric(&self) -> bool {
        matches!(self, Self::Symmetric)
    }

    /// Whether direct hole punching is likely to succeed
    #[must_use] 
    pub const fn supports_direct_punch(&self) -> bool {
        matches!(
            self,
            Self::None | Self::FullCone | Self::AddressRestricted | Self::PortRestricted
        )
    }

    /// Parse from a STUN detection result string
    #[must_use] 
    pub fn from_detection(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" | "public" | "open" => Self::None,
            "full_cone" | "full-cone" | "fullcone" => Self::FullCone,
            "address_restricted" | "address-restricted" => Self::AddressRestricted,
            "port_restricted" | "port-restricted" => Self::PortRestricted,
            "symmetric" => Self::Symmetric,
            _ => Self::Unknown,
        }
    }
}

/// Port allocation pattern detected by multi-probe STUN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortPattern {
    /// Ports allocated sequentially (step between consecutive allocations)
    Sequential {
        /// Step between consecutive port allocations
        step: i32,
        /// Last observed port
        last_port: u16,
        /// Predicted next port
        predicted_next: u16,
        /// Confidence in prediction (0.0 - 1.0)
        confidence: f64,
    },
    /// Random port allocation (unpredictable)
    Random {
        /// Observed ports
        observed: Vec<u16>,
    },
    /// Could not determine pattern
    Unknown,
}

impl PortPattern {
    /// Whether the pattern is predictable enough for coordinated punch
    #[must_use] 
    pub fn is_predictable(&self) -> bool {
        matches!(
            self,
            Self::Sequential { confidence, .. } if *confidence >= 0.6
        )
    }

    /// Parse from a JSON value returned by `stun.probe_port_pattern`
    pub fn from_json(value: &serde_json::Value) -> Self {
        if let Some(pattern_type) = value.get("type").and_then(|v| v.as_str()) {
            match pattern_type {
                "sequential" => {
                    let step = value
                        .get("step")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(1) as i32;
                    let last_port = value
                        .get("last_port")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as u16;
                    let predicted_next = value
                        .get("predicted_next")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as u16;
                    let confidence = value
                        .get("confidence")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0);
                    Self::Sequential {
                        step,
                        last_port,
                        predicted_next,
                        confidence,
                    }
                }
                "random" => {
                    let observed = value
                        .get("observed")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().map(|n| n as u16))
                                .collect()
                        })
                        .unwrap_or_default();
                    Self::Random { observed }
                }
                _ => Self::Unknown,
            }
        } else {
            Self::Unknown
        }
    }
}

/// Connection info exchanged during rendezvous
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectionInfo {
    /// STUN-discovered public address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stun_results: Option<StunResults>,
    /// Family relay endpoint (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay_endpoint: Option<String>,
    /// Self-hosted STUN server address (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stun_server: Option<String>,
}

/// STUN probe results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunResults {
    /// Our public address as seen by STUN
    pub public_addr: String,
    /// Detected NAT type
    pub nat_type: String,
}
