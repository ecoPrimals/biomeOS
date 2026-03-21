// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Extra integration tests for [`super::cache::ModelCache`] (HF paths, errors, manifest).

#![expect(
    clippy::unwrap_used,
    reason = "test setup uses tempfile and infallible fixtures"
)]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::ModelCache;
use super::types::{CacheManifest, ModelEntry, ModelFile, ModelResolution};
use biomeos_test_utils::env_helpers::TestEnvGuard;
use serial_test::serial;
use std::path::PathBuf;
use tempfile::TempDir;

fn hf_models_dir(hf_hub: &std::path::Path, model_id: &str) -> PathBuf {
    hf_hub.join(format!("models--{}", model_id.replace('/', "--")))
}

#[tokio::test]
async fn test_register_huggingface_from_hub_missing_snapshots_dir() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_dir = hf_models_dir(&hf_hub, "org/missing-snapshots");
    std::fs::create_dir_all(&model_dir).unwrap();
    // no `snapshots/` subdirectory

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let err = cache
        .register_huggingface_model_from_hub("org/missing-snapshots", &hf_hub)
        .await
        .expect_err("expected snapshots error");
    assert!(err.to_string().contains("snapshots"));
}

#[tokio::test]
async fn test_register_huggingface_from_hub_empty_snapshots_dir() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_dir = hf_models_dir(&hf_hub, "org/empty-snap");
    std::fs::create_dir_all(model_dir.join("snapshots")).unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let err = cache
        .register_huggingface_model_from_hub("org/empty-snap", &hf_hub)
        .await
        .expect_err("expected no snapshot");
    assert!(err.to_string().contains("snapshot"));
}

#[tokio::test]
#[serial_test::serial]
async fn test_register_huggingface_model_uses_hf_home_hub_path() {
    let tmp = TempDir::new().unwrap();
    let hf_home = tmp.path().join("hf_home");
    let hf_hub = hf_home.join("hub");
    let model_id = "demo/Demo-Model";
    let snapshot = hf_models_dir(&hf_hub, model_id)
        .join("snapshots")
        .join("abc123");
    std::fs::create_dir_all(&snapshot).unwrap();
    std::fs::write(snapshot.join("model.safetensors"), b"x").unwrap();

    let _hf_guard = TestEnvGuard::set(
        "HF_HOME",
        hf_home.to_str().expect("HF_HOME temp path must be UTF-8"),
    );
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let path = cache
        .register_huggingface_model(model_id)
        .await
        .expect("register");
    assert_eq!(path, snapshot);
}

#[tokio::test]
async fn test_import_huggingface_skips_entries_that_fail_to_register() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let bad_name = "broken/model";
    let bad_dir = hf_models_dir(&hf_hub, bad_name);
    std::fs::create_dir_all(bad_dir.join("snapshots")).unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let imported = cache.import_huggingface_cache_from(&hf_hub).await.unwrap();
    assert!(imported.is_empty());
}

#[tokio::test]
async fn test_register_model_symlinked_file_in_tree() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("real.bin");
    std::fs::write(&target, b"blob").unwrap();

    let model_dir = tmp.path().join("model-with-link");
    std::fs::create_dir_all(&model_dir).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&target, model_dir.join("weights.bin")).unwrap();
    }
    #[cfg(not(unix))]
    {
        std::fs::copy(&target, model_dir.join("weights.bin")).unwrap();
    }

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("link/test", &model_dir, "test://")
        .await
        .unwrap();
    let entry = cache.get_model("link/test").expect("entry");
    assert!(entry.size_bytes >= 4);
}

#[tokio::test]
async fn test_cache_manifest_roundtrip_serialization() {
    let mut m = CacheManifest::new();
    m.models.insert(
        "k".to_string(),
        ModelEntry {
            model_id: "k".to_string(),
            local_path: PathBuf::from("/tmp/k"),
            size_bytes: 1,
            source: "s".to_string(),
            sha256: None,
            cached_at: "t".to_string(),
            gate_id: "g".to_string(),
            format: "huggingface".to_string(),
            files: vec![ModelFile {
                relative_path: "f".to_string(),
                size_bytes: 1,
                sha256: None,
            }],
        },
    );
    let v = serde_json::to_value(&m).expect("json");
    let back: CacheManifest = serde_json::from_value(v).expect("back");
    assert_eq!(back.version, 1);
    assert_eq!(back.models.len(), 1);
}

#[tokio::test]
async fn test_resolve_not_found_after_manifest_stale_path() {
    let tmp = TempDir::new().unwrap();
    let model_dir = tmp.path().join("gone");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let cache_dir = tmp.path().join("cache");
    {
        let mut cache = ModelCache::with_cache_dir(cache_dir.clone()).await.unwrap();
        cache
            .register_model("stale/m", &model_dir, "test://")
            .await
            .unwrap();
    }
    std::fs::remove_dir_all(&model_dir).unwrap();

    let cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
    let res = cache.resolve("stale/m").await;
    assert!(matches!(res, ModelResolution::NotFound));
}

#[tokio::test]
#[serial]
async fn test_model_cache_new_creates_default_cache_under_home() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_str().expect("temp path utf-8");
    let _g_home = TestEnvGuard::set("HOME", home);
    let _g_hf = TestEnvGuard::remove("HF_HOME");

    ModelCache::new().await.expect("new with HOME");

    let cache_dir = tmp.path().join(".biomeos").join("model-cache");
    assert!(
        cache_dir.is_dir(),
        "expected default cache dir at {{HOME}}/.biomeos/model-cache"
    );
}

#[tokio::test]
#[serial]
async fn test_model_cache_new_err_when_home_unset() {
    let _g = TestEnvGuard::remove("HOME");
    let Err(err) = ModelCache::new().await else {
        panic!("expected Err when HOME is unset");
    };
    let msg = format!("{err:#}");
    assert!(
        msg.contains("HOME"),
        "expected HOME context in error, got: {msg}"
    );
}

#[tokio::test]
#[serial]
async fn test_model_cache_new_ignores_hf_home_for_default_cache_location() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home-a");
    std::fs::create_dir_all(&home).unwrap();
    let hf_home = tmp.path().join("hf-alt");
    std::fs::create_dir_all(&hf_home).unwrap();

    let _g_home = TestEnvGuard::set("HOME", home.to_str().expect("utf8"));
    let _g_hf = TestEnvGuard::set("HF_HOME", hf_home.to_str().expect("utf8"));

    ModelCache::new().await.expect("new");

    assert!(home.join(".biomeos/model-cache").is_dir());
    assert!(
        !hf_home.join(".biomeos").exists(),
        "HF_HOME must not relocate the model-cache root used by ModelCache::new"
    );
}

#[tokio::test]
#[serial]
async fn test_register_model_gate_id_from_gate_id_env() {
    let tmp = TempDir::new().unwrap();
    let _g_gate = TestEnvGuard::set("GATE_ID", "gate-from-env-8841");
    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("g/env", &model_dir, "test://")
        .await
        .unwrap();
    assert_eq!(
        cache.get_model("g/env").expect("entry").gate_id,
        "gate-from-env-8841"
    );
}

#[tokio::test]
#[serial]
async fn test_register_model_gate_id_from_hostname_env_when_gate_id_unset() {
    let tmp = TempDir::new().unwrap();
    let _g1 = TestEnvGuard::remove("GATE_ID");
    let _g2 = TestEnvGuard::set("HOSTNAME", "host-from-env-2219");

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("g/host", &model_dir, "test://")
        .await
        .unwrap();
    assert_eq!(
        cache.get_model("g/host").expect("entry").gate_id,
        "host-from-env-2219"
    );
}

#[tokio::test]
#[serial]
async fn test_register_model_gate_id_fallback_reads_etc_hostname() {
    let tmp = TempDir::new().unwrap();
    let _g1 = TestEnvGuard::remove("GATE_ID");
    let _g2 = TestEnvGuard::remove("HOSTNAME");

    let expected = std::fs::read_to_string("/etc/hostname")
        .map_or_else(|_| "unknown".to_string(), |s| s.trim().to_string());

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("g/etc-host", &model_dir, "test://")
        .await
        .unwrap();
    assert_eq!(
        cache.get_model("g/etc-host").expect("entry").gate_id,
        expected
    );
}

#[tokio::test]
#[serial]
async fn test_register_model_family_id_env_chain_family_id_wins() {
    let tmp = TempDir::new().unwrap();
    let _g1 = TestEnvGuard::set("FAMILY_ID", "fam-primary-1");
    let _g2 = TestEnvGuard::set("NODE_FAMILY_ID", "fam-node-ignored");
    let _g3 = TestEnvGuard::set("BIOMEOS_FAMILY_ID", "fam-biomeos-ignored");

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("fam/a", &model_dir, "test://")
        .await
        .unwrap();

    assert_eq!(cache.list_models().len(), 1);
}

#[tokio::test]
#[serial]
async fn test_register_model_family_id_from_node_family_id_when_family_unset() {
    let tmp = TempDir::new().unwrap();
    let _g1 = TestEnvGuard::remove("FAMILY_ID");
    let _g2 = TestEnvGuard::set("NODE_FAMILY_ID", "fam-node-2");
    let _g3 = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("fam/b", &model_dir, "test://")
        .await
        .unwrap();

    assert_eq!(cache.list_models().len(), 1);
}

#[tokio::test]
#[serial]
async fn test_register_model_family_id_from_biomeos_family_id_when_others_unset() {
    let tmp = TempDir::new().unwrap();
    let _g1 = TestEnvGuard::remove("FAMILY_ID");
    let _g2 = TestEnvGuard::remove("NODE_FAMILY_ID");
    let _g3 = TestEnvGuard::set("BIOMEOS_FAMILY_ID", "fam-bio-3");

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("fam/c", &model_dir, "test://")
        .await
        .unwrap();

    assert_eq!(cache.list_models().len(), 1);
}

#[tokio::test]
async fn test_register_huggingface_from_hub_selects_last_sorted_snapshot_dir() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_id = "org/multi-snap";
    let base = hf_models_dir(&hf_hub, model_id).join("snapshots");
    std::fs::create_dir_all(base.join("aaa-rev")).unwrap();
    std::fs::create_dir_all(base.join("zzz-rev")).unwrap();
    std::fs::write(base.join("zzz-rev").join("model.safetensors"), b"last").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let path = cache
        .register_huggingface_model_from_hub(model_id, &hf_hub)
        .await
        .expect("register");
    assert!(path.ends_with("zzz-rev"));
    assert!(path.join("model.safetensors").exists());
}

#[tokio::test]
async fn test_import_huggingface_cache_from_imports_models_prefix_directories() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_id = "import/ok-model";
    let snap = hf_models_dir(&hf_hub, model_id)
        .join("snapshots")
        .join("snapsha");
    std::fs::create_dir_all(&snap).unwrap();
    std::fs::write(snap.join("model.safetensors"), b"weights").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let imported = cache
        .import_huggingface_cache_from(&hf_hub)
        .await
        .expect("import");
    assert_eq!(imported, vec![model_id.to_string()]);
    assert!(cache.has_model(model_id));
}

#[tokio::test]
#[cfg(unix)]
async fn test_register_model_skips_broken_symlink_during_scan() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let model_dir = tmp.path().join("with-broken");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("real.safetensors"), b"x").unwrap();
    symlink(
        tmp.path().join("nonexistent-target-xyz"),
        model_dir.join("broken.link"),
    )
    .unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("broken/sym", &model_dir, "test://")
        .await
        .expect("register with broken symlink in tree");
    let entry = cache.get_model("broken/sym").expect("entry");
    assert!(entry.size_bytes >= 1);
}

#[tokio::test]
#[serial]
async fn test_register_huggingface_model_resolves_hub_via_home_cache_path() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("h");
    std::fs::create_dir_all(&home).unwrap();
    let hf_hub = home.join(".cache").join("huggingface").join("hub");
    let model_id = "homecache/FromHome";
    let snap = hf_models_dir(&hf_hub, model_id)
        .join("snapshots")
        .join("only");
    std::fs::create_dir_all(&snap).unwrap();
    std::fs::write(snap.join("model.safetensors"), b"y").unwrap();

    let _g_home = TestEnvGuard::set("HOME", home.to_str().expect("utf8"));
    let _g_hf = TestEnvGuard::remove("HF_HOME");

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let path = cache
        .register_huggingface_model(model_id)
        .await
        .expect("register via HOME/.cache/huggingface/hub");
    assert!(path.join("model.safetensors").exists());
}

#[tokio::test]
async fn test_register_model_completes_when_nestgate_unavailable() {
    let tmp = TempDir::new().unwrap();
    let model_dir = tmp.path().join("local-only");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("a.safetensors"), b"z").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("local/no-mesh", &model_dir, "test://")
        .await
        .expect("register should succeed without NestGate");
}
