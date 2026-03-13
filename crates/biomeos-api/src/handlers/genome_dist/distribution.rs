// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Genome binary distribution and download logic.
//!
//! Handles binary downloads and LiveSpore updates.

use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio_util::io::ReaderStream;
use tracing::{error, info, warn};

use super::discovery;
use super::error::DistError;
use super::manifest;

/// Update LiveSpore with all genomes
///
/// Copies all genome binaries to a target LiveSpore directory.
/// Used for preparing USB deployment media.
#[derive(Debug, Deserialize)]
pub struct UpdateLiveSporeRequest {
    /// Target LiveSpore directory
    pub target_path: PathBuf,
    /// Architectures to include (default: all)
    pub architectures: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateLiveSporeResponse {
    /// Success status
    pub success: bool,
    /// Number of binaries copied
    pub binaries_copied: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Per-architecture summary
    pub by_arch: HashMap<String, ArchSummary>,
    /// Message
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ArchSummary {
    /// Number of binaries
    pub count: usize,
    /// Total size
    pub size: u64,
    /// Primals included
    pub primals: Vec<String>,
}

/// Download a genome binary
///
/// PROTECTED endpoint - requires Dark Forest token for lineage verification.
/// The Dark Forest gate middleware handles authentication before this handler.
pub async fn download_binary(
    Path((primal, version, arch)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, Json<DistError>)> {
    info!("📥 Download request: {}/{}/{}", primal, version, arch);

    let genome_bin = discovery::get_genome_bin_path().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(DistError {
                error: "Genome distribution not configured".to_string(),
                code: "GENOMEBIN_NOT_FOUND".to_string(),
            }),
        )
    })?;

    // Resolve "latest" to actual version
    let actual_version = if version == "latest" {
        let manifest = manifest::get_manifest().await?;
        manifest
            .primals
            .get(&primal)
            .map(|p| p.latest.clone())
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(DistError {
                        error: format!("Primal not found: {}", primal),
                        code: "PRIMAL_NOT_FOUND".to_string(),
                    }),
                )
            })?
    } else {
        version.clone()
    };

    // Build binary path: primals/{primal}/v{version}/{primal}-{arch}
    let binary_filename = format!("{}-{}", primal, arch);
    let binary_path = genome_bin
        .join("primals")
        .join(&primal)
        .join(format!("v{}", actual_version))
        .join(&binary_filename);

    if !binary_path.exists() {
        warn!("Binary not found: {}", binary_path.display());
        return Err((
            StatusCode::NOT_FOUND,
            Json(DistError {
                error: format!("Binary not found: {}/{}/{}", primal, actual_version, arch),
                code: "BINARY_NOT_FOUND".to_string(),
            }),
        ));
    }

    // Get file metadata
    let metadata = fs::metadata(&binary_path).await.map_err(|e| {
        error!("Failed to get file metadata: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DistError {
                error: "Failed to access binary".to_string(),
                code: "FILE_ACCESS_ERROR".to_string(),
            }),
        )
    })?;

    // Open file for streaming
    let file = fs::File::open(&binary_path).await.map_err(|e| {
        error!("Failed to open binary: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DistError {
                error: "Failed to open binary".to_string(),
                code: "FILE_OPEN_ERROR".to_string(),
            }),
        )
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    info!(
        "📤 Serving binary: {} ({} bytes)",
        binary_filename,
        metadata.len()
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", binary_filename),
        )
        .header(header::CONTENT_LENGTH, metadata.len())
        .header("X-Primal-Name", &primal)
        .header("X-Primal-Version", &actual_version)
        .header("X-Primal-Arch", &arch)
        .body(body)
        .unwrap())
}

/// Update a LiveSpore with all genomes from genomeBin
pub async fn update_livespore(
    Json(req): Json<UpdateLiveSporeRequest>,
) -> Result<Json<UpdateLiveSporeResponse>, (StatusCode, Json<DistError>)> {
    info!("🔄 Updating LiveSpore at: {}", req.target_path.display());

    let genome_bin = discovery::get_genome_bin_path().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(DistError {
                error: "Genome distribution not configured".to_string(),
                code: "GENOMEBIN_NOT_FOUND".to_string(),
            }),
        )
    })?;

    // Create target directories
    let target_primals = req.target_path.join("primals");
    fs::create_dir_all(&target_primals).await.map_err(|e| {
        error!("Failed to create target directory: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DistError {
                error: "Failed to create target directory".to_string(),
                code: "DIR_CREATE_ERROR".to_string(),
            }),
        )
    })?;

    // Get manifest for primal list
    let manifest = manifest::get_manifest().await?;

    // Determine architectures to copy
    let arches: Vec<String> = req.architectures.unwrap_or_else(|| {
        vec![
            "x86_64-linux-musl".to_string(),
            "aarch64-linux-musl".to_string(),
        ]
    });

    let mut binaries_copied = 0usize;
    let mut total_size = 0u64;
    let mut by_arch: HashMap<String, ArchSummary> = HashMap::new();

    for arch in &arches {
        // Create arch-specific directory
        let arch_short = arch.split('-').next().unwrap_or(arch);
        let arch_dir = target_primals.join(arch_short);
        fs::create_dir_all(&arch_dir).await.ok();

        let mut arch_summary = ArchSummary {
            count: 0,
            size: 0,
            primals: vec![],
        };

        for (primal_name, primal_info) in &manifest.primals {
            let version = &primal_info.latest;
            let binary_filename = format!("{}-{}", primal_name, arch);
            let source_path = genome_bin
                .join("primals")
                .join(primal_name)
                .join(format!("v{}", version))
                .join(&binary_filename);

            if source_path.exists() {
                let target_path = arch_dir.join(primal_name);

                // Copy binary
                match fs::copy(&source_path, &target_path).await {
                    Ok(size) => {
                        info!(
                            "  ✅ Copied: {} -> {} ({} bytes)",
                            binary_filename,
                            target_path.display(),
                            size
                        );
                        binaries_copied += 1;
                        total_size += size;
                        arch_summary.count += 1;
                        arch_summary.size += size;
                        arch_summary.primals.push(primal_name.clone());
                    }
                    Err(e) => {
                        warn!("  ⚠️ Failed to copy {}: {}", binary_filename, e);
                    }
                }
            }
        }

        by_arch.insert(arch_short.to_string(), arch_summary);
    }

    // Copy manifest and checksums
    let manifest_src = genome_bin.join("manifest.toml");
    let checksums_src = genome_bin.join("checksums.toml");

    if manifest_src.exists() {
        fs::copy(&manifest_src, req.target_path.join("manifest.toml"))
            .await
            .ok();
    }
    if checksums_src.exists() {
        fs::copy(&checksums_src, req.target_path.join("checksums.toml"))
            .await
            .ok();
    }

    info!(
        "🎉 LiveSpore updated: {} binaries, {} bytes",
        binaries_copied, total_size
    );

    Ok(Json(UpdateLiveSporeResponse {
        success: true,
        binaries_copied,
        total_size,
        by_arch,
        message: format!(
            "Updated LiveSpore with {} binaries ({} MB)",
            binaries_copied,
            total_size / 1024 / 1024
        ),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::genome_dist::discovery::GENOMEBIN_PATH_LOCK;
    use std::io::Write;

    #[test]
    fn test_update_livespore_request_deserialize() {
        let json = r#"{"target_path": "/media/usb/livespore"}"#;
        let req: UpdateLiveSporeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.target_path, PathBuf::from("/media/usb/livespore"));
        assert!(req.architectures.is_none());
    }

    #[test]
    fn test_update_livespore_request_with_arches() {
        let json = r#"{
            "target_path": "/media/usb",
            "architectures": ["x86_64-linux-musl", "aarch64-linux-musl"]
        }"#;
        let req: UpdateLiveSporeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.architectures.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_update_livespore_response_serialization() {
        let mut by_arch = HashMap::new();
        by_arch.insert(
            "x86_64".to_string(),
            ArchSummary {
                count: 2,
                size: 1024,
                primals: vec!["beardog".to_string(), "songbird".to_string()],
            },
        );
        let resp = UpdateLiveSporeResponse {
            success: true,
            binaries_copied: 2,
            total_size: 2048,
            by_arch,
            message: "Updated".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"binaries_copied\":2"));
        assert!(json.contains("x86_64"));
    }

    #[test]
    fn test_arch_summary_serialization() {
        let summary = ArchSummary {
            count: 1,
            size: 512,
            primals: vec!["beardog".to_string()],
        };
        let json = serde_json::to_string(&summary).expect("serialize");
        assert!(json.contains("\"count\":1"));
        assert!(json.contains("\"size\":512"));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_download_binary_file_not_found() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = download_binary(Path((
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
            panic!("expected Err when binary missing");
        };
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.code, "BINARY_NOT_FOUND");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_download_binary_success() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let primal_dir = temp.path().join("primals").join("beardog").join("v0.9.0");
        std::fs::create_dir_all(&primal_dir).expect("create dir");
        let binary_path = primal_dir.join("beardog-x86_64-linux-musl");
        let mut f = std::fs::File::create(&binary_path).expect("create binary");
        f.write_all(b"fake binary content").expect("write");
        drop(f);
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = download_binary(Path((
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
        let response = result.expect("download should succeed");
        assert_eq!(response.status(), StatusCode::OK);
        let ct = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(ct, "application/octet-stream");
        let cd = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(cd.contains("beardog-x86_64-linux-musl"));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_update_livespore_success() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        std::fs::write(temp.path().join("checksums.toml"), "[checksums]\n")
            .expect("write checksums");
        let target = tempfile::tempdir().expect("target temp dir");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = update_livespore(Json(UpdateLiveSporeRequest {
            target_path: target.path().to_path_buf(),
            architectures: Some(vec![
                "x86_64-linux-musl".to_string(),
                "aarch64-linux-musl".to_string(),
            ]),
        }))
        .await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let json = result.expect("update_livespore should succeed");
        assert!(json.success);
        assert!(json.message.contains("Updated"));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_download_binary_latest_resolution() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        // Test that "latest" version resolves from manifest
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
        let primal_dir = temp.path().join("primals").join("beardog").join("v0.9.0");
        std::fs::create_dir_all(&primal_dir).expect("create dir");
        std::fs::write(primal_dir.join("beardog-x86_64-linux-musl"), b"binary")
            .expect("write binary");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = download_binary(Path((
            "beardog".to_string(),
            "latest".to_string(),
            "x86_64-linux-musl".to_string(),
        )))
        .await;
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        let response = result.expect("download with latest should succeed");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
