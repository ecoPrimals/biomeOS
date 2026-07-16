use super::super::*;
use crate::primal_adapter::types::{HealthCheckConfig, LifecycleCapabilities, PortConfigMethod, PrimalState};
use std::time::Duration;

// ============================================================================
// PortConfigMethod Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_port_config_env_var() {
    let config = PortConfigMethod::EnvVar("PRIMAL_PORT".to_string());

    match config {
        PortConfigMethod::EnvVar(name) => {
            assert_eq!(name, "PRIMAL_PORT");
        }
        _ => panic!("Expected EnvVar variant"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_port_config_cli_flag() {
    let config = PortConfigMethod::CliFlag("--port".to_string());

    match config {
        PortConfigMethod::CliFlag(flag) => {
            assert_eq!(flag, "--port");
        }
        _ => panic!("Expected CliFlag variant"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_port_config_config_file() {
    let config = PortConfigMethod::ConfigFile {
        path: "/etc/primal/config.yaml".to_string(),
        format: "yaml".to_string(),
    };

    match config {
        PortConfigMethod::ConfigFile { path, format } => {
            assert_eq!(path, "/etc/primal/config.yaml");
            assert_eq!(format, "yaml");
        }
        _ => panic!("Expected ConfigFile variant"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_port_config_multiple() {
    let methods = vec![
        PortConfigMethod::EnvVar("PORT".to_string()),
        PortConfigMethod::CliFlag("--port".to_string()),
        PortConfigMethod::ConfigFile {
            path: "config.yaml".to_string(),
            format: "yaml".to_string(),
        },
    ];

    let config = PortConfigMethod::Multiple(methods);

    match config {
        PortConfigMethod::Multiple(m) => {
            assert_eq!(m.len(), 3);
        }
        _ => panic!("Expected Multiple variant"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_port_config_unknown() {
    let config = PortConfigMethod::Unknown;
    assert!(matches!(config, PortConfigMethod::Unknown));
}

// ============================================================================
// HealthCheckConfig Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_check_config_url_replacement() {
    let config = HealthCheckConfig {
        url_pattern: "http://localhost:PORT/health".to_string(),
        expected_status: 200,
        timeout: Duration::from_secs(2),
    };

    let url_9000 = config.url_pattern.replace("PORT", "9000");
    let url_9010 = config.url_pattern.replace("PORT", "9010");

    assert_eq!(url_9000, "http://localhost:9000/health");
    assert_eq!(url_9010, "http://localhost:9010/health");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_check_config_status_codes() {
    let config_200 = HealthCheckConfig {
        url_pattern: "http://localhost:PORT/health".to_string(),
        expected_status: 200,
        timeout: Duration::from_secs(2),
    };

    let config_204 = HealthCheckConfig {
        url_pattern: "http://localhost:PORT/health".to_string(),
        expected_status: 204,
        timeout: Duration::from_secs(2),
    };

    assert_eq!(config_200.expected_status, 200);
    assert_eq!(config_204.expected_status, 204);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_check_config_timeout() {
    let config = HealthCheckConfig {
        url_pattern: "http://localhost:PORT/health".to_string(),
        expected_status: 200,
        timeout: Duration::from_secs(5),
    };

    assert_eq!(config.timeout, Duration::from_secs(5));
}

// ============================================================================
// PrimalState Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_not_started() {
    let state = PrimalState::NotStarted;
    assert!(matches!(state, PrimalState::NotStarted));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_starting() {
    let state = PrimalState::Starting {
        started_at: chrono::Utc::now(),
    };
    assert!(matches!(state, PrimalState::Starting { .. }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_running() {
    let state = PrimalState::Running {
        pid: Some(12345),
        port: 9000,
    };

    match state {
        PrimalState::Running { pid, port } => {
            assert_eq!(pid, Some(12345));
            assert_eq!(port, 9000);
        }
        _ => panic!("Expected Running state"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_unhealthy() {
    let state = PrimalState::Unhealthy {
        port: 9000,
        reason: "connection timeout".to_string(),
    };

    match state {
        PrimalState::Unhealthy { port, reason } => {
            assert_eq!(port, 9000);
            assert_eq!(reason, "connection timeout");
        }
        _ => panic!("Expected Unhealthy state"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_stopping() {
    let state = PrimalState::Stopping;
    assert!(matches!(state, PrimalState::Stopping));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_stopped() {
    let state = PrimalState::Stopped;
    assert!(matches!(state, PrimalState::Stopped));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_unknown() {
    let state = PrimalState::Unknown;
    assert!(matches!(state, PrimalState::Unknown));
}
