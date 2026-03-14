// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! DNS configuration utilities

/// Parse nameserver lines from resolv.conf content (testable)
pub(crate) fn parse_resolv_conf(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.trim().starts_with("nameserver"))
        .filter_map(|line| line.split_whitespace().nth(1).map(String::from))
        .collect()
}
