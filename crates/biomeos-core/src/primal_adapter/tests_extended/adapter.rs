use super::super::*;
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
