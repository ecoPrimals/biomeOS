// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Pure Rust system metrics via /proc (ecoBin v3).
//! Replaces sysinfo for topology.metrics.

use std::fs;

/// Parse /proc/stat cpu line into (total_jiffies, idle_plus_iowait).
/// Testable with const fixtures.
pub fn parse_stat_cpu(s: &str) -> Option<(u64, u64)> {
    let line = s.lines().next()?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 || parts[0] != "cpu" {
        return None;
    }
    let user: u64 = parts.get(1)?.parse().ok()?;
    let nice: u64 = parts.get(2)?.parse().ok()?;
    let system: u64 = parts.get(3)?.parse().ok()?;
    let idle: u64 = parts.get(4)?.parse().ok()?;
    let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
    let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
    let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);
    let guest: u64 = parts.get(9).and_then(|s| s.parse().ok()).unwrap_or(0);
    let guest_nice: u64 = parts.get(10).and_then(|s| s.parse().ok()).unwrap_or(0);

    let total = user + nice + system + idle + iowait + irq + softirq + steal + guest + guest_nice;
    Some((total, idle + iowait))
}

/// Parse /proc/meminfo into (total_bytes, used_bytes). Used = total - available.
/// Testable with const fixtures.
pub fn parse_meminfo_bytes(s: &str) -> (u64, u64) {
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;
    for line in s.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(kb) = line.split_whitespace().nth(1) {
                let _ = kb.parse::<u64>().map(|k| total_kb = k);
            }
        } else if line.starts_with("MemAvailable:") {
            if let Some(kb) = line.split_whitespace().nth(1) {
                let _ = kb.parse::<u64>().map(|k| avail_kb = k);
            }
        }
    }
    let total = total_kb * 1024;
    let used = total.saturating_sub(avail_kb * 1024);
    (total, used)
}

/// Parse /proc/uptime first field (seconds). Testable with const fixtures.
pub fn parse_uptime_seconds(s: &str) -> u64 {
    if let Some(first) = s.split_whitespace().next() {
        if let Ok(secs) = first.parse::<f64>() {
            return secs as u64;
        }
    }
    0
}

/// CPU usage 0-100 from /proc/stat (requires two samples)
pub async fn cpu_percent() -> f64 {
    #[cfg(target_os = "linux")]
    {
        let (t1, i1) = read_cpu_jiffies().unwrap_or((0, 0));
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let (t2, i2) = read_cpu_jiffies().unwrap_or((0, 0));
        let total_delta = t2.saturating_sub(t1);
        let idle_delta = i2.saturating_sub(i1);
        if total_delta == 0 {
            return 0.0;
        }
        let usage = 1.0 - (idle_delta as f64 / total_delta as f64);
        usage.clamp(0.0, 1.0) * 100.0
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = tokio::time::Duration::from_millis(1);
        0.0
    }
}

#[cfg(target_os = "linux")]
fn read_cpu_jiffies() -> Option<(u64, u64)> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    parse_stat_cpu(&stat)
}

/// (total_bytes, used_bytes) from /proc/meminfo
pub fn memory_bytes() -> (u64, u64) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            return parse_meminfo_bytes(&meminfo);
        }
    }

    #[cfg(not(target_os = "linux"))]
    {}
    (0, 0)
}

/// Uptime in seconds from /proc/uptime
pub fn uptime_seconds() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(uptime) = fs::read_to_string("/proc/uptime") {
            return parse_uptime_seconds(&uptime);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    // First line is aggregate CPU; format: cpu user nice system idle iowait irq softirq steal guest guest_nice
    const STAT_SAMPLE: &str = "cpu 100 50 200 5000 100 0 25 0 0 0\ncpu0 10 1 20 50 1 0 0 0 0 0\n";
    const MEMINFO_SAMPLE: &str = "MemTotal:       16384000 kB\nMemAvailable:    4096000 kB\n";
    const UPTIME_SAMPLE: &str = "12345.67 98765.43\n";

    #[test]
    fn test_parse_stat_cpu_valid() {
        let (total, idle_plus) = parse_stat_cpu(STAT_SAMPLE).unwrap();
        // user=100 + nice=50 + system=200 + idle=5000 + iowait=100 + irq=0 + softirq=25
        assert_eq!(total, 100 + 50 + 200 + 5000 + 100 + 25);
        assert_eq!(idle_plus, 5000 + 100); // idle + iowait
    }

    #[test]
    fn test_parse_stat_cpu_minimal() {
        let s = "cpu  100 0 0 500 0\n";
        let (total, idle) = parse_stat_cpu(s).unwrap();
        assert_eq!(total, 600);
        assert_eq!(idle, 500);
    }

    #[test]
    fn test_parse_stat_cpu_empty() {
        assert!(parse_stat_cpu("").is_none());
    }

    #[test]
    fn test_parse_stat_cpu_wrong_prefix() {
        // cpu0 is per-CPU line, not aggregate - aggregate must start with "cpu "
        assert!(parse_stat_cpu("cpu0 100 0 0 0 0 0 0 0 0 0\n").is_none());
    }

    #[test]
    fn test_parse_stat_cpu_too_few_fields() {
        assert!(parse_stat_cpu("cpu 1 2 3\n").is_none());
    }

    #[test]
    fn test_parse_meminfo_bytes_valid() {
        let (total, used) = parse_meminfo_bytes(MEMINFO_SAMPLE);
        assert_eq!(total, 16_384_000 * 1024);
        assert_eq!(used, (16_384_000 - 4_096_000) * 1024);
    }

    #[test]
    fn test_parse_meminfo_bytes_empty() {
        let (total, used) = parse_meminfo_bytes("");
        assert_eq!(total, 0);
        assert_eq!(used, 0);
    }

    #[test]
    fn test_parse_meminfo_bytes_malformed() {
        let (total, used) = parse_meminfo_bytes("MemTotal: xyz\nMemAvailable: 1000 kB\n");
        assert_eq!(total, 0);
        assert_eq!(used, 0);
    }

    #[test]
    fn test_parse_uptime_seconds_valid() {
        assert_eq!(parse_uptime_seconds(UPTIME_SAMPLE), 12345);
    }

    #[test]
    fn test_parse_uptime_seconds_empty() {
        assert_eq!(parse_uptime_seconds(""), 0);
    }

    #[test]
    fn test_parse_uptime_seconds_malformed() {
        assert_eq!(parse_uptime_seconds("invalid"), 0);
    }
}
