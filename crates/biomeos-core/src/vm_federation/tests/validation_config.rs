// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::time::Duration;

use super::ValidationConfig;

#[test]
fn test_validation_config_default() {
    let config = ValidationConfig::default();
    assert_eq!(config.cloud_init_timeout.as_secs(), 600);
    assert_eq!(config.ssh_timeout.as_secs(), 300);
    assert_eq!(config.ssh_retry_interval.as_secs(), 30);
    assert_eq!(config.ssh_max_retries, 20);
}

#[test]
fn test_validation_config_custom() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(120),
        ssh_timeout: Duration::from_secs(60),
        ssh_retry_interval: Duration::from_secs(10),
        ssh_max_retries: 5,
    };
    assert_eq!(config.cloud_init_timeout.as_secs(), 120);
    assert_eq!(config.ssh_max_retries, 5);
}

#[test]
fn test_validation_config_debug() {
    let config = ValidationConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ValidationConfig"));
}

#[test]
fn test_validation_config_clone() {
    let config = ValidationConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.ssh_max_retries, config.ssh_max_retries);
    assert_eq!(cloned.cloud_init_timeout, config.cloud_init_timeout);
}

#[test]
fn test_validation_config_builder_pattern() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(900),
        ssh_timeout: Duration::from_secs(600),
        ssh_retry_interval: Duration::from_secs(15),
        ssh_max_retries: 40,
    };
    assert_eq!(config.cloud_init_timeout.as_secs(), 900);
    assert_eq!(config.ssh_max_retries, 40);
}

#[test]
fn test_validation_config_extreme_retries_zero() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(1),
        ssh_timeout: Duration::from_secs(1),
        ssh_retry_interval: Duration::from_secs(1),
        ssh_max_retries: 0,
    };
    assert_eq!(config.ssh_max_retries, 0);
}

#[test]
fn test_validation_config_extreme_durations() {
    let c = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(u64::MAX / 4),
        ssh_timeout: Duration::from_secs(1),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: u32::MAX,
    };
    assert!(c.cloud_init_timeout > Duration::from_secs(1_000_000));
}
