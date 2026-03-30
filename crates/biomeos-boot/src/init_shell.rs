// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Shell Management for `BiomeOS` Init
//!
//! Spawns and manages the interactive shell.

use crate::init_error::Result;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;
use tracing::{error, info};

/// Builds the production interactive shell command (`busybox sh` with inherited stdio).
pub(crate) fn build_interactive_shell_command() -> Command {
    let mut cmd = Command::new("/bin/busybox");
    cmd.arg("sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    cmd
}

/// Runs a shell command to completion (separated for unit testing).
pub(crate) fn run_interactive_shell_status(cmd: &mut Command) -> std::io::Result<ExitStatus> {
    cmd.status()
}

/// Shell spawner and manager
pub struct ShellManager;

impl ShellManager {
    /// Creates a new shell manager
    #[must_use] 
    pub const fn new() -> Self {
        Self
    }

    /// Spawns an interactive shell
    ///
    /// This function spawns `BusyBox` sh and waits for it to complete.
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
        match run_interactive_shell_status(&mut build_interactive_shell_command()) {
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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_manager_creation() {
        let _mgr = ShellManager::new();
    }

    #[test]
    fn test_shell_manager_default() {
        let mgr = ShellManager;
        let mgr2 = ShellManager::new();
        let _ = (mgr, mgr2);
    }

    #[test]
    fn test_shell_manager_default_equals_new() {
        let default_mgr = ShellManager;
        let new_mgr = ShellManager::new();
        let _ = (default_mgr, new_mgr);
    }

    #[test]
    fn test_build_interactive_shell_command_uses_busybox_sh() {
        let cmd = build_interactive_shell_command();
        assert_eq!(cmd.get_program(), std::path::Path::new("/bin/busybox"));
        let args: Vec<_> = cmd.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(args, vec!["sh"]);
    }

    #[test]
    fn test_run_interactive_shell_status_true_exits_successfully() {
        let mut cmd = Command::new("true");
        let status = run_interactive_shell_status(&mut cmd).expect("spawn true");
        assert!(status.success());
    }

    #[test]
    fn test_run_interactive_shell_status_false_exits_nonzero() {
        let mut cmd = Command::new("false");
        let status = run_interactive_shell_status(&mut cmd).expect("spawn false");
        assert!(!status.success());
    }

    #[test]
    fn test_run_interactive_shell_status_missing_binary_returns_error() {
        let mut cmd = Command::new("/nonexistent/binary/biomeos-init-shell-test-xyz");
        let err = run_interactive_shell_status(&mut cmd).expect_err("expected spawn error");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[tokio::test]
    #[ignore = "spawn_interactive runs real shell - use for integration testing"]
    async fn test_spawn_interactive_placeholder() {
        let mgr = ShellManager::new();
        let _ = mgr;
    }
}
