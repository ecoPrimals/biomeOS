//! System Monitor - Live Data Collection
//!
//! Collects real system metrics from /proc filesystem and system calls.
//! Replaces all mock data with actual system information.

use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SystemMonitor {
    last_cpu_times: Option<CpuTimes>,
    last_network_stats: HashMap<String, NetworkStats>,
    last_update: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct LiveSystemMetrics {
    pub timestamp: f64,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub disk_usage_percent: f32,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub load_average: (f32, f32, f32),
    pub network_rx_mbps: f32,
    pub network_tx_mbps: f32,
    pub uptime_hours: f32,
    pub process_count: u32,
    pub thread_count: u32,
}

#[derive(Debug, Clone)]
struct CpuTimes {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

#[derive(Debug, Clone)]
struct NetworkStats {
    rx_bytes: u64,
    tx_bytes: u64,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            last_cpu_times: None,
            last_network_stats: HashMap::new(),
            last_update: std::time::Instant::now(),
        }
    }

    pub fn collect_metrics(&mut self) -> Result<LiveSystemMetrics, Box<dyn std::error::Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();

        // CPU usage calculation
        let cpu_usage = self.get_cpu_usage()?;

        // Memory usage
        let (memory_usage, memory_used, memory_total) = self.get_memory_usage()?;

        // Disk usage
        let (disk_usage, disk_used, disk_total) = self.get_disk_usage()?;

        // Load average
        let load_average = self.get_load_average()?;

        // Network I/O
        let (network_rx, network_tx) = self.get_network_io()?;

        // Uptime
        let uptime_hours = self.get_uptime()?;

        // Process and thread counts
        let (process_count, thread_count) = self.get_process_counts()?;

        Ok(LiveSystemMetrics {
            timestamp: now,
            cpu_usage_percent: cpu_usage,
            memory_usage_percent: memory_usage,
            memory_used_gb: memory_used,
            memory_total_gb: memory_total,
            disk_usage_percent: disk_usage,
            disk_used_gb: disk_used,
            disk_total_gb: disk_total,
            load_average,
            network_rx_mbps: network_rx,
            network_tx_mbps: network_tx,
            uptime_hours,
            process_count,
            thread_count,
        })
    }

    fn get_cpu_usage(&mut self) -> Result<f32, Box<dyn std::error::Error>> {
        let stat_content = fs::read_to_string("/proc/stat")?;
        let cpu_line = stat_content.lines().next().ok_or("No CPU line found")?;

        let parts: Vec<&str> = cpu_line.split_whitespace().collect();
        if parts.len() < 8 {
            return Err("Invalid CPU stat format".into());
        }

        let current_times = CpuTimes {
            user: parts[1].parse()?,
            nice: parts[2].parse()?,
            system: parts[3].parse()?,
            idle: parts[4].parse()?,
            iowait: parts[5].parse()?,
            irq: parts[6].parse()?,
            softirq: parts[7].parse()?,
            steal: parts.get(8).unwrap_or(&"0").parse().unwrap_or(0),
        };

        let cpu_usage = if let Some(last_times) = &self.last_cpu_times {
            let total_diff = (current_times.user
                + current_times.nice
                + current_times.system
                + current_times.idle
                + current_times.iowait
                + current_times.irq
                + current_times.softirq
                + current_times.steal)
                - (last_times.user
                    + last_times.nice
                    + last_times.system
                    + last_times.idle
                    + last_times.iowait
                    + last_times.irq
                    + last_times.softirq
                    + last_times.steal);

            let idle_diff = current_times.idle - last_times.idle;

            if total_diff > 0 {
                ((total_diff - idle_diff) as f32 / total_diff as f32) * 100.0
            } else {
                0.0
            }
        } else {
            // First measurement, return a reasonable default
            5.0
        };

        self.last_cpu_times = Some(current_times);
        Ok(cpu_usage.min(100.0).max(0.0))
    }

    fn get_memory_usage(&self) -> Result<(f32, f32, f32), Box<dyn std::error::Error>> {
        let meminfo_content = fs::read_to_string("/proc/meminfo")?;
        let mut mem_total = 0u64;
        let mut mem_available = 0u64;

        for line in meminfo_content.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                mem_available = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            }
        }

        if mem_total == 0 {
            return Err("Could not read memory information".into());
        }

        let mem_used = mem_total - mem_available;
        let usage_percent = (mem_used as f32 / mem_total as f32) * 100.0;
        let used_gb = mem_used as f32 / 1024.0 / 1024.0;
        let total_gb = mem_total as f32 / 1024.0 / 1024.0;

        Ok((usage_percent, used_gb, total_gb))
    }

    fn get_disk_usage(&self) -> Result<(f32, f32, f32), Box<dyn std::error::Error>> {
        use nix::sys::statvfs::statvfs;
        use std::path::Path;

        let stat = statvfs(Path::new("/"))?;

        let f_frsize = stat.fragment_size();
        let f_blocks = stat.blocks();
        let f_bavail = stat.blocks_available();

        // Validate that we got reasonable values to avoid division by zero
        if f_frsize == 0 || f_blocks == 0 {
            return Err("Invalid filesystem statistics returned".into());
        }

        let total_bytes = f_blocks * f_frsize;
        let available_bytes = f_bavail * f_frsize;
        let used_bytes = total_bytes.saturating_sub(available_bytes);

        let usage_percent = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        let used_gb = used_bytes as f32 / 1024.0 / 1024.0 / 1024.0;
        let total_gb = total_bytes as f32 / 1024.0 / 1024.0 / 1024.0;

        Ok((usage_percent, used_gb, total_gb))
    }

    fn get_load_average(&self) -> Result<(f32, f32, f32), Box<dyn std::error::Error>> {
        let loadavg_content = fs::read_to_string("/proc/loadavg")?;
        let parts: Vec<&str> = loadavg_content.split_whitespace().collect();

        if parts.len() < 3 {
            return Err("Invalid loadavg format".into());
        }

        let load1: f32 = parts[0].parse()?;
        let load5: f32 = parts[1].parse()?;
        let load15: f32 = parts[2].parse()?;

        Ok((load1, load5, load15))
    }

    fn get_network_io(&mut self) -> Result<(f32, f32), Box<dyn std::error::Error>> {
        let net_dev_content = fs::read_to_string("/proc/net/dev")?;
        let mut total_rx_bytes = 0u64;
        let mut total_tx_bytes = 0u64;

        for line in net_dev_content.lines().skip(2) {
            // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let interface = parts[0].trim_end_matches(':');
                if interface != "lo" {
                    // Skip loopback interface
                    let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
                    let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
                    total_rx_bytes += rx_bytes;
                    total_tx_bytes += tx_bytes;
                }
            }
        }

        // Calculate speeds based on previous measurement
        let now = std::time::Instant::now();
        let time_diff = now.duration_since(self.last_update).as_secs_f32();

        let (rx_mbps, tx_mbps) = if time_diff > 0.0 && !self.last_network_stats.is_empty() {
            let last_total_rx = self
                .last_network_stats
                .values()
                .map(|s| s.rx_bytes)
                .sum::<u64>();
            let last_total_tx = self
                .last_network_stats
                .values()
                .map(|s| s.tx_bytes)
                .sum::<u64>();

            let rx_diff = total_rx_bytes.saturating_sub(last_total_rx);
            let tx_diff = total_tx_bytes.saturating_sub(last_total_tx);

            let rx_mbps = (rx_diff as f32 * 8.0) / (time_diff * 1_000_000.0); // Convert to Mbps
            let tx_mbps = (tx_diff as f32 * 8.0) / (time_diff * 1_000_000.0);

            (rx_mbps, tx_mbps)
        } else {
            (0.0, 0.0)
        };

        // Update stored stats
        self.last_network_stats.clear();
        self.last_network_stats.insert(
            "total".to_string(),
            NetworkStats {
                rx_bytes: total_rx_bytes,
                tx_bytes: total_tx_bytes,
            },
        );
        self.last_update = now;

        Ok((rx_mbps, tx_mbps))
    }

    fn get_uptime(&self) -> Result<f32, Box<dyn std::error::Error>> {
        let uptime_content = fs::read_to_string("/proc/uptime")?;
        let uptime_seconds: f32 = uptime_content
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse()?;

        Ok(uptime_seconds / 3600.0) // Convert to hours
    }

    fn get_process_counts(&self) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        let mut process_count = 0u32;
        let mut thread_count = 0u32;

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.chars().all(|c| c.is_ascii_digit()) {
                            process_count += 1;

                            // Count threads for this process
                            let stat_path = format!("/proc/{}/stat", filename);
                            if let Ok(stat_content) = fs::read_to_string(&stat_path) {
                                let parts: Vec<&str> = stat_content.split_whitespace().collect();
                                if parts.len() > 19 {
                                    if let Ok(threads) = parts[19].parse::<u32>() {
                                        thread_count += threads;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((process_count, thread_count))
    }
}
