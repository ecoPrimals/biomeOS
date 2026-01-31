// biomeos-genomebin-v3/src/runtime.rs
// Runtime extraction and execution
//
// Deep Debt Principles:
// - Platform-agnostic installation paths (etcetera)
// - Runtime discovery (no hardcoding)
// - Clear error messages

use crate::{Arch, GenomeBin};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

impl GenomeBin {
    /// Extract binary for current architecture to directory
    pub fn extract(&self, install_dir: &Path) -> Result<PathBuf> {
        let arch = Arch::detect();
        tracing::info!("Extracting {:?} binary to: {}", arch, install_dir.display());
        
        // Get binary for current arch
        let compressed = self.binaries.get(&arch)
            .with_context(|| format!("No binary for current architecture: {:?}", arch))?;
        
        // Decompress
        let decompressed = compressed.decompress()
            .context("Failed to decompress binary")?;
        
        // Create install directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;
        
        // Write binary
        let binary_path = install_dir.join(&self.manifest.name);
        std::fs::write(&binary_path, decompressed)
            .with_context(|| format!("Failed to write binary: {}", binary_path.display()))?;
        
        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = binary_path.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }
        
        tracing::info!("✅ Extracted to: {}", binary_path.display());
        
        // If this has embedded genomes, extract them too
        for embedded in &self.embedded_genomes {
            let embedded_dir = install_dir.join(&embedded.manifest.name);
            embedded.extract(&embedded_dir)
                .with_context(|| format!("Failed to extract embedded genome: {}", embedded.manifest.name))?;
        }
        
        Ok(binary_path)
    }
    
    /// Get default install directory for current platform
    /// 
    /// Deep Debt: Uses etcetera for platform-agnostic paths
    pub fn default_install_dir() -> Result<PathBuf> {
        use etcetera::BaseStrategy;
        
        // Check for Android
        if std::path::Path::new("/system/build.prop").exists() {
            tracing::debug!("Detected Android platform");
            return Ok(PathBuf::from("/data/local/tmp"));
        }
        
        // Use XDG base directories (etcetera - Pure Rust)
        let strategy = etcetera::base_strategy::choose_base_strategy()
            .context("Failed to determine base directory strategy")?;
        
        // Check if root
        let is_root = std::env::var("USER").map(|u| u == "root").unwrap_or(false);
        
        if is_root {
            // Root: Use /opt
            Ok(PathBuf::from("/opt"))
        } else {
            // Non-root: Use ~/.local
            let data_dir = strategy.data_dir();
            Ok(data_dir.join("..").join(".local"))
        }
    }
    
    /// Run in-place without full extraction (basic implementation)
    /// 
    /// Note: Full zero-copy mmap implementation is future enhancement
    pub fn run_in_place(&self, args: &[String]) -> Result<()> {
        tracing::info!("Running {} in-place", self.manifest.name);
        
        // For now: Extract to temp directory, execute, cleanup
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;
        
        let binary_path = self.extract(temp_dir.path())
            .context("Failed to extract to temp directory")?;
        
        // Execute
        let status = std::process::Command::new(&binary_path)
            .args(args)
            .status()
            .with_context(|| format!("Failed to execute: {}", binary_path.display()))?;
        
        if !status.success() {
            anyhow::bail!("Process exited with status: {}", status);
        }
        
        // Temp dir automatically cleaned up on drop
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_install_dir() {
        let dir = GenomeBin::default_install_dir().unwrap();
        assert!(dir.is_absolute());
        // Should be /opt for root or ~/.local for non-root
        let path_str = dir.to_str().unwrap();
        assert!(path_str.contains("/opt") || path_str.contains("/.local") || path_str.contains("/data/local/tmp"));
    }
}
