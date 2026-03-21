// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! BiomeOS VM Verification CLI
//!
//! Modern idiomatic Rust replacement for verify-primals.sh
//!
//! Usage:
//!   biomeos-verify --serial-log /tmp/vm1-serial.log
//!   biomeos-verify --serial-log /tmp/vm1-serial.log --rootfs biomeos-root/

use biomeos_deploy::verify::{VerifyConfig, VmVerifier};
use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser, Debug)]
#[command(name = "biomeos-verify")]
#[command(about = "BiomeOS VM verification tool", long_about = None)]
struct Cli {
    /// Path to serial log file
    #[arg(short, long, default_value = "/tmp/biomeos-verify.log")]
    serial_log: PathBuf,

    /// Optional root filesystem directory to check for primals
    #[arg(short, long)]
    rootfs: Option<PathBuf>,

    /// Boot timeout in seconds
    #[arg(short = 't', long, default_value = "30")]
    boot_timeout: u64,

    /// Expected boot message
    #[arg(short = 'm', long, default_value = "BiomeOS initialization complete")]
    expected_message: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::from_default_env()
            .add_directive("biomeos_deploy=debug".parse()?)
            .add_directive("biomeos_verify=debug".parse()?)
    } else {
        EnvFilter::from_default_env()
            .add_directive("biomeos_deploy=info".parse()?)
            .add_directive("biomeos_verify=info".parse()?)
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    // Banner
    println!("\n🔍 BiomeOS VM Verifier");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    info!("Serial log: {}", cli.serial_log.display());
    if let Some(ref rootfs) = cli.rootfs {
        info!("Root filesystem: {}", rootfs.display());
    }

    // Create verifier config
    let config = VerifyConfig {
        serial_log: cli.serial_log.clone(),
        rootfs_dir: cli.rootfs.clone(),
        boot_timeout: cli.boot_timeout,
        expected_boot_message: cli.expected_message.clone(),
        ..Default::default()
    };

    // Run verification
    let verifier = VmVerifier::new(config);

    println!("🚀 Verifying VM boot...\n");

    match verifier.verify_boot().await {
        Ok(result) => {
            // Print summary
            println!("{}", result.summary());

            if result.is_ok() {
                println!("🎯 Verification: PASSED ✅\n");
                std::process::exit(0);
            } else {
                println!("🎯 Verification: FAILED ❌\n");
                println!("💡 Troubleshooting:");
                println!("   • Check serial log: {}", cli.serial_log.display());
                println!("   • Ensure VM has sufficient time to boot");
                println!("   • Verify boot configuration\n");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Verification error: {}", e);
            eprintln!("\n❌ Verification failed: {e}\n");
            std::process::exit(1);
        }
    }
}
