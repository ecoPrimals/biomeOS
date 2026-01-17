// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Encrypted Storage Layer for NUCLEUS
//!
//! This module provides encryption-by-default for all data stored in NestGate.
//! All operations transparently encrypt data before storage and decrypt after retrieval.
//!
//! # Architecture
//!
//! ```text
//! Application
//!     ↓
//! EncryptedStorage (this module)
//!     ├→ BearDog (encryption/decryption)
//!     └→ NestGate (storage backend)
//! ```
//!
//! # Features
//!
//! - **Transparent Encryption**: Applications don't need to change
//! - **Hardware Accelerated**: Uses CPU AES-NI for <5% overhead
//! - **Ephemeral Keys**: Per-dataset keys managed by BearDog
//! - **Authenticated Encryption**: AES-256-GCM prevents tampering
//! - **Zero-Knowledge Metadata**: Only hashes and key refs stored
//!
//! # Performance
//!
//! - **Latency**: <100µs per MB (with AES-NI)
//! - **Throughput**: >1 GB/s (hardware-accelerated)
//! - **Overhead**: <5% vs plaintext storage
//!
//! # Example
//!
//! ```no_run
//! use biomeos_core::encrypted_storage::EncryptedStorage;
//! use biomeos_core::clients::beardog::BearDogClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize BearDog and encrypted storage
//!     let beardog = BearDogClient::discover("nat0").await?;
//!     let storage = EncryptedStorage::new(beardog).await?;
//!     
//!     // Store data (automatically encrypted)
//!     storage.store("my-dataset", b"sensitive data").await?;
//!     
//!     // Retrieve data (automatically decrypted)
//!     let data = storage.retrieve("my-dataset").await?;
//!     
//!     assert_eq!(data, b"sensitive data");
//!     
//!     Ok(())
//! }
//! ```

pub mod backend;
pub mod metadata;
pub mod types;

#[cfg(test)]
mod tests;

pub use backend::EncryptedStorage;
pub use metadata::EncryptionMetadata;
pub use types::{EncryptedBlob, StorageBackend};
