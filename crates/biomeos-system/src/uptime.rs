// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! System uptime.

use std::fs;

use biomeos_types::{BiomeError, BiomeResult};

/// Get system uptime by reading from /proc/uptime on Linux.
///
/// Returns an error if /proc/uptime is not available (e.g. on non-Linux)
/// or cannot be read/parsed.
pub fn get_uptime() -> BiomeResult<std::time::Duration> {
    let uptime_str = fs::read_to_string("/proc/uptime").map_err(|e| {
        BiomeError::internal_error(
            format!("Cannot read /proc/uptime: {e}"),
            Some("UPTIME_READ_FAILED"),
        )
    })?;

    let uptime_seconds = uptime_str.split_whitespace().next().ok_or_else(|| {
        BiomeError::internal_error(
            "Empty or invalid /proc/uptime content",
            Some("UPTIME_PARSE_EMPTY"),
        )
    })?;

    let seconds: f64 = uptime_seconds.parse().map_err(|_| {
        BiomeError::internal_error(
            format!("Invalid uptime value in /proc/uptime: {uptime_seconds}"),
            Some("UPTIME_PARSE_INVALID"),
        )
    })?;

    Ok(std::time::Duration::from_secs_f64(seconds))
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime() {
        let uptime = get_uptime().expect("get_uptime should succeed");
        assert!(uptime.as_secs() > 0, "uptime should be positive");
    }
}
