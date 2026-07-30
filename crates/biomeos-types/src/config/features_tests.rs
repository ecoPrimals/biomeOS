
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
    let debug = format!("{flags:?}");
    assert!(debug.contains("ai_first"));
}
