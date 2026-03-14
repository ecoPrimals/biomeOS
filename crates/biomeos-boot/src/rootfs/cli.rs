// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Root filesystem CLI

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use super::builder::RootFsBuilder;
use super::config::RootFsConfig;

/// CLI for biomeos-rootfs binary
#[derive(Debug, Parser)]
#[command(name = "biomeos-rootfs")]
#[command(about = "Build BiomeOS root filesystem images", long_about = None)]
pub struct RootFsCli {
    /// Size of the root filesystem (e.g., "8G", "10G")
    #[arg(short, long, default_value = "8G")]
    pub size: String,

    /// Output path for the image
    #[arg(short, long, default_value = "biomeos-root.qcow2")]
    pub output: PathBuf,

    /// Directory containing primal binaries
    #[arg(short, long)]
    pub primals: Option<PathBuf>,

    /// Filesystem type
    #[arg(short, long, default_value = "ext4")]
    pub fs_type: String,
}

impl RootFsCli {
    /// Execute the CLI command
    pub async fn execute(self) -> Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into()),
            )
            .init();

        let config = RootFsConfig {
            size: self.size,
            output: self.output,
            primals_dir: self.primals,
            fs_type: self.fs_type,
            ..Default::default()
        };

        let builder = RootFsBuilder::new(config);
        builder.build().await?;

        Ok(())
    }
}
