//! BiomeOS CLI Library
//!
//! This crate provides command-line interface utilities for BiomeOS,
//! including TUI components, discovery, formatting, and health monitoring.

#![warn(missing_docs)]
#![deny(unsafe_code)]

/// CLI command implementations
pub mod commands;
/// Service discovery utilities
pub mod discovery;
/// Output formatting utilities
pub mod formatting;
/// Health monitoring utilities
pub mod health;
/// Terminal user interface components
pub mod tui;

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
            "info" => tracing::Level::INFO,
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
