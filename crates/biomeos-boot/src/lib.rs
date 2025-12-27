//! BiomeOS Boot Infrastructure
//! 
//! Pure Rust boot system for BiomeOS. Provides:
//! - PID 1 init system
//! - Initramfs generation
//! - Bootable USB/ISO creation
//! - Hardware detection
//! - Network configuration
//! 
//! Zero bash scripts. Zero external dependencies (except kernel). 100% Rust.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

pub mod bootable;
pub mod init_console;
pub mod init_error;
pub mod init_filesystem;
pub mod init_hardware;
pub mod init_network;
pub mod init_params;
pub mod init_shell;
pub mod initramfs;
pub mod boot_logger;
pub mod rootfs;

pub use bootable::{BootableMediaBuilder, BootTarget};
pub use init_console::ConsoleWriter;
pub use init_error::{BootError, Result};
pub use init_filesystem::FilesystemManager;
pub use init_hardware::{Architecture, HardwareInfo};
pub use init_network::NetworkManager;
pub use init_params::{BootMode, BootParams};
pub use init_shell::ShellManager;
pub use initramfs::{InitramfsBuilder, KernelManager};
pub use boot_logger::{BootLogger, BootStage, LogLevel};
pub use rootfs::{RootFsBuilder, RootFsConfig, RootFsCli};
