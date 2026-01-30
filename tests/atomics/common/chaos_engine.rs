// Chaos Engineering Infrastructure for NUCLEUS Testing

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use anyhow::{Result, Context};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

/// Chaos testing engine
pub struct ChaosEngine {
    active_scenarios: Vec<ActiveScenario>,
}

/// Active chaos scenario
struct ActiveScenario {
    scenario: ChaosScenario,
    cleanup: Option<Box<dyn FnOnce() -> Result<()>>>,
}

/// Chaos scenario types
#[derive(Debug, Clone)]
pub enum ChaosScenario {
    /// Kill process with specified signal
    ProcessTermination {
        pid: u32,
        signal: Signal,
    },
    
    /// Simulate network partition
    NetworkPartition {
        duration: Duration,
    },
    
    /// Corrupt socket file
    SocketCorruption {
        socket_path: PathBuf,
    },
    
    /// Inject CPU load
    CpuLoad {
        percentage: u8,
        duration: Duration,
    },
    
    /// Create memory pressure
    MemoryPressure {
        mb: usize,
        duration: Duration,
    },
    
    /// Fill disk space
    DiskFull {
        path: PathBuf,
        size_mb: usize,
    },
    
    /// Inject network latency
    LatencyInjection {
        delay_ms: u64,
    },
}

impl ChaosEngine {
    /// Create new chaos engine
    pub fn new() -> Self {
        Self {
            active_scenarios: Vec::new(),
        }
    }
    
    /// Inject chaos scenario
    pub async fn inject(&mut self, scenario: ChaosScenario) -> Result<()> {
        match &scenario {
            ChaosScenario::ProcessTermination { pid, signal } => {
                self.kill_process(*pid, *signal)?;
            }
            
            ChaosScenario::NetworkPartition { duration } => {
                self.partition_network(*duration).await?;
            }
            
            ChaosScenario::SocketCorruption { socket_path } => {
                self.corrupt_socket(socket_path)?;
            }
            
            ChaosScenario::CpuLoad { percentage, duration } => {
                self.inject_cpu_load(*percentage, *duration).await?;
            }
            
            ChaosScenario::MemoryPressure { mb, duration } => {
                self.inject_memory_pressure(*mb, *duration).await?;
            }
            
            ChaosScenario::DiskFull { path, size_mb } => {
                self.fill_disk(path, *size_mb)?;
            }
            
            ChaosScenario::LatencyInjection { delay_ms } => {
                self.inject_latency(*delay_ms).await?;
            }
        }
        
        Ok(())
    }
    
    /// Kill process with signal
    fn kill_process(&self, pid: u32, signal: Signal) -> Result<()> {
        signal::kill(Pid::from_raw(pid as i32), signal)
            .context(format!("Failed to send signal {:?} to PID {}", signal, pid))?;
        Ok(())
    }
    
    /// Simulate network partition using iptables
    async fn partition_network(&self, duration: Duration) -> Result<()> {
        // Drop all outgoing packets
        Command::new("sudo")
            .args(&["iptables", "-A", "OUTPUT", "-j", "DROP"])
            .output()
            .context("Failed to inject network partition")?;
        
        // Wait for duration
        tokio::time::sleep(duration).await;
        
        // Restore network
        Command::new("sudo")
            .args(&["iptables", "-D", "OUTPUT", "-j", "DROP"])
            .output()
            .context("Failed to restore network")?;
        
        Ok(())
    }
    
    /// Corrupt socket file
    fn corrupt_socket(&self, socket_path: &PathBuf) -> Result<()> {
        // Change permissions to make socket inaccessible
        std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o000))
            .context("Failed to corrupt socket permissions")?;
        Ok(())
    }
    
    /// Inject CPU load using stress-ng
    async fn inject_cpu_load(&self, percentage: u8, duration: Duration) -> Result<()> {
        let cpu_count = num_cpus::get();
        let workers = (cpu_count as f64 * percentage as f64 / 100.0).ceil() as usize;
        
        let mut child = Command::new("stress-ng")
            .args(&[
                "--cpu", &workers.to_string(),
                "--timeout", &format!("{}s", duration.as_secs()),
            ])
            .spawn()
            .context("Failed to start stress-ng")?;
        
        tokio::time::sleep(duration).await;
        let _ = child.kill();
        
        Ok(())
    }
    
    /// Inject memory pressure
    async fn inject_memory_pressure(&self, mb: usize, duration: Duration) -> Result<()> {
        let mut child = Command::new("stress-ng")
            .args(&[
                "--vm", "1",
                "--vm-bytes", &format!("{}M", mb),
                "--timeout", &format!("{}s", duration.as_secs()),
            ])
            .spawn()
            .context("Failed to start memory pressure")?;
        
        tokio::time::sleep(duration).await;
        let _ = child.kill();
        
        Ok(())
    }
    
    /// Fill disk with dummy file
    fn fill_disk(&self, path: &PathBuf, size_mb: usize) -> Result<()> {
        let dummy_file = path.join("chaos_dummy.bin");
        
        Command::new("dd")
            .args(&[
                "if=/dev/zero",
                &format!("of={}", dummy_file.display()),
                "bs=1M",
                &format!("count={}", size_mb),
            ])
            .output()
            .context("Failed to fill disk")?;
        
        Ok(())
    }
    
    /// Inject network latency using tc (traffic control)
    async fn inject_latency(&self, delay_ms: u64) -> Result<()> {
        // Add latency to loopback interface
        Command::new("sudo")
            .args(&[
                "tc", "qdisc", "add", "dev", "lo", "root",
                "netem", "delay", &format!("{}ms", delay_ms),
            ])
            .output()
            .context("Failed to inject latency")?;
        
        Ok(())
    }
    
    /// Remove network latency
    pub async fn clear_latency(&self) -> Result<()> {
        Command::new("sudo")
            .args(&["tc", "qdisc", "del", "dev", "lo", "root"])
            .output()
            .context("Failed to clear latency")?;
        
        Ok(())
    }
    
    /// Recover from chaos scenario
    pub async fn recover(&mut self, scenario: ChaosScenario) -> Result<()> {
        match scenario {
            ChaosScenario::SocketCorruption { socket_path } => {
                // Restore socket permissions
                std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o755))?;
            }
            
            ChaosScenario::DiskFull { path, .. } => {
                // Remove dummy file
                let dummy_file = path.join("chaos_dummy.bin");
                std::fs::remove_file(dummy_file)?;
            }
            
            ChaosScenario::LatencyInjection { .. } => {
                self.clear_latency().await?;
            }
            
            _ => {
                // Most scenarios auto-recover or require manual intervention
            }
        }
        
        Ok(())
    }
}

impl Default for ChaosEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Helper to get the mode for permissions
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
trait PermissionsMode {
    fn from_mode(mode: u32) -> Self;
}

#[cfg(unix)]
impl PermissionsMode for std::fs::Permissions {
    fn from_mode(mode: u32) -> Self {
        std::fs::Permissions::from_mode(mode)
    }
}
