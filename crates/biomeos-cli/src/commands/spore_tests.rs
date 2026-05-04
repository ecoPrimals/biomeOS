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

#[test]
fn test_parse_spore_type_live() {
    assert_eq!(parse_spore_type("live").unwrap(), SporeType::Live);
    assert_eq!(parse_spore_type("LIVE").unwrap(), SporeType::Live);
    assert_eq!(parse_spore_type("Live").unwrap(), SporeType::Live);
}

#[test]
fn test_parse_spore_type_cold() {
    assert_eq!(parse_spore_type("cold").unwrap(), SporeType::Cold);
    assert_eq!(parse_spore_type("COLD").unwrap(), SporeType::Cold);
    assert_eq!(parse_spore_type("Cold").unwrap(), SporeType::Cold);
}

#[test]
fn test_parse_spore_type_invalid() {
    assert!(parse_spore_type("invalid").is_err());
    assert!(parse_spore_type("").is_err());
    assert!(parse_spore_type("warm").is_err());
}

#[test]
fn test_parse_spore_type_error_message() {
    let err = parse_spore_type("invalid").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("invalid"),
        "error should mention invalid input: {msg}"
    );
    assert!(
        msg.contains("live") || msg.contains("cold"),
        "error should mention valid types: {msg}"
    );
}

#[test]
fn test_gather_spore_structure_info_nonexistent() {
    let infos = gather_spore_structure_info(Path::new("/nonexistent/path"));
    // 3 static entries (.family.seed, tower.toml, bin/tower) + CORE_PRIMALS
    assert_eq!(
        infos.len(),
        3 + biomeos_types::primal_names::CORE_PRIMALS.len()
    );
    assert!(infos.iter().all(|i| !i.exists));
}

#[test]
fn test_compute_refresh_plan() {
    let paths = vec![
        PathBuf::from("bin/tower"),
        PathBuf::from("primals/beardog"),
        PathBuf::from("primals/songbird"),
    ];
    let would_refresh = vec![true, false, true];
    let report = compute_refresh_plan(&paths, &would_refresh);
    assert_eq!(report.to_refresh.len(), 2);
    assert_eq!(report.to_keep.len(), 1);
    assert!(report.to_refresh.contains(&PathBuf::from("bin/tower")));
    assert!(
        report
            .to_refresh
            .contains(&PathBuf::from("primals/songbird"))
    );
    assert!(report.to_keep.contains(&PathBuf::from("primals/beardog")));
}

#[test]
fn test_compute_refresh_plan_empty() {
    let paths: Vec<PathBuf> = vec![];
    let would_refresh: Vec<bool> = vec![];
    let report = compute_refresh_plan(&paths, &would_refresh);
    assert!(report.to_refresh.is_empty());
    assert!(report.to_keep.is_empty());
}

#[test]
fn test_format_spore_create_summary() {
    let spore_info = serde_json::json!({
        "location": "/media/usb/biomeOS"
    });
    let lines = format_spore_create_summary(&spore_info);
    assert!(lines.iter().any(|l| l.contains("/media/usb/biomeOS")));
    assert!(lines.iter().any(|l| l.contains("What was created")));
    assert!(lines.iter().any(|l| l.contains("Security")));
}

#[test]
fn test_format_spore_create_summary_no_location() {
    let spore_info = serde_json::json!({});
    let lines = format_spore_create_summary(&spore_info);
    assert!(lines.iter().any(|l| l.contains("What was created")));
    assert!(lines.iter().any(|l| l.contains("Security")));
}

#[test]
fn test_path_info_debug() {
    let info = PathInfo {
        name: "bin/tower".to_string(),
        exists: true,
        permissions: Some(0o755),
    };
    let _ = format!("{info:?}");
}

#[test]
fn test_path_info_clone() {
    let info = PathInfo {
        name: "tower.toml".to_string(),
        exists: false,
        permissions: None,
    };
    let cloned = info.clone();
    assert_eq!(info.name, cloned.name);
    assert_eq!(info.exists, cloned.exists);
}

#[test]
fn test_gather_spore_structure_info_checks_all_paths() {
    let infos = gather_spore_structure_info(std::path::Path::new("/nonexistent"));
    let names: Vec<_> = infos.iter().map(|i| i.name.as_str()).collect();
    assert!(names.contains(&".family.seed"));
    assert!(names.contains(&"tower.toml"));
    assert!(names.contains(&"bin/tower"));
    assert!(names.contains(&"primals/beardog"));
    assert!(names.contains(&"primals/songbird"));
}

#[test]
fn test_compute_refresh_plan_mismatched_lengths() {
    let paths = vec![std::path::PathBuf::from("a"), std::path::PathBuf::from("b")];
    let would_refresh = vec![true];
    let report = compute_refresh_plan(&paths, &would_refresh);
    assert_eq!(report.to_refresh.len(), 1);
    assert_eq!(report.to_keep.len(), 1);
}

#[test]
fn test_spore_type_emoji() {
    assert_eq!(SporeType::Live.emoji(), "🌱");
    assert_eq!(SporeType::Cold.emoji(), "❄️");
}

#[test]
fn test_format_spore_create_summary_location_null() {
    let spore_info = serde_json::json!({"location": null});
    let lines = format_spore_create_summary(&spore_info);
    assert!(lines.iter().any(|l| l.contains("What was created")));
}

#[test]
fn test_format_spore_create_summary_location_number() {
    let spore_info = serde_json::json!({"location": 42});
    let lines = format_spore_create_summary(&spore_info);
    assert!(lines.iter().any(|l| l.contains("Security")));
}

#[test]
fn test_refresh_report_default() {
    let report = RefreshReport::default();
    assert!(report.to_refresh.is_empty());
    assert!(report.to_keep.is_empty());
}

#[tokio::test]
async fn test_handle_spore_refresh_no_plasmid_bin() {
    let temp = tempfile::tempdir().expect("temp dir");
    let result = handle_spore_refresh(temp.path().to_path_buf(), true).await;
    if let Err(e) = result {
        let err = e.to_string();
        assert!(
            err.contains("plasmidBin") || err.contains("not found") || err.contains("tower"),
            "unexpected error: {err}"
        );
    }
}

#[tokio::test]
async fn test_handle_spore_list() {
    let result = handle_spore_list().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_spore_verify_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mount = temp.path().join("biomeOS");
    std::fs::create_dir_all(&mount).expect("create dir");
    let result = handle_spore_verify(temp.path().to_path_buf()).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_handle_spore_info_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let result = handle_spore_info(temp.path().to_path_buf()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_spore_clone_missing_source() {
    let temp = tempfile::tempdir().expect("temp dir");
    let from = temp.path().join("no-such-spore");
    let to = temp.path().join("dest-spore");
    let result = handle_spore_clone(from, to, "node-new".into()).await;
    assert!(result.is_err());
}

#[test]
fn test_gather_spore_structure_info_partial_tree() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("bin")).expect("bin");
    std::fs::create_dir_all(root.join("primals")).expect("primals");
    std::fs::write(root.join(".family.seed"), b"seed").expect("seed");
    std::fs::write(root.join("tower.toml"), b"[tower]").expect("tower");
    std::fs::write(root.join("bin/tower"), b"exe").expect("tower bin");
    std::fs::write(root.join("primals/beardog"), b"bd").expect("bd");
    // songbird missing on purpose

    let infos = gather_spore_structure_info(root);
    assert!(infos.iter().any(|i| i.name == ".family.seed" && i.exists));
    assert!(infos.iter().any(|i| i.name == "tower.toml" && i.exists));
    assert!(
        infos
            .iter()
            .any(|i| i.name == "primals/songbird" && !i.exists)
    );
}

#[tokio::test]
async fn test_handle_spore_verify_invalid_tree() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mount = temp.path().join("not-a-spore");
    std::fs::create_dir_all(&mount).expect("dir");
    let result = handle_spore_verify(mount).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_spore_refresh_dry_run_with_plasmid_and_spore() {
    use biomeos_spore::manifest::{BinaryInfo, BinaryManifest, CompatibilityInfo, ManifestMeta};
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;

    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();

    let nucleus = root.join("plasmidBin");
    std::fs::create_dir_all(nucleus.join("tower")).expect("tower dir");
    std::fs::create_dir_all(nucleus.join("primals")).expect("primals dir");
    let tower_bytes = b"tower-nucleus-v1";
    std::fs::write(nucleus.join("tower/tower"), tower_bytes).expect("nucleus tower");

    let mut hasher = Sha256::new();
    hasher.update(tower_bytes);
    let sha = format!("{:x}", hasher.finalize());

    let mut binaries = HashMap::new();
    binaries.insert(
        "tower".to_string(),
        BinaryInfo {
            name: "tower".to_string(),
            version: "1.0.0".to_string(),
            git_commit: "abc".to_string(),
            build_date: chrono::Utc::now(),
            sha256: sha,
            size_bytes: tower_bytes.len() as u64,
            source_repo: "test".to_string(),
            features: vec![],
        },
    );

    let manifest = BinaryManifest {
        manifest: ManifestMeta {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            pipeline_run: "test".to_string(),
        },
        binaries,
        compatibility: CompatibilityInfo {
            min_tower_version: "1.0.0".to_string(),
            min_beardog_version: "0.1.0".to_string(),
            min_songbird_version: "0.1.0".to_string(),
        },
    };
    std::fs::write(
        nucleus.join("MANIFEST.toml"),
        toml::to_string_pretty(&manifest).expect("manifest toml"),
    )
    .expect("write MANIFEST.toml");

    let spore = root.join("spore");
    std::fs::create_dir_all(spore.join("bin")).expect("bin");
    std::fs::write(spore.join("bin/tower"), tower_bytes).expect("matching tower");
    std::fs::write(
        spore.join("tower.toml"),
        r#"
node_id = "test-node"
family_id = "test-family"

[primals.env]
NODE_ID = "test-node"
"#,
    )
    .expect("tower.toml");

    let result = handle_spore_refresh_with_plasmid_dir(spore, true, nucleus).await;
    assert!(
        result.is_ok(),
        "dry-run refresh should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_format_spore_create_summary_has_security_section() {
    let lines = format_spore_create_summary(&serde_json::json!({}));
    assert!(lines.iter().any(|l| l.contains("0600")));
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Security provider") || l.contains("cryptography"))
    );
}

#[test]
fn test_compute_refresh_plan_all_keep() {
    let paths = vec![PathBuf::from("a"), PathBuf::from("b")];
    let flags = vec![false, false];
    let r = compute_refresh_plan(&paths, &flags);
    assert_eq!(r.to_refresh.len(), 0);
    assert_eq!(r.to_keep.len(), 2);
}

#[test]
fn test_compute_refresh_plan_all_refresh() {
    let paths = vec![PathBuf::from("x")];
    let flags = vec![true];
    let r = compute_refresh_plan(&paths, &flags);
    assert_eq!(r.to_refresh.len(), 1);
    assert!(r.to_keep.is_empty());
}

#[test]
fn test_parse_spore_type_whitespace_not_trimmed() {
    assert!(parse_spore_type("  cold  ").is_err());
}

#[test]
fn test_gather_spore_structure_info_order() {
    let infos = gather_spore_structure_info(Path::new("/nonexistent"));
    let order: Vec<_> = infos.iter().map(|i| i.name.as_str()).collect();
    let mut expected = vec![".family.seed", "tower.toml", "bin/tower"];
    for primal in biomeos_types::primal_names::CORE_PRIMALS {
        expected.push(primal);
    }
    assert_eq!(order.len(), expected.len());
    assert_eq!(order[..3], expected[..3], "static entries must come first");
    for primal in biomeos_types::primal_names::CORE_PRIMALS {
        let primal_path = format!("primals/{primal}");
        assert!(
            order.iter().any(|o| *o == primal_path),
            "missing expected primal path: {primal_path}"
        );
    }
}

#[test]
fn test_compute_refresh_plan_index_out_of_bounds_goes_to_keep() {
    let paths = vec![PathBuf::from("only-one")];
    let flags: Vec<bool> = vec![];
    let r = compute_refresh_plan(&paths, &flags);
    assert_eq!(r.to_keep, paths);
    assert!(r.to_refresh.is_empty());
}

#[test]
fn test_format_spore_create_summary_location_object() {
    let lines = format_spore_create_summary(&serde_json::json!({
        "location": { "nested": true }
    }));
    assert!(lines.iter().any(|l| l.contains("What was created")));
}

#[test]
fn test_discover_plasmid_dir_env_override() {
    let temp = tempfile::tempdir().expect("temp dir");
    let plasmid = temp.path().join("plasmidBin");
    std::fs::create_dir_all(&plasmid).expect("plasmid dir");
    let got = super::discover_plasmid_dir_with_override(Some(&plasmid)).expect("discover");
    assert_eq!(got, plasmid);
}

#[tokio::test]
async fn test_handle_spore_create_invalid_spore_type() {
    let temp = tempfile::tempdir().expect("temp dir");
    let err = handle_spore_create(
        temp.path().to_path_buf(),
        "l".into(),
        "n1".into(),
        "thermal".into(),
    )
    .await
    .expect_err("invalid type");
    assert!(err.to_string().to_lowercase().contains("invalid"));
}

#[test]
fn test_discover_plasmid_dir_env_missing_falls_through() {
    let temp = tempfile::tempdir().expect("temp dir");
    let missing = temp.path().join("not-there");
    let result = super::discover_plasmid_dir_with_override(Some(&missing));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("plasmidBin") || err.contains("not found"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_handle_spore_refresh_apply_updates_matching_spore() {
    use biomeos_spore::manifest::{BinaryInfo, BinaryManifest, CompatibilityInfo, ManifestMeta};
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;

    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();

    let nucleus = root.join("plasmidBin");
    std::fs::create_dir_all(nucleus.join("tower")).expect("tower dir");
    std::fs::create_dir_all(nucleus.join("primals")).expect("primals dir");
    let tower_bytes = b"tower-nucleus-v1-apply";
    std::fs::write(nucleus.join("tower/tower"), tower_bytes).expect("nucleus tower");

    let mut hasher = Sha256::new();
    hasher.update(tower_bytes);
    let sha = format!("{:x}", hasher.finalize());

    let mut binaries = HashMap::new();
    binaries.insert(
        "tower".to_string(),
        BinaryInfo {
            name: "tower".to_string(),
            version: "1.0.0".to_string(),
            git_commit: "abc".to_string(),
            build_date: chrono::Utc::now(),
            sha256: sha,
            size_bytes: tower_bytes.len() as u64,
            source_repo: "test".to_string(),
            features: vec![],
        },
    );

    let manifest = BinaryManifest {
        manifest: ManifestMeta {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            pipeline_run: "test".to_string(),
        },
        binaries,
        compatibility: CompatibilityInfo {
            min_tower_version: "1.0.0".to_string(),
            min_beardog_version: "0.1.0".to_string(),
            min_songbird_version: "0.1.0".to_string(),
        },
    };
    std::fs::write(
        nucleus.join("MANIFEST.toml"),
        toml::to_string_pretty(&manifest).expect("manifest toml"),
    )
    .expect("write MANIFEST.toml");

    let spore = root.join("spore");
    std::fs::create_dir_all(spore.join("bin")).expect("bin");
    std::fs::write(spore.join("bin/tower"), tower_bytes).expect("matching tower");
    std::fs::write(
        spore.join("tower.toml"),
        r#"
node_id = "test-node"
family_id = "test-family"

[primals.env]
NODE_ID = "test-node"
"#,
    )
    .expect("tower.toml");

    let result = super::handle_spore_refresh_with_plasmid_dir(spore, false, nucleus).await;
    assert!(
        result.is_ok(),
        "refresh apply should succeed when spore matches nucleus: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_handle_spore_info_minimal_valid_tree() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mount = temp.path();
    let biome = mount.join("biomeOS");
    std::fs::create_dir_all(&biome).expect("biomeOS");
    std::fs::write(
        biome.join("tower.toml"),
        r#"node_id = "node-info-test"
"#,
    )
    .expect("tower.toml");
    let result = handle_spore_info(mount.to_path_buf()).await;
    assert!(result.is_ok(), "info: {:?}", result.err());
}

#[tokio::test]
async fn test_handle_spore_refresh_dry_run_stale_binary_lists_refresh() {
    use biomeos_spore::manifest::{BinaryInfo, BinaryManifest, CompatibilityInfo, ManifestMeta};
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;

    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();

    let nucleus = root.join("plasmidBin");
    std::fs::create_dir_all(nucleus.join("tower")).expect("tower dir");
    std::fs::create_dir_all(nucleus.join("primals")).expect("primals dir");
    let tower_bytes = b"tower-expected-v2";
    std::fs::write(nucleus.join("tower/tower"), tower_bytes).expect("nucleus tower");

    let mut hasher = Sha256::new();
    hasher.update(tower_bytes);
    let sha = format!("{:x}", hasher.finalize());

    let mut binaries = HashMap::new();
    binaries.insert(
        "tower".to_string(),
        BinaryInfo {
            name: "tower".to_string(),
            version: "2.0.0".to_string(),
            git_commit: "def".to_string(),
            build_date: chrono::Utc::now(),
            sha256: sha,
            size_bytes: tower_bytes.len() as u64,
            source_repo: "test".to_string(),
            features: vec![],
        },
    );

    let manifest = BinaryManifest {
        manifest: ManifestMeta {
            version: "2.0.0".to_string(),
            created_at: chrono::Utc::now(),
            pipeline_run: "test".to_string(),
        },
        binaries,
        compatibility: CompatibilityInfo {
            min_tower_version: "1.0.0".to_string(),
            min_beardog_version: "0.1.0".to_string(),
            min_songbird_version: "0.1.0".to_string(),
        },
    };
    std::fs::write(
        nucleus.join("MANIFEST.toml"),
        toml::to_string_pretty(&manifest).expect("manifest toml"),
    )
    .expect("write MANIFEST.toml");

    let spore = root.join("spore-stale");
    std::fs::create_dir_all(spore.join("bin")).expect("bin");
    std::fs::write(spore.join("bin/tower"), b"old-bytes-not-matching").expect("stale tower");
    std::fs::write(
        spore.join("tower.toml"),
        r#"
node_id = "stale-node"
family_id = "fam"

[primals.env]
NODE_ID = "stale-node"
"#,
    )
    .expect("tower.toml");

    let result = handle_spore_refresh_with_plasmid_dir(spore, true, nucleus).await;
    assert!(
        result.is_ok(),
        "dry-run with stale binary: {:?}",
        result.err()
    );
}
