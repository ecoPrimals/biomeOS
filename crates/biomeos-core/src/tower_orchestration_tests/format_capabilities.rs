// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::format_capabilities;

#[test]
fn format_capabilities_returns_known_categories() {
    let caps = format_capabilities();
    assert!(caps.len() >= 7);
    assert!(caps.iter().any(|(name, _)| *name == "Security"));
    assert!(caps.iter().any(|(name, _)| *name == "Discovery"));
    assert!(caps.iter().any(|(name, _)| *name == "AI"));
}

#[test]
fn format_capabilities_includes_all_known_categories() {
    let caps = format_capabilities();
    assert!(caps.len() >= 8);
    let names: Vec<&str> = caps.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"Security"));
    assert!(names.contains(&"Discovery"));
    assert!(names.contains(&"Compute"));
    assert!(names.contains(&"AI"));
    assert!(names.contains(&"Storage"));
    assert!(names.contains(&"Observability"));
    assert!(names.contains(&"Federation"));
    assert!(names.contains(&"Network"));
}
