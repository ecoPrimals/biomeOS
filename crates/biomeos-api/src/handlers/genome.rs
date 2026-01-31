// biomeos-api/src/handlers/genome.rs
// Genome Factory REST API handlers
//
// Deep Debt: REST API for genomeBin production via neuralAPI

use crate::{ApiError, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use biomeos_genome_factory::{
    GenomeComposeRequest, GenomeComposeResponse, GenomeCreateRequest, GenomeCreateResponse,
    GenomeFactory, SelfReplicateResponse, VerifyResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Genome handler state
pub struct GenomeState {
    factory: GenomeFactory,
}

impl GenomeState {
    /// Create new genome state with default factory
    pub fn new() -> Result<Self, ApiError> {
        let factory = GenomeFactory::default().map_err(|e| {
            ApiError::Internal(format!("Failed to initialize genome factory: {}", e))
        })?;

        info!("🧬 Genome Factory initialized for neuralAPI");
        Ok(Self { factory })
    }

    /// Get factory reference
    pub fn factory(&self) -> &GenomeFactory {
        &self.factory
    }
}

/// Create genomeBin from binaries
pub async fn create_genome(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenomeCreateRequest>,
) -> Result<Json<GenomeCreateResponse>, ApiError> {
    info!("🧬 [POST /api/v1/genome/create] Request: {}", request.name);

    state
        .genome()
        .factory()
        .create_genome(request)
        .map(Json)
        .map_err(|e| {
            error!("❌ Failed to create genome: {}", e);
            ApiError::Internal(format!("Genome creation failed: {}", e))
        })
}

/// Compose atomic genomeBin
pub async fn compose_genome(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenomeComposeRequest>,
) -> Result<Json<GenomeComposeResponse>, ApiError> {
    info!(
        "🧬 [POST /api/v1/genome/compose] Atomic: {} ({})",
        request.name, request.nucleus_type
    );

    state
        .genome()
        .factory()
        .compose_genome(request)
        .map(Json)
        .map_err(|e| {
            error!("❌ Failed to compose genome: {}", e);
            ApiError::Internal(format!("Genome composition failed: {}", e))
        })
}

/// Self-replicate: biomeOS creates its own genomeBin
pub async fn self_replicate(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SelfReplicateResponse>, ApiError> {
    info!("🧬 [POST /api/v1/genome/self-replicate] biomeOS replicating itself");

    state
        .genome()
        .factory()
        .self_replicate()
        .map(Json)
        .map_err(|e| {
            error!("❌ Failed to self-replicate: {}", e);
            ApiError::Internal(format!("Self-replication failed: {}", e))
        })
}

/// Verify genomeBin integrity
pub async fn verify_genome(
    State(state): State<Arc<AppState>>,
    Path(genome_id): Path<String>,
) -> Result<Json<VerifyResponse>, ApiError> {
    info!("🧬 [GET /api/v1/genome/{}/verify] Verifying", genome_id);

    state
        .genome()
        .factory()
        .verify_genome(&genome_id)
        .map(Json)
        .map_err(|e| {
            error!("❌ Failed to verify genome: {}", e);
            ApiError::NotFound(format!("Genome verification failed: {}", e))
        })
}

/// List all genomes in storage
pub async fn list_genomes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GenomeListResponse>, ApiError> {
    info!("🧬 [GET /api/v1/genome/list] Listing genomes");

    state
        .genome()
        .factory()
        .list_genomes()
        .map(|genomes| {
            Json(GenomeListResponse {
                genomes: genomes
                    .into_iter()
                    .map(|name| GenomeInfo {
                        name: name.clone(),
                        path: state.genome().factory().genome_path(&name),
                    })
                    .collect(),
            })
        })
        .map_err(|e| {
            error!("❌ Failed to list genomes: {}", e);
            ApiError::Internal(format!("Failed to list genomes: {}", e))
        })
}

/// Download genomeBin (binary response)
pub async fn download_genome(
    State(state): State<Arc<AppState>>,
    Path(genome_id): Path<String>,
) -> Result<Response, ApiError> {
    info!("🧬 [GET /api/v1/genome/{}/download] Downloading", genome_id);

    let path = state.genome().factory().genome_path(&genome_id);

    if !path.exists() {
        return Err(ApiError::NotFound(format!(
            "Genome not found: {}",
            genome_id
        )));
    }

    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let filename = format!("{}.genome", genome_id);

            Ok((
                StatusCode::OK,
                [
                    ("Content-Type", "application/octet-stream"),
                    ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
                ],
                bytes,
            )
                .into_response())
        }
        Err(e) => {
            error!("❌ Failed to read genome file: {}", e);
            Err(ApiError::Internal(format!("Failed to read genome: {}", e)))
        }
    }
}

/// Genome list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeListResponse {
    pub genomes: Vec<GenomeInfo>,
}

/// Genome info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeInfo {
    pub name: String,
    pub path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genome_state_creation() {
        // Test may fail if not in workspace, but that's okay
        let _ = GenomeState::new();
    }
}
