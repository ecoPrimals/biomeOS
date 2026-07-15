// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::scope_covers_method;

#[test]
fn scope_wildcard_matches_all() {
    let scope = vec!["*".to_owned()];
    assert!(scope_covers_method(&scope, "anything.here"));
    assert!(scope_covers_method(&scope, "graph.execute"));
}

#[test]
fn scope_prefix_matches_domain() {
    let scope = vec!["compute.*".to_owned()];
    assert!(scope_covers_method(&scope, "compute.submit"));
    assert!(scope_covers_method(&scope, "compute.status"));
    assert!(!scope_covers_method(&scope, "storage.get"));
    assert!(!scope_covers_method(&scope, "compute_x.submit"));
}

#[test]
fn scope_exact_matches() {
    let scope = vec!["graph.execute".to_owned()];
    assert!(scope_covers_method(&scope, "graph.execute"));
    assert!(!scope_covers_method(&scope, "graph.save"));
}

#[test]
fn scope_empty_denies_all() {
    assert!(!scope_covers_method(&[], "anything"));
}

#[test]
fn scope_multiple_patterns() {
    let scope = vec!["compute.*".to_owned(), "storage.get".to_owned()];
    assert!(scope_covers_method(&scope, "compute.submit"));
    assert!(scope_covers_method(&scope, "storage.get"));
    assert!(!scope_covers_method(&scope, "storage.put"));
}
