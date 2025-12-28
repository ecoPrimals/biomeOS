use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Result of executing a CLI command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CliResult {
    /// Check if command succeeded
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get stdout as string
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Get stderr as string
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Get exit code
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

/// Generic CLI adapter for executing command-line tools
#[derive(Debug, Clone)]
pub struct CliAdapter {
    /// Path to the binary
    binary_path: PathBuf,
    /// Default timeout for commands (in seconds)
    default_timeout: Duration,
    /// Working directory for commands
    working_dir: Option<PathBuf>,
}

impl CliAdapter {
    /// Create a new CLI adapter
    pub fn new<P: Into<PathBuf>>(binary_path: P) -> Self {
        CliAdapter {
            binary_path: binary_path.into(),
            default_timeout: Duration::from_secs(30),
            working_dir: None,
        }
    }

    /// Set default timeout for commands
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.default_timeout = Duration::from_secs(timeout_secs);
        self
    }

    /// Set working directory for commands
    pub fn with_working_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Get the binary path
    pub fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }

    /// Execute a command with arguments
    pub async fn execute(&self, args: &[&str]) -> Result<CliResult> {
        self.execute_with_timeout(args, self.default_timeout).await
    }

    /// Execute a command with a custom timeout
    pub async fn execute_with_timeout(
        &self,
        args: &[&str],
        timeout_duration: Duration,
    ) -> Result<CliResult> {
        let mut command = Command::new(&self.binary_path);
        command.args(args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        if let Some(ref dir) = self.working_dir {
            command.current_dir(dir);
        }

        let execution = async {
            let output = command
                .output()
                .await
                .context("Failed to execute command")?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            let success = output.status.success();

            Ok(CliResult {
                stdout,
                stderr,
                exit_code,
                success,
            })
        };

        timeout(timeout_duration, execution)
            .await
            .context("Command execution timed out")?
    }

    /// Execute a command and return only stdout (convenience method)
    pub async fn execute_stdout(&self, args: &[&str]) -> Result<String> {
        let result = self.execute(args).await?;
        if result.is_success() {
            Ok(result.stdout)
        } else {
            anyhow::bail!(
                "Command failed with exit code {}: {}",
                result.exit_code,
                result.stderr
            )
        }
    }

    /// Execute a command with stdin input
    pub async fn execute_with_stdin(&self, args: &[&str], stdin_data: &str) -> Result<CliResult> {
        let mut command = Command::new(&self.binary_path);
        command.args(args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        if let Some(ref dir) = self.working_dir {
            command.current_dir(dir);
        }

        let mut child = command.spawn().context("Failed to spawn command")?;

        // Write to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(stdin_data.as_bytes())
                .await
                .context("Failed to write to stdin")?;
            stdin.flush().await.context("Failed to flush stdin")?;
            drop(stdin); // Close stdin
        }

        let execution = async {
            let output = child
                .wait_with_output()
                .await
                .context("Failed to wait for command")?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            let success = output.status.success();

            Ok(CliResult {
                stdout,
                stderr,
                exit_code,
                success,
            })
        };

        timeout(self.default_timeout, execution)
            .await
            .context("Command execution timed out")?
    }

    /// Check if the binary exists and is executable
    ///
    /// Supports both full paths and command names (searched in PATH)
    pub fn verify_binary(&self) -> Result<()> {
        // Check if it's a full path or contains path separators
        if self.binary_path.is_absolute() || self.binary_path.components().count() > 1 {
            // Full or relative path - check file exists
            if !self.binary_path.exists() {
                anyhow::bail!("Binary not found: {:?}", self.binary_path);
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = std::fs::metadata(&self.binary_path)
                    .context("Failed to read binary metadata")?;
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 == 0 {
                    anyhow::bail!("Binary is not executable: {:?}", self.binary_path);
                }
            }

            Ok(())
        } else {
            // Command name - try to execute with --version to verify it exists in PATH
            let result = std::process::Command::new(&self.binary_path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            match result {
                Ok(_) => Ok(()), // Command executed, so it exists
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    anyhow::bail!("Binary not found in PATH: {:?}", self.binary_path)
                }
                Err(_) => {
                    // Other errors (permission, etc.) still mean the binary was found
                    Ok(())
                }
            }
        }
    }

    /// Get version information (tries --version flag)
    pub async fn get_version(&self) -> Result<String> {
        let result = self.execute(&["--version"]).await;
        match result {
            Ok(r) if r.is_success() => Ok(r.stdout.trim().to_string()),
            Ok(r) => Ok(format!("Version check failed: {}", r.stderr)),
            Err(e) => Ok(format!("Version unavailable: {}", e)),
        }
    }

    /// Get help information (tries --help flag)
    pub async fn get_help(&self) -> Result<String> {
        let result = self.execute(&["--help"]).await;
        match result {
            Ok(r) if r.is_success() => Ok(r.stdout.trim().to_string()),
            Ok(r) if !r.stderr.is_empty() => Ok(r.stderr.trim().to_string()),
            Ok(r) => Ok(r.stdout.trim().to_string()),
            Err(e) => anyhow::bail!("Help unavailable: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_adapter_echo() {
        let adapter = CliAdapter::new("echo");
        let result = adapter.execute(&["hello", "world"]).await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.stdout.trim(), "hello world");
    }

    #[tokio::test]
    async fn test_cli_adapter_stdin() {
        let adapter = CliAdapter::new("cat");
        let result = adapter.execute_with_stdin(&[], "test input").await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.stdout.trim(), "test input");
    }

    #[tokio::test]
    async fn test_cli_adapter_timeout() {
        let adapter = CliAdapter::new("sleep").with_timeout(1);
        let result = adapter.execute(&["10"]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_cli_adapter_verify_binary() {
        let adapter = CliAdapter::new("echo");
        assert!(adapter.verify_binary().is_ok());

        let bad_adapter = CliAdapter::new("/nonexistent/binary");
        assert!(bad_adapter.verify_binary().is_err());
    }
}
