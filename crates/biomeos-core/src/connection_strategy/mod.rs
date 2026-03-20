// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Connection Strategy Orchestrator
//!
//! Selects the optimal connection tier for reaching a peer, using the Neural API
//! for capability routing. This is the biomeOS-owned "brain" that decides WHAT to
//! do — primals (Songbird, BearDog) decide HOW.
//!
//! ## Multi-Tier Strategy
//!
//! ```text
//! Tier 1: LAN Direct      — mesh.auto_discover → direct socket/TCP
//! Tier 2: Direct Punch    — Both non-symmetric NAT → punch.request
//! Tier 3: Coordinated     — Either symmetric NAT → relay + stun.probe_port_pattern
//!         Punch              → punch.coordinate with predicted ports
//! Tier 4: Pure Relay      — Random NAT / punch fails → relay.allocate (always works)
//! ```
//!
//! ## Ownership Boundary
//!
//! biomeOS decides which tier to attempt. It calls Neural API `capability.call`
//! for each step. The actual UDP/STUN/relay operations are performed by Songbird.
//! Cryptographic authorization is performed by BearDog.
//!
//! See: `ecoPrimals/wateringHole/handoffs/archive/` (relay/coordinated punch handoff)

#![forbid(unsafe_code)]

mod connect;
mod types;

#[cfg(test)]
mod tests;

pub use connect::connect_to_peer;
pub use types::*;
