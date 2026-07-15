// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::SystemInspector;

#[test]
fn test_hostname_retrieval() {
    let hostname = SystemInspector::get_hostname().expect("get_hostname should succeed");
    assert!(!hostname.is_empty(), "hostname should not be empty");
}

#[test]
fn test_kernel_info() {
    let kernel_info = SystemInspector::get_kernel_info().expect("get_kernel_info should succeed");

    assert!(
        !kernel_info.name.is_empty(),
        "kernel name should not be empty"
    );
    assert!(
        !kernel_info.architecture.is_empty(),
        "architecture should not be empty"
    );
    assert_eq!(
        kernel_info.architecture,
        std::env::consts::ARCH,
        "architecture should match target"
    );
}

#[test]
fn test_get_hostname_with_override() {
    let h = SystemInspector::get_hostname_with(Some("env-host-test")).expect("hostname");
    assert_eq!(h, "env-host-test");
}
