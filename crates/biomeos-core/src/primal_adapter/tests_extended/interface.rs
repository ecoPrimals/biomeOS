use crate::primal_adapter::types::{
    HealthCheckConfig, InterfacePattern, LifecycleCapabilities, PortConfigMethod,
    PrimalCapabilities, PrimalInterface,
};
use std::time::Duration;

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
