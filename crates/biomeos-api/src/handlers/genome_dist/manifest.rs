// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Manifest parsing and querying for genome distribution.
//!
//! Handles manifest.toml, checksums.toml, and version lookups.

use axum::{Json, extract::Path, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info, warn};

use super::discovery;
use super::error::DistError;

fn genome_bin_not_found_err() -> (StatusCode, Json<DistError>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(DistError {
            error: "Genome distribution not configured".to_string(),
            code: "GENOMEBIN_NOT_FOUND".to_string(),
        }),
    )
}

/// Genome distribution manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistManifest {
    /// Manifest version
    pub version: String,
    /// Generation timestamp
    pub generated: String,
    /// Available primals
    pub primals: HashMap<String, PrimalInfo>,
    /// Available atomics
    pub atomics: HashMap<String, AtomicInfo>,
}

/// Primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Latest version
    pub latest: String,
    /// All available versions
    pub versions: Vec<String>,
    /// Supported architectures
    pub architectures: Vec<String>,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// ecoBin grade
    pub ecobin_grade: String,
}

/// Atomic bundle information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicInfo {
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Included primals
    pub primals: Vec<String>,
    /// Latest version
    pub latest: String,
    /// Available versions
    pub versions: Vec<String>,
}

/// Checksum response
#[derive(Debug, Serialize)]
pub struct ChecksumResponse {
    /// Primal name
    pub primal: String,
    /// Version
    pub version: String,
    /// Architecture
    pub arch: String,
    /// SHA256 checksum
    pub sha256: String,
    /// File size in bytes
    pub size: u64,
}

/// Get the distribution manifest
///
/// Returns the master manifest listing all available primals, versions, and architectures.
/// This endpoint is PUBLIC (no Dark Forest token required).
pub async fn get_manifest() -> Result<Json<DistManifest>, (StatusCode, Json<DistError>)> {
    info!("📦 Genome distribution manifest requested");

    let genome_bin = discovery::get_genome_bin_path().ok_or_else(|| {
        error!("genomeBin path not found");
        genome_bin_not_found_err()
    })?;

    get_manifest_from(genome_bin).await.map(Json)
}

#[expect(clippy::too_many_lines, reason = "manifest parsing and validation")]
pub async fn get_manifest_from(
    genome_bin: impl AsRef<std::path::Path>,
) -> Result<DistManifest, (StatusCode, Json<DistError>)> {
    let genome_bin = genome_bin.as_ref();
    let manifest_path = genome_bin.join("manifest.toml");
    let manifest_content = fs::read_to_string(&manifest_path).await.map_err(|e| {
        error!("Failed to read manifest: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DistError {
                error: "Failed to read manifest".to_string(),
                code: "MANIFEST_READ_ERROR".to_string(),
            }),
        )
    })?;

    // Parse TOML manifest
    let manifest: toml::Value = toml::from_str(&manifest_content).map_err(|e| {
        error!("Failed to parse manifest: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DistError {
                error: "Failed to parse manifest".to_string(),
                code: "MANIFEST_PARSE_ERROR".to_string(),
            }),
        )
    })?;

    // Build response from parsed TOML
    let mut primals = HashMap::new();
    let mut atomics = HashMap::new();

    // Parse primals section
    if let Some(primals_table) = manifest.get("primals").and_then(|v| v.as_table()) {
        for (key, value) in primals_table {
            if let Some(table) = value.as_table() {
                primals.insert(
                    key.clone(),
                    PrimalInfo {
                        name: table
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(key)
                            .to_string(),
                        description: table
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        latest: table
                            .get("latest")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.0.0")
                            .to_string(),
                        versions: table
                            .get("versions")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        architectures: table
                            .get("architectures")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        capabilities: table
                            .get("capabilities")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        ecobin_grade: table
                            .get("ecobin_grade")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    },
                );
            }
        }
    }

    // Parse atomics section
    if let Some(atomics_table) = manifest.get("atomics").and_then(|v| v.as_table()) {
        for (key, value) in atomics_table {
            if let Some(table) = value.as_table() {
                atomics.insert(
                    key.clone(),
                    AtomicInfo {
                        name: table
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(key)
                            .to_string(),
                        description: table
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        primals: table
                            .get("primals")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        latest: table
                            .get("latest")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.0.0")
                            .to_string(),
                        versions: table
                            .get("versions")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    },
                );
            }
        }
    }

    let manifest_version = manifest
        .get("manifest")
        .and_then(|v| v.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();

    let generated = manifest
        .get("manifest")
        .and_then(|v| v.get("generated"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    info!(
        "📦 Serving manifest: {} primals, {} atomics",
        primals.len(),
        atomics.len()
    );

    Ok(DistManifest {
        version: manifest_version,
        generated,
        primals,
        atomics,
    })
}

/// Get latest version for a primal
pub async fn get_latest(
    Path(primal): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<DistError>)> {
    info!("📦 Getting latest version for: {}", primal);

    let genome_bin = discovery::get_genome_bin_path().ok_or_else(|| {
        error!("genomeBin path not found");
        genome_bin_not_found_err()
    })?;

    get_latest_from(genome_bin, primal).await
}

pub async fn get_latest_from(
    genome_bin: impl AsRef<std::path::Path>,
    primal: String,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<DistError>)> {
    let manifest = get_manifest_from(genome_bin).await?;

    if let Some(info) = manifest.primals.get(&primal) {
        Ok(Json(serde_json::json!({
            "primal": primal,
            "latest": info.latest,
            "architectures": info.architectures,
        })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(DistError {
                error: format!("Primal not found: {primal}"),
                code: "PRIMAL_NOT_FOUND".to_string(),
            }),
        ))
    }
}

/// Get checksum for a specific binary
///
/// PUBLIC endpoint - no Dark Forest token required.
pub async fn get_checksum(
    Path((primal, version, arch)): Path<(String, String, String)>,
) -> Result<Json<ChecksumResponse>, (StatusCode, Json<DistError>)> {
    info!("📦 Getting checksum for: {}/{}/{}", primal, version, arch);

    let genome_bin = discovery::get_genome_bin_path().ok_or_else(|| {
        error!("genomeBin path not found");
        genome_bin_not_found_err()
    })?;

    get_checksum_from(genome_bin, primal, version, arch).await
}

pub async fn get_checksum_from(
    genome_bin: PathBuf,
    primal: String,
    version: String,
    arch: String,
) -> Result<Json<ChecksumResponse>, (StatusCode, Json<DistError>)> {
    let checksums_path = genome_bin.join("checksums.toml");
    let checksums_content = fs::read_to_string(&checksums_path).await.map_err(|e| {
        error!("Failed to read checksums: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DistError {
                error: "Failed to read checksums".to_string(),
                code: "CHECKSUMS_READ_ERROR".to_string(),
            }),
        )
    })?;

    let checksums: toml::Value = toml::from_str(&checksums_content).map_err(|e| {
        error!("Failed to parse checksums: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DistError {
                error: "Failed to parse checksums".to_string(),
                code: "CHECKSUMS_PARSE_ERROR".to_string(),
            }),
        )
    })?;

    // Look up checksum: [primal][version][arch]
    let checksum_entry = checksums
        .get(&primal)
        .and_then(|p| p.get(&version))
        .and_then(|v| v.get(&arch))
        .ok_or_else(|| {
            warn!("Checksum not found for: {}/{}/{}", primal, version, arch);
            (
                StatusCode::NOT_FOUND,
                Json(DistError {
                    error: format!("Checksum not found: {primal}/{version}/{arch}"),
                    code: "CHECKSUM_NOT_FOUND".to_string(),
                }),
            )
        })?;

    let sha256 = checksum_entry
        .get("sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let size = checksum_entry
        .get("size")
        .and_then(toml::Value::as_integer)
        .unwrap_or(0) as u64;

    Ok(Json(ChecksumResponse {
        primal,
        version,
        arch,
        sha256,
        size,
    }))
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
