// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

/// Evaluate a simple condition.
pub(crate) fn evaluate_condition(condition: &str, env: &HashMap<String, String>) -> bool {
    // Handle == comparison
    if let Some((left, right)) = condition.split_once("==") {
        let left = resolve_var(left.trim(), env);
        let right = right.trim();
        return left.as_str() == right;
    }

    // Handle != comparison
    if let Some((left, right)) = condition.split_once("!=") {
        let left = resolve_var(left.trim(), env);
        let right = right.trim();
        return left.as_str() != right;
    }

    // If no operator, treat as truthy check
    let value = resolve_var(condition.trim(), env);
    !value.is_empty() && value != "false" && value != "0"
}

/// Resolve a variable reference.
pub(crate) fn resolve_var(s: &str, env: &HashMap<String, String>) -> String {
    if s.starts_with("${") && s.ends_with('}') {
        let var_name = &s[2..s.len() - 1];
        env.get(var_name)
            .cloned()
            .or_else(|| std::env::var(var_name).ok())
            .unwrap_or_default()
    } else {
        s.to_string()
    }
}
