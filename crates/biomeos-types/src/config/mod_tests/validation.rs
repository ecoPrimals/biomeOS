// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::config::*;

#[test]
fn test_validate_default_config_passes() {
    let config = BiomeOSConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_validate_port_zero_fails() {
    let mut config = BiomeOSConfig::default();
    config.network.port = 0;
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("Port cannot be 0"));
}

#[test]
fn test_validate_request_timeout_zero_fails() {
    let mut config = BiomeOSConfig::default();
    config.system.timeouts.default_request_timeout = std::time::Duration::from_secs(0);
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("Request timeout cannot be 0"));
}

#[test]
fn test_validate_max_connections_zero_fails() {
    let mut config = BiomeOSConfig::default();
    config.system.limits.max_connections = 0;
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("Max connections cannot be 0"));
}

#[test]
fn test_validate_metrics_interval_zero_fails() {
    let mut config = BiomeOSConfig::default();
    config.observability.metrics.enabled = true;
    config.observability.metrics.interval = std::time::Duration::from_secs(0);
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("Metrics interval cannot be 0"));
}

#[test]
fn test_validate_metrics_disabled_zero_interval_ok() {
    let mut config = BiomeOSConfig::default();
    config.observability.metrics.enabled = false;
    config.observability.metrics.interval = std::time::Duration::from_secs(0);
    // Should pass because metrics are disabled
    assert!(config.validate().is_ok());
}

#[test]
fn test_validate_session_timeout_zero_fails() {
    let mut config = BiomeOSConfig::default();
    config.security.session.timeout = std::time::Duration::from_secs(0);
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("Session timeout cannot be 0"));
}

#[test]
fn test_validate_custom_port_passes() {
    let mut config = BiomeOSConfig::default();
    config.network.port = 9090;
    assert!(config.validate().is_ok());
}
