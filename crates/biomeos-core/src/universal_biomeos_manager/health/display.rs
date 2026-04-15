// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Pure helpers for mapping [`Health`](biomeos_types::Health) to display strings and percentages.

use biomeos_types::Health;

/// Map Health enum to display string (testable pure function)
pub(crate) const fn health_to_status_string(health: &Health) -> &'static str {
    match health {
        Health::Healthy => "Healthy",
        Health::Degraded { .. } => "Degraded",
        Health::Unhealthy { .. } => "Unhealthy",
        _ => "Unknown",
    }
}

/// Map Health to quick scan status ("ok" or "issue")
pub(crate) const fn health_to_quick_status(health: &Health) -> &'static str {
    match health {
        Health::Healthy => "ok",
        _ => "issue",
    }
}

/// Compute health percentage from counts
pub(crate) fn health_percentage(healthy: usize, total: usize) -> f64 {
    if total > 0 {
        (healthy as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}
