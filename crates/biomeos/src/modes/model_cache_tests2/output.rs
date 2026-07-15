// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{nestgate_status_label, print_resolve_model_resolution};
use biomeos_core::model_cache::{ModelEntry, ModelFile, ModelResolution};

#[test]
fn nestgate_status_label_connected_when_mesh_registry_active() {
    assert_eq!(
        nestgate_status_label(false),
        "connected (mesh registry active)"
    );
}

#[test]
fn print_resolve_model_resolution_local_with_files_branch() {
    let entry = ModelEntry {
        model_id: "test/files-branch".to_string(),
        local_path: std::path::PathBuf::from("/tmp/model"),
        size_bytes: 2_097_152,
        source: "test".to_string(),
        sha256: None,
        cached_at: "2025-01-01".to_string(),
        gate_id: "gate-local".to_string(),
        format: "huggingface".to_string(),
        files: vec![
            ModelFile {
                relative_path: "a.bin".to_string(),
                size_bytes: 1,
                sha256: None,
            },
            ModelFile {
                relative_path: "b.bin".to_string(),
                size_bytes: 2,
                sha256: None,
            },
        ],
    };
    print_resolve_model_resolution("test/files-branch", &ModelResolution::Local(entry));
}

#[test]
fn print_resolve_model_resolution_remote_branch() {
    let entry = ModelEntry {
        model_id: "remote/m".to_string(),
        local_path: std::path::PathBuf::from("/elsewhere"),
        size_bytes: 1_048_576,
        source: "mesh".to_string(),
        sha256: None,
        cached_at: "2025-01-01".to_string(),
        gate_id: "remote-gate-7".to_string(),
        format: "gguf".to_string(),
        files: vec![],
    };
    print_resolve_model_resolution("remote/m", &ModelResolution::Remote(entry));
}
