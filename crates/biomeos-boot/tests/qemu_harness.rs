//! QEMU Test Harness
//!
//! Automated testing framework for booting BiomeOS in QEMU

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

/// QEMU VM instance for testing
pub struct QemuVm {
    process: Child,
    serial_log: PathBuf,
    pid: u32,
}

impl QemuVm {
    /// Launch BiomeOS in QEMU
    pub fn launch(iso: &Path, disk: Option<&Path>) -> Result<Self> {
        let serial_log = PathBuf::from(format!("/tmp/biomeos-test-{}.log", std::process::id()));

        let mut cmd = Command::new("qemu-system-x86_64");
        cmd.arg("-cdrom").arg(iso);
        cmd.arg("-m").arg("512");
        cmd.arg("-serial")
            .arg(format!("file:{}", serial_log.display()));
        cmd.arg("-display").arg("none"); // Headless for testing
        cmd.arg("-nographic");

        if let Some(disk_path) = disk {
            cmd.arg("-drive")
                .arg(format!("file={},format=qcow2,if=ide", disk_path.display()));
        }

        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to launch QEMU")?;

        let pid = child.id();

        Ok(Self {
            process: child,
            serial_log,
            pid,
        })
    }

    /// Wait for a pattern in serial output
    pub fn wait_for_output(&self, pattern: &str, timeout: Duration) -> Result<bool> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if self.serial_log.exists() {
                let content = std::fs::read_to_string(&self.serial_log)?;
                if content.contains(pattern) {
                    return Ok(true);
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }

        Ok(false)
    }

    /// Get full serial output
    pub fn serial_output(&self) -> Result<String> {
        if !self.serial_log.exists() {
            return Ok(String::new());
        }
        std::fs::read_to_string(&self.serial_log).context("Failed to read serial log")
    }

    /// Check if VM is still running
    pub fn is_running(&self) -> bool {
        Command::new("ps")
            .arg("-p")
            .arg(self.pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Drop for QemuVm {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        let _ = std::fs::remove_file(&self.serial_log);
    }
}

/// Test that QEMU can boot the ISO
#[test]
#[ignore] // Requires built ISO
fn test_qemu_boot_iso() -> Result<()> {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Find latest ISO
    let dist_dir = project_root.join("dist");
    let iso = std::fs::read_dir(&dist_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "iso")
                .unwrap_or(false)
        })
        .max_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        })
        .context("No ISO found in dist/")?
        .path();

    println!("Testing ISO: {}", iso.display());

    let vm = QemuVm::launch(&iso, None)?;

    // Wait for kernel boot messages
    let found_kernel = vm.wait_for_output("Linux version", Duration::from_secs(30))?;
    assert!(found_kernel, "Kernel did not boot");

    let output = vm.serial_output()?;
    println!("Serial output:\n{}", output);

    // Check for our init
    assert!(output.contains("Run /init as init process") || output.contains("BiomeOS"));

    Ok(())
}

/// Test boot with root disk
#[test]
#[ignore] // Requires built ISO and disk
fn test_qemu_boot_with_disk() -> Result<()> {
    let binding = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = binding.parent().unwrap().parent().unwrap();

    let disk = project_root.join("vm-testing/biomeos-root.qcow2");
    if !disk.exists() {
        eprintln!("⚠️  Skipping: root disk not found");
        return Ok(());
    }

    // Find latest ISO
    let dist_dir = project_root.join("dist");
    let iso = std::fs::read_dir(&dist_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "iso")
                .unwrap_or(false)
        })
        .max_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        })
        .context("No ISO found in dist/")?
        .path();

    println!("Testing ISO: {}", iso.display());
    println!("With disk: {}", disk.display());

    let vm = QemuVm::launch(&iso, Some(&disk))?;

    // Wait for root mount
    let found_mount = vm.wait_for_output("VFS: Mounted root", Duration::from_secs(30))?;
    assert!(found_mount, "Root filesystem not mounted");

    // Wait for init
    let found_init = vm.wait_for_output("Run /init", Duration::from_secs(10))?;
    assert!(found_init, "Init not executed");

    let output = vm.serial_output()?;
    println!("Serial output:\n{}", output);

    Ok(())
}
