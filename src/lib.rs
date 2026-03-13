// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS - Universal Operating System
//!
//! A capability-based orchestration layer for managing primals and ecosystems

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// Re-export core types
pub use biomeos_core::UniversalBiomeOSManager;
pub use biomeos_types::BiomeOSConfig;

/// UI interaction modes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum UIMode {
    /// Automatically detect best UI mode
    #[default]
    Auto,

    /// Terminal/CLI interface
    Terminal,

    /// Web interface
    Web,

    /// Desktop GUI
    Desktop,

    /// Mobile interface
    Mobile,

    /// Headless (no UI)
    Headless,
}

/// BiomeOS runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSRuntime {
    /// Version information
    pub version: String,

    /// UI mode
    pub ui_mode: UIMode,

    /// Whether to enable AI-first API
    pub ai_first_enabled: bool,

    /// Whether to enable sovereignty guardian
    pub sovereignty_enabled: bool,
}

impl Default for BiomeOSRuntime {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            ui_mode: UIMode::Auto,
            ai_first_enabled: true,
            sovereignty_enabled: true,
        }
    }
}
