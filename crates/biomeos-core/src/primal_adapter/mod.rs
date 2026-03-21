// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Primal Adapter Pattern - CLI Agnostic Primal Integration
//!
//! This module provides adaptive integration with primals, learning their interfaces
//! rather than enforcing a universal contract. Respects primal sovereignty while
//! enabling seamless BiomeOS orchestration.
//!
//! # Philosophy
//!
//! - **CLI Agnostic**: Learn each primal's interface pattern
//! - **Future-Proof**: Handle primal evolution automatically
//! - **Sovereignty-First**: Primals control their own interfaces
//! - **Graceful**: Missing capabilities = degradation, not failure
//!
//! # Example
//!
//! ```no_run
//! use biomeos_core::primal_adapter::discover_primal_interface;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Discover how to interact with a primal
//! let mut adapter = discover_primal_interface(Path::new("./squirrel-bin")).await?;
//!
//! // Start the primal using discovered interface
//! adapter.start(9010).await?;
//!
//! // Check health
//! if adapter.check_health().await? {
//!     println!("Primal is healthy!");
//! }
//! # Ok(())
//! # }
//! ```

mod cache;
mod discovery;
mod lifecycle;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_extended;

pub use cache::{AdapterCache, load_adapter, save_adapter};
pub use discovery::{discover_primal_interface, probe_interface_patterns};
pub use lifecycle::{LifecycleRequest, LifecycleResponse, TransitionReason};
pub use types::{
    InterfacePattern, LifecycleCapabilities, PrimalAdapter, PrimalCapabilities, PrimalInterface,
    PrimalState,
};

use anyhow::Result;
use std::path::Path;

/// Quick discovery and start - convenience function
pub async fn discover_and_start(binary: &Path, port: u16) -> Result<PrimalAdapter> {
    let mut adapter = discover_primal_interface(binary).await?;
    adapter.start(port).await?;
    Ok(adapter)
}

/// Check if primal is compatible with BiomeOS
pub async fn check_compatibility(binary: &Path) -> Result<bool> {
    match discover_primal_interface(binary).await {
        Ok(adapter) => {
            // Compatible if we can at least start it
            Ok(adapter.capabilities.lifecycle.can_start)
        }
        Err(_) => Ok(false),
    }
}
