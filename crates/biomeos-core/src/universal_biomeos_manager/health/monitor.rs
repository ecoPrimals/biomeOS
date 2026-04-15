// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! [`HealthMonitor`] — system-wide health report construction.

use std::collections::HashMap;
use std::sync::Arc;

use biomeos_types::{BiomeOSConfig, Health, HealthReport};

/// Health Monitor for system-wide health tracking
#[derive(Debug, Clone)]
pub struct HealthMonitor {
    /// Reserved for health monitoring configuration (interval, thresholds, etc.).
    /// Planned: wire up for configurable health check intervals and thresholds.
    _config: Arc<BiomeOSConfig>,
}

impl HealthMonitor {
    /// Create new health monitor with Arc-wrapped config for zero-copy sharing
    #[must_use]
    pub const fn new(config: Arc<BiomeOSConfig>) -> Self {
        Self { _config: config }
    }

    /// Get system health report
    pub fn get_system_health(&self) -> HealthReport {
        use biomeos_types::health::HealthMetrics;
        use biomeos_types::{HealthSubject, HealthSubjectType};
        use uuid::Uuid;

        HealthReport {
            id: Uuid::new_v4(),
            subject: HealthSubject {
                id: "system".to_string(),
                subject_type: HealthSubjectType::System,
                name: "BiomeOS System".to_string(),
                version: "1.0.0".to_string(),
            },
            health: Health::Healthy,
            components: HashMap::new(),
            metrics: HealthMetrics {
                response_time: None,
                resources: None,
                errors: None,
                availability: None,
                custom: HashMap::new(),
            },
            history: vec![],
            generated_at: chrono::Utc::now(),
            next_check_at: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        }
    }
}
