// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::ModelCache;
use super::super::types::ModelCacheConfig;
use tempfile::TempDir;

#[tokio::test]
async fn test_register_model_gate_id_from_gate_id_env() {
    let tmp = TempDir::new().unwrap();
    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "default".into(),
        gate_id: "gate-from-env-8841".into(),
        hf_home: None,
    };
    let mut cache = ModelCache::with_config(config).await.unwrap();
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
async fn test_register_model_gate_id_from_hostname_env_when_gate_id_unset() {
    let tmp = TempDir::new().unwrap();

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "default".into(),
        gate_id: "host-from-env-2219".into(),
        hf_home: None,
    };
    let mut cache = ModelCache::with_config(config).await.unwrap();
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
async fn test_register_model_gate_id_fallback_reads_etc_hostname() {
    let tmp = TempDir::new().unwrap();

    let expected = std::fs::read_to_string("/etc/hostname")
        .map_or_else(|_| "unknown".to_string(), |s| s.trim().to_string());

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "default".into(),
        gate_id: expected.clone(),
        hf_home: None,
    };
    let mut cache = ModelCache::with_config(config).await.unwrap();
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
async fn test_register_model_family_id_env_chain_family_id_wins() {
    let tmp = TempDir::new().unwrap();

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "fam-primary-1".into(),
        gate_id: "g".into(),
        hf_home: None,
    };
    let mut cache = ModelCache::with_config(config).await.unwrap();
    cache
        .register_model("fam/a", &model_dir, "test://")
        .await
        .unwrap();

    assert_eq!(cache.family_id(), "fam-primary-1");
    assert_eq!(cache.list_models().len(), 1);
}

#[tokio::test]
async fn test_register_model_family_id_from_node_family_id_when_family_unset() {
    let tmp = TempDir::new().unwrap();

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "fam-node-2".into(),
        gate_id: "g".into(),
        hf_home: None,
    };
    let mut cache = ModelCache::with_config(config).await.unwrap();
    cache
        .register_model("fam/b", &model_dir, "test://")
        .await
        .unwrap();

    assert_eq!(cache.family_id(), "fam-node-2");
    assert_eq!(cache.list_models().len(), 1);
}

#[tokio::test]
async fn test_register_model_family_id_from_biomeos_family_id_when_others_unset() {
    let tmp = TempDir::new().unwrap();

    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "fam-bio-3".into(),
        gate_id: "g".into(),
        hf_home: None,
    };
    let mut cache = ModelCache::with_config(config).await.unwrap();
    cache
        .register_model("fam/c", &model_dir, "test://")
        .await
        .unwrap();

    assert_eq!(cache.family_id(), "fam-bio-3");
    assert_eq!(cache.list_models().len(), 1);
}
