// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Ratatui rendering implementations for TUI widgets.
//!
//! This module contains the WidgetRenderer struct and its render_* methods that
//! draw to ratatui Frames and Rects.

/// Advanced widget renderer for ecosystem interface
pub struct WidgetRenderer;

mod impl_full;
#[cfg(all(test, feature = "deprecated-tui"))]
mod tests;
