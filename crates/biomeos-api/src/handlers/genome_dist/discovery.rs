// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    use biomeos_test_utils::env_helpers::TestEnvGuard;
    use serial_test::serial;
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

    #[test]
    fn test_get_genome_bin_path_with_no_env_searches_paths() {
        let temp = tempfile::tempdir().expect("temp dir");
        let genome_dir = temp.path().join("genomeBin");
        std::fs::create_dir_all(&genome_dir).expect("create dir");
        std::fs::write(
            genome_dir.join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let search_paths = vec![genome_dir.as_path()];
        let result = get_genome_bin_path_with(None, &search_paths);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), genome_dir);
    }

    #[test]
    fn test_get_genome_bin_path_with_env_existing_dir_returns_it() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir_with_manifest = temp.path().join("with-manifest");
        std::fs::create_dir_all(&dir_with_manifest).expect("create dir");
        std::fs::write(
            dir_with_manifest.join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let search_paths: Vec<&Path> = vec![];
        let result =
            get_genome_bin_path_with(Some(dir_with_manifest.to_str().unwrap()), &search_paths);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), dir_with_manifest);
    }

    #[test]
    fn test_get_genome_bin_path_with_env_nonexistent_falls_through() {
        let nonexistent = "/tmp/nonexistent-genomebin-xyz-98765";
        let temp = tempfile::tempdir().expect("temp dir");
        let genome_dir = temp.path().join("genomeBin");
        std::fs::create_dir_all(&genome_dir).expect("create dir");
        std::fs::write(
            genome_dir.join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let search_paths = vec![genome_dir.as_path()];
        let result = get_genome_bin_path_with(Some(nonexistent), &search_paths);
        assert!(result.is_some(), "should fall through to search_paths");
        assert_eq!(result.unwrap(), genome_dir);
    }

    #[test]
    fn test_get_genome_bin_path_with_empty_search_paths() {
        let result = get_genome_bin_path_with(None, &[]);
        assert!(result.is_none());
    }

    /// Exercises [`get_genome_bin_path`] (env + default search paths), not only
    /// [`get_genome_bin_path_with`].
    #[test]
    #[serial]
    fn test_get_genome_bin_path_wrapper_respects_genomebin_path_env() {
        let temp = tempfile::tempdir().expect("create temp dir");
        std::fs::write(
            temp.path().join("manifest.toml"),
            "[manifest]\nversion = \"1.0\"",
        )
        .expect("write manifest");
        let _guard = TestEnvGuard::set("GENOMEBIN_PATH", temp.path().to_str().expect("utf8 path"));
        let result = get_genome_bin_path();
        assert_eq!(result, Some(temp.path().to_path_buf()));
    }
}
