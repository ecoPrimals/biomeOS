// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Genome Distribution API Handlers
//!
//! REST endpoints for genome binary distribution via nestgate.io.
//! Serves binaries from the centralized `wateringHole/genomeBin/` repository.
//!
//! # Security
//!
//! - Public: manifest, version info, checksums
//! - Dark Forest: binary downloads (lineage verification required)
//!
//! AGPL-3.0-only License

mod discovery;
mod distribution;
mod error;
mod manifest;

// Re-export all public items for external consumers (preserve public API)
#[expect(
    unused_imports,
    reason = "re-exports for public API surface; consumed by external crates"
)]
pub use {
    discovery::get_genome_bin_path,
    distribution::{
        ArchSummary, UpdateLiveSporeRequest, UpdateLiveSporeResponse, download_binary,
        update_livespore,
    },
    error::DistError,
    manifest::{
        AtomicInfo, ChecksumResponse, DistManifest, PrimalInfo, get_checksum, get_latest,
        get_manifest,
    },
};
