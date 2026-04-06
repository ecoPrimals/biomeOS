// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Genome state for storing built genomes

use std::collections::HashMap;
use std::path::PathBuf;

use biomeos_genomebin_v3::GenomeBin;
use tokio::sync::RwLock;
use tracing::{info, warn};

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

    pub fn new() -> Result<Self, String> {
        let storage_dir = Self::default_storage_dir();
        Self::with_storage(storage_dir)
    }

    pub fn with_storage(storage_dir: PathBuf) -> Result<Self, String> {
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir)
                .map_err(|e| format!("Failed to create genome storage: {e}"))?;
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
    pub async fn save_genome(&self, id: &str, genome: &GenomeBin) -> Result<(), String> {
        let path = self.genome_path(id);
        genome
            .save(&path)
            .map_err(|e| format!("Failed to save genome: {e}"))?;

        let mut cache = self.genomes.write().await;
        cache.insert(id.to_string(), genome.clone());

        info!("💾 Saved genome to: {}", path.display());
        Ok(())
    }

    /// Load genome from persistent storage
    pub async fn load_genome(&self, id: &str) -> Result<GenomeBin, String> {
        {
            let cache = self.genomes.read().await;
            if let Some(genome) = cache.get(id) {
                return Ok(genome.clone());
            }
        }

        let path = self.genome_path(id);
        if !path.exists() {
            return Err(format!("Genome not found: {id}"));
        }

        let genome = GenomeBin::load(&path).map_err(|e| format!("Failed to load genome: {e}"))?;

        {
            let mut cache = self.genomes.write().await;
            cache.insert(id.to_string(), genome.clone());
        }

        Ok(genome)
    }

    /// List all genomes in storage
    pub fn list_all(&self) -> Result<Vec<(String, GenomeBin)>, String> {
        let mut genomes = Vec::new();

        if !self.storage_dir.exists() {
            return Ok(genomes);
        }

        let entries = std::fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read storage dir: {e}"))?;

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
