// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! PseudoSpore envelope validation and param mapping.

use anyhow::Result;
use biomeos_pseudospore::{self as pseudospore, PseudoSporeManifest, SporeStatus};
use std::path::{Path, PathBuf};

/// Validated pseudoSpore envelope metadata, mapped from `PseudoSporeManifest`.
#[derive(Debug)]
pub(super) struct Envelope {
    pub scope_id: String,
    pub pseudospore_dir: PathBuf,
    pub data_file_count: usize,
    pub checksums: Vec<(String, String)>,
    pub(super) scope_json: serde_json::Value,
}

impl Envelope {
    pub fn from_manifest(manifest: &PseudoSporeManifest) -> Self {
        Self {
            scope_id: manifest.scope.artifact.name.clone(),
            pseudospore_dir: manifest.root.clone(),
            data_file_count: manifest.checksums.len(),
            checksums: manifest
                .checksums
                .iter()
                .map(|c| (c.path.clone(), c.hash.clone()))
                .collect(),
            scope_json: serde_json::json!({
                "name": manifest.scope.artifact.name,
                "version": manifest.scope.artifact.version,
                "type": manifest.scope.artifact.artifact_type,
                "date": manifest.scope.artifact.date,
                "origin": manifest.scope.artifact.origin,
                "spring": manifest.ferment.spring,
                "dataset_id": manifest.ferment.dataset_id,
            }),
        }
    }

    pub fn to_params(&self) -> serde_json::Value {
        serde_json::json!({
            "scope_id": self.scope_id,
            "source_dir": self.pseudospore_dir.display().to_string(),
            "data_file_count": self.data_file_count,
            "checksums": self.checksums.iter()
                .map(|(path, hash)| serde_json::json!({"file": path, "blake3": hash}))
                .collect::<Vec<_>>(),
            "manifest": self.scope_json,
        })
    }
}

/// Validate a pseudoSpore directory using the canonical pseudoSpore 2.0 standard.
pub(super) fn validate_envelope(dir: &Path) -> Result<Envelope> {
    anyhow::ensure!(
        dir.is_dir(),
        "pseudoSpore path is not a directory: {}",
        dir.display()
    );

    let mut manifest = pseudospore::load_pseudospore(dir);

    if manifest.status == SporeStatus::Invalid {
        anyhow::bail!(
            "Invalid pseudoSpore at {}: {}",
            dir.display(),
            manifest.errors.join("; ")
        );
    }

    if !pseudospore::verify_checksums(&mut manifest) {
        anyhow::bail!(
            "Checksum verification failed for {}: {}",
            dir.display(),
            manifest.errors.join("; ")
        );
    }

    Ok(Envelope::from_manifest(&manifest))
}
