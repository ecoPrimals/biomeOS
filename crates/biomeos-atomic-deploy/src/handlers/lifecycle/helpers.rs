// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shared lifecycle state serialization helpers.

use serde_json::{Value, json};

use crate::lifecycle_manager::LifecycleState;

/// Get state-specific details
pub(crate) fn state_details(state: &LifecycleState) -> Value {
    match state {
        LifecycleState::Germinating => json!({}),
        LifecycleState::Incubating {
            started_at,
            timeout_ms,
        } => json!({
            "started_at": started_at.to_rfc3339(),
            "timeout_ms": timeout_ms
        }),
        LifecycleState::Active {
            since,
            last_health_check,
        } => json!({
            "since": since.to_rfc3339(),
            "last_health_check": last_health_check.to_rfc3339()
        }),
        LifecycleState::Degraded {
            since,
            reason,
            resurrection_attempts,
        } => json!({
            "since": since.to_rfc3339(),
            "reason": reason,
            "resurrection_attempts": resurrection_attempts
        }),
        LifecycleState::Apoptosis { reason, started_at } => json!({
            "reason": format!("{:?}", reason),
            "started_at": started_at.to_rfc3339()
        }),
        LifecycleState::Dead { since, reason } => json!({
            "since": since.to_rfc3339(),
            "reason": reason
        }),
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Copyright 2025-2026 ecoPrimals Project

    use super::*;
    use crate::lifecycle_manager::ApoptosisReason;

    fn fixed_time() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-01-15T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    #[test]
    fn germinating_returns_empty_object() {
        let val = state_details(&LifecycleState::Germinating);
        assert_eq!(val, json!({}));
    }

    #[test]
    fn incubating_contains_started_at_and_timeout() {
        let t = fixed_time();
        let val = state_details(&LifecycleState::Incubating {
            started_at: t,
            timeout_ms: 5000,
        });
        assert_eq!(val["timeout_ms"], 5000);
        assert_eq!(val["started_at"].as_str().unwrap(), t.to_rfc3339());
    }

    #[test]
    fn active_contains_since_and_last_health_check() {
        let t = fixed_time();
        let val = state_details(&LifecycleState::Active {
            since: t,
            last_health_check: t,
        });
        assert_eq!(val["since"].as_str().unwrap(), t.to_rfc3339());
        assert_eq!(val["last_health_check"].as_str().unwrap(), t.to_rfc3339());
    }

    #[test]
    fn degraded_contains_reason_and_attempts() {
        let t = fixed_time();
        let val = state_details(&LifecycleState::Degraded {
            since: t,
            reason: "high latency".to_string(),
            resurrection_attempts: 3,
        });
        assert_eq!(val["reason"], "high latency");
        assert_eq!(val["resurrection_attempts"], 3);
        assert_eq!(val["since"].as_str().unwrap(), t.to_rfc3339());
    }

    #[test]
    fn apoptosis_contains_debug_reason() {
        let t = fixed_time();
        for reason in [
            ApoptosisReason::UserRequest,
            ApoptosisReason::EcosystemHealth,
            ApoptosisReason::ResourcePressure,
            ApoptosisReason::DependencyDeath("beardog".to_string()),
            ApoptosisReason::ResurrectionExhausted,
            ApoptosisReason::SystemShutdown,
        ] {
            let expected_debug = format!("{reason:?}");
            let val = state_details(&LifecycleState::Apoptosis {
                reason,
                started_at: t,
            });
            assert_eq!(val["reason"].as_str().unwrap(), expected_debug);
            assert_eq!(val["started_at"].as_str().unwrap(), t.to_rfc3339());
        }
    }

    #[test]
    fn dead_contains_since_and_reason() {
        let t = fixed_time();
        let val = state_details(&LifecycleState::Dead {
            since: t,
            reason: "OOM killed".to_string(),
        });
        assert_eq!(val["reason"], "OOM killed");
        assert_eq!(val["since"].as_str().unwrap(), t.to_rfc3339());
    }

    #[test]
    fn all_variants_return_json_object() {
        let t = fixed_time();
        let states = vec![
            LifecycleState::Germinating,
            LifecycleState::Incubating {
                started_at: t,
                timeout_ms: 1000,
            },
            LifecycleState::Active {
                since: t,
                last_health_check: t,
            },
            LifecycleState::Degraded {
                since: t,
                reason: "x".to_string(),
                resurrection_attempts: 0,
            },
            LifecycleState::Apoptosis {
                reason: ApoptosisReason::SystemShutdown,
                started_at: t,
            },
            LifecycleState::Dead {
                since: t,
                reason: "crash".to_string(),
            },
        ];
        for state in &states {
            let val = state_details(state);
            assert!(val.is_object(), "expected object for {state:?}");
        }
    }
}
