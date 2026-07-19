// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{run_with, run_with_config};
use super::common::model_cache_config_for_home;
use crate::ModelCacheCommand;

#[tokio::test]
async fn test_run_list_uses_home_for_default_model_cache() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cfg = model_cache_config_for_home(temp.path());
    let result = run_with_config(cfg, ModelCacheCommand::List).await;
    assert!(
        result.is_ok(),
        "production run(List) with isolated HOME should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_status_production_uses_home_for_hf_scan() {
    let temp = tempfile::tempdir().expect("temp dir");
    let home = temp.path();
    let hub = home.join(".cache/huggingface/hub");
    std::fs::create_dir_all(hub.join("models--org--model-xyz")).expect("hf hub layout");
    let cfg = model_cache_config_for_home(home);
    let result = run_with_config(cfg, ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "production run(Status) with HF cache under HOME: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_resolve_production_not_found() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cfg = model_cache_config_for_home(temp.path());
    let result = run_with_config(
        cfg,
        ModelCacheCommand::Resolve {
            model_id: "no/such/model-for-run-test".to_string(),
        },
    )
    .await;
    assert!(result.is_ok(), "resolve NotFound path: {:?}", result.err());
}

#[tokio::test]
async fn test_run_import_hf_production_shows_already_cached_section() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let cache_dir = home.join(".cache/biomeos/models");
    std::fs::create_dir_all(&cache_dir).expect("cache");

    let model_dir = temp.path().join("pre-for-import-prod");
    std::fs::create_dir_all(&model_dir).expect("create");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/import-prod-existing".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let hub = home.join(".cache/huggingface/hub");
    std::fs::create_dir_all(&hub).expect("hub");

    let cfg = model_cache_config_for_home(home);
    let result = run_with_config(cfg, ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "production ImportHf with prior cache: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_import_hf_production_imports_from_default_hf_hub_layout() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let cache_dir = home.join(".cache/biomeos/models");
    std::fs::create_dir_all(&cache_dir).expect("cache");

    let hub = home.join(".cache/huggingface/hub");
    let snap = hub
        .join("models--prod--default-hub")
        .join("snapshots")
        .join("snapsha");
    std::fs::create_dir_all(&snap).expect("hf layout");
    std::fs::write(snap.join("config.json"), "{}").expect("config");

    let cfg = model_cache_config_for_home(home);
    let result = run_with_config(cfg, ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "production ImportHf from ~/.cache/huggingface/hub: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_resolve_production_local_entry() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let cache_dir = home.join(".cache/biomeos/models");
    std::fs::create_dir_all(&cache_dir).expect("cache");

    let model_dir = temp.path().join("resolve-prod-model");
    std::fs::create_dir_all(&model_dir).expect("create");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/resolve-production-local".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let cfg = model_cache_config_for_home(home);
    let result = run_with_config(
        cfg,
        ModelCacheCommand::Resolve {
            model_id: "test/resolve-production-local".to_string(),
        },
    )
    .await;
    assert!(
        result.is_ok(),
        "production resolve local branch: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_status_production_hf_unregistered_count_and_hint() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let hub = home.join(".cache/huggingface/hub");
    std::fs::create_dir_all(hub.join("models--orphan--model-xyz")).expect("hf model dir");

    let cfg = model_cache_config_for_home(home);
    let result = run_with_config(cfg, ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "production status with unregistered HF dirs: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_register_production_success_path() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    std::fs::create_dir_all(home.join(".cache/biomeos/models")).expect("cache");

    let model_dir = temp.path().join("register-prod-model");
    std::fs::create_dir_all(&model_dir).expect("create");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write");

    let cfg = model_cache_config_for_home(home);
    let result = run_with_config(
        cfg,
        ModelCacheCommand::Register {
            model_id: "test/register-production-path".to_string(),
            path: model_dir,
        },
    )
    .await;
    assert!(result.is_ok(), "production register: {:?}", result.err());
}
