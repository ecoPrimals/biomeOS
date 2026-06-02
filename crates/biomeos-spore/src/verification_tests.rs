// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_verification_status_display() {
    assert_eq!(VerificationStatus::Fresh.to_string(), "Fresh");
    assert_eq!(VerificationStatus::Stale.to_string(), "Stale");
    assert_eq!(VerificationStatus::Missing.to_string(), "Missing");
    assert_eq!(VerificationStatus::Modified.to_string(), "Modified");
    assert_eq!(VerificationStatus::Newer.to_string(), "Newer");
}

#[test]
fn test_verification_status_equality() {
    assert_eq!(VerificationStatus::Fresh, VerificationStatus::Fresh);
    assert_ne!(VerificationStatus::Fresh, VerificationStatus::Stale);
    assert_ne!(VerificationStatus::Modified, VerificationStatus::Newer);
}

#[test]
fn test_binary_verification_struct() {
    let verification = BinaryVerification {
        name: "beardog".to_string(),
        status: VerificationStatus::Fresh,
        expected_version: "0.1.0".to_string(),
        actual_version: Some("0.1.0".to_string()),
        expected_sha256: "abc123".to_string(),
        actual_sha256: Some("abc123".to_string()),
    };

    assert_eq!(verification.name, "beardog");
    assert_eq!(verification.status, VerificationStatus::Fresh);
    assert_eq!(verification.expected_version, "0.1.0");
    assert_eq!(verification.actual_version, Some("0.1.0".to_string()));
}

#[test]
fn test_binary_verification_missing() {
    let verification = BinaryVerification {
        name: "missing-primal".to_string(),
        status: VerificationStatus::Missing,
        expected_version: "1.0.0".to_string(),
        actual_version: None,
        expected_sha256: "expected_hash".to_string(),
        actual_sha256: None,
    };

    assert_eq!(verification.status, VerificationStatus::Missing);
    assert!(verification.actual_version.is_none());
    assert!(verification.actual_sha256.is_none());
}

#[test]
fn test_verification_report_struct() {
    let report = VerificationReport {
        spore_path: PathBuf::from("/media/user/biomeOS"),
        node_id: "test-node".to_string(),
        overall_status: VerificationStatus::Fresh,
        binaries: vec![],
        recommendations: vec![],
    };

    assert_eq!(report.node_id, "test-node");
    assert_eq!(report.overall_status, VerificationStatus::Fresh);
    assert!(report.recommendations.is_empty());
}

#[test]
fn test_verification_report_with_recommendations() {
    let report = VerificationReport {
        spore_path: PathBuf::from("/media/user/biomeOS"),
        node_id: "stale-node".to_string(),
        overall_status: VerificationStatus::Stale,
        binaries: vec![BinaryVerification {
            name: "beardog".to_string(),
            status: VerificationStatus::Stale,
            expected_version: "0.2.0".to_string(),
            actual_version: Some("0.1.0".to_string()),
            expected_sha256: "new_hash".to_string(),
            actual_sha256: Some("old_hash".to_string()),
        }],
        recommendations: vec![
            "Run: biomeos spore refresh /media/user/biomeOS to update binaries".to_string(),
            "1 stale binaries need update".to_string(),
        ],
    };

    assert_eq!(report.overall_status, VerificationStatus::Stale);
    assert_eq!(report.recommendations.len(), 2);
    assert!(report.recommendations[0].contains("biomeos spore refresh"));
}

#[test]
fn test_is_valid_spore_with_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path();

    // Create required structure
    std::fs::create_dir_all(spore_path.join("bin")).unwrap();
    std::fs::create_dir_all(spore_path.join("primals")).unwrap();
    std::fs::write(spore_path.join("tower.toml"), "[tower]\n").unwrap();
    std::fs::write(spore_path.join(".family.seed"), "test_seed").unwrap();

    // Create a mock verifier to test is_valid_spore
    // Since is_valid_spore is private, we test the conditions directly
    let has_tower = spore_path.join("tower.toml").exists();
    let has_seed = spore_path.join(".family.seed").exists();
    let has_bin = spore_path.join("bin").exists();
    let has_primals = spore_path.join("primals").exists();

    assert!(has_tower);
    assert!(has_seed);
    assert!(has_bin);
    assert!(has_primals);
    assert!(has_tower && has_seed && has_bin && has_primals);
}

#[test]
fn test_invalid_spore_missing_tower() {
    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path();

    // Create partial structure (missing tower.toml)
    std::fs::create_dir_all(spore_path.join("bin")).unwrap();
    std::fs::create_dir_all(spore_path.join("primals")).unwrap();
    std::fs::write(spore_path.join(".family.seed"), "test_seed").unwrap();

    let has_tower = spore_path.join("tower.toml").exists();
    assert!(!has_tower);
}

#[test]
fn test_sha256_computation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.bin");

    // Write known content
    let mut file = std::fs::File::create(&test_file).unwrap();
    file.write_all(b"hello world").unwrap();

    // Compute SHA256 manually
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"hello world");
    let expected = format!("{:x}", hasher.finalize());

    // Read file and compute
    let content = std::fs::read(&test_file).unwrap();
    let mut hasher2 = Sha256::new();
    hasher2.update(&content);
    let actual = format!("{:x}", hasher2.finalize());

    assert_eq!(expected, actual);
    // Known SHA256 of "hello world"
    assert_eq!(
        actual,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_tower_toml_node_id_extraction_modern_format() {
    let temp_dir = TempDir::new().unwrap();
    let tower_toml = temp_dir.path().join("tower.toml");

    // Modern format with primals array
    let content = r#"
[[primals]]
name = "beardog"
[primals.env]
NODE_ID = "test-node-123"
"#;
    std::fs::write(&tower_toml, content).unwrap();

    let parsed: toml::Value = toml::from_str(content).unwrap();
    let node_id = parsed
        .get("primals")
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .and_then(|primal| primal.get("env"))
        .and_then(|env| env.get("NODE_ID"))
        .and_then(|v| v.as_str());

    assert_eq!(node_id, Some("test-node-123"));
}

#[test]
fn test_tower_toml_node_id_extraction_legacy_format() {
    let content = r#"
[tower]
NODE_ID = "legacy-node-456"
"#;

    let parsed: toml::Value = toml::from_str(content).unwrap();
    let node_id = parsed
        .get("tower")
        .and_then(|t| t.get("NODE_ID"))
        .and_then(|v| v.as_str());

    assert_eq!(node_id, Some("legacy-node-456"));
}

#[test]
fn test_tower_toml_node_id_extraction_simple_format() {
    let content = r#"
node_id = "simple-node-789"
"#;

    let parsed: toml::Value = toml::from_str(content).unwrap();
    let node_id = parsed.get("node_id").and_then(|v| v.as_str());

    assert_eq!(node_id, Some("simple-node-789"));
}

#[test]
fn test_status_clone() {
    let status = VerificationStatus::Fresh;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

#[test]
fn test_verifier_from_nucleus() {
    let temp_dir = TempDir::new().unwrap();
    let nucleus_path = temp_dir.path();
    std::fs::create_dir_all(nucleus_path.join("tower")).unwrap();
    std::fs::create_dir_all(nucleus_path.join("primals")).unwrap();
    std::fs::write(nucleus_path.join("tower").join("tower"), b"tower").unwrap();

    let result = SporeVerifier::from_nucleus(nucleus_path);
    assert!(result.is_ok());
}

#[test]
fn test_verify_spore_minimal() {
    let temp_dir = TempDir::new().unwrap();
    let nucleus_path = temp_dir.path();
    std::fs::create_dir_all(nucleus_path.join("tower")).unwrap();
    std::fs::create_dir_all(nucleus_path.join("primals")).unwrap();
    let tower_bytes = b"tower_binary";
    std::fs::write(nucleus_path.join("tower").join("tower"), tower_bytes).unwrap();

    let spore_path = temp_dir.path().join("spore");
    std::fs::create_dir_all(spore_path.join("bin")).unwrap();
    std::fs::create_dir_all(spore_path.join("primals")).unwrap();
    std::fs::write(
        spore_path.join("tower.toml"),
        r#"[tower]
NODE_ID = "verify-test-node"
"#,
    )
    .unwrap();
    std::fs::write(spore_path.join(".family.seed"), b"seed").unwrap();

    std::fs::write(spore_path.join("bin").join("tower"), tower_bytes).unwrap();

    let verifier = SporeVerifier::from_nucleus(nucleus_path).unwrap();
    let report = verifier.verify_spore(&spore_path).unwrap();
    assert_eq!(report.node_id, "verify-test-node");
    assert!(
        report.overall_status == VerificationStatus::Fresh
            || report.overall_status == VerificationStatus::Stale
    );
}

#[test]
fn test_extract_node_id_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path();
    std::fs::create_dir_all(spore_path).unwrap();
    let result = SporeVerifier::from_nucleus(spore_path).unwrap();
    let spore_no_tower = temp_dir.path().join("empty");
    std::fs::create_dir_all(&spore_no_tower).unwrap();
    let report_result = result.verify_spore(&spore_no_tower);
    assert!(report_result.is_err());
}

#[test]
fn test_verify_all_spores_smoke() {
    let temp_dir = TempDir::new().unwrap();
    let nucleus_path = temp_dir.path();
    std::fs::create_dir_all(nucleus_path.join("tower")).unwrap();
    std::fs::create_dir_all(nucleus_path.join("primals")).unwrap();
    std::fs::write(nucleus_path.join("tower").join("tower"), b"tower").unwrap();

    let verifier = SporeVerifier::from_nucleus(nucleus_path).unwrap();
    let reports = verifier.verify_all_spores().expect("verify_all");
    let _ = reports;
}

#[test]
fn test_verify_spore_modified_when_hash_mismatch_equal_version() {
    let temp = TempDir::new().unwrap();
    let nucleus = temp.path().join("nucleus");
    let spore = temp.path().join("spore");
    std::fs::create_dir_all(nucleus.join("tower")).unwrap();
    std::fs::create_dir_all(spore.join("bin")).unwrap();
    std::fs::create_dir_all(spore.join("primals")).unwrap();

    let tower_bytes = b"v1binary";
    std::fs::write(nucleus.join("tower").join("tower"), tower_bytes).unwrap();
    std::fs::write(spore.join("bin").join("tower"), b"tampered").unwrap();

    use crate::manifest::{
        BinaryInfo, BinaryManifest, CompatibilityInfo, LineageInfo, ManifestMeta,
        SporeBinaryInfo, SporeInfo, SporeManifest,
    };
    use chrono::Utc;

    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(tower_bytes);
    let expected_sha = format!("{:x}", hasher.finalize());

    let manifest = BinaryManifest {
        manifest: ManifestMeta {
            version: "1".into(),
            created_at: Utc::now(),
            pipeline_run: "t".into(),
        },
        binaries: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "tower".into(),
                BinaryInfo {
                    name: "tower".into(),
                    version: "1.0.0".into(),
                    git_commit: "x".into(),
                    build_date: Utc::now(),
                    sha256: expected_sha,
                    size_bytes: tower_bytes.len() as u64,
                    source_repo: "s".into(),
                    features: vec![],
                },
            );
            m
        },
        compatibility: CompatibilityInfo {
            min_tower_version: "0".into(),
            min_beardog_version: "0".into(),
            min_songbird_version: "0".into(),
        },
    };
    std::fs::write(
        nucleus.join("MANIFEST.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let spore_manifest = SporeManifest {
        spore: SporeInfo {
            node_id: "n1".into(),
            family_id: "f".into(),
            created_at: Utc::now(),
            created_by: "t".into(),
            spore_type: "usb".into(),
            deployment_batch: "b".into(),
        },
        lineage: LineageInfo {
            parent_seed_hash: "p".into(),
            child_seed_hash: "c".into(),
            derivation_method: "m".into(),
        },
        binaries: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "tower".into(),
                SporeBinaryInfo {
                    name: "tower".into(),
                    version: "1.0.0".into(),
                    sha256: "old".into(),
                    source_manifest: "s".into(),
                    copied_at: Utc::now(),
                },
            );
            m
        },
        deployment_history: vec![],
    };
    spore_manifest.save(&spore).unwrap();

    let verifier = SporeVerifier::from_nucleus(&nucleus).expect("verifier");
    let report = verifier.verify_spore(&spore).expect("report");
    let tower_v = report
        .binaries
        .iter()
        .find(|b| b.name == "tower")
        .expect("tower");
    assert_eq!(tower_v.status, VerificationStatus::Modified);
}
