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
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Auto-detect from system
    Auto,
    /// Custom theme name
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
    /// Grid-based layout
    Grid,
    /// List-based layout
    List,
    /// Card-based layout
    Cards,
    /// Custom layout
    Custom(String),
}

/// Dashboard views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardView {
    /// System overview
    Overview,
    /// Metrics dashboard
    Metrics,
    /// Log viewer
    Logs,
    /// Services status
    Services,
    /// Configuration editor
    Configuration,
    /// Custom view
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
    /// System statistics widget
    SystemStats,
    /// Service status widget
    ServiceStatus,
    /// Log viewer widget
    LogViewer,
    /// Metrics chart widget
    MetricsChart,
    /// Alert summary widget
    AlertSummary,
    /// Custom widget
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
                    size: WidgetSize {
                        width: 2,
                        height: 1,
                    },
                    config: HashMap::new(),
                },
                WidgetConfig {
                    id: "service_status".to_string(),
                    widget_type: WidgetType::ServiceStatus,
                    position: WidgetPosition { row: 0, column: 2 },
                    size: WidgetSize {
                        width: 2,
                        height: 1,
                    },
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ═══════════════════════════════════════════════════════════════════════
    // FeatureFlags Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_feature_flags_default() {
        let flags = FeatureFlags::default();
        assert!(!flags.experimental);
        assert!(!flags.debug);
        assert!(flags.telemetry);
        assert!(flags.ai_first);
        assert!(!flags.auto_scaling);
        assert!(!flags.federation);
        assert!(flags.sandboxing);
        assert!(!flags.crypto_locks);
        assert!(flags.custom.is_empty());
    }

    #[test]
    fn test_feature_flags_serialization() {
        let flags = FeatureFlags::default();
        let json = serde_json::to_string(&flags).expect("serialize");
        let deserialized: FeatureFlags = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.ai_first);
    }

    #[test]
    fn test_feature_flags_custom() {
        let mut flags = FeatureFlags::default();
        flags.custom.insert("beta_feature".to_string(), true);
        flags.custom.insert("alpha_feature".to_string(), false);
        assert_eq!(flags.custom.len(), 2);
        assert_eq!(flags.custom.get("beta_feature"), Some(&true));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UITheme Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_ui_theme_serialization() {
        for theme in [
            UITheme::Light,
            UITheme::Dark,
            UITheme::Auto,
            UITheme::Custom("dracula".to_string()),
        ] {
            let json = serde_json::to_string(&theme).expect("serialize");
            let _: UITheme = serde_json::from_str(&json).expect("deserialize");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UIConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_ui_config_default() {
        let config = UIConfig::default();
        assert!(config.enabled);
        assert!(matches!(config.theme, UITheme::Auto));
        assert_eq!(config.language, "en");
        assert!(config.timezone.is_none());
        assert!(config.custom.is_empty());
    }

    #[test]
    fn test_ui_config_serialization() {
        let config = UIConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: UIConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.enabled);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DashboardConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert!(config.enabled);
        assert!(matches!(config.layout, DashboardLayout::Grid));
        assert_eq!(config.refresh_interval, Duration::from_secs(30));
        assert!(matches!(config.default_view, DashboardView::Overview));
        assert_eq!(config.widgets.len(), 2);
    }

    #[test]
    fn test_dashboard_layout_serialization() {
        for layout in [
            DashboardLayout::Grid,
            DashboardLayout::List,
            DashboardLayout::Cards,
            DashboardLayout::Custom("masonry".to_string()),
        ] {
            let json = serde_json::to_string(&layout).expect("serialize");
            let _: DashboardLayout = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_dashboard_view_serialization() {
        for view in [
            DashboardView::Overview,
            DashboardView::Metrics,
            DashboardView::Logs,
            DashboardView::Services,
            DashboardView::Configuration,
            DashboardView::Custom("primals".to_string()),
        ] {
            let json = serde_json::to_string(&view).expect("serialize");
            let _: DashboardView = serde_json::from_str(&json).expect("deserialize");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WidgetConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_widget_config_creation() {
        let widget = WidgetConfig {
            id: "cpu_chart".to_string(),
            widget_type: WidgetType::MetricsChart,
            position: WidgetPosition { row: 1, column: 0 },
            size: WidgetSize {
                width: 4,
                height: 2,
            },
            config: HashMap::new(),
        };
        assert_eq!(widget.id, "cpu_chart");
        assert_eq!(widget.position.row, 1);
        assert_eq!(widget.size.width, 4);
    }

    #[test]
    fn test_widget_type_serialization() {
        for wt in [
            WidgetType::SystemStats,
            WidgetType::ServiceStatus,
            WidgetType::LogViewer,
            WidgetType::MetricsChart,
            WidgetType::AlertSummary,
            WidgetType::Custom("primal_status".to_string()),
        ] {
            let json = serde_json::to_string(&wt).expect("serialize");
            let _: WidgetType = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_widget_position_serialization() {
        let pos = WidgetPosition { row: 2, column: 3 };
        let json = serde_json::to_string(&pos).expect("serialize");
        let deserialized: WidgetPosition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.row, 2);
        assert_eq!(deserialized.column, 3);
    }

    #[test]
    fn test_widget_size_serialization() {
        let size = WidgetSize {
            width: 4,
            height: 2,
        };
        let json = serde_json::to_string(&size).expect("serialize");
        let deserialized: WidgetSize = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.width, 4);
        assert_eq!(deserialized.height, 2);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UIAuthConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_ui_auth_config_default() {
        let config = UIAuthConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.session_timeout, Duration::from_secs(3600));
        assert!(config.remember_me);
    }

    #[test]
    fn test_login_page_config_default() {
        let config = LoginPageConfig::default();
        assert!(config.logo_url.is_none());
        assert_eq!(config.title, Some("BiomeOS".to_string()));
        assert!(config.footer.is_none());
        assert!(config.background_url.is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AccessibilityConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_accessibility_config_default() {
        let config = AccessibilityConfig::default();
        assert!(!config.high_contrast);
        assert!(!config.large_fonts);
        assert!(config.screen_reader);
        assert!(config.keyboard_navigation);
        assert!(!config.reduce_motion);
    }

    #[test]
    fn test_accessibility_config_serialization() {
        let config = AccessibilityConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: AccessibilityConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.screen_reader);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EnvironmentConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_environment_config_creation() {
        let mut variables = HashMap::new();
        variables.insert("API_URL".to_string(), "https://api.example.com".to_string());

        let config = EnvironmentConfig {
            name: "production".to_string(),
            description: Some("Production environment".to_string()),
            variables,
            features: FeatureFlags::default(),
            limits: EnvironmentLimits::default(),
            endpoints: HashMap::new(),
        };
        assert_eq!(config.name, "production");
        assert!(config.description.is_some());
    }

    #[test]
    fn test_environment_limits_default() {
        let limits = EnvironmentLimits::default();
        assert!(limits.max_users.is_none());
        assert!(limits.max_sessions.is_none());
        assert!(limits.rate_limit.is_none());
        assert_eq!(limits.retention_days, Some(30));
        assert!(limits.storage_limit.is_none());
    }

    #[test]
    fn test_environment_limits_custom() {
        let limits = EnvironmentLimits {
            max_users: Some(1000),
            max_sessions: Some(5000),
            rate_limit: Some(1000),
            retention_days: Some(90),
            storage_limit: Some(100 * 1024 * 1024 * 1024),
        };
        assert_eq!(limits.max_users, Some(1000));
        assert_eq!(limits.storage_limit, Some(100 * 1024 * 1024 * 1024));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Clone & Debug Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_feature_flags_clone() {
        let original = FeatureFlags::default();
        let cloned = original.clone();
        assert_eq!(cloned.ai_first, original.ai_first);
    }

    #[test]
    fn test_ui_config_clone() {
        let original = UIConfig::default();
        let cloned = original.clone();
        assert_eq!(cloned.language, original.language);
    }

    #[test]
    fn test_dashboard_config_clone() {
        let original = DashboardConfig::default();
        let cloned = original.clone();
        assert_eq!(cloned.widgets.len(), original.widgets.len());
    }

    #[test]
    fn test_feature_flags_debug() {
        let flags = FeatureFlags::default();
        let debug = format!("{:?}", flags);
        assert!(debug.contains("ai_first"));
    }
}
