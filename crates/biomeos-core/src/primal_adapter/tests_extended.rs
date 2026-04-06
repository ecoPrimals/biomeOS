// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

// Additional comprehensive tests for primal adapter
use super::*;
use crate::primal_adapter::types::{HealthCheckConfig, PortConfigMethod, PrimalState};
use std::path::PathBuf;
use std::time::Duration;

// ============================================================================
// PrimalAdapter Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_adapter_new() {
    let adapter = PrimalAdapter::new("test".to_string(), PathBuf::from("/bin/test"));

    assert_eq!(adapter.name, "test");
    assert_eq!(adapter.binary, PathBuf::from("/bin/test"));
    assert!(!adapter.interface.is_known());
    assert!(matches!(adapter.state, PrimalState::NotStarted));
    assert!(adapter.version.is_none());
    assert!(adapter.capabilities.lifecycle.can_refuse);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_adapter_state_transitions() {
    let mut adapter = PrimalAdapter::new("test".to_string(), PathBuf::from("/bin/test"));

    // Initial state
    assert!(matches!(adapter.state, PrimalState::NotStarted));

    // Simulate starting
    adapter.state = PrimalState::Starting {
        started_at: chrono::Utc::now(),
    };
    assert!(matches!(adapter.state, PrimalState::Starting { .. }));

    // Simulate running
    adapter.state = PrimalState::Running {
        pid: Some(12345),
        port: 9000,
    };
    assert!(matches!(adapter.state, PrimalState::Running { .. }));

    // Simulate unhealthy
    adapter.state = PrimalState::Unhealthy {
        port: 9000,
        reason: "timeout".to_string(),
    };
    assert!(matches!(adapter.state, PrimalState::Unhealthy { .. }));

    // Simulate stopped
    adapter.state = PrimalState::Stopped;
    assert!(matches!(adapter.state, PrimalState::Stopped));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_adapter_with_version() {
    let mut adapter = PrimalAdapter::new("test".to_string(), PathBuf::from("/bin/test"));
    adapter.version = Some("1.2.3".to_string());

    assert_eq!(adapter.version, Some("1.2.3".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_adapter_health_check_no_config() {
    let mut adapter = PrimalAdapter::new("test".to_string(), PathBuf::from("/bin/test"));
    adapter.state = PrimalState::Running {
        pid: Some(12345),
        port: 9000,
    };

    // No health check configured, should return true if running
    let result = adapter.check_health().unwrap();
    assert!(result);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_adapter_health_check_not_running() {
    let adapter = PrimalAdapter::new("test".to_string(), PathBuf::from("/bin/test"));

    // Not running, should return false
    let result = adapter.check_health().unwrap();
    assert!(!result);
}

// ============================================================================
// PrimalInterface Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_direct() {
    let interface = PrimalInterface::Direct {
        args: vec!["--config".to_string(), "test.yaml".to_string()],
    };

    assert!(interface.is_known());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_subcommand() {
    let interface = PrimalInterface::Subcommand {
        start_cmd: "serve".to_string(),
        stop_cmd: Some("stop".to_string()),
    };

    assert!(interface.is_known());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_service() {
    let interface = PrimalInterface::Service {
        service_name: "biomeos-nestgate".to_string(),
    };

    assert!(interface.is_known());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_docker() {
    let interface = PrimalInterface::Docker {
        image: "biomeos/nestgate:latest".to_string(),
        container: "nestgate-1".to_string(),
    };

    assert!(interface.is_known());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_api() {
    let interface = PrimalInterface::Api {
        endpoint: "http://localhost:9000".to_string(),
        start_path: "/api/start".to_string(),
        stop_path: Some("/api/stop".to_string()),
    };

    assert!(interface.is_known());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_unknown() {
    let interface = PrimalInterface::Unknown {
        attempted_patterns: vec![InterfacePattern::Direct, InterfacePattern::SubcommandServe],
    };

    assert!(!interface.is_known());
}

// ============================================================================
// InterfacePattern Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_patterns_all_variants() {
    let patterns = [
        InterfacePattern::Direct,
        InterfacePattern::SubcommandServe,
        InterfacePattern::SubcommandService,
        InterfacePattern::SubcommandStart,
        InterfacePattern::SubcommandRun,
        InterfacePattern::Systemd,
        InterfacePattern::Docker,
        InterfacePattern::ApiLifecycle,
    ];

    assert_eq!(patterns.len(), 8);
}

// ============================================================================
// PrimalCapabilities Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_capabilities_full() {
    let caps = PrimalCapabilities {
        lifecycle: LifecycleCapabilities {
            can_start: true,
            can_stop: true,
            can_restart: true,
            graceful_shutdown: true,
            can_refuse: true,
        },
        health_check: Some(HealthCheckConfig {
            url_pattern: "http://localhost:PORT/health".to_string(),
            expected_status: 200,
            timeout: Duration::from_secs(2),
        }),
        port_config: PortConfigMethod::EnvVar("PORT".to_string()),
        has_version_cmd: true,
        has_fast_help: true,
    };

    assert!(caps.lifecycle.can_start);
    assert!(caps.lifecycle.can_stop);
    assert!(caps.lifecycle.can_restart);
    assert!(caps.lifecycle.graceful_shutdown);
    assert!(caps.health_check.is_some());
    assert!(caps.has_version_cmd);
    assert!(caps.has_fast_help);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lifecycle_capabilities_minimal() {
    let caps = LifecycleCapabilities {
        can_start: true,
        can_stop: false,
        can_restart: false,
        graceful_shutdown: false,
        can_refuse: true,
    };

    assert!(caps.can_start);
    assert!(!caps.can_stop);
    assert!(!caps.can_restart);
    assert!(!caps.graceful_shutdown);
    assert!(caps.can_refuse); // Always true for sovereignty
}

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

// ============================================================================
// Default Implementation Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_state_default() {
    let state = PrimalState::default();
    assert!(matches!(state, PrimalState::NotStarted));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_capabilities_default_sovereignty() {
    let caps = PrimalCapabilities::default();

    // Sovereignty check: can_refuse should ALWAYS be true
    assert!(
        caps.lifecycle.can_refuse,
        "Sovereignty violated: primals must always be able to refuse requests"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lifecycle_capabilities_default_sovereignty() {
    let caps = LifecycleCapabilities::default();

    // Sovereignty: can_refuse is ALWAYS true
    assert!(
        caps.can_refuse,
        "Sovereignty violated: can_refuse must default to true"
    );

    // Safety: other capabilities default to false (conservative)
    assert!(!caps.can_start);
    assert!(!caps.can_stop);
    assert!(!caps.can_restart);
    assert!(!caps.graceful_shutdown);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_adapter_with_all_capabilities() {
    let mut adapter = PrimalAdapter::new("full_featured".to_string(), PathBuf::from("/bin/primal"));

    // Set up full capabilities
    adapter.interface = PrimalInterface::Subcommand {
        start_cmd: "serve".to_string(),
        stop_cmd: Some("stop".to_string()),
    };

    adapter.capabilities = PrimalCapabilities {
        lifecycle: LifecycleCapabilities {
            can_start: true,
            can_stop: true,
            can_restart: true,
            graceful_shutdown: true,
            can_refuse: true,
        },
        health_check: Some(HealthCheckConfig {
            url_pattern: "http://localhost:PORT/health".to_string(),
            expected_status: 200,
            timeout: Duration::from_secs(2),
        }),
        port_config: PortConfigMethod::Multiple(vec![
            PortConfigMethod::EnvVar("PORT".to_string()),
            PortConfigMethod::CliFlag("--port".to_string()),
        ]),
        has_version_cmd: true,
        has_fast_help: true,
    };

    adapter.version = Some("2.1.0".to_string());

    // Verify all capabilities
    assert!(adapter.interface.is_known());
    assert!(adapter.capabilities.lifecycle.can_start);
    assert!(adapter.capabilities.lifecycle.can_stop);
    assert!(adapter.capabilities.lifecycle.can_restart);
    assert!(adapter.capabilities.lifecycle.graceful_shutdown);
    assert!(adapter.capabilities.health_check.is_some());
    assert!(adapter.capabilities.has_version_cmd);
    assert!(adapter.capabilities.has_fast_help);
    assert_eq!(adapter.version, Some("2.1.0".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_adapter_serialization() {
    let adapter = PrimalAdapter::new("test".to_string(), PathBuf::from("/bin/test"));

    // Test serialization
    let json = serde_json::to_string(&adapter);
    assert!(json.is_ok());

    // Test deserialization
    let json_str = json.unwrap();
    let deserialized: Result<PrimalAdapter, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let deserialized_adapter = deserialized.unwrap();
    assert_eq!(deserialized_adapter.name, "test");
}
