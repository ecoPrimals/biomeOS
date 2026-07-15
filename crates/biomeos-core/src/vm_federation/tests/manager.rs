// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::{ValidationConfig, VmFederationManager};

#[test]
fn test_manager_creation() {
    let manager = VmFederationManager::new();
    // Manager creation requires benchscale directory to exist
    // This is a valid requirement, so we just verify the Result type works
    match manager {
        Ok(_) => {
            // benchscale exists - great!
        }
        Err(e) => {
            // benchscale doesn't exist - expected in CI/test environments
            assert!(
                e.to_string().contains("benchscale not found"),
                "Error should be about missing benchscale, got: {e}"
            );
        }
    }
}

#[test]
fn test_with_validation_config_requires_benchscale() {
    let config = ValidationConfig::default();
    let result = VmFederationManager::with_validation_config(config);
    match result {
        Ok(_) => {}
        Err(e) => {
            assert!(
                e.to_string().contains("benchscale not found") || e.to_string().contains("parent"),
                "Expected benchscale or path error, got: {e}"
            );
        }
    }
}
