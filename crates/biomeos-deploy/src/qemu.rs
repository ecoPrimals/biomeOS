// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    #[must_use] 
    pub fn new(config: QemuConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            process: None,
        }
    }

    /// Get instance ID
    #[must_use] 
    pub const fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get VM name
    #[must_use] 
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
    pub fn start(&mut self) -> Result<()> {
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
        cmd.arg("-netdev")
            .arg(format!("bridge,id=net0,br={}", self.config.bridge_name));
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
                message: format!("Failed to spawn QEMU: {e}"),
            })?;

        self.process = Some(child);

        info!(
            "✅ QEMU instance {} started (ID: {})",
            self.config.name, self.id
        );

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
                use rustix::process::{Pid, Signal, kill_process};

                let pid_i32 = i32::try_from(process.id()).unwrap_or(-1);
                if let Some(pid) = Pid::from_raw(pid_i32) {
                    kill_process(pid, Signal::Term).map_err(DeployError::Process)?;
                }
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
                            warn!(
                                "QEMU instance {} did not stop gracefully, killing",
                                self.config.name
                            );
                            process.kill().map_err(|e| DeployError::QemuProcess {
                                message: format!("Failed to kill QEMU process: {e}"),
                            })?;
                            return Ok(());
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        return Err(DeployError::QemuProcess {
                            message: format!("Failed to wait for QEMU process: {e}"),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Get serial log path
    #[must_use] 
    pub const fn serial_log_path(&self) -> &PathBuf {
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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    fn sample_qemu_config() -> QemuConfig {
        QemuConfig {
            name: "test-vm".to_string(),
            memory: 1024,
            cpus: 2,
            disk_image: PathBuf::from("/tmp/test.qcow2"),
            bridge_name: "br0".to_string(),
            mac_address: "52:54:00:00:00:01".to_string(),
            serial_log: PathBuf::from("/tmp/vm.log"),
            enable_kvm: false,
            extra_args: vec!["-no-reboot".to_string()],
        }
    }

    #[test]
    fn test_qemu_config_construction() {
        let config = sample_qemu_config();
        assert_eq!(config.name, "test-vm");
        assert_eq!(config.memory, 1024);
        assert_eq!(config.cpus, 2);
        assert_eq!(config.bridge_name, "br0");
        assert!(!config.enable_kvm);
        assert_eq!(config.extra_args.len(), 1);
    }

    #[test]
    fn test_qemu_instance_new() {
        let config = sample_qemu_config();
        let instance = QemuInstance::new(config.clone());
        assert_eq!(instance.name(), "test-vm");
        assert_eq!(instance.serial_log_path(), &config.serial_log);
        assert!(instance.id().as_u128() != 0);
    }

    #[test]
    fn test_qemu_instance_is_running_stopped() {
        let config = sample_qemu_config();
        let mut instance = QemuInstance::new(config);
        assert!(!instance.is_running());
    }

    #[test]
    fn test_qemu_instance_serial_log_path() {
        let config = sample_qemu_config();
        let instance = QemuInstance::new(config);
        let path = instance.serial_log_path();
        assert!(path.ends_with("vm.log"));
    }

    #[test]
    fn test_qemu_instance_unique_ids() {
        let config = sample_qemu_config();
        let instance1 = QemuInstance::new(config.clone());
        let instance2 = QemuInstance::new(config);
        assert_ne!(instance1.id(), instance2.id());
    }

    #[test]
    fn test_qemu_config_clone() {
        let config = sample_qemu_config();
        let cloned = config.clone();
        assert_eq!(config.name, cloned.name);
        assert_eq!(config.memory, cloned.memory);
        assert_eq!(config.cpus, cloned.cpus);
        assert_eq!(config.extra_args, cloned.extra_args);
    }

    #[test]
    fn test_qemu_config_debug() {
        let config = sample_qemu_config();
        let debug = format!("{config:?}");
        assert!(debug.contains("QemuConfig"));
        assert!(debug.contains("test-vm"));
        assert!(debug.contains("1024"));
    }

    #[test]
    fn test_qemu_config_with_empty_extra_args() {
        let config = QemuConfig {
            name: "minimal".to_string(),
            memory: 512,
            cpus: 1,
            disk_image: PathBuf::from("/tmp/disk.qcow2"),
            bridge_name: "virbr0".to_string(),
            mac_address: "52:54:00:11:22:33".to_string(),
            serial_log: PathBuf::from("/tmp/serial.log"),
            enable_kvm: true,
            extra_args: vec![],
        };
        let instance = QemuInstance::new(config);
        assert_eq!(instance.name(), "minimal");
    }

    #[test]
    fn test_qemu_config_with_kvm() {
        let config = QemuConfig {
            name: "kvm-vm".to_string(),
            memory: 2048,
            cpus: 4,
            disk_image: PathBuf::from("/tmp/kvm.qcow2"),
            bridge_name: "br0".to_string(),
            mac_address: "52:54:00:aa:bb:cc".to_string(),
            serial_log: PathBuf::from("/tmp/kvm.log"),
            enable_kvm: true,
            extra_args: vec!["-no-reboot".to_string(), "-no-shutdown".to_string()],
        };
        let instance = QemuInstance::new(config);
        assert_eq!(instance.name(), "kvm-vm");
    }

    #[tokio::test]
    async fn test_qemu_stop_when_not_running() {
        let config = sample_qemu_config();
        let mut instance = QemuInstance::new(config);
        let result = instance.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_qemu_start_twice_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let config = QemuConfig {
            name: "double-start".to_string(),
            memory: 128,
            cpus: 1,
            disk_image: temp.path().join("disk.qcow2"),
            bridge_name: "virbr0".to_string(),
            mac_address: "52:54:00:00:00:99".to_string(),
            serial_log: temp.path().join("serial.log"),
            enable_kvm: false,
            extra_args: vec!["-no-reboot".to_string()],
        };
        std::fs::write(&config.disk_image, b"").expect("create disk");
        let mut instance = QemuInstance::new(config);
        let first = instance.start();
        if first.is_ok() {
            let second = instance.start();
            assert!(second.is_err());
            let _ = instance.stop().await;
        }
    }
}
