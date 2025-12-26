//! BiomeOS Boot Infrastructure
//! 
//! Pure Rust boot system for BiomeOS. Provides:
//! - PID 1 init system
//! - Initramfs generation
//! - Bootable USB/ISO creation
//! - Hardware detection
//! - Network configuration
//! 
//! Zero bash scripts. Zero external dependencies. 100% Rust.

#![deny(unsafe_code)]

pub mod bootable;
pub mod initramfs;

pub use bootable::BootableMediaBuilder;
pub use initramfs::{InitramfsBuilder, KernelManager};
