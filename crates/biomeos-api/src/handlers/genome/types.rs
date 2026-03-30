// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Genome API request/response types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Build genomeBin request
#[derive(Debug, Deserialize)]
pub struct BuildRequest {
    /// Genome name
    pub name: String,
    /// Optional version
    pub version: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Binary path and architecture pairs
    pub binaries: Vec<BinarySpec>,
}

/// Binary specification
#[derive(Debug, Deserialize)]
pub struct BinarySpec {
    /// Architecture (`x86_64`, aarch64, arm, riscv64)
    pub arch: String,
    /// Path to binary file
    pub path: PathBuf,
}

/// Build genomeBin response
#[derive(Debug, Serialize)]
pub struct BuildResponse {
    pub success: bool,
    pub genome_id: String,
    pub message: String,
}

/// Verify genomeBin request
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// Path to genomeBin file
    pub path: PathBuf,
}

/// Verify genomeBin response
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub message: String,
}

/// `GenomeBin` info response
#[derive(Debug, Serialize)]
pub struct GenomeInfoResponse {
    pub name: String,
    pub version: String,
    pub architectures: Vec<String>,
}

/// Download response
#[derive(Debug, Serialize)]
pub struct DownloadResponse {
    pub url: String,
    pub size: u64,
}

/// Create genome request
#[derive(Debug, Deserialize)]
pub struct CreateGenomeRequest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Create genome response
#[derive(Debug, Serialize)]
pub struct CreateGenomeResponse {
    pub success: bool,
    pub genome_id: String,
    pub message: String,
}

/// Compose genomes request
#[derive(Debug, Deserialize)]
pub struct ComposeRequest {
    pub name: String,
    pub nucleus_type: String,
    pub genomes: Vec<String>,
}

/// Compose genomes response
#[derive(Debug, Serialize)]
pub struct ComposeResponse {
    pub success: bool,
    pub genome_id: String,
    pub embedded_count: usize,
    pub message: String,
}

/// Self-replicate response
#[derive(Debug, Serialize)]
pub struct SelfReplicateResponse {
    pub success: bool,
    pub genome_id: String,
    pub size: u64,
    pub message: String,
}

/// List genomes response
#[derive(Debug, Serialize)]
pub struct ListGenomesResponse {
    pub genomes: Vec<GenomeSummary>,
}

/// Genome summary
#[derive(Debug, Serialize)]
pub struct GenomeSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub architectures: Vec<String>,
}
