// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::*;
use biomeos_types::HealthSubjectType;

#[tokio::test]
async fn test_system_info_collection() {
    let system_info = SystemInspector::get_system_info()
        .await
        .expect("get_system_info should succeed");

    assert!(
        !system_info.hostname.is_empty(),
        "hostname should not be empty"
    );
    assert!(
        !system_info.kernel_info.name.is_empty(),
        "kernel name should not be empty"
    );
    assert!(
        !system_info.kernel_info.architecture.is_empty(),
        "architecture should not be empty"
    );
    assert!(
        system_info.kernel_info.architecture == std::env::consts::ARCH,
        "kernel architecture should match target architecture"
    );
    assert!(
        system_info.cpu_info.cores >= 1,
        "should have at least 1 core"
    );
    assert!(
        system_info.memory_info.total_gb >= 0.0,
        "total memory should be non-negative"
    );
    assert!(
        system_info.memory_info.usage_percent >= 0.0
            && system_info.memory_info.usage_percent <= 1.0,
        "memory usage_percent should be in 0-1 range"
    );
    assert!(
        !system_info.disk_info.is_empty(),
        "should have at least one disk"
    );
    assert!(
        !system_info.network_info.is_empty(),
        "should have at least one network interface"
    );
    assert!(
        system_info.uptime.as_secs() > 0,
        "uptime should be positive"
    );
    assert!(
        system_info.load_average.load_1m >= 0.0
            && system_info.load_average.load_5m >= 0.0
            && system_info.load_average.load_15m >= 0.0,
        "load averages should be non-negative"
    );
}

#[tokio::test]
async fn test_resource_usage() {
    let resource_usage = SystemInspector::get_resource_usage()
        .await
        .expect("get_resource_usage should succeed");

    assert!(
        resource_usage.cpu_usage.is_some(),
        "cpu_usage should be present"
    );
    assert!(
        resource_usage.memory_usage.is_some(),
        "memory_usage should be present"
    );
    assert!(
        resource_usage.disk_usage.is_some(),
        "disk_usage should be present"
    );
    assert!(
        resource_usage.network_io.is_some(),
        "network_io should be present"
    );

    let cpu = resource_usage.cpu_usage.unwrap();
    assert!(
        (0.0..=1.0).contains(&cpu),
        "cpu_usage should be in 0-1 range, got {cpu}"
    );

    let memory = resource_usage.memory_usage.unwrap();
    assert!(
        (0.0..=1.0).contains(&memory),
        "memory_usage should be in 0-1 range, got {memory}"
    );

    let disk = resource_usage.disk_usage.unwrap();
    assert!(
        (0.0..=1.0).contains(&disk),
        "disk_usage should be in 0-1 range, got {disk}"
    );

    let network = resource_usage.network_io.as_ref().unwrap();
    assert!(
        network.bytes_in_per_sec >= 0.0 && network.bytes_out_per_sec >= 0.0,
        "network I/O should be non-negative"
    );
}

#[tokio::test]
async fn test_system_health() {
    let health_report = SystemInspector::get_system_health()
        .await
        .expect("get_system_health should succeed");

    assert_eq!(
        health_report.subject.subject_type,
        HealthSubjectType::System,
        "subject type should be System"
    );
    assert_eq!(
        health_report.subject.name, "biomeOS System",
        "subject name should match"
    );
    assert!(
        !health_report.subject.id.is_empty(),
        "subject id should not be empty"
    );
    assert!(
        !health_report.components.is_empty(),
        "should have components"
    );

    assert!(
        health_report.components.contains_key("cpu"),
        "should have CPU component"
    );
    assert!(
        health_report.components.contains_key("memory"),
        "should have memory component"
    );

    assert!(
        health_report.metrics.resources.is_some(),
        "metrics should include resources"
    );
    assert!(
        health_report.metrics.availability.is_some(),
        "metrics should include availability"
    );

    let availability = health_report.metrics.availability.as_ref().unwrap();
    assert!(
        availability.uptime_percentage >= 0.0 && availability.uptime_percentage <= 1.0,
        "uptime_percentage should be in 0-1 range"
    );
}

#[tokio::test]
async fn test_system_info_and_health_consistency() {
    let system_info = SystemInspector::get_system_info()
        .await
        .expect("get_system_info should succeed");
    let health_report = SystemInspector::get_system_health()
        .await
        .expect("get_system_health should succeed");

    assert_eq!(
        system_info.hostname, health_report.subject.id,
        "health report subject id should match system hostname"
    );
}
