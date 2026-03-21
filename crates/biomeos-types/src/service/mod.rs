// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unified Service System
//!
//! This module consolidates all service-related types from across the biomeOS ecosystem,
//! providing a comprehensive service abstraction layer. The service system has been split
//! into logical modules for better maintainability and to stay within the 2000-line limit per file.
//!
//! ## Module Structure
//!
//! - `core`: Core service definitions (UniversalService, ServiceMetadata, ServiceSpec)
//! - `runtime`: Runtime configurations (containers, binaries, WASM, VMs)
//! - `security`: Security configurations (authentication, authorization, encryption)
//! - `health`: Health checks, probes, and monitoring
//! - `networking`: Service networking, discovery, and load balancing

// Re-export all service modules
pub mod core;
pub mod health;
pub mod networking;
pub mod runtime;
pub mod security;

// Re-export all types for convenience
pub use core::*;
pub use health::*;
pub use networking::*;
pub use runtime::*;
pub use security::*;

// Additional test module
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_service() {
        let service = UniversalService::default();
        assert_eq!(service.metadata.name, "default-service");
        assert_eq!(service.spec.scaling.min_replicas, 1);
        assert_eq!(service.spec.scaling.max_replicas, 1);
    }

    #[test]
    fn test_service_serialization() {
        let service = UniversalService::default();
        let json = serde_json::to_string(&service).unwrap();
        let deserialized: UniversalService = serde_json::from_str(&json).unwrap();
        assert_eq!(service.metadata.name, deserialized.metadata.name);
    }

    #[test]
    fn test_service_runtime_default() {
        let runtime = ServiceRuntime::default();
        match runtime.runtime_type {
            RuntimeType::Binary { ref executable, .. } => {
                assert_eq!(executable, "service");
            }
            _ => panic!("Expected Binary runtime type"),
        }
    }

    #[test]
    fn test_service_security_default() {
        let security = ServiceSecurity::default();
        assert_eq!(security.security_context.run_as_user, Some(1000));
        assert!(security.security_context.run_as_non_root);
        assert!(security.security_context.read_only_root_fs);
        assert!(!security.security_context.allow_privilege_escalation);
    }

    #[test]
    fn test_service_health_default() {
        let health = ServiceHealth::default();
        assert!(health.health_checks.is_empty());
        assert!(health.reporting.enabled);
        assert_eq!(health.reporting.interval, 30);
    }

    #[test]
    fn test_service_networking_default() {
        let networking = ServiceNetworking::default();
        match networking.network_mode {
            NetworkMode::Bridge => {}
            _ => panic!("Expected Bridge network mode"),
        }
        assert!(networking.discovery.enabled);
        assert!(networking.ports.is_empty());
    }
}
