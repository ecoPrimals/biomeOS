//! # biomeOS - Universal Biological Computing Platform
//!
//! biomeOS provides a unified platform for orchestrating the five Primals
//! (Songbird, NestGate, Toadstool, BearDog, Squirrel) into a cohesive
//! biological computing environment.

pub use biomeos_core::*;
pub mod universal_adapter;

/// Universal UI types for examples - minimal implementation
pub mod universal_ui {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UIFeatures {
        pub ai_assistant: bool,
        pub real_time_monitoring: bool,
        pub deployment_wizard: bool,
        pub service_management: bool,
        pub log_viewer: bool,
        pub metrics_dashboard: bool,
        pub custom_dashboards: bool,
        pub multi_primal_coordination: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CustomPrimalConfig {
        pub name: String,
        pub endpoints: Vec<String>,
        pub capabilities: Vec<String>,
        pub configuration: serde_json::Value,
        pub auth_config: Option<serde_json::Value>,
        pub description: String,
        pub ui_config: PrimalUIConfig,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UniversalUIConfig {
        pub theme: String,
        pub mode: crate::UIMode,
        pub features: crate::UIFeatures,
        pub custom_primals: std::collections::HashMap<String, CustomPrimalConfig>,
        pub primal_endpoints: std::collections::HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PrimalUIConfig {
        pub name: String,
        pub enabled: bool,
        pub display_name: String,
        pub icon: String,
        pub color: String,
        pub dashboard_widgets: Vec<WidgetConfig>,
        pub custom_actions: Vec<ActionConfig>,
        pub metrics_config: MetricsConfig,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WidgetConfig {
        pub widget_type: String,
        pub title: String,
        pub api_endpoint: String,
        pub refresh_interval_secs: u64,
        pub display_config: HashMap<String, serde_json::Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ActionConfig {
        pub action_id: String,
        pub label: String,
        pub icon: String,
        pub api_endpoint: String,
        pub method: String,
        pub parameters: Vec<ParameterConfig>,
        pub confirmation_message: Option<String>,
        pub confirmation_required: bool,
        pub display_name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ParameterConfig {
        pub name: String,
        pub param_type: String,
        pub required: bool,
        pub description: String,
        pub default_value: Option<serde_json::Value>,
        pub validation: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MetricsConfig {
        pub enabled: bool,
        pub chart_types: Vec<String>,
        pub metrics_endpoint: String,
        pub default_time_range: String,
    }

    impl Default for ActionConfig {
        fn default() -> Self {
            Self {
                action_id: String::new(),
                label: String::new(),
                icon: String::new(),
                api_endpoint: String::new(),
                method: "GET".to_string(),
                parameters: Vec::new(),
                confirmation_message: None,
                confirmation_required: false,
                display_name: String::new(),
            }
        }
    }

    impl Default for MetricsConfig {
        fn default() -> Self {
            Self {
                enabled: true,
                chart_types: vec!["line".to_string(), "bar".to_string()],
                metrics_endpoint: "/metrics".to_string(),
                default_time_range: "1h".to_string(),
            }
        }
    }

    impl Default for UniversalUIConfig {
        fn default() -> Self {
            Self {
                theme: "default".to_string(),
                mode: crate::UIMode::Auto,
                features: crate::UIFeatures::default(),
                custom_primals: std::collections::HashMap::new(),
                primal_endpoints: std::collections::HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UniversalUIManager {
        pub config: UniversalUIConfig,
        pub status: crate::SystemStatus,
    }

    impl UniversalUIManager {
        pub fn new(config: UniversalUIConfig) -> Self {
            Self {
                config,
                status: crate::SystemStatus {
                    healthy: true,
                    uptime: std::time::Duration::from_secs(0),
                    primals: vec![],
                    total_primals: 0,
                    healthy_primals: 0,
                    ui_mode: crate::UIMode::Auto,
                    last_discovery: None,
                },
            }
        }

        pub async fn handle_user_input(
            &self,
            _input: crate::UserInput,
        ) -> Result<crate::UIResponse, anyhow::Error> {
            Ok(crate::UIResponse {
                success: true,
                message: "Input processed".to_string(),
                data: None,
            })
        }

        pub async fn get_system_status(&self) -> Result<crate::SystemStatus, anyhow::Error> {
            Ok(self.status.clone())
        }

        pub async fn start(&self) -> Result<(), anyhow::Error> {
            // Mock implementation for examples
            Ok(())
        }
    }
}

/// Re-export ecosystem integration for external use
pub mod ecosystem {
    pub use biomeos_core::ecosystem_integration::*;
}

/// Re-export universal adapter for federation coordination
pub mod federation {
    pub use crate::universal_adapter::{
        BiomeOSUniversalAdapter, CoordinationSession, DeploymentStatus, FederationStatus,
        SessionStatus, UniversalCoordination,
    };
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!("biomeOS v", env!("CARGO_PKG_VERSION"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));  // Version should contain dots
    }

    #[test]
    fn test_build_info() {
        assert!(BUILD_INFO.contains("biomeOS"));
    }
}
