// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Genome Factory API Handlers
//!
//! REST/WebSocket endpoints for genomeBin operations.
//! Implements XDG-compliant file storage for genomeBins.
//!
//! AGPL-3.0-only License

mod build;
mod retrieval;
mod state;
mod types;
mod validation;

pub use build::{build_genome, compose_genome, create_genome, self_replicate};
pub use retrieval::{download_genome, get_genome_info, list_genomes};
pub use state::GenomeState;
pub use validation::{verify_genome, verify_genome_file};

#[cfg(test)]
mod tests;
