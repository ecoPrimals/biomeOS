// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Monitor Command Handlers
//!
//! Handles monitoring operations including system monitoring,
//! dashboard display, log handling, exec operations, and scaling.

mod display;
mod format;
mod handlers;
#[cfg(test)]
mod tests;

pub use handlers::{handle_dashboard, handle_exec, handle_logs, handle_monitor, handle_scale};
