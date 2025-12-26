// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Primal client implementations
//!
//! This module contains client implementations for communicating with all primals
//! in the ecoPrimals ecosystem:
//!
//! - **Songbird**: Service discovery and coordination
//! - **ToadStool**: Compute execution and resource metrics
//! - **Squirrel**: AI and intelligence services
//! - **NestGate**: Storage and persistence
//! - **BearDog**: Cryptography and security
//!
//! # Architecture
//!
//! All clients implement the `PrimalClient` trait and use the shared HTTP client
//! infrastructure for consistent communication patterns.
//!
//! # Discovery
//!
//! Clients are typically discovered at runtime via Songbird, though they can also
//! be configured explicitly via environment variables for development.
//!
//! # Example
//!
//! ```no_run
//! use biomeos_core::clients::songbird::SongbirdClient;
//! use biomeos_core::primal_client::PrimalClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create client from environment or discovery
//!     let songbird = SongbirdClient::new("http://localhost:3000");
//!
//!     // Check availability
//!     if songbird.is_available().await {
//!         println!("Songbird is available!");
//!
//!         // Use specialized methods
//!         let services = songbird.discover_by_capability("compute").await?;
//!         println!("Found {} compute services", services.len());
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod base;
pub mod beardog;
pub mod nestgate;
pub mod songbird;
pub mod squirrel;
pub mod toadstool;

// Re-export commonly used types
pub use base::PrimalHttpClient;
pub use beardog::BearDogClient;
pub use nestgate::NestGateClient;
pub use songbird::SongbirdClient;
pub use squirrel::SquirrelClient;
pub use toadstool::ToadStoolClient;
