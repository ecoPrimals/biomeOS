// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;
use std::sync::Arc;

use biomeos_primal_sdk::PrimalCapability;
use biomeos_types::{BiomeOSConfig, Health, PrimalType};
use chrono::Utc;

use super::*;
use crate::universal_biomeos_manager::{PrimalInfo, UniversalBiomeOSManager};

#[test]
fn test_health_to_status_string() {
    assert_eq!(health_to_status_string(&Health::Healthy), "Healthy");
    assert_eq!(
        health_to_status_string(&Health::Degraded {
            issues: vec![],
            impact_score: None,
        }),
        "Degraded"
    );
    assert_eq!(
        health_to_status_string(&Health::Unhealthy {
            issues: vec![],
            failed_at: Utc::now(),
        }),
        "Unhealthy"
    );
    assert_eq!(
        health_to_status_string(&Health::Unknown {
            reason: "test".into(),
            last_known: None,
        }),
        "Unknown"
    );
}

#[test]
fn test_health_to_quick_status() {
    assert_eq!(health_to_quick_status(&Health::Healthy), "ok");
    assert_eq!(
        health_to_quick_status(&Health::Degraded {
            issues: vec![],
            impact_score: None,
        }),
        "issue"
    );
}

#[test]
fn test_health_percentage() {
    assert!((health_percentage(0, 0) - 0.0).abs() < f64::EPSILON);
    assert!((health_percentage(5, 10) - 50.0).abs() < f64::EPSILON);
    assert!((health_percentage(10, 10) - 100.0).abs() < f64::EPSILON);
    let p = health_percentage(1, 3);
    assert!((p - 33.333).abs() < 0.001, "expected ~33.33, got {p}");
}

#[test]
fn test_health_monitor_construction() {
    let config = Arc::new(BiomeOSConfig::default());
    let monitor = HealthMonitor::new(config);
    assert!(std::mem::size_of_val(&monitor) > 0);
}

#[tokio::test]
async fn test_health_monitor_get_system_health() {
    let config = Arc::new(BiomeOSConfig::default());
    let monitor = HealthMonitor::new(config);
    let report = monitor.get_system_health();
    assert_eq!(report.subject.id, "system");
    assert!(matches!(report.health, Health::Healthy));
}

#[test]
fn test_health_to_status_string_critical() {
    let health = Health::Critical {
        issues: vec![],
        affected_capabilities: vec!["compute".to_string()],
    };
    assert_eq!(health_to_status_string(&health), "Unknown");
}

#[test]
fn test_health_to_status_string_starting() {
    let health = Health::Starting {
        phase: biomeos_types::StartupPhase::Initializing,
        progress: 50,
    };
    assert_eq!(health_to_status_string(&health), "Unknown");
}

#[test]
fn test_health_to_quick_status_unknown() {
    let health = Health::Unknown {
        reason: "test".into(),
        last_known: None,
    };
    assert_eq!(health_to_quick_status(&health), "issue");
}

fn test_primal_info(id: &str, name: &str, endpoint: &str, health: Health) -> PrimalInfo {
    PrimalInfo {
        id: id.to_string(),
        name: name.to_string(),
        primal_type: PrimalType::from_discovered("compute", name, "1.0.0"),
        endpoint: endpoint.to_string(),
        capabilities: vec![PrimalCapability::new("compute", "execution", "1.0")],
        health,
        last_seen: Utc::now(),
        discovered_at: Utc::now(),
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_manager_get_system_health() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");
    let report = manager.get_system_health();
    assert_eq!(report.subject.id, "system");
    assert!(matches!(report.health, Health::Healthy));
}

#[tokio::test]
async fn test_manager_probe_endpoint_nonexistent() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");
    // Probing a non-existent socket should fail gracefully
    let result = manager
        .probe_endpoint("unix:///tmp/biomeos_health_test_absent.sock")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_manager_probe_endpoint_unsupported_scheme() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");
    let result = manager.probe_endpoint("ftp://invalid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_check_service_health_found_unreachable() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let primal = test_primal_info(
        "health-1",
        "health-svc",
        "unix:///tmp/biomeos_health_test_absent.sock",
        Health::Healthy,
    );
    manager.register_primal(primal).await.expect("register");

    let result = manager
        .check_service_health("health-svc")
        .await
        .expect("check health");
    // Socket doesn't exist → probe fails → status is Unreachable
    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("Unreachable")
    );
}

#[tokio::test]
async fn test_check_service_health_not_found() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let result = manager
        .check_service_health("nonexistent")
        .await
        .expect("check health");
    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("Not Found")
    );
    assert!(result.contains_key("error"));
}

#[tokio::test]
async fn test_check_system_health() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let result = manager
        .check_system_health()
        .await
        .expect("check system health");
    assert!(result.contains_key("overall_status"));
    assert!(result.contains_key("timestamp"));
    assert!(result.contains_key("services"));
    assert!(result.contains_key("service_summary"));
    assert!(result.contains_key("system_metrics"));
}

#[tokio::test]
async fn test_check_system_health_with_primals() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let primal = test_primal_info("p1", "svc1", "unix:///a.sock", Health::Healthy);
    manager.register_primal(primal).await.expect("register");

    let result = manager.check_system_health().await.expect("check");
    let summary = result.get("service_summary").expect("summary");
    assert_eq!(summary["total"].as_u64(), Some(1));
    assert_eq!(summary["healthy"].as_u64(), Some(1));
}

#[tokio::test]
async fn test_probe_service_health_found() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let primal = test_primal_info("probe-1", "probe-svc", "unix:///x.sock", Health::Healthy);
    manager.register_primal(primal).await.expect("register");

    let result = manager
        .probe_service_health("probe-svc", 5)
        .await
        .expect("probe");
    assert!(result.contains_key("connectivity"));
}

#[tokio::test]
async fn test_probe_service_health_not_found() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let result = manager
        .probe_service_health("nonexistent", 5)
        .await
        .expect("probe");
    assert!(result.contains_key("error"));
}

#[tokio::test]
async fn test_quick_system_scan() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let result = manager.quick_system_scan().await.expect("scan");
    assert_eq!(
        result.get("scan_type").and_then(|v| v.as_str()),
        Some("quick")
    );
    assert!(result.contains_key("services_scanned"));
    assert!(result.contains_key("issues_count"));
}

#[tokio::test]
async fn test_quick_system_scan_with_issues() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let degraded = test_primal_info(
        "d1",
        "degraded-svc",
        "unix:///d.sock",
        Health::Degraded {
            issues: vec![],
            impact_score: None,
        },
    );
    manager.register_primal(degraded).await.expect("register");

    let result = manager.quick_system_scan().await.expect("scan");
    assert_eq!(
        result
            .get("issues_count")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
}

#[tokio::test]
async fn test_comprehensive_system_scan() {
    let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
    manager.initialize().expect("init");

    let result = manager.comprehensive_system_scan().await.expect("scan");
    assert_eq!(
        result.get("scan_type").and_then(|v| v.as_str()),
        Some("comprehensive")
    );
    assert!(result.contains_key("system_health"));
}
