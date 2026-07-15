// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::{Health, SystemInspector};

#[test]
fn test_cpu_component_health_at_upper_noncritical_before_degraded_band() {
    // > 0.9 is Critical; exactly 0.9 falls through to the > 0.7 Degraded branch
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.9)),
        Health::Degraded { .. }
    ));
}

#[test]
fn test_memory_component_health_at_critical_threshold() {
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.96)),
        Health::Critical { .. }
    ));
}

#[test]
fn test_cpu_component_health_branches() {
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.91)),
        Health::Critical { .. }
    ));
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.75)),
        Health::Degraded { .. }
    ));
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.5)),
        Health::Healthy
    ));
    assert!(matches!(
        SystemInspector::cpu_component_health(None),
        Health::Healthy
    ));
}

#[test]
fn test_memory_component_health_branches() {
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.96)),
        Health::Critical { .. }
    ));
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.81)),
        Health::Degraded { .. }
    ));
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.5)),
        Health::Healthy
    ));
    assert!(matches!(
        SystemInspector::memory_component_health(None),
        Health::Healthy
    ));
}

#[test]
fn test_memory_component_health_at_95_percent_is_degraded() {
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.95)),
        Health::Degraded { .. }
    ));
}
