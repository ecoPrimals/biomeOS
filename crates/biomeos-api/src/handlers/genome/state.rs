// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Genome state for storing built genomes

use std::collections::HashMap;
use std::path::PathBuf;

use biomeos_genomebin_v3::GenomeBin;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Errors that can occur during genome state operations.
#[derive(Debug, thiserror::Error)]
pub enum GenomeStateError {
    /// Failed to create or read the genome storage directory.
    #[error("genome storage I/O: {0}")]
    StorageIo(#[from] std::io::Error),

    /// Genome not found in cache or on disk.
    #[error("genome not found: {0}")]
    NotFound(String),

    /// Failed to serialize/deserialize a genome binary.
    #[error("genome codec: {0}")]
    Codec(#[source] anyhow::Error),
}

/// Genome state for storing built genomes
#[derive(Debug)]
pub struct GenomeState {
    /// In-memory cache of genomes
    genomes: RwLock<HashMap<String, GenomeBin>>,
    /// Storage directory for persistent genomes (XDG-compliant)
    pub storage_dir: PathBuf,
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
    pub fn default_storage_dir() -> PathBuf {
        biomeos_types::paths::SystemPaths::new_lazy()
            .data_dir()
            .join("genomes")
    }

    pub fn new() -> Result<Self, GenomeStateError> {
        let storage_dir = Self::default_storage_dir();
        Self::with_storage(storage_dir)
    }

    pub fn with_storage(storage_dir: PathBuf) -> Result<Self, GenomeStateError> {
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir)?;
        }
        Ok(Self {
            genomes: RwLock::new(HashMap::new()),
            storage_dir,
        })
    }

    /// Get path for a genome file
    pub fn genome_path(&self, id: &str) -> PathBuf {
        self.storage_dir.join(format!("{id}.genome"))
    }

    /// Save genome to persistent storage
    pub async fn save_genome(&self, id: &str, genome: &GenomeBin) -> Result<(), GenomeStateError> {
        let path = self.genome_path(id);
        genome
            .save(&path)
            .map_err(GenomeStateError::Codec)?;

        let mut cache = self.genomes.write().await;
        cache.insert(id.to_string(), genome.clone());

        info!("Saved genome to: {}", path.display());
        Ok(())
    }

    /// Load genome from persistent storage
    pub async fn load_genome(&self, id: &str) -> Result<GenomeBin, GenomeStateError> {
        {
            let cache = self.genomes.read().await;
            if let Some(genome) = cache.get(id) {
                return Ok(genome.clone());
            }
        }

        let path = self.genome_path(id);
        if !path.exists() {
            return Err(GenomeStateError::NotFound(id.to_string()));
        }

        let genome =
            GenomeBin::load(&path).map_err(GenomeStateError::Codec)?;

        {
            let mut cache = self.genomes.write().await;
            cache.insert(id.to_string(), genome.clone());
        }

        Ok(genome)
    }

    /// List all genomes in storage
    pub fn list_all(&self) -> Result<Vec<(String, GenomeBin)>, GenomeStateError> {
        let mut genomes = Vec::new();

        if !self.storage_dir.exists() {
            return Ok(genomes);
        }

        let entries = std::fs::read_dir(&self.storage_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "genome") {
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

/// Thread-safe global genome state
static GENOME_STATE: std::sync::OnceLock<GenomeState> = std::sync::OnceLock::new();

/// Get the global genome state
pub fn genome_state() -> &'static GenomeState {
    GENOME_STATE.get_or_init(GenomeState::default)
}
