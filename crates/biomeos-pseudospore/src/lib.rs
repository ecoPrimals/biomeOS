// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! pseudoSpore envelope validation for biomeOS NUCLEUS gateway.
//!
//! Implements the pseudoSpore 2.0 standard: structural validation, BLAKE3
//! checksum verification, and module completeness checking. Types and logic
//! are compatible with `litho_core::pseudospore` — when lithoSpore ships
//! `pseudospore-core` as a standalone crate, this becomes a thin re-export.
//!
//! Required directory layout:
//! ```text
//! pseudoSpore_<name>/
//! ├── scope.toml                    # [artifact] type = "pseudoSpore"
//! ├── validation.json               # module results
//! ├── receipts/
//! │   ├── environment.toml          # [hardware] + [software]
//! │   └── checksums.blake3          # BLAKE3 of content files
//! ├── provenance/
//! │   └── ferment_transcript.json   # dataset_id + spring
//! └── README.md
//! ```

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Scope types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PseudoSporeScope {
    pub artifact: ArtifactIdentity,
    #[serde(default)]
    pub target: Option<TargetPaper>,
    #[serde(default)]
    pub module: Vec<PseudoModule>,
    #[serde(default)]
    pub evolution: Option<EvolutionTiers>,
    #[serde(default)]
    pub source: Option<SourceRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtifactIdentity {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub experiment: Option<u32>,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TargetPaper {
    #[serde(default)]
    pub paper_doi: String,
    #[serde(default)]
    pub paper_title: String,
    #[serde(default)]
    pub paper_authors: String,
    #[serde(default)]
    pub paper_year: Option<u16>,
    #[serde(default)]
    pub paper_pdb: Option<String>,
    #[serde(default)]
    pub paper_system: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PseudoModule {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub checks: Option<u32>,
    #[serde(default)]
    pub checks_total: Option<u32>,
    #[serde(default)]
    pub checks_passed: Option<u32>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvolutionTiers {
    #[serde(default)]
    pub tier_0: Option<String>,
    #[serde(default)]
    pub tier_1: Option<String>,
    #[serde(default)]
    pub tier_2: Option<String>,
    #[serde(default)]
    pub tier_3: Option<String>,
    #[serde(default)]
    pub acceptance_test: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceRef {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub branch: String,
}

// ---------------------------------------------------------------------------
// Validation result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationDoc {
    #[serde(default)]
    pub artifact: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub modules: Vec<ValidationModule>,
    #[serde(default)]
    pub summary: Option<ValidationSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationModule {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub checks_total: Option<u32>,
    #[serde(default)]
    pub checks_passed: Option<u32>,
    #[serde(default)]
    pub checks: Vec<serde_json::Value>,
    #[serde(default)]
    pub errata: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationSummary {
    #[serde(default)]
    pub modules_total: u32,
    #[serde(default)]
    pub modules_pass: u32,
    #[serde(default)]
    pub modules_in_flight: u32,
}

// ---------------------------------------------------------------------------
// Environment receipt
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvironmentReceipt {
    #[serde(default)]
    pub hardware: Option<BTreeMap<String, toml::Value>>,
    #[serde(default)]
    pub software: Option<BTreeMap<String, toml::Value>>,
    #[serde(default)]
    pub timestamps: Option<BTreeMap<String, toml::Value>>,
}

// ---------------------------------------------------------------------------
// Ferment transcript
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FermentTranscript {
    #[serde(default)]
    pub dataset_id: String,
    #[serde(default)]
    pub spring: String,
    #[serde(default)]
    pub spring_version: Option<String>,
    #[serde(default)]
    pub braid_id: Option<String>,
    #[serde(default)]
    pub dag_session_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

// ---------------------------------------------------------------------------
// Checksum entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ChecksumEntry {
    pub hash: String,
    pub path: String,
}

// ---------------------------------------------------------------------------
// Spore status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SporeStatus {
    Valid,
    Verified,
    Complete,
    Invalid,
}

impl std::fmt::Display for SporeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "VALID"),
            Self::Verified => write!(f, "VERIFIED"),
            Self::Complete => write!(f, "COMPLETE"),
            Self::Invalid => write!(f, "INVALID"),
        }
    }
}

// ---------------------------------------------------------------------------
// Composite manifest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PseudoSporeManifest {
    pub root: PathBuf,
    pub scope: PseudoSporeScope,
    pub validation: ValidationDoc,
    pub environment: EnvironmentReceipt,
    pub ferment: FermentTranscript,
    pub checksums: Vec<ChecksumEntry>,
    pub status: SporeStatus,
    pub errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Parsing and validation
// ---------------------------------------------------------------------------

/// Parse `checksums.blake3` file content into entries.
pub fn parse_checksums(content: &str) -> Vec<ChecksumEntry> {
    content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let mut parts = line.splitn(2, "  ");
            let hash = parts.next()?.trim().to_string();
            let path = parts.next()?.trim().to_string();
            if hash.is_empty() || path.is_empty() {
                return None;
            }
            Some(ChecksumEntry { hash, path })
        })
        .collect()
}

/// Validate a pseudoSpore directory structure. Returns a manifest with status.
pub fn load_pseudospore(root: &Path) -> PseudoSporeManifest {
    let mut errors = Vec::new();

    let scope_path = root.join("scope.toml");
    let scope: PseudoSporeScope = match std::fs::read_to_string(&scope_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("scope.toml parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("scope.toml not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if scope.artifact.artifact_type != "pseudoSpore"
        && scope.artifact.artifact_type != "pseudo-lithoSpore"
    {
        errors.push(format!(
            "scope.toml type is '{}', expected 'pseudoSpore'",
            scope.artifact.artifact_type
        ));
    }

    let val_path = root.join("validation.json");
    let validation: ValidationDoc = match std::fs::read_to_string(&val_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("validation.json parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("validation.json not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if validation.modules.is_empty() {
        errors.push("validation.json has no modules".to_string());
    }

    let env_path = root.join("receipts/environment.toml");
    let environment: EnvironmentReceipt = match std::fs::read_to_string(&env_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("receipts/environment.toml parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("receipts/environment.toml not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if environment.hardware.is_none() {
        errors.push("receipts/environment.toml missing [hardware]".to_string());
    }
    if environment.software.is_none() {
        errors.push("receipts/environment.toml missing [software]".to_string());
    }

    let cksum_path = root.join("receipts/checksums.blake3");
    let checksums: Vec<ChecksumEntry> = match std::fs::read_to_string(&cksum_path) {
        Ok(content) => parse_checksums(&content),
        Err(_) => {
            errors.push("receipts/checksums.blake3 not found".to_string());
            Vec::new()
        }
    };

    let ferment_path = root.join("provenance/ferment_transcript.json");
    let ferment: FermentTranscript = match std::fs::read_to_string(&ferment_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(f) => f,
            Err(e) => {
                errors.push(format!(
                    "provenance/ferment_transcript.json parse error: {e}"
                ));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("provenance/ferment_transcript.json not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if ferment.dataset_id.is_empty() {
        errors.push("ferment_transcript.json missing dataset_id".to_string());
    }
    if ferment.spring.is_empty() {
        errors.push("ferment_transcript.json missing spring".to_string());
    }

    let readme_path = root.join("README.md");
    if !readme_path.exists() {
        errors.push("README.md not found".to_string());
    } else if std::fs::metadata(&readme_path)
        .map(|m| m.len())
        .unwrap_or(0)
        == 0
    {
        errors.push("README.md is empty".to_string());
    }

    let status = if errors.is_empty() {
        SporeStatus::Valid
    } else {
        SporeStatus::Invalid
    };

    PseudoSporeManifest {
        root: root.to_path_buf(),
        scope,
        validation,
        environment,
        ferment,
        checksums,
        status,
        errors,
    }
}

/// Verify BLAKE3 checksums against actual files. Upgrades status to `Verified`.
pub fn verify_checksums(manifest: &mut PseudoSporeManifest) -> bool {
    if manifest.status == SporeStatus::Invalid {
        return false;
    }

    if manifest.checksums.is_empty() {
        manifest.errors.push("No checksums to verify".to_string());
        return false;
    }

    let mut all_ok = true;
    for entry in &manifest.checksums {
        let file_path = manifest.root.join(&entry.path);
        match std::fs::read(&file_path) {
            Ok(data) => {
                let computed = blake3::hash(&data).to_hex().to_string();
                if computed != entry.hash {
                    manifest.errors.push(format!(
                        "Checksum mismatch: {} (expected {}, got {})",
                        entry.path,
                        &entry.hash[..std::cmp::min(12, entry.hash.len())],
                        &computed[..12]
                    ));
                    all_ok = false;
                }
            }
            Err(_) => {
                manifest
                    .errors
                    .push(format!("Missing file: {}", entry.path));
                all_ok = false;
            }
        }
    }

    if all_ok {
        manifest.status = SporeStatus::Verified;
    }
    all_ok
}

/// Check completeness: all validation modules PASS or SKIP, none IN_FLIGHT.
pub fn check_completeness(manifest: &mut PseudoSporeManifest) -> bool {
    if manifest.status == SporeStatus::Invalid {
        return false;
    }

    let all_done = manifest.validation.modules.iter().all(|m| {
        let s = m.status.to_uppercase();
        s == "PASS" || s == "SKIP"
    });

    if all_done
        && (manifest.status == SporeStatus::Verified || manifest.status == SporeStatus::Valid)
    {
        manifest.status = SporeStatus::Complete;
    }
    all_done
}

/// Compute BLAKE3 checksums for all files under specified subdirectories.
pub fn compute_checksums(root: &Path, dirs: &[&str]) -> Vec<ChecksumEntry> {
    let mut entries = Vec::new();
    for dir_name in dirs {
        let dir = root.join(dir_name);
        if !dir.exists() {
            continue;
        }
        if let Ok(files) = walk_dir(&dir) {
            for file_path in files {
                if let Ok(data) = std::fs::read(&file_path) {
                    let hash = blake3::hash(&data).to_hex().to_string();
                    let rel = file_path
                        .strip_prefix(root)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();
                    entries.push(ChecksumEntry { hash, path: rel });
                }
            }
        }
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries
}

/// Format checksum entries into `checksums.blake3` file content.
pub fn format_checksums(entries: &[ChecksumEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}  {}", e.hash, e.path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn walk_dir(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_dir(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn invalid_manifest(root: &Path, errors: Vec<String>) -> PseudoSporeManifest {
    PseudoSporeManifest {
        root: root.to_path_buf(),
        scope: PseudoSporeScope {
            artifact: ArtifactIdentity {
                name: String::new(),
                version: String::new(),
                artifact_type: String::new(),
                date: String::new(),
                origin: String::new(),
                experiment: None,
                license: String::new(),
            },
            target: None,
            module: Vec::new(),
            evolution: None,
            source: None,
        },
        validation: ValidationDoc {
            artifact: String::new(),
            version: String::new(),
            date: String::new(),
            modules: Vec::new(),
            summary: None,
        },
        environment: EnvironmentReceipt {
            hardware: None,
            software: None,
            timestamps: None,
        },
        ferment: FermentTranscript {
            dataset_id: String::new(),
            spring: String::new(),
            spring_version: None,
            braid_id: None,
            dag_session_id: None,
            timestamp: None,
        },
        checksums: Vec::new(),
        status: SporeStatus::Invalid,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_checksums_basic() {
        let input = "abc123def456  outputs/foo.dat\n789012345678  provenance/bar.json\n";
        let entries = parse_checksums(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc123def456");
        assert_eq!(entries[0].path, "outputs/foo.dat");
        assert_eq!(entries[1].path, "provenance/bar.json");
    }

    #[test]
    fn parse_checksums_skips_comments_and_empty() {
        let input = "# header comment\nabc123  file.dat\n\n# another comment\n";
        let entries = parse_checksums(input);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn format_checksums_roundtrip() {
        let entries = vec![
            ChecksumEntry {
                hash: "aaa".to_string(),
                path: "a.txt".to_string(),
            },
            ChecksumEntry {
                hash: "bbb".to_string(),
                path: "b.txt".to_string(),
            },
        ];
        let formatted = format_checksums(&entries);
        assert_eq!(formatted, "aaa  a.txt\nbbb  b.txt");
    }

    fn create_valid_pseudospore(dir: &Path) {
        std::fs::write(
            dir.join("scope.toml"),
            r#"[artifact]
name = "test-spore"
version = "1.0.0"
type = "pseudoSpore"
date = "2026-05-27"
origin = "biomeOS-test"
license = "AGPL-3.0"
"#,
        )
        .unwrap();

        std::fs::write(
            dir.join("validation.json"),
            r#"{"artifact":"test-spore","version":"1.0.0","date":"2026-05-27","modules":[{"name":"structural","status":"PASS","checks_total":3,"checks_passed":3}]}"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("receipts")).unwrap();
        std::fs::write(
            dir.join("receipts/environment.toml"),
            r#"[hardware]
cpu = "x86_64"
cores = 8

[software]
os = "Linux"
rust = "1.82"
"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("data")).unwrap();
        std::fs::write(dir.join("data/payload.bin"), b"hello world").unwrap();

        let hash = blake3::hash(b"hello world").to_hex().to_string();
        std::fs::write(
            dir.join("receipts/checksums.blake3"),
            format!("{hash}  data/payload.bin\n"),
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("provenance")).unwrap();
        std::fs::write(
            dir.join("provenance/ferment_transcript.json"),
            r#"{"dataset_id":"ds-001","spring":"hotSpring","spring_version":"1.5.0"}"#,
        )
        .unwrap();

        std::fs::write(dir.join("README.md"), "# Test pseudoSpore\n").unwrap();
    }

    #[test]
    fn load_valid_pseudospore() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_valid_pseudospore(tmp.path());

        let manifest = load_pseudospore(tmp.path());
        assert_eq!(manifest.status, SporeStatus::Valid);
        assert!(manifest.errors.is_empty(), "errors: {:?}", manifest.errors);
        assert_eq!(manifest.scope.artifact.name, "test-spore");
        assert_eq!(manifest.checksums.len(), 1);
    }

    #[test]
    fn verify_checksums_pass() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_valid_pseudospore(tmp.path());

        let mut manifest = load_pseudospore(tmp.path());
        assert!(verify_checksums(&mut manifest));
        assert_eq!(manifest.status, SporeStatus::Verified);
    }

    #[test]
    fn verify_checksums_fail_on_tamper() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_valid_pseudospore(tmp.path());
        std::fs::write(tmp.path().join("data/payload.bin"), b"tampered").unwrap();

        let mut manifest = load_pseudospore(tmp.path());
        assert!(!verify_checksums(&mut manifest));
        assert!(manifest.errors.iter().any(|e| e.contains("mismatch")));
    }

    #[test]
    fn check_completeness_pass() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_valid_pseudospore(tmp.path());

        let mut manifest = load_pseudospore(tmp.path());
        let _ = verify_checksums(&mut manifest);
        assert!(check_completeness(&mut manifest));
        assert_eq!(manifest.status, SporeStatus::Complete);
    }

    #[test]
    fn load_missing_scope_returns_invalid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let manifest = load_pseudospore(tmp.path());
        assert_eq!(manifest.status, SporeStatus::Invalid);
        assert!(manifest.errors.iter().any(|e| e.contains("scope.toml")));
    }

    #[test]
    fn load_wrong_type_reports_error() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_valid_pseudospore(tmp.path());
        std::fs::write(
            tmp.path().join("scope.toml"),
            "[artifact]\nname = \"x\"\nversion = \"1\"\ntype = \"liveSpore\"\n",
        )
        .unwrap();

        let manifest = load_pseudospore(tmp.path());
        assert!(manifest.errors.iter().any(|e| e.contains("expected 'pseudoSpore'")));
    }

    #[test]
    fn compute_checksums_works() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("data")).unwrap();
        std::fs::write(tmp.path().join("data/a.bin"), b"aaa").unwrap();
        std::fs::write(tmp.path().join("data/b.bin"), b"bbb").unwrap();

        let entries = compute_checksums(tmp.path(), &["data"]);
        assert_eq!(entries.len(), 2);
        assert!(entries[0].path.contains("data/"));
    }
}
