// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Socket Discovery Engine Tests
//!
//! Extracted from engine.rs to maintain files under 1000 lines.
//! These tests cover the SocketDiscovery functionality including:
//! - Socket path building
//! - Port calculation
//! - Environment hint discovery
//! - Cache functionality
//! - TCP and Unix socket verification
//! - Manifest and registry discovery
//! - XDG and family tmp path discovery
//!
//! Additional tests: `engine_tests2.rs`.
mod path_building;
mod env_hints;
mod cache;
mod discovery_verify;
