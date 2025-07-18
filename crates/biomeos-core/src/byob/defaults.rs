//! BYOB (Bring Your Own Biome) default implementations
//!
//! This module contains all the default implementations for BYOB types and
//! configuration structures.

use chrono::Utc;
use std::collections::HashMap;

use crate::HealthStatus;
use super::types::*;

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_cpu_cores: 8.0,
            max_memory_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            max_storage_bytes: 100 * 1024 * 1024 * 1024, // 100GB
            max_network_bandwidth_mbps: 1000,
            max_deployments: 10,
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_cores: 0.0,
            memory_bytes: 0,
            storage_bytes: 0,
            network_bandwidth_mbps: 0,
            active_deployments: 0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for TeamHealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: 30,
            alerting: TeamAlertConfig::default(),
            auto_scaling: AutoScalingConfig::default(),
            toadstool_integration: ToadstoolIntegrationConfig::default(),
        }
    }
}

impl Default for TeamAlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thresholds: AlertThresholds::default(),
            notification_channels: vec!["email".to_string()],
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 0.8,
            memory_threshold: 0.8,
            storage_threshold: 0.9,
            network_threshold: 0.8,
        }
    }
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_instances: 1,
            max_instances: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            cooldown_seconds: 300,
        }
    }
}

impl Default for ToadstoolIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "http://localhost:8090".to_string(),
            features: ToadstoolFeatures::default(),
        }
    }
}

impl Default for ToadstoolFeatures {
    fn default() -> Self {
        Self {
            container_orchestration: true,
            service_mesh: false,
            load_balancing: true,
            auto_recovery: true,
        }
    }
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            network_isolation: true,
            resource_isolation: true,
            secret_isolation: true,
        }
    }
}

impl Default for DeploymentHealthStatus {
    fn default() -> Self {
        Self {
            overall_health: HealthStatus::Unknown,
            service_health: HashMap::new(),
            resource_utilization: ResourceUtilization::default(),
            health_events: Vec::new(),
            last_health_check: Utc::now(),
        }
    }
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            storage_usage: 0.0,
            network_usage: 0.0,
        }
    }
}

impl Default for ServiceMeshConfig {
    fn default() -> Self {
        Self {
            namespace: "default".to_string(),
            service_discovery: true,
            traffic_management: true,
            security_policies: true,
        }
    }
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            lb_type: LoadBalancerType::HealthBased,
            health_check: LoadBalancerHealthCheck::default(),
            sticky_sessions: false,
        }
    }
}

impl Default for LoadBalancerHealthCheck {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 10,
            healthy_threshold: 2,
            unhealthy_threshold: 3,
        }
    }
}

impl Default for SimpleBiomeManifest {
    fn default() -> Self {
        Self {
            metadata: SimpleBiomeMetadata::default(),
            services: Vec::new(),
            resources: SimpleBiomeResources::default(),
        }
    }
}

impl Default for SimpleBiomeMetadata {
    fn default() -> Self {
        Self {
            name: "default-biome".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

impl Default for SimpleBiomeResources {
    fn default() -> Self {
        Self {
            cpu_cores: 1.0,
            memory_mb: 512,
            storage_gb: 10,
        }
    }
} 