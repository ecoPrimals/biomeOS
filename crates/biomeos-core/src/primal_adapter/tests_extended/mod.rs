// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

// Additional comprehensive tests for primal adapter
use super::*;
use crate::primal_adapter::types::{HealthCheckConfig, PortConfigMethod, PrimalState};
use std::path::PathBuf;
use std::time::Duration;

mod adapter;
mod config;
mod interface;
