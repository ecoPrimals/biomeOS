// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_port_pattern_sequential_from_json() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 1,
        "last_port": 41204,
        "predicted_next": 41205,
        "confidence": 0.85
    });

    let pattern = PortPattern::from_json(&json);
    assert!(pattern.is_predictable());

    if let PortPattern::Sequential {
        step,
        last_port,
        predicted_next,
        confidence,
    } = pattern
    {
        assert_eq!(step, 1);
        assert_eq!(last_port, 41204);
        assert_eq!(predicted_next, 41205);
        assert!((confidence - 0.85).abs() < f64::EPSILON);
    } else {
        panic!("Expected Sequential pattern");
    }
}

#[test]
fn test_port_pattern_random_from_json() {
    let json = serde_json::json!({
        "type": "random",
        "observed": [41200, 52300, 10500, 33000]
    });

    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());

    if let PortPattern::Random { observed } = pattern {
        assert_eq!(observed.len(), 4);
        assert_eq!(observed[0], 41200);
    } else {
        panic!("Expected Random pattern");
    }
}

#[test]
fn test_port_pattern_unknown_from_json() {
    let json = serde_json::json!({});
    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());
    assert!(matches!(pattern, PortPattern::Unknown));
}

#[test]
fn test_port_pattern_low_confidence_not_predictable() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 3,
        "last_port": 50000,
        "predicted_next": 50003,
        "confidence": 0.3
    });

    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());
}

#[test]
fn test_port_pattern_from_json_sequential_defaults() {
    let json = serde_json::json!({"type": "sequential"});
    let pattern = PortPattern::from_json(&json);
    if let PortPattern::Sequential {
        step,
        last_port,
        predicted_next,
        confidence,
    } = pattern
    {
        assert_eq!(step, 1);
        assert_eq!(last_port, 0);
        assert_eq!(predicted_next, 0);
        assert!((confidence - 0.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected Sequential with defaults");
    }
}

#[test]
fn test_port_pattern_from_json_random_empty() {
    let json = serde_json::json!({"type": "random"});
    let pattern = PortPattern::from_json(&json);
    assert!(matches!(pattern, PortPattern::Random { observed } if observed.is_empty()));
}

#[test]
fn port_pattern_random_non_numeric_observed_entries_skipped() {
    let json = serde_json::json!({
        "type": "random",
        "observed": ["not-a-number", 41200, null]
    });
    let pattern = PortPattern::from_json(&json);
    match pattern {
        PortPattern::Random { observed } => assert_eq!(observed, vec![41200u16]),
        _ => panic!("expected Random"),
    }
}

#[test]
fn test_port_pattern_sequential_exact_confidence_threshold() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 2,
        "last_port": 50000,
        "predicted_next": 50002,
        "confidence": 0.6
    });
    let pattern = PortPattern::from_json(&json);
    assert!(pattern.is_predictable());
}

#[test]
fn test_port_pattern_sequential_just_below_threshold() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 1,
        "last_port": 40000,
        "predicted_next": 40001,
        "confidence": 0.59
    });
    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());
}

#[test]
fn test_port_pattern_unknown_type() {
    let json = serde_json::json!({"type": "custom_unknown"});
    let pattern = PortPattern::from_json(&json);
    assert!(matches!(pattern, PortPattern::Unknown));
    assert!(!pattern.is_predictable());
}
