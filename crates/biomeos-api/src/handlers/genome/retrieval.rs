// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Genome retrieval handlers

use axum::{extract::Path, http::StatusCode, response::Json};
use tracing::{error, info};

use super::state::genome_state;
use super::types::{DownloadResponse, GenomeInfoResponse, GenomeSummary, ListGenomesResponse};

/// Get genomeBin info
pub async fn get_genome_info(
    Path(id): Path<String>,
) -> Result<Json<GenomeInfoResponse>, StatusCode> {
    info!("Getting genome info: {}", id);

    match genome_state().load_genome(&id).await {
        Ok(genome) => {
            let archs: Vec<String> = genome
                .binaries
                .keys()
                .map(|a| format!("{a:?}").to_lowercase())
                .collect();

            Ok(Json(GenomeInfoResponse {
                name: genome.manifest.name.clone(),
                version: genome.manifest.version,
                architectures: archs,
            }))
        }
        Err(e) => {
            error!("Genome not found: {} - {}", id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Download genomeBin
pub async fn download_genome(Path(id): Path<String>) -> Result<Json<DownloadResponse>, StatusCode> {
    info!("Download request for genome: {}", id);

    let path = genome_state().genome_path(&id);
    if !path.exists() {
        error!("Genome not found: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }

    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    Ok(Json(DownloadResponse {
        url: format!("/api/v1/genome/{id}/data"),
        size,
    }))
}

/// List all genomes
pub async fn list_genomes() -> Result<Json<ListGenomesResponse>, StatusCode> {
    info!("Listing all genomes");

    match genome_state().list_all() {
        Ok(genomes) => {
            let summaries: Vec<GenomeSummary> = genomes
                .iter()
                .map(|(id, genome)| {
                    let archs: Vec<String> = genome
                        .binaries
                        .keys()
                        .map(|a| format!("{a:?}").to_lowercase())
                        .collect();
                    GenomeSummary {
                        id: id.clone(),
                        name: genome.manifest.name.clone(),
                        version: genome.manifest.version.clone(),
                        architectures: archs,
                    }
                })
                .collect();

            Ok(Json(ListGenomesResponse { genomes: summaries }))
        }
        Err(e) => {
            error!("Failed to list genomes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
