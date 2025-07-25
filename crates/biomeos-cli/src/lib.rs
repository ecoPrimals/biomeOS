use anyhow::Result;
use biomeos_core::{BiomeOSConfig, SystemHealth, UniversalBiomeOSManager};
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth};
use chrono::{DateTime, Utc};
use colored::*;
use comfy_table::{presets::UTF8_FULL, Table};
use thiserror::Error;
use tracing::Level;

pub mod discovery;
pub mod formatting;
pub mod health;
pub mod monitoring;
pub mod tui;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("BiomeOS core error: {0}")]
    BiomeOSCore(#[from] biomeos_core::BiomeError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// CLI utilities and common functionality
pub struct CliUtils;

impl CliUtils {
    /// Initialize logging with specified level
    pub fn init_logging(level: &str) -> Result<(), CliError> {
        let log_level = match level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };

        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .with_target(false)
            .init();

        Ok(())
    }

    /// Create default BiomeOS configuration
    pub fn default_config() -> BiomeOSConfig {
        BiomeOSConfig::default()
    }

    /// Initialize UniversalBiomeOSManager
    pub fn init_manager() -> UniversalBiomeOSManager {
        let config = Self::default_config();
        UniversalBiomeOSManager::new(config)
    }

    /// Format PrimalHealth with colors
    pub fn format_health(health: &PrimalHealth) -> String {
        match health {
            PrimalHealth::Healthy => "● Healthy".green().to_string(),
            PrimalHealth::Degraded => "◐ Degraded".yellow().to_string(),
            PrimalHealth::Unhealthy => "● Unhealthy".red().to_string(),
            PrimalHealth::Unknown => "? Unknown".bright_black().to_string(),
        }
    }

    /// Format system health status
    pub fn format_system_health_status(health: &SystemHealth) -> String {
        format!("{:?}", health.overall_status)
            .color(match health.overall_status {
                biomeos_core::HealthStatus::Healthy => "green",
                biomeos_core::HealthStatus::Degraded => "yellow",
                biomeos_core::HealthStatus::Warning => "yellow",
                biomeos_core::HealthStatus::Critical => "red",
                biomeos_core::HealthStatus::Unhealthy => "red",
                biomeos_core::HealthStatus::Unknown => "bright_black",
            })
            .to_string()
    }

    /// Format capabilities list
    pub fn format_capabilities(capabilities: &[PrimalCapability]) -> String {
        capabilities
            .iter()
            .map(|cap| format!("{}/{}", cap.domain.cyan(), cap.name.green()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format timestamp
    pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
        timestamp
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string()
            .bright_black()
            .to_string()
    }

    /// Format duration in human-readable format
    pub fn format_duration(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Format bytes in human-readable format
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit = 0;

        while size >= 1024.0 && unit < UNITS.len() - 1 {
            size /= 1024.0;
            unit += 1;
        }

        format!("{:.1} {}", size, UNITS[unit])
    }

    /// Create a styled table
    pub fn create_table() -> Table {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        // Note: Removed specific styling calls that were causing compilation errors
        // The UTF8_FULL preset provides good default styling
        table
    }

    /// Print colored header
    pub fn print_header(text: &str) {
        println!("{}", text.bright_cyan().bold());
        println!("{}", "=".repeat(text.len()).bright_black());
    }

    /// Print colored section
    pub fn print_section(text: &str) {
        println!("\n{}", text.yellow().bold());
        println!("{}", "-".repeat(text.len()).bright_black());
    }

    /// Print success message
    pub fn print_success(text: &str) {
        println!("{} {}", "✅".green(), text);
    }

    /// Print warning message
    pub fn print_warning(text: &str) {
        println!("{} {}", "⚠️".yellow(), text.yellow());
    }

    /// Print error message
    pub fn print_error(text: &str) {
        println!("{} {}", "❌".red(), text.red());
    }

    /// Print info message
    pub fn print_info(text: &str) {
        println!("{} {}", "ℹ️".blue(), text);
    }
}
