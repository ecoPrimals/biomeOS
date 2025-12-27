//! Shell Management for BiomeOS Init
//!
//! Spawns and manages the interactive shell.

use crate::init_error::{BootError, Result};
use std::time::Duration;
use tracing::{error, info};

/// Shell spawner and manager
pub struct ShellManager;

impl ShellManager {
    /// Creates a new shell manager
    pub fn new() -> Self {
        Self
    }

    /// Spawns an interactive shell
    ///
    /// This function spawns BusyBox sh and waits for it to complete.
    /// If the shell fails or exits, this enters an infinite loop to
    /// prevent PID 1 from exiting (which would cause a kernel panic).
    ///
    /// # Errors
    ///
    /// Returns an error if the shell cannot be spawned.
    pub async fn spawn_interactive(&self) -> Result<()> {
        info!("🐚 Spawning shell...");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("");

        // Try to spawn busybox sh
        match std::process::Command::new("/bin/busybox")
            .arg("sh")
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
        {
            Ok(status) => {
                if !status.success() {
                    error!("Shell exited with: {}", status);
                }
                // Shell exited - enter infinite loop (PID 1 must not exit)
                self.infinite_wait().await;
            }
            Err(e) => {
                error!("Failed to spawn shell: {}", e);
                error!("Entering infinite wait loop to prevent kernel panic...");
                self.infinite_wait().await;
            }
        }

        Ok(())
    }

    /// Infinite wait loop (for when shell exits/fails)
    ///
    /// PID 1 must never exit, so we wait forever if the shell fails.
    async fn infinite_wait(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    }
}

impl Default for ShellManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_manager_creation() {
        let _mgr = ShellManager::new();
        // Just verify it can be created
    }
}

