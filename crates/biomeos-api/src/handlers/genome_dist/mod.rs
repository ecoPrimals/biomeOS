// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
#[allow(unused_imports)]
pub use {
    discovery::get_genome_bin_path,
    distribution::{
        download_binary, update_livespore, ArchSummary, UpdateLiveSporeRequest,
        UpdateLiveSporeResponse,
    },
    error::DistError,
    manifest::{
        get_checksum, get_latest, get_manifest, AtomicInfo, ChecksumResponse, DistManifest,
        PrimalInfo,
    },
};
