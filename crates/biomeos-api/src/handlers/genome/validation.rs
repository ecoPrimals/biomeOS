// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Genome validation handlers

use axum::{extract::Path, http::StatusCode, response::Json};
use biomeos_genomebin_v3::GenomeBin;
use tracing::{error, info};

use super::state::genome_state;
use super::types::{VerifyRequest, VerifyResponse};

/// Verify genomeBin integrity by ID
pub async fn verify_genome(Path(id): Path<String>) -> Result<Json<VerifyResponse>, StatusCode> {
    info!("Verifying genome: {}", id);

    match genome_state().load_genome(&id).await {
        Ok(genome) => match genome.is_valid() {
            Ok(valid) => {
                if valid {
                    info!("✅ Genome verified: {}", id);
                    Ok(Json(VerifyResponse {
                        valid: true,
                        message: "All checksums valid".to_string(),
                    }))
                } else {
                    error!("❌ Genome verification failed: {}", id);
                    Ok(Json(VerifyResponse {
                        valid: false,
                        message: "Checksum verification failed".to_string(),
                    }))
                }
            }
            Err(e) => {
                error!("Verification error: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            error!("Genome not found: {} - {}", id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Verify genomeBin integrity from file (legacy)
pub async fn verify_genome_file(
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, StatusCode> {
    info!("Verifying genomeBin: {}", req.path.display());

    if !req.path.exists() {
        error!("GenomeBin not found: {}", req.path.display());
        return Err(StatusCode::NOT_FOUND);
    }

    match GenomeBin::load(&req.path) {
        Ok(genome) => match genome.is_valid() {
            Ok(valid) => {
                if valid {
                    info!("✅ GenomeBin verified: {}", req.path.display());
                    Ok(Json(VerifyResponse {
                        valid: true,
                        message: "All checksums valid".to_string(),
                    }))
                } else {
                    error!("❌ GenomeBin verification failed: {}", req.path.display());
                    Ok(Json(VerifyResponse {
                        valid: false,
                        message: "Checksum verification failed".to_string(),
                    }))
                }
            }
            Err(e) => {
                error!("Verification error: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            error!("Failed to load genomeBin: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}
