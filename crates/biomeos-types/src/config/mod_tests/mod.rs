// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Configuration module tests (split from `mod_tests.rs`).
//!
//! Domain-focused modules:
//! - [`defaults_metadata`] — defaults, metadata, clone/debug
//! - [`validation`] — config validation rules
//! - [`builder`] — builder pattern
//! - [`merge`] — merge behavior
//! - [`serialization`] — serde round-trips
//! - [`env_vars`] — environment variable overrides
//! - [`file_io`] — file load/save
//! - [`environments`] — environment-specific configuration
//! - [`subconfigs`] — sub-config defaults and variants

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

mod builder;
mod common;
mod defaults_metadata;
mod env_vars;
mod environments;
mod file_io;
mod merge;
mod serialization;
mod subconfigs;
mod validation;
