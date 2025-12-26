//! BiomeOS CLI Library
//!
//! This crate provides command-line interface utilities for BiomeOS,
//! including TUI components, discovery, formatting, and health monitoring.

// Core CLI modules
pub mod commands;
pub mod discovery;
pub mod formatting;
pub mod health;
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
    #[value(name = "json")]
    Json,
    #[value(name = "yaml")]
    Yaml,
    #[value(name = "pretty")]
    Pretty,
}
