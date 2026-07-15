// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

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

    let result = super::super::handle_spore_refresh_with_plasmid_dir(spore, false, nucleus).await;
    assert!(
        result.is_ok(),
        "refresh apply should succeed when spore matches nucleus: {:?}",
        result.err()
    );
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
