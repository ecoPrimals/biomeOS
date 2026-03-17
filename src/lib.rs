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
#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_mode_default_is_auto() {
        let mode = UIMode::default();
        assert!(matches!(mode, UIMode::Auto));
    }

    #[test]
    fn runtime_default_has_sovereignty_enabled() {
        let rt = BiomeOSRuntime::default();
        assert!(rt.sovereignty_enabled);
        assert!(rt.ai_first_enabled);
        assert!(matches!(rt.ui_mode, UIMode::Auto));
        assert!(!rt.version.is_empty());
    }

    #[test]
    fn runtime_serde_roundtrip() {
        let rt = BiomeOSRuntime::default();
        let json = serde_json::to_string(&rt).expect("serialize");
        let parsed: BiomeOSRuntime = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.version, rt.version);
        assert_eq!(parsed.ai_first_enabled, rt.ai_first_enabled);
        assert_eq!(parsed.sovereignty_enabled, rt.sovereignty_enabled);
    }

    #[test]
    fn ui_mode_serde_all_variants() {
        let variants = vec![
            UIMode::Auto,
            UIMode::Terminal,
            UIMode::Web,
            UIMode::Desktop,
            UIMode::Mobile,
            UIMode::Headless,
        ];
        for mode in variants {
            let json = serde_json::to_string(&mode).expect("serialize");
            let parsed: UIMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(
                std::mem::discriminant(&mode),
                std::mem::discriminant(&parsed)
            );
        }
    }

    #[test]
    fn runtime_version_matches_cargo_pkg() {
        let rt = BiomeOSRuntime::default();
        assert_eq!(rt.version, env!("CARGO_PKG_VERSION"));
    }
}
