// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Pure Rust system metrics via /proc (ecoBin v3).
//!
//! Replaces sysinfo with direct /proc filesystem reads.

use std::fs;
use std::path::Path;

/// Parse MemTotal (KB) from /proc/meminfo format. Testable with const fixtures.
pub fn parse_meminfo_total(s: &str) -> u64 {
    for line in s.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(kb) = line.split_whitespace().nth(1) {
                if let Ok(k) = kb.parse::<u64>() {
                    return k * 1024;
                }
            }
        }
    }
    0
}

/// Parse MemAvailable (KB) from /proc/meminfo format. Testable with const fixtures.
pub fn parse_meminfo_available(s: &str) -> u64 {
    for line in s.lines() {
        if line.starts_with("MemAvailable:") {
            if let Some(kb) = line.split_whitespace().nth(1) {
                if let Ok(k) = kb.parse::<u64>() {
                    return k * 1024;
                }
            }
        }
    }
    0
}

/// Parse first field (1-min load) from /proc/loadavg format. Testable with const fixtures.
pub fn parse_loadavg_first(s: &str) -> f64 {
    if let Some(first) = s.split_whitespace().next() {
        if let Ok(v) = first.parse::<f64>() {
            return v;
        }
    }
    0.0
}

/// Total memory in bytes from /proc/meminfo MemTotal
pub fn total_memory() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            return parse_meminfo_total(&meminfo);
        }
    }
    0
}

/// Available memory in bytes from /proc/meminfo MemAvailable
pub fn available_memory() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            return parse_meminfo_available(&meminfo);
        }
    }
    0
}

/// CPU core count from /proc/cpuinfo
pub fn cpu_count() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            return cpuinfo
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count()
                .max(1);
        }
    }
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}

/// 1-minute load average from /proc/loadavg
pub fn load_average_one() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            return parse_loadavg_first(&loadavg);
        }
    }
    0.0
}

/// Root disk (/) total and available bytes via /proc/mounts + statvfs
pub fn root_disk_bytes() -> Option<(u64, u64)> {
    #[cfg(target_os = "linux")]
    {
        let mounts = fs::read_to_string("/proc/mounts").ok()?;
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "/" {
                let st = rustix::fs::statvfs(Path::new("/")).ok()?;
                let total = st.f_blocks.saturating_mul(st.f_frsize);
                let available = st.f_bavail.saturating_mul(st.f_frsize);
                return Some((total, available));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    const MEMINFO_SAMPLE: &str =
        "MemTotal:       16384000 kB\nMemFree:         2048000 kB\nMemAvailable:    4096000 kB\n";
    const LOADAVG_SAMPLE: &str = "1.25 0.95 0.80 2/150 12345\n";

    #[test]
    fn test_parse_meminfo_total_valid() {
        assert_eq!(parse_meminfo_total(MEMINFO_SAMPLE), 16_384_000 * 1024);
    }

    #[test]
    fn test_parse_meminfo_total_empty() {
        assert_eq!(parse_meminfo_total(""), 0);
    }

    #[test]
    fn test_parse_meminfo_total_malformed() {
        assert_eq!(parse_meminfo_total("MemTotal: invalid\n"), 0);
    }

    #[test]
    fn test_parse_meminfo_total_no_memtotal() {
        assert_eq!(
            parse_meminfo_total("MemAvailable: 1000 kB\nMemFree: 500 kB\n"),
            0
        );
    }

    #[test]
    fn test_parse_meminfo_available_valid() {
        assert_eq!(parse_meminfo_available(MEMINFO_SAMPLE), 4_096_000 * 1024);
    }

    #[test]
    fn test_parse_meminfo_available_empty() {
        assert_eq!(parse_meminfo_available(""), 0);
    }

    #[test]
    fn test_parse_meminfo_available_malformed() {
        assert_eq!(parse_meminfo_available("MemAvailable: xyz\n"), 0);
    }

    #[test]
    fn test_parse_loadavg_first_valid() {
        assert!((parse_loadavg_first(LOADAVG_SAMPLE) - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_loadavg_first_empty() {
        assert!((parse_loadavg_first("") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_loadavg_first_malformed() {
        assert!((parse_loadavg_first("not-a-number 1 2\n") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_loadavg_first_single_value() {
        assert!((parse_loadavg_first("2.50") - 2.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_meminfo_total_extra_whitespace() {
        assert_eq!(
            parse_meminfo_total("MemTotal:    16384000 kB\n"),
            16_384_000 * 1024
        );
    }

    #[test]
    fn test_parse_meminfo_total_only_label_no_value() {
        assert_eq!(parse_meminfo_total("MemTotal:\n"), 0);
    }

    #[test]
    fn test_parse_meminfo_available_only_label_no_value() {
        assert_eq!(parse_meminfo_available("MemAvailable:\n"), 0);
    }

    #[test]
    fn test_parse_meminfo_total_multiple_fields_uses_second() {
        assert_eq!(
            parse_meminfo_total("MemTotal: 8192 kB extra stuff\n"),
            8192 * 1024
        );
    }

    #[test]
    fn test_parse_loadavg_first_whitespace_only() {
        assert!((parse_loadavg_first("   \n\t  ") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_loadavg_first_trailing_newline() {
        assert!((parse_loadavg_first("1.50\n") - 1.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_loadavg_first_negative_parsed() {
        assert!((parse_loadavg_first("-0.5") - (-0.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_meminfo_total_memtotal_after_other_lines() {
        let s = "MemFree: 1000 kB\nMemTotal: 2048 kB\nMemAvailable: 500 kB\n";
        assert_eq!(parse_meminfo_total(s), 2048 * 1024);
    }

    #[test]
    fn test_parse_meminfo_available_memavailable_after_other_lines() {
        let s = "MemTotal: 1000 kB\nMemAvailable: 3000 kB\nMemFree: 500 kB\n";
        assert_eq!(parse_meminfo_available(s), 3000 * 1024);
    }
}
