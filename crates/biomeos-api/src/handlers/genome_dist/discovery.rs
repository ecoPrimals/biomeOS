//! Genome binary discovery and path resolution.
//!
//! Locates the `wateringHole/genomeBin/` directory for genome distribution.

use std::path::PathBuf;

/// Get the genome distribution base path
///
/// Searches for `wateringHole/genomeBin/` relative to the workspace.
pub fn get_genome_bin_path() -> Option<PathBuf> {
    // Try environment variable first
    if let Ok(path) = std::env::var("GENOMEBIN_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Try relative paths from current directory
    let search_paths = [
        "../../../wateringHole/genomeBin", // From binary location
        "../../wateringHole/genomeBin",    // From biomeOS root
        "../wateringHole/genomeBin",       // From phase2
        "wateringHole/genomeBin",          // Direct
    ];

    for path in &search_paths {
        let p = PathBuf::from(path);
        if p.exists() && p.join("manifest.toml").exists() {
            return Some(p);
        }
    }

    None
}

/// Serializes tests that mutate GENOMEBIN_PATH to avoid races when running in parallel.
#[cfg(test)]
pub(crate) static GENOMEBIN_PATH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_genome_bin_path_env_var() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        let temp = tempfile::tempdir().expect("create temp dir");
        let manifest_path = temp.path().join("manifest.toml");
        std::fs::write(&manifest_path, "[manifest]\nversion = \"1.0\"").expect("write manifest");
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        std::env::set_var("GENOMEBIN_PATH", temp.path());
        let result = get_genome_bin_path();
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        assert!(
            result.is_some(),
            "GENOMEBIN_PATH with valid dir should return Some"
        );
        assert_eq!(result.unwrap(), temp.path());
    }

    #[test]
    fn test_get_genome_bin_path_env_var_nonexistent_does_not_return_it() {
        let _guard = GENOMEBIN_PATH_LOCK.lock().expect("lock");
        // When GENOMEBIN_PATH points to nonexistent path, we fall through to search_paths.
        // If we get a result, it must not be the nonexistent path we set.
        let saved = std::env::var("GENOMEBIN_PATH").ok();
        let nonexistent = "/nonexistent/genomebin/path/12345";
        std::env::set_var("GENOMEBIN_PATH", nonexistent);
        let result = get_genome_bin_path();
        if let Some(prev) = saved {
            std::env::set_var("GENOMEBIN_PATH", prev);
        } else {
            std::env::remove_var("GENOMEBIN_PATH");
        }
        if let Some(p) = result {
            assert_ne!(
                p,
                PathBuf::from(nonexistent),
                "should not return nonexistent path when env points to missing dir"
            );
        }
    }
}
