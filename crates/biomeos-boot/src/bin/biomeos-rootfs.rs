//! BiomeOS Root Filesystem Builder
//!
//! Creates a complete BiomeOS root filesystem with:
//! - Base system (BusyBox)
//! - BiomeOS init system
//! - Primal binaries
//! - System configuration

use biomeos_boot::rootfs::RootFsCli;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = RootFsCli::parse();
    cli.execute().await
}
