// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::pid_file_path;
use super::common::mock_env;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn pid_file_path_uses_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/run/user/1000/membrane/tower.pid"));
}

#[test]
fn pid_file_path_falls_back_to_family_id() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "nat0".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/membrane-nat0/tower.pid"));
}

#[test]
fn pid_file_path_falls_back_to_default() {
    let env: HashMap<String, String> = HashMap::new();
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/membrane-default/tower.pid"));
}

#[test]
fn pid_file_path_prefers_biomeos_family_over_family_id() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "preferred".to_string());
    env.insert("FAMILY_ID".to_string(), "fallback".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/membrane-preferred/tower.pid"));
}

#[test]
fn pid_file_path_uses_family_id_env_alone() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "only_family".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/membrane-only_family/tower.pid"));
}

#[test]
fn pid_file_path_xdg_takes_precedence_over_both_family_vars() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/xdg/rt".to_string());
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "fam-a".to_string());
    env.insert("FAMILY_ID".to_string(), "fam-b".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/xdg/rt/membrane/tower.pid"));
}
