// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::ModelCache`] (local cache, HF import, mesh stubs).

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::ModelCache;
use super::types::{ModelCacheConfig, ModelResolution};
mod basic_queries;
mod huggingface;
mod mesh_manifest;
mod register_local;
