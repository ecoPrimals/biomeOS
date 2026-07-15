// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::dark_forest_gate::DarkForestGateConfig;
use std::collections::HashMap;

/// Gate config with sovereign checks disabled (equivalent to `BIOMEOS_SOVEREIGN=false`).
pub(super) fn gate_disabled() -> DarkForestGateConfig {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_SOVEREIGN".to_string(), "false".to_string());
    DarkForestGateConfig::from_env_map(&env)
}
