// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Network Services Specifications
//!
//! This module contains network service types including DNS, IPAM,
//! Service Mesh, and advanced networking configurations.

pub mod dns_ipam;
pub mod mesh;
pub mod routing;
pub mod traffic;

pub use dns_ipam::*;
pub use mesh::*;
pub use routing::*;
pub use traffic::*;

#[cfg(test)]
#[path = "../networking_services_tests.rs"]
mod tests;
