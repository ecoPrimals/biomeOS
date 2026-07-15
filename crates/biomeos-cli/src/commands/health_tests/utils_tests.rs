// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_status_to_icon_all_variants() {
    assert_eq!(status_to_icon("Healthy"), "✅");
    assert_eq!(status_to_icon("Degraded"), "⚠️");
    assert_eq!(status_to_icon("Critical"), "🔴");
    assert_eq!(status_to_icon("Unhealthy"), "❌");
    assert_eq!(status_to_icon("Starting"), "🔄");
    assert_eq!(status_to_icon("Stopping"), "⏹️");
    assert_eq!(status_to_icon("Maintenance"), "🔧");
    assert_eq!(status_to_icon("Unknown"), "❓");
    assert_eq!(status_to_icon("custom"), "🔹");
    assert_eq!(status_to_icon(""), "🔹");
}

#[test]
fn test_compute_memory_percent() {
    assert!((compute_memory_percent(0, 0) - 0.0).abs() < f64::EPSILON);
    assert!((compute_memory_percent(512, 1024) - 50.0).abs() < f64::EPSILON);
    assert!((compute_memory_percent(256, 1024) - 25.0).abs() < f64::EPSILON);
    assert!((compute_memory_percent(1024, 1024) - 100.0).abs() < f64::EPSILON);
    let p = compute_memory_percent(1, 3);
    assert!((p - 33.333).abs() < 0.001, "expected ~33.333, got {p}");
}

#[test]
fn test_format_bytes_edge_cases() {
    use super::super::super::utils::format_bytes;
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
}

#[test]
fn test_compute_memory_percent_one_byte() {
    let p = compute_memory_percent(1, 1024);
    assert!((p - (100.0 / 1024.0)).abs() < 1e-9);
}

#[test]
fn test_status_to_icon_edge() {
    assert_eq!(status_to_icon("RandomStatus"), "🔹");
}

#[test]
fn test_compute_memory_percent_large_values() {
    let p = compute_memory_percent(1_000_000_000, 2_000_000_000);
    assert!((p - 50.0).abs() < 1e-6);
}

#[test]
fn test_status_to_icon_maintenance_and_stopping() {
    assert_eq!(status_to_icon("Maintenance"), "🔧");
    assert_eq!(status_to_icon("Stopping"), "⏹️");
}
