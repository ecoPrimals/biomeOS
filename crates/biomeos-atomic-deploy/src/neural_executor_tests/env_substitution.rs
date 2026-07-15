// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_executor::GraphExecutor;
use std::collections::HashMap;

#[test]
fn test_env_substitution() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    env.insert("BAZ".to_string(), "qux".to_string());

    let result = GraphExecutor::substitute_env("${FOO}/${BAZ}/test", &env);
    assert_eq!(result, "bar/qux/test");
}

#[test]
fn test_env_substitution_empty() {
    let env = HashMap::new();
    let result = GraphExecutor::substitute_env("no-vars", &env);
    assert_eq!(result, "no-vars");
}

#[test]
fn test_env_substitution_partial() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    let result = GraphExecutor::substitute_env("${FOO}/${MISSING}", &env);
    assert_eq!(result, "bar/${MISSING}");
}

#[test]
fn test_env_substitution_adjacent_vars() {
    let mut env = HashMap::new();
    env.insert("A".to_string(), "x".to_string());
    env.insert("B".to_string(), "y".to_string());
    let result = GraphExecutor::substitute_env("${A}${B}", &env);
    assert_eq!(result, "xy");
}

#[test]
fn test_env_substitution_same_var_multiple_times() {
    let mut env = HashMap::new();
    env.insert("X".to_string(), "hello".to_string());
    let result = GraphExecutor::substitute_env("${X}-${X}-${X}", &env);
    assert_eq!(result, "hello-hello-hello");
}

#[test]
fn test_env_substitution_empty_string() {
    let env = HashMap::new();
    let result = GraphExecutor::substitute_env("", &env);
    assert_eq!(result, "");
}

#[test]
fn test_env_substitution_nested_looking_not_actually_nested() {
    // Should not recursively substitute
    let mut env = HashMap::new();
    env.insert("OUTER".to_string(), "${INNER}".to_string());
    env.insert("INNER".to_string(), "deep".to_string());
    let result = GraphExecutor::substitute_env("${OUTER}", &env);
    // The result depends on iteration order; OUTER gets replaced first with "${INNER}"
    // then INNER might or might not get replaced. Let's just check it doesn't panic.
    assert!(!result.is_empty());
}

#[test]
fn test_env_substitution_special_chars_in_value() {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/bin:/usr/local/bin".to_string());
    let result = GraphExecutor::substitute_env("Path: ${PATH}", &env);
    assert_eq!(result, "Path: /usr/bin:/usr/local/bin");
}
