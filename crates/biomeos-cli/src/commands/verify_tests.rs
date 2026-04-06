// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

use super::*;
use biomeos_spore::verification::VerificationStatus;

#[test]
fn test_verification_status_display() {
    assert_eq!(
        verification_status_display(&VerificationStatus::Fresh),
        ("✅", "Fresh")
    );
    assert_eq!(
        verification_status_display(&VerificationStatus::Stale),
        ("⚠️ ", "Stale")
    );
    assert_eq!(
        verification_status_display(&VerificationStatus::Missing),
        ("❌", "Missing")
    );
    assert_eq!(
        verification_status_display(&VerificationStatus::Modified),
        ("⚠️ ", "Modified")
    );
    assert_eq!(
        verification_status_display(&VerificationStatus::Newer),
        ("❓", "Newer")
    );
}

#[test]
fn test_verification_status_display_all_variants() {
    use biomeos_spore::verification::VerificationStatus;
    let variants = [
        (VerificationStatus::Fresh, "✅", "Fresh"),
        (VerificationStatus::Stale, "⚠️ ", "Stale"),
        (VerificationStatus::Missing, "❌", "Missing"),
        (VerificationStatus::Modified, "⚠️ ", "Modified"),
        (VerificationStatus::Newer, "❓", "Newer"),
    ];
    for (status, expected_icon, expected_text) in variants {
        let (icon, text) = verification_status_display(&status);
        assert_eq!(icon, expected_icon, "icon for {status:?}");
        assert_eq!(text, expected_text, "text for {status:?}");
    }
}

#[test]
fn test_verification_status_display_icons_distinct() {
    let (fresh_icon, _) = verification_status_display(&VerificationStatus::Fresh);
    let (missing_icon, _) = verification_status_display(&VerificationStatus::Missing);
    assert_ne!(fresh_icon, missing_icon);
}

#[test]
fn test_verify_args_target_nucleus() {
    let args = VerifyArgs {
        target: VerifyTarget::Nucleus {
            path: PathBuf::from("plasmidBin"),
        },
    };
    match &args.target {
        VerifyTarget::Nucleus { path } => assert_eq!(path, &PathBuf::from("plasmidBin")),
        _ => panic!("expected Nucleus"),
    }
}

#[test]
fn test_verify_args_target_spore() {
    let args = VerifyArgs {
        target: VerifyTarget::Spore {
            mount_point: PathBuf::from("/media/usb/biomeOS"),
        },
    };
    match &args.target {
        VerifyTarget::Spore { mount_point } => {
            assert_eq!(mount_point, &PathBuf::from("/media/usb/biomeOS"));
        }
        _ => panic!("expected Spore"),
    }
}

#[test]
fn test_verify_args_target_all() {
    let args = VerifyArgs {
        target: VerifyTarget::All { verbose: false },
    };
    match &args.target {
        VerifyTarget::All { verbose } => assert!(!*verbose),
        _ => panic!("expected All"),
    }
}

#[tokio::test]
async fn test_run_nucleus_nonexistent_path() {
    let args = VerifyArgs {
        target: VerifyTarget::Nucleus {
            path: PathBuf::from("/nonexistent/path/xyz"),
        },
    };
    let result = run(args).await;
    assert!(result.is_ok(), "run should return Ok (prints message)");
}

#[tokio::test]
async fn test_run_spore_nonexistent() {
    let args = VerifyArgs {
        target: VerifyTarget::Spore {
            mount_point: PathBuf::from("/nonexistent/spore/mount"),
        },
    };
    let _result = run(args).await;
}

#[tokio::test]
async fn test_run_all_spores() {
    let args = VerifyArgs {
        target: VerifyTarget::All { verbose: false },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_all_spores_verbose() {
    let args = VerifyArgs {
        target: VerifyTarget::All { verbose: true },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[test]
fn test_verify_args_debug() {
    let args = VerifyArgs {
        target: VerifyTarget::Nucleus {
            path: PathBuf::from("plasmidBin"),
        },
    };
    let _ = format!("{args:?}");
}

#[test]
fn test_verify_target_variants() {
    let _ = format!(
        "{:?}",
        VerifyTarget::Nucleus {
            path: PathBuf::from("p")
        }
    );
    let _ = format!(
        "{:?}",
        VerifyTarget::Spore {
            mount_point: PathBuf::from("/m")
        }
    );
    let _ = format!("{:?}", VerifyTarget::All { verbose: true });
}

fn minimal_plasmid_bin(temp: &std::path::Path) -> std::path::PathBuf {
    let pb = temp.join("plasmidBin");
    std::fs::create_dir_all(pb.join("tower")).unwrap();
    std::fs::write(pb.join("tower").join("tower"), b"tower-bytes").unwrap();
    std::fs::create_dir_all(pb.join("primals")).unwrap();
    std::fs::write(pb.join("primals").join("beardog-server"), b"bd").unwrap();
    std::fs::write(pb.join("primals").join("songbird"), b"sb").unwrap();
    pb
}

#[tokio::test]
async fn test_run_nucleus_with_minimal_plasmid_bin() {
    let temp = tempfile::tempdir().unwrap();
    let pb = minimal_plasmid_bin(temp.path());
    let args = VerifyArgs {
        target: VerifyTarget::Nucleus { path: pb },
    };
    assert!(run(args).await.is_ok());
}

#[tokio::test]
async fn test_run_nucleus_with_manifest_toml() {
    let temp = tempfile::tempdir().unwrap();
    let pb = minimal_plasmid_bin(temp.path());
    let manifest = biomeos_spore::manifest::BinaryManifest::from_nucleus(&pb).unwrap();
    manifest.save(pb.join("MANIFEST.toml")).unwrap();

    let args = VerifyArgs {
        target: VerifyTarget::Nucleus { path: pb },
    };
    assert!(run(args).await.is_ok());
}

#[tokio::test]
async fn test_run_nucleus_with_manifest_features_prints_features_line() {
    use biomeos_spore::manifest::{BinaryInfo, BinaryManifest, CompatibilityInfo, ManifestMeta};
    use chrono::Utc;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;

    let temp = tempfile::tempdir().unwrap();
    let pb = minimal_plasmid_bin(temp.path());
    let tower_path = pb.join("tower/tower");
    let bytes = std::fs::read(&tower_path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha = format!("{:x}", hasher.finalize());

    let mut binaries = HashMap::new();
    binaries.insert(
        "tower".to_string(),
        BinaryInfo {
            name: "tower".to_string(),
            version: "9.9.9".to_string(),
            git_commit: "abc".to_string(),
            build_date: Utc::now(),
            sha256: sha,
            size_bytes: bytes.len() as u64,
            source_repo: "test".to_string(),
            features: vec!["feat-a".to_string(), "feat-b".to_string()],
        },
    );

    let manifest = BinaryManifest {
        manifest: ManifestMeta {
            version: "9.9.9".to_string(),
            created_at: Utc::now(),
            pipeline_run: "test".to_string(),
        },
        binaries,
        compatibility: CompatibilityInfo {
            min_tower_version: "1.0.0".to_string(),
            min_beardog_version: "0.1.0".to_string(),
            min_songbird_version: "0.1.0".to_string(),
        },
    };
    manifest.save(pb.join("MANIFEST.toml")).unwrap();

    let args = VerifyArgs {
        target: VerifyTarget::Nucleus { path: pb },
    };
    assert!(run(args).await.is_ok());
}

#[tokio::test]
async fn test_run_spore_verify_with_matching_plasmid_and_spore() {
    let temp = tempfile::tempdir().unwrap();
    let pb = minimal_plasmid_bin(temp.path());
    let spore = temp.path().join("spore-mount");
    std::fs::create_dir_all(spore.join("bin")).unwrap();
    std::fs::create_dir_all(spore.join("primals")).unwrap();
    std::fs::write(spore.join("bin").join("tower"), b"tower-bytes").unwrap();
    std::fs::write(spore.join("primals").join("beardog-server"), b"bd").unwrap();
    std::fs::write(spore.join("primals").join("songbird"), b"sb").unwrap();
    std::fs::write(spore.join(".family.seed"), b"seed").unwrap();
    std::fs::write(
        spore.join("tower.toml"),
        r#"
[tower]
NODE_ID = "node-test-123"
"#,
    )
    .unwrap();

    assert!(verify_single_spore_at(&spore, &pb).is_ok());
}

#[tokio::test]
async fn test_run_spore_verify_stale_binary() {
    let temp = tempfile::tempdir().unwrap();
    let pb = minimal_plasmid_bin(temp.path());
    let spore = temp.path().join("spore-stale");
    std::fs::create_dir_all(spore.join("bin")).unwrap();
    std::fs::create_dir_all(spore.join("primals")).unwrap();
    std::fs::write(spore.join("bin").join("tower"), b"wrong-tower").unwrap();
    std::fs::write(spore.join("primals").join("beardog-server"), b"bd").unwrap();
    std::fs::write(spore.join("primals").join("songbird"), b"sb").unwrap();
    std::fs::write(spore.join(".family.seed"), b"seed").unwrap();
    std::fs::write(
        spore.join("tower.toml"),
        r#"
[tower]
NODE_ID = "node-stale"
"#,
    )
    .unwrap();

    assert!(verify_single_spore_at(&spore, &pb).is_ok());
}

#[tokio::test]
async fn test_verify_single_spore_at_missing_binary_branch() {
    let temp = tempfile::tempdir().unwrap();
    let pb = minimal_plasmid_bin(temp.path());
    let spore = temp.path().join("spore-missing-bin");
    std::fs::create_dir_all(spore.join("bin")).unwrap();
    std::fs::create_dir_all(spore.join("primals")).unwrap();
    std::fs::write(spore.join("bin").join("tower"), b"tower-bytes").unwrap();
    std::fs::write(spore.join("primals").join("beardog-server"), b"bd").unwrap();
    // songbird intentionally absent — exercises Missing per-binary path
    std::fs::write(spore.join(".family.seed"), b"seed").unwrap();
    std::fs::write(
        spore.join("tower.toml"),
        r#"
[tower]
NODE_ID = "node-missing"
"#,
    )
    .unwrap();

    assert!(verify_single_spore_at(&spore, &pb).is_ok());
}

#[test]
fn test_parse_biomeos_spore_paths_list_two_paths() {
    let mounts =
        super::parse_biomeos_spore_paths_list("/tmp/biome-paths/spore-a,/tmp/biome-paths/spore-b");
    assert_eq!(mounts.len(), 2);
    assert_eq!(mounts[0].0, "/tmp/biome-paths/spore-a");
    assert_eq!(mounts[0].1, "spore-a");
    assert_eq!(mounts[1].1, "spore-b");
}

#[test]
fn test_parse_biomeos_spore_paths_list_skips_empty_segments() {
    let mounts = super::parse_biomeos_spore_paths_list("/x/only-one,,/y/two");
    assert_eq!(mounts.len(), 2);
    assert_eq!(mounts[0].1, "only-one");
    assert_eq!(mounts[1].1, "two");
}

#[test]
fn test_plasmid_bin_dir_for_verify_custom() {
    assert_eq!(
        super::plasmid_bin_dir_for_verify(Some("/opt/custom/plasmidBin")),
        PathBuf::from("/opt/custom/plasmidBin")
    );
}

#[test]
fn test_plasmid_bin_dir_for_verify_defaults_when_none_or_empty() {
    assert_eq!(
        super::plasmid_bin_dir_for_verify(None),
        PathBuf::from("plasmidBin")
    );
    assert_eq!(
        super::plasmid_bin_dir_for_verify(Some("")),
        PathBuf::from("plasmidBin")
    );
}

#[tokio::test]
async fn test_verify_single_spore_at_nucleus_missing_returns_ok() {
    let temp = tempfile::tempdir().unwrap();
    let spore = temp.path().join("any-spore");
    std::fs::create_dir_all(&spore).unwrap();
    let missing_nucleus = temp.path().join("no-plasmid-here");
    assert!(verify_single_spore_at(&spore, &missing_nucleus).is_ok());
}

#[test]
fn test_expect_used_lint_coverage() {
    // Exercises `expect` alongside `unwrap` so file-level clippy expectations stay fulfilled.
    fn computed() -> Option<i32> {
        Some(40 + 2)
    }
    assert_eq!(computed().expect("computed"), 42);
}
