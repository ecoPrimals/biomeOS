// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! BearDog client for security, cryptography, and BTSP tunneling
//!
//! BearDog is the security and cryptography primal. It provides:
//! - Encryption and decryption
//! - Key management
//! - Digital signatures
//! - Access control validation
//! - BTSP (BirdSong Tunnel Protocol) tunnel management
//! - BirdSong genetic cryptography (lineage-aware encryption)
//!
//! # Architecture
//!
//! This module is organized by domain:
//!
//! - **[`client`]** - Core client, discovery, connection management
//! - **[`crypto`]** - Encryption, decryption, signing, verification
//! - **[`keys`]** - Key generation and lifecycle management
//! - **[`access`]** - Access control and audit logging
//! - **[`tunnels`]** - Low-level BTSP tunnel operations
//! - **[`btsp`]** - High-level BTSP tunnel API (user-facing)
//! - **[`types`]** - Shared types and data structures
//!
//! # Transport Evolution
//!
//! **NEW**: Auto-discovery via Unix socket (JSON-RPC 2.0)
//! - **PRIMARY**: JSON-RPC over Unix socket (100x faster, secure)
//! - **FALLBACK**: HTTP REST API (deprecated, legacy only)
//!
//! # Quick Start
//!
//! ```no_run
//! use biomeos_core::clients::beardog::BearDogClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Auto-discover via Unix socket
//!     let beardog = BearDogClient::discover("nat0").await?;
//!     
//!     // Encrypt data
//!     let encrypted = beardog.encrypt("secret data", "my-key").await?;
//!     
//!     // Decrypt data
//!     let decrypted = beardog.decrypt(&encrypted.ciphertext, "my-key").await?;
//!     
//!     // Establish tunnel
//!     let tunnel = beardog.establish_tunnel("peer-node-1", "192.168.1.100:9091").await?;
//!     
//!     Ok(())
//! }
//! ```

// Module declarations
pub mod access;
pub mod btsp;
pub mod client;
pub mod crypto;
pub mod keys;
pub mod tunnels;
pub mod types;

// Re-export main types
pub use client::BearDogClient;
pub use types::{
    AccessDecision, AccessRequest, AuditEntry, EncryptedData, KeyInfo, Signature, TunnelInfo,
    TunnelStatus,
};
