// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Service Networking Configurations
//!
//! This module contains networking-related types including service networking,
//! discovery, load balancing, and traffic management.

mod service_core;
mod traffic;

pub use service_core::*;
pub use traffic::*;

#[cfg(test)]
#[path = "networking_tests.rs"]
mod networking_tests;
