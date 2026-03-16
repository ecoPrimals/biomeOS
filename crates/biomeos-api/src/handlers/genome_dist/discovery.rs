// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Genome binary discovery and path resolution.
//!
//! Locates the `wateringHole/genomeBin/` directory for genome distribution.

use std::path::{Path, PathBuf};

const DEFAULT_SEARCH_PATHS: &[&str] = &[
    "../../../wateringHole/genomeBin",
    "../../wateringHole/genomeBin",
    "../wateringHole/genomeBin",
    "wateringHole/genomeBin",
];

pub fn get_genome_bin_path_with(env_path: Option<&str>, search_paths: &[&Path]) -> Option<PathBuf> {
    if let Some(path) = env_path {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    for path in search_paths {
        let p = PathBuf::from(*path);
        if p.exists() && p.join("manifest.toml").exists() {
            return Some(p);
        }
    }

    None
}

/// Get the genome distribution base path
///
/// Searches for `wateringHole/genomeBin/` relative to the workspace.
pub fn get_genome_bin_path() -> Option<PathBuf> {
    let env_path = std::env::var("GENOMEBIN_PATH").ok();
    let search_paths: Vec<&Path> = DEFAULT_SEARCH_PATHS.iter().map(Path::new).collect();
    get_genome_bin_path_with(env_path.as_deref(), &search_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_get_genome_bin_path_env_var() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let manifest_path = temp.path().join("manifest.toml");
        std::fs::write(&manifest_path, "[manifest]\nversion = \"1.0\"").expect("write manifest");
        let result = get_genome_bin_path_with(Some(temp.path().to_str().unwrap()), &[]);
        assert!(
            result.is_some(),
            "GENOMEBIN_PATH with valid dir should return Some"
        );
        assert_eq!(result.unwrap(), temp.path());
    }

    #[test]
    fn test_get_genome_bin_path_env_var_nonexistent_does_not_return_it() {
        let nonexistent = "/nonexistent/genomebin/path/12345";
        let search_paths: Vec<&Path> = DEFAULT_SEARCH_PATHS.iter().map(Path::new).collect();
        let result = get_genome_bin_path_with(Some(nonexistent), &search_paths);
        if let Some(p) = result {
            assert_ne!(
                p,
                PathBuf::from(nonexistent),
                "should not return nonexistent path when env points to missing dir"
            );
        }
    }
}
