// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Networking Specifications
//!
//! This module contains networking-related manifest types including NetworkSpec,
//! network policies, and routing configurations.
//!
//! The networking types have been split into logical modules:
//! - `networking_core` - Core networking types and configurations
//! - `networking_policies` - Network policy and security specifications  
//! - `networking_services` - DNS, IPAM, and Service Mesh configurations

// Re-export all types from the split modules
pub use super::networking_core::*;
pub use super::networking_policies::*; 
pub use super::networking_services::*; 