//! Genome Factory API Handlers
//!
//! REST/WebSocket endpoints for genomeBin operations.
//! Implements XDG-compliant file storage for genomeBins.
//!
//! AGPL-3.0-only License

use axum::{extract::Path, http::StatusCode, response::Json};
use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeBinBuilder, GenomeManifest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Genome state for storing built genomes
#[derive(Debug)]
pub struct GenomeState {
    /// In-memory cache of genomes
    genomes: RwLock<HashMap<String, GenomeBin>>,
    /// Storage directory for persistent genomes (XDG-compliant)
    storage_dir: PathBuf,
}

impl Default for GenomeState {
    fn default() -> Self {
        Self {
            genomes: RwLock::new(HashMap::new()),
            storage_dir: Self::default_storage_dir(),
        }
    }
}

impl GenomeState {
    /// Get XDG-compliant default storage directory
    ///
    /// Uses SystemPaths for consistent XDG path resolution across all of biomeOS.
    fn default_storage_dir() -> PathBuf {
        biomeos_types::paths::SystemPaths::new_lazy()
            .data_dir()
            .join("genomes")
    }

    pub fn new() -> Result<Self, String> {
        let storage_dir = Self::default_storage_dir();
        Self::with_storage(storage_dir)
    }

    pub fn with_storage(storage_dir: PathBuf) -> Result<Self, String> {
        // Ensure directory exists
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir)
                .map_err(|e| format!("Failed to create genome storage: {}", e))?;
        }
        Ok(Self {
            genomes: RwLock::new(HashMap::new()),
            storage_dir,
        })
    }

    /// Get path for a genome file
    fn genome_path(&self, id: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.genome", id))
    }

    /// Save genome to persistent storage
    async fn save_genome(&self, id: &str, genome: &GenomeBin) -> Result<(), String> {
        let path = self.genome_path(id);
        genome
            .save(&path)
            .map_err(|e| format!("Failed to save genome: {}", e))?;

        // Also cache in memory
        let mut cache = self.genomes.write().await;
        cache.insert(id.to_string(), genome.clone());

        info!("💾 Saved genome to: {}", path.display());
        Ok(())
    }

    /// Load genome from persistent storage
    async fn load_genome(&self, id: &str) -> Result<GenomeBin, String> {
        // Check cache first
        {
            let cache = self.genomes.read().await;
            if let Some(genome) = cache.get(id) {
                return Ok(genome.clone());
            }
        }

        // Load from disk
        let path = self.genome_path(id);
        if !path.exists() {
            return Err(format!("Genome not found: {}", id));
        }

        let genome = GenomeBin::load(&path).map_err(|e| format!("Failed to load genome: {}", e))?;

        // Cache for future access
        {
            let mut cache = self.genomes.write().await;
            cache.insert(id.to_string(), genome.clone());
        }

        Ok(genome)
    }

    /// List all genomes in storage
    async fn list_all(&self) -> Result<Vec<(String, GenomeBin)>, String> {
        let mut genomes = Vec::new();

        if !self.storage_dir.exists() {
            return Ok(genomes);
        }

        let entries = std::fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read storage dir: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "genome").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    match GenomeBin::load(&path) {
                        Ok(genome) => {
                            genomes.push((stem.to_string(), genome));
                        }
                        Err(e) => {
                            warn!("Failed to load genome {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(genomes)
    }
}

// Thread-safe global state
// DEEP DEBT EVOLUTION: Replaced lazy_static! with std::sync::OnceLock (stable Rust, no external dep)
static GENOME_STATE: std::sync::OnceLock<GenomeState> = std::sync::OnceLock::new();

fn genome_state() -> &'static GenomeState {
    GENOME_STATE.get_or_init(GenomeState::default)
}

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
    /// Architecture (x86_64, aarch64, arm, riscv64)
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

/// Build a new genomeBin
pub async fn build_genome(
    Json(req): Json<BuildRequest>,
) -> Result<Json<BuildResponse>, StatusCode> {
    info!("Building genomeBin: {}", req.name);

    let mut builder = GenomeBinBuilder::new(&req.name);

    if let Some(version) = &req.version {
        builder = builder.version(version.clone());
    }

    if let Some(description) = &req.description {
        builder = builder.description(description.clone());
    }

    // Parse and add binaries
    for spec in req.binaries {
        let arch = match spec.arch.as_str() {
            "x86_64" => Arch::X86_64,
            "aarch64" => Arch::Aarch64,
            "arm" => Arch::Arm,
            "riscv64" => Arch::Riscv64,
            _ => {
                error!("Invalid architecture: {}", spec.arch);
                return Err(StatusCode::BAD_REQUEST);
            }
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

            // Save to persistent storage
            if let Err(e) = genome_state().save_genome(&genome_id, &genome).await {
                error!("Failed to save genome: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            info!("✅ Built and saved genomeBin: {}", genome_id);

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
                .map(|a| format!("{:?}", a).to_lowercase())
                .collect();

            Ok(Json(GenomeInfoResponse {
                name: genome.manifest.name.clone(),
                version: genome.manifest.version.clone(),
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
        url: format!("/api/v1/genome/{}/data", id),
        size,
    }))
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

/// GenomeBin info response
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

// ============================================================================
// Public Handler Functions (used by router)
// ============================================================================

/// Create a new genome
#[derive(Debug, Deserialize)]
pub struct CreateGenomeRequest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateGenomeResponse {
    pub success: bool,
    pub genome_id: String,
    pub message: String,
}

pub async fn create_genome(
    Json(req): Json<CreateGenomeRequest>,
) -> Result<Json<CreateGenomeResponse>, StatusCode> {
    info!("Creating genome: {}", req.name);

    let manifest = GenomeManifest::new(&req.name)
        .version(req.version.unwrap_or_else(|| "0.1.0".to_string()))
        .description(req.description.unwrap_or_default());

    let genome = GenomeBin::with_manifest(manifest);
    let genome_id = format!("{}-{}", genome.manifest.name, genome.manifest.version);

    // Save to persistent storage
    if let Err(e) = genome_state().save_genome(&genome_id, &genome).await {
        error!("Failed to save genome: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(CreateGenomeResponse {
        success: true,
        genome_id: genome_id.clone(),
        message: format!("Created genome: {}", genome_id),
    }))
}

/// Compose genomes into atomic
#[derive(Debug, Deserialize)]
pub struct ComposeRequest {
    pub name: String,
    pub nucleus_type: String,
    pub genomes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ComposeResponse {
    pub success: bool,
    pub genome_id: String,
    pub embedded_count: usize,
    pub message: String,
}

pub async fn compose_genome(
    Json(req): Json<ComposeRequest>,
) -> Result<Json<ComposeResponse>, StatusCode> {
    info!(
        "Composing {} atomic from {} genomes",
        req.nucleus_type,
        req.genomes.len()
    );

    // Load all source genomes
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

    // Create composed genome manifest
    let manifest = GenomeManifest::new(&req.name)
        .version("1.0.0")
        .description(format!(
            "{} atomic composed from {} genomes",
            req.nucleus_type,
            source_genomes.len()
        ))
        .nucleus_atomic(&req.nucleus_type);

    let mut composed = GenomeBin::with_manifest(manifest);

    // Merge binaries from all source genomes
    let mut embedded_count = 0;
    for source in &source_genomes {
        for (arch, binary) in &source.binaries {
            // Only add if not already present
            if !composed.binaries.contains_key(arch) {
                composed.add_binary_bytes(*arch, &binary.data);
                embedded_count += 1;
            }
        }
    }

    let genome_id = format!("{}-composed", req.name);

    // Save composed genome
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
#[derive(Debug, Serialize)]
pub struct SelfReplicateResponse {
    pub success: bool,
    pub genome_id: String,
    pub size: u64,
    pub message: String,
}

pub async fn self_replicate() -> Result<Json<SelfReplicateResponse>, StatusCode> {
    info!("Self-replication initiated");

    // Get current executable
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

    // Save to persistent storage
    if let Err(e) = genome_state().save_genome(&genome_id, &genome).await {
        error!("Failed to save self-replicated genome: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(SelfReplicateResponse {
        success: true,
        genome_id: genome_id.clone(),
        size,
        message: format!("Self-replicated biomeOS: {} bytes", size),
    }))
}

/// List all genomes
#[derive(Debug, Serialize)]
pub struct ListGenomesResponse {
    pub genomes: Vec<GenomeSummary>,
}

#[derive(Debug, Serialize)]
pub struct GenomeSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub architectures: Vec<String>,
}

pub async fn list_genomes() -> Result<Json<ListGenomesResponse>, StatusCode> {
    info!("Listing all genomes");

    match genome_state().list_all().await {
        Ok(genomes) => {
            let summaries: Vec<GenomeSummary> = genomes
                .iter()
                .map(|(id, genome)| {
                    let archs: Vec<String> = genome
                        .binaries
                        .keys()
                        .map(|a| format!("{:?}", a).to_lowercase())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== GenomeState Tests ==========

    #[test]
    fn test_genome_state_default_storage_dir() {
        let dir = GenomeState::default_storage_dir();
        assert!(dir.to_string_lossy().contains("biomeos/genomes"));
    }

    #[test]
    fn test_genome_state_with_storage() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let storage = temp_dir.path().join("genomes");
        let state = GenomeState::with_storage(storage.clone()).expect("create state");
        assert!(storage.exists(), "with_storage should create the directory");
        assert_eq!(state.storage_dir, storage);
    }

    #[test]
    fn test_genome_state_genome_path() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");
        let path = state.genome_path("test-genome");
        assert_eq!(path, temp_dir.path().join("test-genome.genome"));
    }

    #[tokio::test]
    async fn test_genome_state_save_and_load() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");

        // Create a genome
        let manifest = GenomeManifest::new("test").version("1.0.0");
        let genome = GenomeBin::with_manifest(manifest);

        // Save it
        state
            .save_genome("test-1.0.0", &genome)
            .await
            .expect("save genome");

        // Verify file was created
        assert!(temp_dir.path().join("test-1.0.0.genome").exists());

        // Load it back
        let loaded = state.load_genome("test-1.0.0").await.expect("load genome");
        assert_eq!(loaded.manifest.name, "test");
        assert_eq!(loaded.manifest.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_genome_state_load_not_found() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");

        let result = state.load_genome("nonexistent").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Genome not found: nonexistent"));
    }

    #[tokio::test]
    async fn test_genome_state_list_all_empty() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");

        let genomes = state.list_all().await.expect("list all");
        assert!(genomes.is_empty());
    }

    #[tokio::test]
    async fn test_genome_state_list_all_with_genomes() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");

        // Save two genomes
        let manifest1 = GenomeManifest::new("genome-a").version("1.0.0");
        let manifest2 = GenomeManifest::new("genome-b").version("2.0.0");

        state
            .save_genome("genome-a-1.0.0", &GenomeBin::with_manifest(manifest1))
            .await
            .expect("save genome-a");
        state
            .save_genome("genome-b-2.0.0", &GenomeBin::with_manifest(manifest2))
            .await
            .expect("save genome-b");

        let genomes = state.list_all().await.expect("list all");
        assert_eq!(genomes.len(), 2);

        let names: Vec<&str> = genomes
            .iter()
            .map(|(_, g)| g.manifest.name.as_str())
            .collect();
        assert!(names.contains(&"genome-a"));
        assert!(names.contains(&"genome-b"));
    }

    #[tokio::test]
    async fn test_genome_state_list_all_nonexistent_dir() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let storage = temp_dir.path().join("does-not-exist");
        // Create state manually to avoid auto-creation
        let state = GenomeState {
            genomes: RwLock::new(HashMap::new()),
            storage_dir: storage,
        };

        let genomes = state.list_all().await.expect("list all");
        assert!(genomes.is_empty());
    }

    // ========== Request/Response Serialization Tests ==========

    #[test]
    fn test_build_request_deserialize() {
        let json = r#"{
            "name": "test-genome",
            "version": "1.0.0",
            "description": "Test genome",
            "binaries": []
        }"#;
        let req: BuildRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "test-genome");
        assert_eq!(req.version, Some("1.0.0".to_string()));
        assert_eq!(req.description, Some("Test genome".to_string()));
        assert!(req.binaries.is_empty());
    }

    #[test]
    fn test_build_request_minimal() {
        let json = r#"{"name": "minimal", "binaries": []}"#;
        let req: BuildRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "minimal");
        assert!(req.version.is_none());
        assert!(req.description.is_none());
    }

    #[test]
    fn test_build_request_with_binaries() {
        let json = r#"{
            "name": "multi-arch",
            "binaries": [
                {"arch": "x86_64", "path": "/tmp/bin-x86"},
                {"arch": "aarch64", "path": "/tmp/bin-arm"}
            ]
        }"#;
        let req: BuildRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.binaries.len(), 2);
        assert_eq!(req.binaries[0].arch, "x86_64");
        assert_eq!(req.binaries[1].arch, "aarch64");
    }

    #[test]
    fn test_build_response_serialize() {
        let resp = BuildResponse {
            success: true,
            genome_id: "test-1.0.0".to_string(),
            message: "Built".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("test-1.0.0"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_verify_request_deserialize() {
        let json = r#"{"path": "/tmp/test.genome"}"#;
        let req: VerifyRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.path, PathBuf::from("/tmp/test.genome"));
    }

    #[test]
    fn test_verify_response_serialize() {
        let resp = VerifyResponse {
            valid: true,
            message: "All checksums valid".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("All checksums valid"));
    }

    #[test]
    fn test_verify_response_invalid() {
        let resp = VerifyResponse {
            valid: false,
            message: "Checksum mismatch".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"valid\":false"));
    }

    #[test]
    fn test_create_genome_request_deserialize() {
        let json = r#"{"name": "my-genome"}"#;
        let req: CreateGenomeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "my-genome");
        assert!(req.version.is_none());
        assert!(req.description.is_none());
    }

    #[test]
    fn test_create_genome_request_full() {
        let json = r#"{
            "name": "full-genome",
            "version": "2.0.0",
            "description": "A fully specified genome"
        }"#;
        let req: CreateGenomeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "full-genome");
        assert_eq!(req.version, Some("2.0.0".to_string()));
        assert_eq!(
            req.description,
            Some("A fully specified genome".to_string())
        );
    }

    #[test]
    fn test_create_genome_response_serialize() {
        let resp = CreateGenomeResponse {
            success: true,
            genome_id: "new-genome-1.0.0".to_string(),
            message: "Created".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("new-genome-1.0.0"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_compose_request_deserialize() {
        let json = r#"{
            "name": "tower-atomic",
            "nucleus_type": "TOWER",
            "genomes": ["beardog-1.0", "songbird-1.0"]
        }"#;
        let req: ComposeRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.name, "tower-atomic");
        assert_eq!(req.nucleus_type, "TOWER");
        assert_eq!(req.genomes.len(), 2);
    }

    #[test]
    fn test_compose_response_serialize() {
        let resp = ComposeResponse {
            success: true,
            genome_id: "composed-1.0".to_string(),
            embedded_count: 3,
            message: "Composed".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("composed-1.0"));
        assert!(json.contains("\"embedded_count\":3"));
    }

    #[test]
    fn test_self_replicate_response_serialize() {
        let resp = SelfReplicateResponse {
            success: true,
            genome_id: "biomeos-self".to_string(),
            size: 50_000_000,
            message: "Self-replicated".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("biomeos-self"));
        assert!(json.contains("50000000"));
    }

    #[test]
    fn test_genome_summary_serialize() {
        let summary = GenomeSummary {
            id: "test-1.0".to_string(),
            name: "test".to_string(),
            version: "1.0".to_string(),
            architectures: vec!["x86_64".to_string()],
        };
        let json = serde_json::to_string(&summary).expect("serialize");
        assert!(json.contains("test-1.0"));
        assert!(json.contains("x86_64"));
    }

    #[test]
    fn test_list_genomes_response_serialize() {
        let resp = ListGenomesResponse {
            genomes: vec![
                GenomeSummary {
                    id: "g1".to_string(),
                    name: "genome1".to_string(),
                    version: "1.0".to_string(),
                    architectures: vec!["x86_64".to_string()],
                },
                GenomeSummary {
                    id: "g2".to_string(),
                    name: "genome2".to_string(),
                    version: "2.0".to_string(),
                    architectures: vec!["aarch64".to_string()],
                },
            ],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("genome1"));
        assert!(json.contains("genome2"));
        assert!(json.contains("aarch64"));
    }

    #[test]
    fn test_download_response_serialize() {
        let resp = DownloadResponse {
            url: "/api/v1/genome/test/data".to_string(),
            size: 12345,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("/api/v1/genome/test/data"));
        assert!(json.contains("12345"));
    }

    #[test]
    fn test_genome_info_response_serialize() {
        let resp = GenomeInfoResponse {
            name: "test-genome".to_string(),
            version: "1.0.0".to_string(),
            architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("test-genome"));
        assert!(json.contains("1.0.0"));
        assert!(json.contains("x86_64"));
        assert!(json.contains("aarch64"));
    }

    // ========== Handler Logic Tests (use global genome_state) ==========

    #[tokio::test]
    async fn test_get_genome_info_not_found() {
        let result = get_genome_info(Path("nonexistent-genome-xyz".to_string())).await;
        assert!(
            matches!(result, Err(StatusCode::NOT_FOUND)),
            "Expected NOT_FOUND for nonexistent genome, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_download_genome_not_found() {
        let result = download_genome(Path("nonexistent-download-xyz".to_string())).await;
        assert!(
            matches!(result, Err(StatusCode::NOT_FOUND)),
            "Expected NOT_FOUND for nonexistent genome download, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_verify_genome_not_found() {
        let result = verify_genome(Path("nonexistent-verify-xyz".to_string())).await;
        assert!(
            matches!(result, Err(StatusCode::NOT_FOUND)),
            "Expected NOT_FOUND for nonexistent genome verify, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_verify_genome_file_nonexistent_path() {
        let req = VerifyRequest {
            path: PathBuf::from("/nonexistent/path/to/genome.genome"),
        };
        let result = verify_genome_file(Json(req)).await;
        assert!(
            matches!(result, Err(StatusCode::NOT_FOUND)),
            "Expected NOT_FOUND for nonexistent file path, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_build_genome_invalid_arch() {
        let req = BuildRequest {
            name: "test".to_string(),
            version: None,
            description: None,
            binaries: vec![BinarySpec {
                arch: "invalid_arch".to_string(),
                path: PathBuf::from("/tmp/some-binary"),
            }],
        };
        let result = build_genome(Json(req)).await;
        assert!(
            matches!(result, Err(StatusCode::BAD_REQUEST)),
            "Expected BAD_REQUEST for invalid architecture, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_build_genome_binary_not_found() {
        let req = BuildRequest {
            name: "test".to_string(),
            version: None,
            description: None,
            binaries: vec![BinarySpec {
                arch: "x86_64".to_string(),
                path: PathBuf::from("/nonexistent/binary/that/does/not/exist"),
            }],
        };
        let result = build_genome(Json(req)).await;
        assert!(
            matches!(result, Err(StatusCode::NOT_FOUND)),
            "Expected NOT_FOUND when binary path does not exist, got: {:?}",
            result
        );
    }

    #[test]
    fn test_binary_spec_deserialize() {
        let json = r#"{"arch": "aarch64", "path": "/usr/bin/example"}"#;
        let spec: BinarySpec = serde_json::from_str(json).expect("deserialize");
        assert_eq!(spec.arch, "aarch64");
        assert_eq!(spec.path, PathBuf::from("/usr/bin/example"));
    }

    #[tokio::test]
    async fn test_genome_state_load_from_cache_after_save() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");

        let manifest = GenomeManifest::new("cache-test").version("1.0.0");
        let genome = GenomeBin::with_manifest(manifest);

        state
            .save_genome("cache-test-1.0.0", &genome)
            .await
            .expect("save genome");

        // Load twice - second should hit cache
        let loaded1 = state
            .load_genome("cache-test-1.0.0")
            .await
            .expect("first load");
        let loaded2 = state
            .load_genome("cache-test-1.0.0")
            .await
            .expect("second load");
        assert_eq!(loaded1.manifest.name, loaded2.manifest.name);
    }
}
