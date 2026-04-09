// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! AI-powered graph advisor using Squirrel integration
//!
//! This module provides AI-driven suggestions and learning for graph modifications.
//! It learns from user modifications and provides intelligent recommendations
//! through Squirrel integration.
//!
//! Deep Debt Principles:
//! - Capability-based AI provider discovery (scan `*-{family}.sock` and `capabilities.list`)
//! - No fallback to a fixed primal name when taxonomy resolution fails
//! - Graceful degradation without an AI primal
//! - Modern async Rust
//! - No unsafe code

#[path = "ai_advisor_core.rs"]
mod core;
#[path = "ai_advisor_discovery.rs"]
mod discovery;
#[path = "ai_advisor_local.rs"]
mod local;
#[path = "ai_advisor_types.rs"]
mod types;

pub use types::{
    AiSuggestion, FeedbackOutcome, GraphSnapshot, ImpactEstimate, LearningEvent,
    SuggestionFeedback, SuggestionType,
};

use local::LocalPattern;
use std::path::PathBuf;
use tokio::time::Duration;

/// AI-powered graph advisor
pub struct AiGraphAdvisor {
    /// Whether Squirrel is available
    squirrel_available: bool,

    /// Resolved AI provider socket (runtime discovery via `capabilities.list`)
    ai_socket_path: Option<PathBuf>,

    /// Timeout for Squirrel requests
    squirrel_timeout: Duration,

    /// Local suggestion cache (fallback when Squirrel unavailable)
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "populated for future local fallback suggestions")
    )]
    local_patterns: Vec<LocalPattern>,
}

#[cfg(test)]
#[path = "ai_advisor_tests.rs"]
mod tests;
