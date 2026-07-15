// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::substitute_env;
use std::collections::HashMap;

#[test]
fn test_substitute_env() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    env.insert("FAMILY_ID".to_string(), "1894e909e454".to_string());

    assert_eq!(substitute_env("${FOO}", &env), "bar");
    assert_eq!(substitute_env("$FOO", &env), "bar");
    assert_eq!(
        substitute_env("prefix-${FAMILY_ID}-suffix", &env),
        "prefix-1894e909e454-suffix"
    );
}

#[test]
fn test_substitute_env_missing() {
    let env = HashMap::new();
    assert_eq!(substitute_env("${MISSING}", &env), "${MISSING}");
}

#[test]
fn test_substitute_env_multiple_vars() {
    let mut env = HashMap::new();
    env.insert("A".to_string(), "alpha".to_string());
    env.insert("B".to_string(), "beta".to_string());
    env.insert("C".to_string(), "gamma".to_string());

    assert_eq!(substitute_env("${A}/${B}/${C}", &env), "alpha/beta/gamma");
}

#[test]
fn test_substitute_env_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string());
    env.insert("FAMILY_ID".to_string(), "cf7e8729".to_string());

    assert_eq!(
        substitute_env("${XDG_RUNTIME_DIR}/biomeos/beardog-${FAMILY_ID}.sock", &env),
        "/run/user/1000/biomeos/beardog-cf7e8729.sock"
    );
}

#[test]
fn test_substitute_env_empty_value() {
    let mut env = HashMap::new();
    env.insert("EMPTY".to_string(), String::new());

    assert_eq!(
        substitute_env("prefix-${EMPTY}-suffix", &env),
        "prefix--suffix"
    );
}

#[test]
fn test_substitute_env_no_vars_in_string() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());

    assert_eq!(
        substitute_env("no variables here", &env),
        "no variables here"
    );
}

#[test]
fn test_substitute_env_dollar_sign_syntax() {
    let mut env = HashMap::new();
    env.insert("PORT".to_string(), "8080".to_string());

    // $PORT syntax (without braces)
    assert_eq!(substitute_env("localhost:$PORT", &env), "localhost:8080");
}

#[test]
fn test_substitute_env_repeated_var() {
    let mut env = HashMap::new();
    env.insert("HOST".to_string(), "gate2".to_string());

    assert_eq!(substitute_env("${HOST}:${HOST}", &env), "gate2:gate2");
}
