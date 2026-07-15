// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::config::*;
use std::collections::HashMap;

/// Helper to create a test EnvironmentConfig
pub(super) fn test_env_config() -> EnvironmentConfig {
    EnvironmentConfig {
        name: "test".to_string(),
        description: None,
        variables: HashMap::new(),
        features: FeatureFlags::default(),
        limits: features::EnvironmentLimits {
            max_users: None,
            max_sessions: None,
            rate_limit: None,
            retention_days: None,
            storage_limit: None,
        },
        endpoints: HashMap::new(),
    }
}
