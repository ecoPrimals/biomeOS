// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! TUI Widgets for Comprehensive Ecosystem Interface
//!
//! Advanced widgets for BiomeOS as the human/AI interface to a headless, AI-first ecosystem.
//! Pure computation/formatting is separated from rendering for testability.

pub mod formatting;
pub mod rendering;

pub use formatting::*;
pub use rendering::WidgetRenderer;
