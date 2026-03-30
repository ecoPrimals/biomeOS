// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Dark Forest Encrypted Beacon System
//!
//! Implements the `BirdSong` Dark Forest trust model:
//! - Encrypted beacons that reveal nothing to outsiders
//! - Only family members can decrypt and understand broadcasts
//! - Lineage verification after discovery
//!
//! ## Architecture
//!
//! ```text
//! Broadcast: [encrypted_beacon]
//!     │
//!     ├── Family member: Decrypt → See node, socket, capabilities
//!     └── Attacker: Decryption fails → See only noise
//! ```

mod beacon;
mod types;

#[cfg(test)]
mod tests;

pub use beacon::DarkForestBeacon;
pub use types::{BeaconPlaintext, DiscoveredPeer, EncryptedBeacon};
