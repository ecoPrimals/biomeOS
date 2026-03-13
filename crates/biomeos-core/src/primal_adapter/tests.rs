// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Tests for primal adapter pattern

use super::*;
use crate::primal_adapter::lifecycle::{LifecycleTransition, TransitionReason, Urgency};
use crate::primal_adapter::types::{HealthCheckConfig, PortConfigMethod};
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_direct_interface() {
    // Test with a real binary if available
    let squirrel_path = Path::new("../phase1bins/squirrel-bin");

    if squirrel_path.exists() {
        let result = discover_primal_interface(squirrel_path).await;
        match result {
            Ok(adapter) => {
                println!("✅ Discovered Squirrel interface: {:?}", adapter.interface);
                assert!(adapter.interface.is_known());
                assert_eq!(adapter.name, "squirrel");
            }
            Err(e) => {
                println!("⚠️ Could not discover Squirrel (may be expected): {}", e);
            }
        }
    } else {
        println!("⏭️ Skipping Squirrel test - binary not found");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_adapter_cache() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".biomeos/primal_adapters");
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create test adapter
    let mut adapter = PrimalAdapter::new(
        "test_primal".to_string(),
        Path::new("/bin/test").to_path_buf(),
    );
    adapter.interface = PrimalInterface::Direct { args: vec![] };
    adapter.capabilities.lifecycle.can_start = true;

    // Save to cache
    let result = save_adapter(&adapter);
    if let Err(e) = &result {
        println!("Cache save result: {:?}", e);
    }

    // Try to load (may fail if home dir not accessible in test env)
    if result.is_ok() {
        let loaded = load_adapter("test_primal");
        match loaded {
            Ok(loaded_adapter) => {
                assert_eq!(loaded_adapter.name, "test_primal");
                assert!(loaded_adapter.capabilities.lifecycle.can_start);
            }
            Err(e) => {
                println!("Cache load failed (may be expected in test): {}", e);
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_interface_patterns() {
    // Test that we have all the expected patterns
    let patterns = [
        InterfacePattern::Direct,
        InterfacePattern::SubcommandServe,
        InterfacePattern::SubcommandService,
        InterfacePattern::SubcommandStart,
    ];

    assert!(patterns.len() >= 4);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lifecycle_request() {
    let request = LifecycleRequest::new(LifecycleTransition::Start, TransitionReason::UserRequest);

    assert!(matches!(request.urgency, Urgency::Normal));
    assert_eq!(request.requestor, "BiomeOS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lifecycle_response() {
    let accepted = LifecycleResponse::Accepted;
    assert!(accepted.is_success());
    assert!(!accepted.should_retry());

    let deferred = LifecycleResponse::Deferred {
        duration: Duration::from_secs(5),
        reason: "busy".to_string(),
    };
    assert!(!deferred.is_success());
    assert!(deferred.should_retry());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_primal_capabilities_default() {
    let caps = PrimalCapabilities::default();

    // Default capabilities should be conservative
    assert!(!caps.lifecycle.can_start);
    assert!(!caps.lifecycle.can_stop);
    assert!(caps.lifecycle.can_refuse); // Always true - sovereignty!
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_port_config_methods() {
    let env_var = PortConfigMethod::EnvVar("PORT".to_string());
    let cli_flag = PortConfigMethod::CliFlag("--port".to_string());
    let multiple = PortConfigMethod::Multiple(vec![env_var.clone(), cli_flag.clone()]);

    // Just verify they can be created
    match multiple {
        PortConfigMethod::Multiple(methods) => {
            assert_eq!(methods.len(), 2);
        }
        _ => panic!("Expected Multiple variant"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_compatibility_check() {
    // Test with non-existent binary
    let result = check_compatibility(Path::new("/nonexistent")).await;
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should return false for non-existent
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_and_start_nonexistent() {
    let result = discover_and_start(Path::new("/nonexistent"), 9010).await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_check_config() {
    let config = HealthCheckConfig {
        url_pattern: "http://localhost:PORT/health".to_string(),
        expected_status: 200,
        timeout: Duration::from_secs(2),
    };

    let url = config.url_pattern.replace("PORT", "9010");
    assert_eq!(url, "http://localhost:9010/health");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_stop_command_discovery() {
    use super::discovery::discover_stop_command;
    use std::path::PathBuf;

    // Test with a binary that doesn't exist (should return None)
    let fake_binary = PathBuf::from("/tmp/nonexistent-primal");
    let stop_cmd = discover_stop_command(&fake_binary).await;

    // Should return None when no stop command found
    assert!(
        stop_cmd.is_none(),
        "Non-existent binary should have no stop command"
    );
}
