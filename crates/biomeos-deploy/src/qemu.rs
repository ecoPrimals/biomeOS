//! QEMU instance management

use crate::error::{DeployError, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tracing::{info, warn};
use uuid::Uuid;

/// QEMU configuration
#[derive(Debug, Clone)]
pub struct QemuConfig {
    /// VM name
    pub name: String,

    /// Memory size (MB)
    pub memory: u32,

    /// Number of CPUs
    pub cpus: u32,

    /// Disk image path
    pub disk_image: PathBuf,

    /// Network bridge name
    pub bridge_name: String,

    /// MAC address
    pub mac_address: String,

    /// Serial log file
    pub serial_log: PathBuf,

    /// Enable KVM acceleration
    pub enable_kvm: bool,

    /// Additional QEMU arguments
    pub extra_args: Vec<String>,
}

/// QEMU instance
pub struct QemuInstance {
    id: Uuid,
    config: QemuConfig,
    process: Option<Child>,
}

impl QemuInstance {
    /// Create a new QEMU instance
    pub fn new(config: QemuConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            process: None,
        }
    }

    /// Get instance ID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get VM name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Check if the instance is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            // Try to check if process is still alive (non-blocking)
            matches!(process.try_wait(), Ok(None))
        } else {
            false
        }
    }

    /// Start the QEMU instance
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(DeployError::QemuProcess {
                message: format!("VM {} is already running", self.config.name),
            });
        }

        info!("Starting QEMU instance: {}", self.config.name);

        // Build QEMU command
        let mut cmd = Command::new("qemu-system-x86_64");

        // Basic configuration
        cmd.arg("-name").arg(&self.config.name);
        cmd.arg("-m").arg(self.config.memory.to_string());
        cmd.arg("-smp").arg(self.config.cpus.to_string());

        // KVM acceleration
        if self.config.enable_kvm {
            cmd.arg("-enable-kvm");
            cmd.arg("-cpu").arg("host");
        }

        // Disk
        cmd.arg("-drive").arg(format!(
            "file={},format=qcow2,if=ide",
            self.config.disk_image.display()
        ));

        // Network (bridge mode)
        cmd.arg("-netdev").arg(format!(
            "bridge,id=net0,br={}",
            self.config.bridge_name
        ));
        cmd.arg("-device").arg(format!(
            "virtio-net-pci,netdev=net0,mac={}",
            self.config.mac_address
        ));

        // Serial console (for logging)
        cmd.arg("-serial")
            .arg(format!("file:{}", self.config.serial_log.display()));

        // No graphics
        cmd.arg("-nographic");

        // Daemonize
        cmd.arg("-daemonize");

        // Extra arguments
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }

        // Spawn process
        info!("QEMU command: {:?}", cmd);

        let child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| DeployError::QemuProcess {
                message: format!("Failed to spawn QEMU: {}", e),
            })?;

        self.process = Some(child);

        info!("✅ QEMU instance {} started (ID: {})", self.config.name, self.id);

        Ok(())
    }

    /// Stop the QEMU instance gracefully
    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running() {
            warn!("VM {} is not running", self.config.name);
            return Ok(());
        }

        info!("Stopping QEMU instance: {}", self.config.name);

        if let Some(mut process) = self.process.take() {
            // Send SIGTERM for graceful shutdown
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let pid = Pid::from_raw(process.id() as i32);
                kill(pid, Signal::SIGTERM).map_err(|e| DeployError::Process(e))?;
            }

            // Wait for process to exit (with timeout)
            let timeout = std::time::Duration::from_secs(10);
            let start = std::time::Instant::now();

            loop {
                match process.try_wait() {
                    Ok(Some(_)) => {
                        info!("✅ QEMU instance {} stopped", self.config.name);
                        return Ok(());
                    }
                    Ok(None) => {
                        if start.elapsed() > timeout {
                            warn!("QEMU instance {} did not stop gracefully, killing", self.config.name);
                            process.kill().map_err(|e| DeployError::QemuProcess {
                                message: format!("Failed to kill QEMU process: {}", e),
                            })?;
                            return Ok(());
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        return Err(DeployError::QemuProcess {
                            message: format!("Failed to wait for QEMU process: {}", e),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Get serial log path
    pub fn serial_log_path(&self) -> &PathBuf {
        &self.config.serial_log
    }
}

impl Drop for QemuInstance {
    fn drop(&mut self) {
        if self.is_running() {
            warn!(
                "QemuInstance {} dropped while still running. Consider calling stop() explicitly.",
                self.config.name
            );
        }
    }
}

