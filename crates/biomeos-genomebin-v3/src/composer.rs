// biomeos-genomebin-v3/src/composer.rs
// Fractal composition - compose atomics from genomeBins
//
// Deep Debt Principles:
// - Fractal by design (recursive composition)
// - Validation at composition time
// - Clear atomic types

use crate::GenomeBin;
use anyhow::{Context, Result};

/// Composer for fractal genomeBin composition
pub struct GenomeBinComposer {
    name: String,
    nucleus_type: Option<String>,
    genomes: Vec<GenomeBin>,
}

impl GenomeBinComposer {
    /// Create new composer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nucleus_type: None,
            genomes: Vec::new(),
        }
    }

    /// Set NUCLEUS atomic type (TOWER, NODE, NEST, NUCLEUS)
    pub fn nucleus_type(mut self, nucleus: impl Into<String>) -> Self {
        self.nucleus_type = Some(nucleus.into());
        self
    }

    /// Add genomeBin to composition
    pub fn add_genome(mut self, genome: GenomeBin) -> Self {
        self.genomes.push(genome);
        self
    }

    /// Build composed genomeBin
    pub fn build(self) -> Result<GenomeBin> {
        tracing::info!(
            "Composing genomeBin: {} ({})",
            self.name,
            self.nucleus_type.as_deref().unwrap_or("standalone")
        );

        // Validate
        if self.genomes.is_empty() {
            anyhow::bail!("Cannot compose genomeBin without any genomes");
        }

        // Create composed genomeBin
        let mut composed = GenomeBin::new(&self.name);
        composed.manifest.nucleus_atomic = self.nucleus_type.clone();

        // Validate atomic composition
        if let Some(ref atomic_type) = self.nucleus_type {
            self.validate_atomic_composition(atomic_type)?;
        }

        // Embed all genomes
        for genome in self.genomes {
            composed.embed(genome).context("Failed to embed genome")?;
        }

        tracing::info!(
            "✅ Composed genomeBin: {} ({} embedded genomes)",
            composed.manifest.name,
            composed.embedded_genomes.len()
        );

        Ok(composed)
    }

    /// Validate atomic composition rules
    fn validate_atomic_composition(&self, atomic_type: &str) -> Result<()> {
        match atomic_type {
            "TOWER" => {
                // TOWER = BearDog + Songbird
                let expected = vec!["beardog", "songbird"];
                self.validate_components(&expected, "TOWER")
            }
            "NODE" => {
                // NODE = TOWER + Toadstool
                // Accept either: (beardog + songbird + toadstool) OR (tower + toadstool)
                let has_tower = self.genomes.iter().any(|g| g.manifest.name == "tower");
                let has_components = self.genomes.iter().any(|g| g.manifest.name == "beardog")
                    && self.genomes.iter().any(|g| g.manifest.name == "songbird");
                let has_toadstool = self.genomes.iter().any(|g| g.manifest.name == "toadstool");

                if !has_toadstool {
                    anyhow::bail!("NODE atomic requires toadstool");
                }

                if !has_tower && !has_components {
                    anyhow::bail!(
                        "NODE atomic requires TOWER (or its components: beardog + songbird)"
                    );
                }

                Ok(())
            }
            "NEST" => {
                // NEST = TOWER + NestGate
                let has_tower = self.genomes.iter().any(|g| g.manifest.name == "tower");
                let has_components = self.genomes.iter().any(|g| g.manifest.name == "beardog")
                    && self.genomes.iter().any(|g| g.manifest.name == "songbird");
                let has_nestgate = self.genomes.iter().any(|g| g.manifest.name == "nestgate");

                if !has_nestgate {
                    anyhow::bail!("NEST atomic requires nestgate");
                }

                if !has_tower && !has_components {
                    anyhow::bail!("NEST atomic requires TOWER (or its components)");
                }

                Ok(())
            }
            "NUCLEUS" => {
                // NUCLEUS = TOWER + NODE + NEST (or all 5 primals)
                let required = vec!["beardog", "songbird", "toadstool", "nestgate"];
                for primal in &required {
                    if !self.genomes.iter().any(|g| g.manifest.name == *primal) {
                        anyhow::bail!("NUCLEUS requires {}", primal);
                    }
                }
                Ok(())
            }
            _ => {
                tracing::warn!("Unknown atomic type '{}', skipping validation", atomic_type);
                Ok(())
            }
        }
    }

    /// Helper: Validate expected components
    fn validate_components(&self, expected: &[&str], atomic_name: &str) -> Result<()> {
        for component in expected {
            if !self.genomes.iter().any(|g| g.manifest.name == *component) {
                anyhow::bail!("{} atomic requires {}", atomic_name, component);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenomeManifest;

    fn create_test_genome(name: &str) -> GenomeBin {
        let mut genome = GenomeBin::new(name);
        genome.manifest = GenomeManifest::new(name).version("1.0.0");
        genome
    }

    #[test]
    fn test_composer_basic() {
        let beardog = create_test_genome("beardog");
        let songbird = create_test_genome("songbird");

        let composed = GenomeBinComposer::new("tower")
            .nucleus_type("TOWER")
            .add_genome(beardog)
            .add_genome(songbird)
            .build()
            .unwrap();

        assert_eq!(composed.manifest.name, "tower");
        assert_eq!(composed.manifest.nucleus_atomic, Some("TOWER".to_string()));
        assert_eq!(composed.embedded_genomes.len(), 2);
    }

    #[test]
    fn test_composer_validation_tower() {
        // TOWER requires beardog + songbird
        let beardog = create_test_genome("beardog");

        let result = GenomeBinComposer::new("tower")
            .nucleus_type("TOWER")
            .add_genome(beardog)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("songbird"));
    }

    #[test]
    fn test_composer_no_genomes() {
        let result = GenomeBinComposer::new("empty").build();

        assert!(result.is_err());
    }
}
