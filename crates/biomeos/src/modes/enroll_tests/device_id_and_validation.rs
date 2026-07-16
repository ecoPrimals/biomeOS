// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_resolve_device_id_explicit() {
    let id = resolve_device_id(Some("custom-device-123"));
    assert_eq!(id, "custom-device-123");
}

#[test]
fn test_resolve_device_id_empty_string_filters() {
    let id = resolve_device_id(Some(""));
    assert!(!id.is_empty());
}

#[test]
fn test_resolve_device_id_whitespace_only_passes() {
    let id = resolve_device_id(Some("   "));
    assert_eq!(id, "   ");
}

#[test]
fn test_resolve_device_id_none_generates() {
    let id = resolve_device_id(None);
    assert!(!id.is_empty());
    assert!(id.len() >= 32);
}

#[test]
fn test_validate_enrollment_paths_family_seed_missing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    let family = temp.path().join("nonexistent.family.seed");
    let result = validate_enrollment_paths(&lineage, &family, false);
    assert!(matches!(
        result,
        Err(EnrollmentValidationError::FamilySeedNotFound)
    ));
}

#[test]
fn test_validate_enrollment_paths_already_enrolled() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    std::fs::write(&lineage, "existing").expect("write lineage");
    let family = temp.path().join(".family.seed");
    std::fs::write(&family, "seed").expect("write family");
    let result = validate_enrollment_paths(&lineage, &family, false);
    assert!(matches!(
        result,
        Err(EnrollmentValidationError::AlreadyEnrolled)
    ));
}

#[test]
fn test_validate_enrollment_paths_force_ok() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    std::fs::write(&lineage, "existing").expect("write lineage");
    let family = temp.path().join(".family.seed");
    std::fs::write(&family, "seed").expect("write family");
    let result = validate_enrollment_paths(&lineage, &family, true);
    assert!(result.is_ok());
}

#[test]
fn test_validate_enrollment_paths_fresh_enrollment() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    let family = temp.path().join(".family.seed");
    std::fs::write(&family, "seed").expect("write family");
    let result = validate_enrollment_paths(&lineage, &family, false);
    assert!(
        result.is_ok(),
        "fresh enrollment (no lineage) should succeed"
    );
}

#[test]
fn test_enrollment_validation_error_display() {
    let already = EnrollmentValidationError::AlreadyEnrolled;
    assert!(
        already.to_string().contains("already enrolled"),
        "AlreadyEnrolled display: {}",
        already
    );
    assert!(already.to_string().contains("force"));

    let not_found = EnrollmentValidationError::FamilySeedNotFound;
    assert!(
        not_found.to_string().contains("not found"),
        "FamilySeedNotFound display: {}",
        not_found
    );
}
