//! Manifest parsing and querying for genome distribution.
//!
//! Handles manifest.toml, checksums.toml, and version lookups.

use axum::{extract::Path, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use tracing::{error, info, warn};

use super::discovery;
use super::error::DistError;

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
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(DistError {
                error: "Genome distribution not configured".to_string(),
                code: "GENOMEBIN_NOT_FOUND".to_string(),
            }),
        )
    })?;

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

    Ok(Json(DistManifest {
        version: manifest_version,
        generated,
        primals,
        atomics,
    }))
}

/// Get latest version for a primal
pub async fn get_latest(
    Path(primal): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<DistError>)> {
    info!("📦 Getting latest version for: {}", primal);

    let manifest = get_manifest().await?;

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
                error: format!("Primal not found: {}", primal),
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
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(DistError {
                error: "Genome distribution not configured".to_string(),
                code: "GENOMEBIN_NOT_FOUND".to_string(),
            }),
        )
    })?;

    // Read checksums.toml
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
                    error: format!("Checksum not found: {}/{}/{}", primal, version, arch),
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
        .and_then(|v| v.as_integer())
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
mod tests {
    use super::*;
    use crate::handlers::genome_dist::discovery::GENOMEBIN_PATH_LOCK;
    use axum::extract::Path;

    #[test]
    fn test_dist_manifest_serialization() {
        let manifest = DistManifest {
            version: "1.0.0".to_string(),
            generated: "2026-02-12".to_string(),
            primals: HashMap::new(),
            atomics: HashMap::new(),
        };
        let json = serde_json::to_string(&manifest).expect("serialize");
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_dist_manifest_with_primals_and_atomics() {
        let mut primals = HashMap::new();
        primals.insert(
            "beardog".to_string(),
            PrimalInfo {
                name: "BearDog".to_string(),
                description: "Security".to_string(),
                latest: "0.9.0".to_string(),
                versions: vec!["0.9.0".to_string()],
                architectures: vec!["x86_64-linux-musl".to_string()],
                capabilities: vec!["crypto".to_string()],
                ecobin_grade: "A++".to_string(),
            },
        );
        let mut atomics = HashMap::new();
        atomics.insert(
            "full-stack".to_string(),
            AtomicInfo {
                name: "Full Stack".to_string(),
                description: "Complete bundle".to_string(),
                primals: vec!["beardog".to_string()],
                latest: "1.0.0".to_string(),
                versions: vec!["1.0.0".to_string()],
            },
        );
        let manifest = DistManifest {
            version: "2.0.0".to_string(),
            generated: "2026-03-11".to_string(),
            primals,
            atomics,
        };
        let json = serde_json::to_string(&manifest).expect("serialize");
        assert!(json.contains("BearDog"));
        assert!(json.contains("full-stack"));
        let deserialized: DistManifest =
            serde_json::from_str(&json).expect("round-trip deserialize");
        assert_eq!(deserialized.version, "2.0.0");
        assert_eq!(deserialized.primals.len(), 1);
        assert_eq!(deserialized.atomics.len(), 1);
    }

    #[test]
    fn test_primal_info_serialization() {
        let info = PrimalInfo {
            name: "BearDog".to_string(),
            description: "Security primal".to_string(),
            latest: "0.9.0".to_string(),
            versions: vec!["0.9.0".to_string()],
            architectures: vec!["x86_64-linux-musl".to_string()],
            capabilities: vec!["crypto".to_string()],
            ecobin_grade: "A++".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("BearDog"));
        assert!(json.contains("A++"));
    }

    #[test]
    fn test_atomic_info_serialization() {
        let info = AtomicInfo {
            name: "Full Stack".to_string(),
            description: "All primals".to_string(),
            primals: vec!["beardog".to_string(), "songbird".to_string()],
            latest: "1.0.0".to_string(),
            versions: vec!["0.9.0".to_string(), "1.0.0".to_string()],
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("Full Stack"));
        assert!(json.contains("beardog"));
        let deserialized: AtomicInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.primals.len(), 2);
    }

    #[test]
    fn test_checksum_response_serialization() {
        let resp = ChecksumResponse {
            primal: "beardog".to_string(),
            version: "0.9.0".to_string(),
            arch: "x86_64-linux-musl".to_string(),
            sha256: "abc123".to_string(),
            size: 1234567,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("abc123"));
        assert!(json.contains("1234567"));
    }

    #[test]
    fn test_dist_error_serialization() {
        let err = DistError {
            error: "Genome distribution not configured".to_string(),
            code: "GENOMEBIN_NOT_FOUND".to_string(),
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(json.contains("GENOMEBIN_NOT_FOUND"));
        assert!(json.contains("Genome distribution not configured"));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_manifest_success() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        let manifest_content = r#"
[manifest]
version = "2.0.0"
generated = "2026-03-11"

[primals.beardog]
name = "BearDog"
description = "Security primal"
latest = "0.9.0"
versions = ["0.9.0", "0.8.0"]
architectures = ["x86_64-linux-musl"]
capabilities = ["crypto"]
ecobin_grade = "A++"

[atomics.full]
name = "Full Stack"
primals = ["beardog"]
latest = "1.0.0"
versions = ["1.0.0"]
"#;
        std::fs::write(temp.path().join("manifest.toml"), manifest_content)
            .expect("write manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_manifest().await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let json = result.expect("get_manifest should succeed");
        assert_eq!(json.version, "2.0.0");
        assert_eq!(json.generated, "2026-03-11");
        assert!(json.primals.contains_key("beardog"));
        assert_eq!(json.primals["beardog"].name, "BearDog");
        assert_eq!(json.primals["beardog"].latest, "0.9.0");
        assert!(json.atomics.contains_key("full"));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_manifest_manifest_file_missing() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        // Dir exists but manifest.toml is missing -> MANIFEST_READ_ERROR
        let temp = tempfile::tempdir().expect("create temp dir");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_manifest().await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let Err((status, body)) = result else {
            panic!("expected Err when manifest.toml missing");
        };
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.code, "MANIFEST_READ_ERROR");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_manifest_parse_error() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(temp.path().join("manifest.toml"), "invalid toml {{{")
            .expect("write bad manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_manifest().await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let Err((status, body)) = result else {
            panic!("expected Err for invalid TOML");
        };
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.code, "MANIFEST_PARSE_ERROR");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_manifest_partial_primal_fields() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        let manifest_content = r#"
[manifest]
version = "1.0"

[primals.minimal]
# No name, description, etc. - should use defaults
latest = "0.1.0"
"#;
        std::fs::write(temp.path().join("manifest.toml"), manifest_content)
            .expect("write manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_manifest().await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let json = result.expect("should succeed with partial fields");
        assert!(json.primals.contains_key("minimal"));
        let p = &json.primals["minimal"];
        assert_eq!(p.name, "minimal");
        assert_eq!(p.description, "");
        assert_eq!(p.latest, "0.1.0");
        assert!(p.versions.is_empty());
        assert!(p.architectures.is_empty());
        assert_eq!(p.ecobin_grade, "unknown");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_latest_success() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        let manifest_content = r#"
[manifest]
version = "1.0"
generated = "2026"

[primals.beardog]
name = "BearDog"
latest = "0.9.0"
versions = ["0.9.0"]
architectures = ["x86_64-linux-musl"]
"#;
        std::fs::write(temp.path().join("manifest.toml"), manifest_content)
            .expect("write manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_latest(Path("beardog".to_string())).await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let json = result.expect("get_latest should succeed");
        assert_eq!(json["primal"], "beardog");
        assert_eq!(json["latest"], "0.9.0");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_latest_primal_not_found() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_latest(Path("nonexistent-primal".to_string())).await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let Err((status, body)) = result else {
            panic!("expected Err for unknown primal");
        };
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.code, "PRIMAL_NOT_FOUND");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_checksum_success() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let checksums_content = r#"
[beardog]
[beardog."0.9.0"]
[beardog."0.9.0"."x86_64-linux-musl"]
sha256 = "deadbeef123"
size = 999
"#;
        std::fs::write(temp.path().join("checksums.toml"), checksums_content)
            .expect("write checksums");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_checksum(Path((
            "beardog".to_string(),
            "0.9.0".to_string(),
            "x86_64-linux-musl".to_string(),
        )))
        .await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let json = result.expect("get_checksum should succeed");
        assert_eq!(json.primal, "beardog");
        assert_eq!(json.version, "0.9.0");
        assert_eq!(json.arch, "x86_64-linux-musl");
        assert_eq!(json.sha256, "deadbeef123");
        assert_eq!(json.size, 999);
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_checksum_not_found() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        std::fs::write(
            temp.path().join("checksums.toml"),
            "[other]\nversion = \"1.0\"",
        )
        .expect("write checksums");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_checksum(Path((
            "beardog".to_string(),
            "0.9.0".to_string(),
            "x86_64-linux-musl".to_string(),
        )))
        .await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let Err((status, body)) = result else {
            panic!("expected Err for missing checksum");
        };
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.code, "CHECKSUM_NOT_FOUND");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_get_checksum_checksums_file_missing() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        // Dir exists with manifest but checksums.toml missing -> CHECKSUMS_READ_ERROR
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_checksum(Path((
            "beardog".to_string(),
            "0.9.0".to_string(),
            "x86_64".to_string(),
        )))
        .await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let Err((status, body)) = result else {
            panic!("expected Err when checksums.toml missing");
        };
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.code, "CHECKSUMS_READ_ERROR");
    }
}
