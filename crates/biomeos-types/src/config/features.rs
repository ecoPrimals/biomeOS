//! Features Configuration
//!
//! This module contains feature flags, UI configuration, and environment-specific settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable experimental features
    pub experimental: bool,

    /// Enable debug features
    pub debug: bool,

    /// Enable telemetry
    pub telemetry: bool,

    /// Enable AI-first features
    pub ai_first: bool,

    /// Enable auto-scaling
    pub auto_scaling: bool,

    /// Enable federation
    pub federation: bool,

    /// Enable sandboxing
    pub sandboxing: bool,

    /// Enable crypto locks
    pub crypto_locks: bool,

    /// Custom feature flags
    pub custom: HashMap<String, bool>,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Enable UI
    pub enabled: bool,

    /// UI theme
    pub theme: UITheme,

    /// UI language
    pub language: String,

    /// UI timezone
    pub timezone: Option<String>,

    /// Dashboard configuration
    pub dashboard: DashboardConfig,

    /// Authentication UI
    pub auth: UIAuthConfig,

    /// Accessibility settings
    pub accessibility: AccessibilityConfig,

    /// Custom UI settings
    pub custom: HashMap<String, serde_json::Value>,
}

/// UI themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UITheme {
    Light,
    Dark,
    Auto,
    Custom(String),
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Enable dashboard
    pub enabled: bool,

    /// Dashboard layout
    pub layout: DashboardLayout,

    /// Refresh interval
    pub refresh_interval: std::time::Duration,

    /// Default view
    pub default_view: DashboardView,

    /// Widget configuration
    pub widgets: Vec<WidgetConfig>,
}

/// Dashboard layouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardLayout {
    Grid,
    List,
    Cards,
    Custom(String),
}

/// Dashboard views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardView {
    Overview,
    Metrics,
    Logs,
    Services,
    Configuration,
    Custom(String),
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// Widget ID
    pub id: String,

    /// Widget type
    pub widget_type: WidgetType,

    /// Widget position
    pub position: WidgetPosition,

    /// Widget size
    pub size: WidgetSize,

    /// Widget configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Widget types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    SystemStats,
    ServiceStatus,
    LogViewer,
    MetricsChart,
    AlertSummary,
    Custom(String),
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    /// Row position
    pub row: u32,

    /// Column position
    pub column: u32,
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    /// Width in grid units
    pub width: u32,

    /// Height in grid units
    pub height: u32,
}

/// UI authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIAuthConfig {
    /// Enable UI authentication
    pub enabled: bool,

    /// Login page customization
    pub login_page: LoginPageConfig,

    /// Session timeout
    pub session_timeout: std::time::Duration,

    /// Remember me feature
    pub remember_me: bool,
}

/// Login page configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPageConfig {
    /// Custom logo URL
    pub logo_url: Option<String>,

    /// Custom title
    pub title: Option<String>,

    /// Custom footer
    pub footer: Option<String>,

    /// Background image URL
    pub background_url: Option<String>,
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// High contrast mode
    pub high_contrast: bool,

    /// Large fonts
    pub large_fonts: bool,

    /// Screen reader support
    pub screen_reader: bool,

    /// Keyboard navigation
    pub keyboard_navigation: bool,

    /// Motion reduction
    pub reduce_motion: bool,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name
    pub name: String,

    /// Environment description
    pub description: Option<String>,

    /// Environment variables
    pub variables: HashMap<String, String>,

    /// Environment-specific feature flags
    pub features: FeatureFlags,

    /// Environment-specific limits
    pub limits: EnvironmentLimits,

    /// Environment-specific endpoints
    pub endpoints: HashMap<String, String>,
}

/// Environment-specific limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentLimits {
    /// Maximum concurrent users
    pub max_users: Option<u32>,

    /// Maximum concurrent sessions
    pub max_sessions: Option<u32>,

    /// Request rate limit
    pub rate_limit: Option<u32>,

    /// Data retention period
    pub retention_days: Option<u32>,

    /// Storage limit in bytes
    pub storage_limit: Option<u64>,
}

/// Default implementations
impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            experimental: false,
            debug: false,
            telemetry: true,
            ai_first: true,
            auto_scaling: false,
            federation: false,
            sandboxing: true,
            crypto_locks: false,
            custom: HashMap::new(),
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: UITheme::Auto,
            language: "en".to_string(),
            timezone: None,
            dashboard: DashboardConfig::default(),
            auth: UIAuthConfig::default(),
            accessibility: AccessibilityConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            layout: DashboardLayout::Grid,
            refresh_interval: std::time::Duration::from_secs(30),
            default_view: DashboardView::Overview,
            widgets: vec![
                WidgetConfig {
                    id: "system_stats".to_string(),
                    widget_type: WidgetType::SystemStats,
                    position: WidgetPosition { row: 0, column: 0 },
                    size: WidgetSize { width: 2, height: 1 },
                    config: HashMap::new(),
                },
                WidgetConfig {
                    id: "service_status".to_string(),
                    widget_type: WidgetType::ServiceStatus,
                    position: WidgetPosition { row: 0, column: 2 },
                    size: WidgetSize { width: 2, height: 1 },
                    config: HashMap::new(),
                },
            ],
        }
    }
}

impl Default for UIAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            login_page: LoginPageConfig::default(),
            session_timeout: std::time::Duration::from_secs(60 * 60), // 1 hour
            remember_me: true,
        }
    }
}

impl Default for LoginPageConfig {
    fn default() -> Self {
        Self {
            logo_url: None,
            title: Some("BiomeOS".to_string()),
            footer: None,
            background_url: None,
        }
    }
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            high_contrast: false,
            large_fonts: false,
            screen_reader: true,
            keyboard_navigation: true,
            reduce_motion: false,
        }
    }
}

impl Default for EnvironmentLimits {
    fn default() -> Self {
        Self {
            max_users: None,
            max_sessions: None,
            rate_limit: None,
            retention_days: Some(30),
            storage_limit: None,
        }
    }
} 