//! Console Output Abstraction
//!
//! Provides reliable console output during early boot, before logging is fully initialized.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};

/// Manages console output during boot
///
/// Writes to multiple outputs to ensure visibility:
/// - stdout (terminal/serial)
/// - stderr (for errors)
/// - /dev/console (direct kernel console)
pub struct ConsoleWriter {
    stdout: io::Stdout,
    stderr: io::Stderr,
    console_device: Option<File>,
}

impl ConsoleWriter {
    /// Creates a new console writer
    ///
    /// Opens `/dev/console` for direct kernel console access. If this fails,
    /// output will only go to stdout/stderr.
    ///
    /// # Errors
    ///
    /// Returns an error if stdout or stderr cannot be locked.
    pub fn new() -> io::Result<Self> {
        let console_device = OpenOptions::new()
            .write(true)
            .open("/dev/console")
            .ok(); // Gracefully handle if /dev/console not available

        Ok(Self {
            stdout: io::stdout(),
            stderr: io::stderr(),
            console_device,
        })
    }

    /// Write a message to all outputs
    ///
    /// # Errors
    ///
    /// Returns an error if writing to any output fails.
    pub fn write_line(&mut self, msg: &str) -> io::Result<()> {
        let line = format!("{}\n", msg);
        self.write_bytes(line.as_bytes())
    }

    /// Write an error message to stderr and console
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_error(&mut self, msg: &str) -> io::Result<()> {
        let line = format!("[ERROR] {}\n", msg);
        
        self.stderr.write_all(line.as_bytes())?;
        self.stderr.flush()?;

        if let Some(ref mut console) = self.console_device {
            console.write_all(line.as_bytes())?;
            console.flush()?;
        }

        Ok(())
    }

    /// Write raw bytes to all outputs
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stdout.write_all(bytes)?;
        self.stdout.flush()?;

        if let Some(ref mut console) = self.console_device {
            console.write_all(bytes)?;
            console.flush()?;
        }

        Ok(())
    }

    /// Write a formatted banner
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_banner(&mut self, title: &str) -> io::Result<()> {
        self.write_line("")?;
        self.write_line(&"=".repeat(60))?;
        self.write_line(title)?;
        self.write_line(&"=".repeat(60))?;
        self.write_line("")
    }
}

impl Default for ConsoleWriter {
    fn default() -> Self {
        Self::new().expect("Failed to create console writer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_writer_creation() {
        // Should not panic even if /dev/console doesn't exist
        let _writer = ConsoleWriter::new();
    }

    #[test]
    fn test_write_to_stdout() -> io::Result<()> {
        let mut writer = ConsoleWriter::new()?;
        // This test just ensures the API works; output verification is manual
        writer.write_line("test message")?;
        Ok(())
    }
}

