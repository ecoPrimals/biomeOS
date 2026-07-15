// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

pub(super) fn mock_env(vars: &HashMap<String, String>) -> impl Fn(&str) -> Option<String> + '_ {
    move |key: &str| vars.get(key).cloned()
}
