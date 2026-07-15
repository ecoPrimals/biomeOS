// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! CapabilityRegistry Tests
//!
//! Extracted from capability_registry.rs to maintain files under 1000 lines.
//! Tests cover registration, discovery, heartbeats, unregistration, and edge cases.
//! Additional tests: `capability_registry_tests2/`.

mod lifecycle;
mod registration;
mod serialization;
