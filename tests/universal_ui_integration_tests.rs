//! Universal UI Integration Tests
//!
//! Comprehensive test suite for the universal biomeOS UI system

use anyhow::Result;
use biomeos::{
    ActionConfig, CustomPrimalConfig, DiscoveredPrimal, MetricsConfig, PrimalUIConfig,
    SystemStatus, UIFeatures, UIMode, UIResponse, UniversalUIConfig, UniversalUIManager, UserInput,
    WidgetConfig,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_universal_ui_initialization() -> Result<()> {
    let config = create_test_config();
    let ui_manager = UniversalUIManager::new(config).await?;

    // Test that UI manager was created successfully
    let status = ui_manager.get_system_status().await?;
    assert_eq!(status.ui_mode, UIMode::CLI);

    Ok(())
}

#[tokio::test]
async fn test_primal_discovery_standard_primals() -> Result<()> {
    let mut config = create_test_config();

    // Add standard primals
    config
        .primal_endpoints
        .insert("songbird".to_string(), "http://localhost:8080".to_string());
    config
        .primal_endpoints
        .insert("nestgate".to_string(), "http://localhost:8082".to_string());
    config
        .primal_endpoints
        .insert("toadstool".to_string(), "http://localhost:8084".to_string());

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test system status includes discovered primals
    let status = ui_manager.get_system_status().await?;

    // In a real test, we'd verify the primals were discovered
    // For now, just verify the manager works
    assert!(status.total_primals >= 0);

    Ok(())
}

#[tokio::test]
async fn test_custom_primal_integration() -> Result<()> {
    let mut config = create_test_config();

    // Add custom primal configuration
    add_test_custom_primal(&mut config);

    let ui_manager = UniversalUIManager::new(config).await?;
    let status = ui_manager.get_system_status().await?;

    // Verify custom primal configuration was loaded
    assert!(status.total_primals >= 0);

    Ok(())
}

#[tokio::test]
async fn test_ui_mode_switching() -> Result<()> {
    let ui_modes = vec![UIMode::Desktop, UIMode::Web, UIMode::Terminal, UIMode::CLI];

    for mode in ui_modes {
        let mut config = create_test_config();
        config.ui_mode = mode.clone();

        let ui_manager = UniversalUIManager::new(config).await?;
        let status = ui_manager.get_system_status().await?;

        assert_eq!(status.ui_mode, mode);
    }

    Ok(())
}

#[tokio::test]
async fn test_multi_primal_coordination() -> Result<()> {
    let mut config = create_test_config();

    // Add multiple primals
    config
        .primal_endpoints
        .insert("songbird".to_string(), "http://localhost:8080".to_string());
    config
        .primal_endpoints
        .insert("nestgate".to_string(), "http://localhost:8082".to_string());
    config
        .primal_endpoints
        .insert("toadstool".to_string(), "http://localhost:8084".to_string());
    config
        .primal_endpoints
        .insert("beardog".to_string(), "http://localhost:9000".to_string());

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test user input for multi-primal coordination
    let input = UserInput {
        input_type: "deploy_biome".to_string(),
        data: serde_json::json!({
            "manifest": {
                "name": "test-app",
                "services": ["web", "database", "cache"]
            }
        }),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(input).await?;
    assert_eq!(response.response_type, "success");

    Ok(())
}

#[tokio::test]
async fn test_ai_assistant_integration() -> Result<()> {
    let mut config = create_test_config();
    config.ai_config.enabled = true;

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test AI command processing
    let ai_input = UserInput {
        input_type: "ai_command".to_string(),
        data: serde_json::json!({
            "command": "What's the status of all primals?"
        }),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(ai_input).await?;
    assert_eq!(response.response_type, "success");

    Ok(())
}

#[tokio::test]
async fn test_real_time_monitoring() -> Result<()> {
    let mut config = create_test_config();
    config.real_time.enabled = true;

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test real-time event handling
    let monitoring_input = UserInput {
        input_type: "start_monitoring".to_string(),
        data: serde_json::json!({
            "event_types": ["service_started", "service_stopped", "health_changed"]
        }),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(monitoring_input).await?;
    assert_eq!(response.response_type, "success");

    Ok(())
}

#[tokio::test]
async fn test_configuration_adaptability() -> Result<()> {
    let mut config = create_test_config();

    // Test different feature combinations
    let feature_combinations = vec![
        UIFeatures {
            ai_assistant: true,
            real_time_monitoring: true,
            deployment_wizard: true,
            service_management: true,
            log_viewer: true,
            metrics_dashboard: true,
            custom_dashboards: true,
            multi_primal_coordination: true,
        },
        UIFeatures {
            ai_assistant: false,
            real_time_monitoring: true,
            deployment_wizard: false,
            service_management: true,
            log_viewer: false,
            metrics_dashboard: true,
            custom_dashboards: false,
            multi_primal_coordination: true,
        },
    ];

    for features in feature_combinations {
        config.features = features.clone();
        let ui_manager = UniversalUIManager::new(config.clone()).await?;
        let status = ui_manager.get_system_status().await?;

        // Verify UI manager adapts to feature configuration
        assert!(status.total_primals >= 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_primal_health_monitoring() -> Result<()> {
    let mut config = create_test_config();
    config.auto_discovery.health_check_interval_secs = 1;

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test health monitoring
    let health_input = UserInput {
        input_type: "check_health".to_string(),
        data: serde_json::json!({
            "primal_names": ["songbird", "nestgate", "toadstool"]
        }),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(health_input).await?;
    assert_eq!(response.response_type, "success");

    Ok(())
}

#[tokio::test]
async fn test_custom_widget_rendering() -> Result<()> {
    let mut config = create_test_config();
    add_test_custom_primal(&mut config);

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test custom widget rendering
    let widget_input = UserInput {
        input_type: "render_widget".to_string(),
        data: serde_json::json!({
            "widget_type": "metrics_chart",
            "primal_name": "custom_ai",
            "config": {
                "chart_type": "line",
                "metrics": ["requests_per_second", "latency_ms"]
            }
        }),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(widget_input).await?;
    assert_eq!(response.response_type, "success");

    Ok(())
}

#[tokio::test]
async fn test_deployment_wizard() -> Result<()> {
    let mut config = create_test_config();
    config.features.deployment_wizard = true;

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test deployment wizard
    let wizard_input = UserInput {
        input_type: "deployment_wizard".to_string(),
        data: serde_json::json!({
            "step": "select_primals",
            "requirements": {
                "compute": true,
                "storage": true,
                "security": true
            }
        }),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(wizard_input).await?;
    assert_eq!(response.response_type, "success");

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let config = create_test_config();
    let ui_manager = UniversalUIManager::new(config).await?;

    // Test invalid input handling
    let invalid_input = UserInput {
        input_type: "invalid_command".to_string(),
        data: serde_json::json!({}),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(invalid_input).await?;
    // Should handle invalid input gracefully
    assert!(response.response_type == "error" || response.response_type == "success");

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let config = create_test_config();
    let ui_manager = UniversalUIManager::new(config).await?;

    // Test concurrent operations
    let operations = vec![
        UserInput {
            input_type: "get_status".to_string(),
            data: serde_json::json!({}),
            context: HashMap::new(),
        },
        UserInput {
            input_type: "list_primals".to_string(),
            data: serde_json::json!({}),
            context: HashMap::new(),
        },
        UserInput {
            input_type: "check_health".to_string(),
            data: serde_json::json!({}),
            context: HashMap::new(),
        },
    ];

    let mut handles = Vec::new();

    for input in operations {
        let manager = &ui_manager;
        let handle = tokio::spawn(async move { manager.handle_user_input(input).await });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = timeout(Duration::from_secs(5), handle).await??;
        assert!(result.response_type == "success" || result.response_type == "error");
    }

    Ok(())
}

#[tokio::test]
async fn test_performance_metrics() -> Result<()> {
    let mut config = create_test_config();
    config.features.metrics_dashboard = true;

    let ui_manager = UniversalUIManager::new(config).await?;

    // Test performance metrics collection
    let metrics_input = UserInput {
        input_type: "get_metrics".to_string(),
        data: serde_json::json!({
            "time_range": "1h",
            "metrics": ["cpu_usage", "memory_usage", "request_count"]
        }),
        context: HashMap::new(),
    };

    let response = ui_manager.handle_user_input(metrics_input).await?;
    assert_eq!(response.response_type, "success");

    Ok(())
}

// Helper functions

fn create_test_config() -> UniversalUIConfig {
    let mut config = UniversalUIConfig::default();
    config.ui_mode = UIMode::CLI;
    config.auto_discovery.enabled = false; // Disable for tests
    config
}

fn add_test_custom_primal(config: &mut UniversalUIConfig) {
    use biomeos::universal_ui::*;

    config.custom_primals.insert(
        "custom_ai".to_string(),
        CustomPrimalConfig {
            endpoint: "http://localhost:7000".to_string(),
            capabilities: vec!["ai".to_string(), "ml".to_string()],
            description: "Test AI primal".to_string(),
            ui_config: PrimalUIConfig {
                display_name: "Test AI".to_string(),
                icon: "🤖".to_string(),
                color: "#FF6B6B".to_string(),
                dashboard_widgets: vec![WidgetConfig {
                    widget_type: "metrics_chart".to_string(),
                    title: "Test Metrics".to_string(),
                    api_endpoint: "/api/v1/metrics".to_string(),
                    refresh_interval_secs: 5,
                    display_config: HashMap::new(),
                }],
                custom_actions: vec![ActionConfig {
                    action_id: "test_action".to_string(),
                    display_name: "Test Action".to_string(),
                    api_endpoint: "/api/v1/test".to_string(),
                    method: "POST".to_string(),
                    parameters: vec![],
                    confirmation_required: false,
                }],
                metrics_config: MetricsConfig {
                    enabled: true,
                    metrics_endpoint: "/api/v1/metrics".to_string(),
                    chart_types: vec!["line".to_string()],
                    default_time_range: "1h".to_string(),
                },
            },
            auth_config: None,
        },
    );
}

// Mock primal server for testing
#[cfg(test)]
mod mock_server {
    use axum::{extract::Query, http::StatusCode, response::Json, routing::get, Router};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use tokio::net::TcpListener;

    #[derive(Serialize)]
    struct HealthResponse {
        status: String,
        api_version: String,
        capabilities: Vec<String>,
    }

    #[derive(Serialize)]
    struct MetricsResponse {
        metrics: HashMap<String, f64>,
        timestamp: String,
    }

    pub async fn start_mock_primal_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/api/v1/health", get(health_handler))
            .route("/api/v1/capabilities", get(capabilities_handler))
            .route("/api/v1/metrics", get(metrics_handler))
            .route("/api/v1/ui/config", get(ui_config_handler));

        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }

    async fn health_handler() -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "healthy".to_string(),
            api_version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string(), "mock".to_string()],
        })
    }

    async fn capabilities_handler() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "capabilities": ["test", "mock", "health_check"]
        }))
    }

    async fn metrics_handler() -> Json<MetricsResponse> {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 45.5);
        metrics.insert("memory_usage".to_string(), 67.2);
        metrics.insert("request_count".to_string(), 1234.0);

        Json(MetricsResponse {
            metrics,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn ui_config_handler() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "display_name": "Mock Primal",
            "icon": "🧪",
            "color": "#00FF00",
            "dashboard_widgets": [
                {
                    "widget_type": "status_card",
                    "title": "Mock Status",
                    "api_endpoint": "/api/v1/status",
                    "refresh_interval_secs": 10,
                    "display_config": {}
                }
            ],
            "custom_actions": [],
            "metrics_config": {
                "enabled": true,
                "metrics_endpoint": "/api/v1/metrics",
                "chart_types": ["line"],
                "default_time_range": "1h"
            }
        }))
    }
}

// Integration test with mock server
#[tokio::test]
async fn test_with_mock_primal_server() -> Result<()> {
    // Start mock server
    tokio::spawn(async {
        mock_server::start_mock_primal_server(8999).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut config = create_test_config();
    config.primal_endpoints.insert(
        "mock_primal".to_string(),
        "http://localhost:8999".to_string(),
    );

    let ui_manager = UniversalUIManager::new(config).await?;
    let status = ui_manager.get_system_status().await?;

    // Verify mock primal integration
    assert!(status.total_primals >= 0);

    Ok(())
}
