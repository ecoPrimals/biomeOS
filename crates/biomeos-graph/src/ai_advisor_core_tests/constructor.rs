// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::time::Duration;

use super::super::*;

#[test]
fn ai_advisor_core_new_constructor_initial_state() {
    let advisor = AiGraphAdvisor::new();
    assert!(!advisor.squirrel_available);
    assert!(advisor.ai_socket_path.is_none());
    assert_eq!(advisor.squirrel_timeout, Duration::from_secs(5));
    assert_eq!(advisor.local_patterns.len(), 3);
}

#[test]
fn ai_advisor_core_advisor_with_timeout_constructor_stores_duration() {
    let timeout_secs = 42;
    let advisor = AiGraphAdvisor::with_timeout(Duration::from_secs(timeout_secs));
    assert!(!advisor.squirrel_available);
    assert_eq!(advisor.local_patterns.len(), 3);
    assert_eq!(advisor.squirrel_timeout, Duration::from_secs(timeout_secs));
}

#[test]
fn ai_advisor_core_advisor_default_matches_new() {
    let from_default = AiGraphAdvisor::default();
    let from_new = AiGraphAdvisor::new();
    assert_eq!(from_default.squirrel_available, from_new.squirrel_available);
    assert_eq!(
        from_default.local_patterns.len(),
        from_new.local_patterns.len()
    );
}
