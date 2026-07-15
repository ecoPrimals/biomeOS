// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::config::*;
use std::collections::HashMap;

#[test]
fn test_config_from_env_all_overrides() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_PORT".to_string(), "8888".to_string());
    let config = BiomeOSConfig::from_env_map(&env);
    assert_eq!(config.network.port, 8888);

    env.clear();
    env.insert("BIOMEOS_PORT".to_string(), "not_a_number".to_string());
    let config = BiomeOSConfig::from_env_map(&env);
    assert!(config.network.port > 0);

    env.clear();
    env.insert("BIOMEOS_BIND_ADDRESS".to_string(), "192.0.2.1".to_string());
    let config = BiomeOSConfig::from_env_map(&env);
    assert_eq!(config.network.bind_address, "192.0.2.1");

    env.clear();
    env.insert("BIOMEOS_DEBUG".to_string(), "true".to_string());
    let config = BiomeOSConfig::from_env_map(&env);
    assert!(config.features.debug);

    env.insert("BIOMEOS_DEBUG".to_string(), "false".to_string());
    let config = BiomeOSConfig::from_env_map(&env);
    assert!(!config.features.debug);

    env.clear();
    env.insert("BIOMEOS_EXPERIMENTAL".to_string(), "true".to_string());
    let config = BiomeOSConfig::from_env_map(&env);
    assert!(config.features.experimental);

    for (level_str, expected_debug) in [
        ("trace", "Trace"),
        ("debug", "Debug"),
        ("info", "Info"),
        ("warn", "Warn"),
        ("error", "Error"),
        ("off", "Off"),
        ("TRACE", "Trace"),
        ("INFO", "Info"),
        ("unknown", "Info"),
    ] {
        env.clear();
        env.insert("BIOMEOS_LOG_LEVEL".to_string(), level_str.to_string());
        let config = BiomeOSConfig::from_env_map(&env);
        assert_eq!(
            format!("{:?}", config.observability.logging.level),
            expected_debug,
            "Failed for log level: {}",
            level_str
        );
    }
}
