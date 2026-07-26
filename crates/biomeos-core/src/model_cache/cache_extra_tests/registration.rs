// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::ModelCache;
use tempfile::TempDir;

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
