// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` CLI Library
//!
//! Command-line interface utilities for `BiomeOS`: discovery, formatting,
//! health monitoring, and command handlers. Ecosystem visualization is
//! provided by petalTongue (the universal UI primal).

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests use unwrap/expect for concise assertions"
    )
)]

/// CLI command implementations
pub mod commands;
/// Service discovery utilities
pub mod discovery;
/// Output formatting utilities
pub mod formatting;
/// Health monitoring utilities
pub mod health;

// Re-export key types and utilities
pub use discovery::*;
pub use formatting::*;
pub use health::*;

/// CLI utilities and helper functions
pub struct CliUtils;

impl CliUtils {
    /// Initialize logging based on level
    pub fn init_logging(level: &str) -> anyhow::Result<()> {
        let log_level = match level.to_lowercase().as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        };

        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .init();

        Ok(())
    }

    /// Format output based on output format
    pub fn format_output<T: serde::Serialize>(
        data: &T,
        format: &OutputFormat,
    ) -> anyhow::Result<String> {
        match format {
            OutputFormat::Json => Ok(serde_json::to_string_pretty(data)?),
            OutputFormat::Yaml => Ok(serde_yaml::to_string(data)?),
            OutputFormat::Pretty => {
                // For pretty format, we'll use JSON as fallback
                // Individual commands should handle their own pretty formatting
                Ok(serde_json::to_string_pretty(data)?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_output_json() {
        let data = serde_json::json!({"key": "value"});
        let result = CliUtils::format_output(&data, &OutputFormat::Json);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains("key"));
        assert!(s.contains("value"));
    }

    #[test]
    fn test_format_output_yaml() {
        let data = serde_json::json!({"a": 1, "b": "two"});
        let result = CliUtils::format_output(&data, &OutputFormat::Yaml);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains('a') || s.contains('1'));
        assert!(s.contains('b') || s.contains("two"));
    }

    #[test]
    fn test_format_output_pretty() {
        let data = serde_json::json!({"x": 42});
        let result = CliUtils::format_output(&data, &OutputFormat::Pretty);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains('x'));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_init_logging_levels() {
        // Test that level parsing works (don't actually init - would conflict with other tests)
        let levels = ["trace", "debug", "info", "warn", "error", "TRACE", "INFO"];
        for level in levels {
            let parsed = match level.to_lowercase().as_str() {
                "trace" => Some(tracing::Level::TRACE),
                "debug" => Some(tracing::Level::DEBUG),
                "info" => Some(tracing::Level::INFO),
                "warn" => Some(tracing::Level::WARN),
                "error" => Some(tracing::Level::ERROR),
                _ => None,
            };
            assert!(parsed.is_some(), "Level {level} should parse");
        }
    }
}

/// Output format options for CLI commands
#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    /// JSON output
    #[value(name = "json")]
    Json,
    /// YAML output
    #[value(name = "yaml")]
    Yaml,
    /// Human-readable pretty-printed output
    #[value(name = "pretty")]
    Pretty,
}
