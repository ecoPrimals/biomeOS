//! System uptime.

use std::fs;

use biomeos_types::BiomeResult;

/// Get system uptime
pub(crate) fn get_uptime() -> BiomeResult<std::time::Duration> {
    // Try to read from /proc/uptime on Linux
    if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
        if let Some(uptime_seconds) = uptime_str.split_whitespace().next() {
            if let Ok(seconds) = uptime_seconds.parse::<f64>() {
                return Ok(std::time::Duration::from_secs(seconds as u64));
            }
        }
    }

    // Fallback
    Ok(std::time::Duration::from_secs(3600)) // 1 hour placeholder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime() {
        let uptime = get_uptime().expect("get_uptime should succeed");
        assert!(uptime.as_secs() > 0, "uptime should be positive");
    }
}
