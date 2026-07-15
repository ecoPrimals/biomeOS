// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_core::model_cache::ModelCacheConfig;

/// Same layout as [`ModelCacheConfig::from_env`] when only `HOME` is set (no `XDG_CACHE_HOME`).
pub(crate) fn model_cache_config_for_home(home: &std::path::Path) -> ModelCacheConfig {
    ModelCacheConfig {
        cache_dir: home.join(".cache/biomeos/models"),
        family_id: "default".to_string(),
        gate_id: "test-gate".to_string(),
        hf_home: Some(home.join(".cache/huggingface")),
    }
}
