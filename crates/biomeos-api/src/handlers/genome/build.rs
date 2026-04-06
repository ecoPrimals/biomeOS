// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Genome build handlers

use axum::{http::StatusCode, response::Json};
use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeBinBuilder, GenomeManifest};
use tracing::error;

use super::state::genome_state;
use super::types::{
    BuildRequest, BuildResponse, ComposeRequest, ComposeResponse, CreateGenomeRequest,
    CreateGenomeResponse, SelfReplicateResponse,
};

/// Build a new genomeBin
pub async fn build_genome(
    Json(req): Json<BuildRequest>,
) -> Result<Json<BuildResponse>, StatusCode> {
    tracing::info!("Building genomeBin: {}", req.name);

    let mut builder = GenomeBinBuilder::new(&req.name);

    if let Some(version) = &req.version {
        builder = builder.version(version.clone());
    }

    if let Some(description) = &req.description {
        builder = builder.description(description.clone());
    }

    for spec in req.binaries {
        let Ok(arch) = parse_arch_for_build(&spec.arch) else {
            error!("Invalid architecture: {}", spec.arch);
            return Err(StatusCode::BAD_REQUEST);
        };

        if !spec.path.exists() {
            error!("Binary not found: {}", spec.path.display());
            return Err(StatusCode::NOT_FOUND);
        }

        builder = builder.add_binary(arch, spec.path);
    }

    match builder.build() {
        Ok(genome) => {
            let genome_id = format!("{}-{}", genome.manifest.name, genome.manifest.version);

            if let Err(e) = genome_state().save_genome(&genome_id, &genome).await {
                error!("Failed to save genome: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            tracing::info!("✅ Built and saved genomeBin: {}", genome_id);

            Ok(Json(BuildResponse {
                success: true,
                genome_id,
                message: format!(
                    "Built genomeBin with {} architectures",
                    genome.binaries.len()
                ),
            }))
        }
        Err(e) => {
            error!("Failed to build genomeBin: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new genome
pub async fn create_genome(
    Json(req): Json<CreateGenomeRequest>,
) -> Result<Json<CreateGenomeResponse>, StatusCode> {
    tracing::info!("Creating genome: {}", req.name);

    let manifest = GenomeManifest::new(&req.name)
        .version(req.version.unwrap_or_else(|| "0.1.0".to_string()))
        .description(req.description.unwrap_or_default());

    let genome = GenomeBin::with_manifest(manifest);
    let genome_id = format!("{}-{}", genome.manifest.name, genome.manifest.version);

    if let Err(e) = genome_state().save_genome(&genome_id, &genome).await {
        error!("Failed to save genome: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(CreateGenomeResponse {
        success: true,
        genome_id: genome_id.clone(),
        message: format!("Created genome: {genome_id}"),
    }))
}

/// Compose genomes into atomic
pub async fn compose_genome(
    Json(req): Json<ComposeRequest>,
) -> Result<Json<ComposeResponse>, StatusCode> {
    tracing::info!(
        "Composing {} atomic from {} genomes",
        req.nucleus_type,
        req.genomes.len()
    );

    let mut source_genomes = Vec::new();
    for genome_id in &req.genomes {
        match genome_state().load_genome(genome_id).await {
            Ok(genome) => source_genomes.push(genome),
            Err(e) => {
                error!("Failed to load genome {}: {}", genome_id, e);
                return Err(StatusCode::NOT_FOUND);
            }
        }
    }

    let manifest = GenomeManifest::new(&req.name)
        .version("1.0.0")
        .description(format!(
            "{} atomic composed from {} genomes",
            req.nucleus_type,
            source_genomes.len()
        ))
        .nucleus_atomic(&req.nucleus_type);

    let mut composed = GenomeBin::with_manifest(manifest);

    let mut embedded_count = 0;
    for source in &source_genomes {
        for (arch, binary) in &source.binaries {
            if !composed.binaries.contains_key(arch) {
                composed.add_binary_bytes(*arch, &binary.data);
                embedded_count += 1;
            }
        }
    }

    let genome_id = format!("{}-composed", req.name);

    if let Err(e) = genome_state().save_genome(&genome_id, &composed).await {
        error!("Failed to save composed genome: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(ComposeResponse {
        success: true,
        genome_id: genome_id.clone(),
        embedded_count,
        message: format!(
            "Composed {} binaries from {} genomes",
            embedded_count,
            req.genomes.len()
        ),
    }))
}

/// Self-replicate biomeOS
pub async fn self_replicate() -> Result<Json<SelfReplicateResponse>, StatusCode> {
    tracing::info!("Self-replication initiated");

    let self_binary = std::env::current_exe().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let arch = Arch::detect();
    let binary_data = std::fs::read(&self_binary).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let manifest = GenomeManifest::new("biomeos")
        .version(env!("CARGO_PKG_VERSION"))
        .description("biomeOS System Orchestrator - Self-Replicated")
        .nucleus_atomic("ORCHESTRATOR")
        .add_capability("orchestration")
        .add_capability("self-replication");

    let mut genome = GenomeBin::with_manifest(manifest);
    genome.add_binary_bytes(arch, &binary_data);

    let genome_id = "biomeos-self".to_string();
    let size = binary_data.len() as u64;

    if let Err(e) = genome_state().save_genome(&genome_id, &genome).await {
        error!("Failed to save self-replicated genome: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(SelfReplicateResponse {
        success: true,
        genome_id: genome_id.clone(),
        size,
        message: format!("Self-replicated biomeOS: {size} bytes"),
    }))
}

/// Parse architecture string for genome build (testable pure function)
pub fn parse_arch_for_build(arch: &str) -> Result<Arch, &'static str> {
    match arch {
        "x86_64" => Ok(Arch::X86_64),
        "aarch64" => Ok(Arch::Aarch64),
        "arm" => Ok(Arch::Arm),
        "riscv64" => Ok(Arch::Riscv64),
        _ => Err("Invalid architecture"),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_arch_for_build_valid() {
        assert!(matches!(
            parse_arch_for_build("x86_64").unwrap(),
            Arch::X86_64
        ));
        assert!(matches!(
            parse_arch_for_build("aarch64").unwrap(),
            Arch::Aarch64
        ));
        assert!(matches!(parse_arch_for_build("arm").unwrap(), Arch::Arm));
        assert!(matches!(
            parse_arch_for_build("riscv64").unwrap(),
            Arch::Riscv64
        ));
    }

    #[test]
    fn test_parse_arch_for_build_invalid() {
        assert!(parse_arch_for_build("invalid").is_err());
        assert!(parse_arch_for_build("").is_err());
        assert!(parse_arch_for_build("x86").is_err());
        assert!(parse_arch_for_build("amd64").is_err());
        assert!(parse_arch_for_build("aarch32").is_err());
    }

    #[test]
    fn test_parse_arch_for_build_case_sensitive() {
        // Build handler expects lowercase
        assert!(parse_arch_for_build("X86_64").is_err());
        assert!(parse_arch_for_build("AArch64").is_err());
    }

    #[test]
    fn test_build_request_deserialization() {
        use crate::handlers::genome::types::BuildRequest;

        let json = r#"{"name":"test-genome","version":"1.0","binaries":[{"arch":"x86_64","path":"/tmp/binary"}]}"#;
        let req: BuildRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "test-genome");
        assert_eq!(req.version.as_deref(), Some("1.0"));
        assert_eq!(req.binaries.len(), 1);
        assert_eq!(req.binaries[0].arch, "x86_64");
    }

    #[test]
    fn test_build_response_serialization() {
        use crate::handlers::genome::types::BuildResponse;

        let resp = BuildResponse {
            success: true,
            genome_id: "test-1.0".to_string(),
            message: "Built".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("test-1.0"));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_compose_request_deserialization() {
        use crate::handlers::genome::types::ComposeRequest;

        let json = r#"{"name":"composed","nucleus_type":"ORCHESTRATOR","genomes":["g1","g2"]}"#;
        let req: ComposeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "composed");
        assert_eq!(req.nucleus_type, "ORCHESTRATOR");
        assert_eq!(req.genomes.len(), 2);
    }

    #[test]
    fn test_create_genome_request_optional_fields() {
        use crate::handlers::genome::types::CreateGenomeRequest;

        let json = r#"{"name":"minimal"}"#;
        let req: CreateGenomeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "minimal");
        assert!(req.version.is_none());
        assert!(req.description.is_none());
    }
}
