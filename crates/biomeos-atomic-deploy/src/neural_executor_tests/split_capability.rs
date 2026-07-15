// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_executor::GraphExecutor;

#[test]
fn test_split_capability_with_dot() {
    let (domain, op) = GraphExecutor::split_capability("ecology.et0_fao56");
    assert_eq!(domain, "ecology");
    assert_eq!(op, "et0_fao56");
}

#[test]
fn test_split_capability_without_dot() {
    let (domain, op) = GraphExecutor::split_capability("single");
    assert_eq!(domain, "single");
    assert_eq!(op, "execute");
}

#[test]
fn test_split_capability_empty() {
    let (domain, op) = GraphExecutor::split_capability("");
    assert_eq!(domain, "");
    assert_eq!(op, "execute");
}

#[test]
fn test_split_capability_multiple_dots() {
    let (domain, op) = GraphExecutor::split_capability("a.b.c");
    assert_eq!(domain, "a");
    assert_eq!(op, "b.c");
}

#[test]
fn test_split_capability_leading_dot() {
    // "domain.op" format - leading dot would be edge case
    let (domain, op) = GraphExecutor::split_capability(".onlyop");
    assert_eq!(domain, "");
    assert_eq!(op, "onlyop");
}

#[test]
fn test_split_capability_trailing_dot() {
    let (domain, op) = GraphExecutor::split_capability("domain.");
    assert_eq!(domain, "domain");
    assert_eq!(op, "");
}
