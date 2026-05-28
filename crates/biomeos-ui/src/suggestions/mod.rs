// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! AI-powered suggestions for Interactive UI
//!
//! Integrates with AI-capable primals to provide intelligent suggestions
//! for device assignments, optimizations, and bottleneck predictions.
//!
//! Deep Debt Principles:
//! - No hardcoding (discover AI provider via capabilities)
//! - Modern async Rust (tokio)
//! - No unsafe code
//! - Graceful degradation (works without AI)

mod manager;
pub mod types;

pub use manager::AISuggestionManager;
pub use types::*;

#[cfg(test)]
#[path = "suggestions_tests.rs"]
mod tests;
