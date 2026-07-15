// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::time::Duration;

use super::super::format::*;
use crate::commands::utils::{format_bytes, format_duration};

#[test]
fn test_should_stop_monitoring_no_duration() {
    assert!(!should_stop_monitoring(Duration::from_secs(100), None));
}

#[test]
fn test_should_stop_monitoring_not_reached() {
    assert!(!should_stop_monitoring(
        Duration::from_secs(50),
        Some(Duration::from_secs(100))
    ));
}

#[test]
fn test_should_stop_monitoring_reached() {
    assert!(should_stop_monitoring(
        Duration::from_secs(100),
        Some(Duration::from_secs(100))
    ));
}

#[test]
fn test_should_stop_monitoring_exceeded() {
    assert!(should_stop_monitoring(
        Duration::from_secs(150),
        Some(Duration::from_secs(100))
    ));
}

#[test]
fn test_service_status_icon_all_variants() {
    assert_eq!(service_status_icon("running"), "✅");
    assert_eq!(service_status_icon("starting"), "🔄");
    assert_eq!(service_status_icon("stopping"), "⏹️");
    assert_eq!(service_status_icon("stopped"), "⏸️");
    assert_eq!(service_status_icon("error"), "❌");
    assert_eq!(service_status_icon("unknown"), "❓");
    assert_eq!(service_status_icon(""), "❓");
}

#[test]
fn test_service_health_icon_all_variants() {
    assert_eq!(service_health_icon("Healthy"), "💚");
    assert_eq!(service_health_icon("Degraded"), "💛");
    assert_eq!(service_health_icon("Critical"), "🧡");
    assert_eq!(service_health_icon("Unhealthy"), "❤️");
    assert_eq!(service_health_icon("unknown"), "🤍");
    assert_eq!(service_health_icon(""), "🤍");
}

#[test]
fn test_alert_severity_icon_all_variants() {
    assert_eq!(alert_severity_icon("critical"), "🔴");
    assert_eq!(alert_severity_icon("warning"), "🟡");
    assert_eq!(alert_severity_icon("info"), "🔵");
    assert_eq!(alert_severity_icon("unknown"), "⚪");
    assert_eq!(alert_severity_icon(""), "⚪");
}

#[test]
fn test_log_level_icon_all_variants() {
    assert_eq!(log_level_icon("error"), "❌");
    assert_eq!(log_level_icon("warn"), "⚠️");
    assert_eq!(log_level_icon("info"), "ℹ️");
    assert_eq!(log_level_icon("debug"), "🐛");
    assert_eq!(log_level_icon("trace"), "🔍");
    assert_eq!(log_level_icon("unknown"), "📝");
    assert_eq!(log_level_icon(""), "📝");
}

#[test]
fn test_scale_status_icon_all_variants() {
    assert_eq!(scale_status_icon("success"), "✅");
    assert_eq!(scale_status_icon("in_progress"), "🔄");
    assert_eq!(scale_status_icon("failed"), "❌");
    assert_eq!(scale_status_icon("unknown"), "🔹");
    assert_eq!(scale_status_icon(""), "🔹");
}

#[test]
fn test_format_bytes_via_utils() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
}

#[test]
fn test_format_duration_via_utils() {
    assert_eq!(format_duration(std::time::Duration::from_secs(45)), "45s");
    assert_eq!(
        format_duration(std::time::Duration::from_secs(125)),
        "2m 5s"
    );
}
