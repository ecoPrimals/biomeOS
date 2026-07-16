// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Extra integration tests for [`super::cache::ModelCache`] (HF paths, errors, manifest).

#![expect(
    clippy::unwrap_used,
    reason = "test setup uses tempfile and infallible fixtures"
)]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use std::path::PathBuf;

pub(super) fn hf_models_dir(hf_hub: &std::path::Path, model_id: &str) -> PathBuf {
    hf_hub.join(format!("models--{}", model_id.replace('/', "--")))
}

mod gate_family_env;
mod huggingface;
mod manifest_resolve;
mod registration;
