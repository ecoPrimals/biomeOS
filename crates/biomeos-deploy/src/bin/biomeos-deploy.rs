//! BiomeOS Federation Deployment CLI
//!
//! Pure Rust replacement for deploy-federation.sh

use anyhow::Result;
use biomeos_deploy::prelude::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser, Debug)]
#[clap(author, version, about = "BiomeOS Federation Deployment", long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deploy a federation from topology file
    Deploy {
        /// Path to topology YAML file
        #[clap(short, long)]
        topology: PathBuf,

        /// Enable KVM acceleration
        #[clap(short, long, default_value = "true")]
        kvm: bool,

        /// Health check timeout (seconds)
        #[clap(long, default_value = "30")]
        health_timeout: u64,

        /// Wait for all VMs to be healthy
        #[clap(short, long, default_value = "true")]
        wait: bool,
    },

    /// Check health of running federation
    Health {
        /// Path to topology YAML file
        #[clap(short, long)]
        topology: PathBuf,
    },

    /// Shutdown a running federation
    Shutdown {
        /// Path to topology YAML file
        #[clap(short, long)]
        topology: PathBuf,
    },

    /// Validate a topology file
    Validate {
        /// Path to topology YAML file
        #[clap(short, long)]
        topology: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    match args.command {
        Commands::Deploy {
            topology,
            kvm,
            health_timeout,
            wait,
        } => {
            let topology = Topology::from_file(&topology)?;
            let config = FederationConfig {
                topology,
                enable_kvm: kvm,
                health_check_timeout: health_timeout,
                wait_for_healthy: wait,
            };

            let mut federation = Federation::new(config);
            federation.deploy().await?;

            println!("✅ Federation deployed successfully!");
            println!("   VMs running: {}", federation.vm_count());
        }

        Commands::Health { topology } => {
            let topology = Topology::from_file(&topology)?;
            let config = FederationConfig {
                topology,
                enable_kvm: true,
                health_check_timeout: 30,
                wait_for_healthy: false,
            };

            let federation = Federation::new(config);
            let health = federation.health_check().await?;

            println!("🔍 Federation Health Check:");
            for vm_health in health {
                println!(
                    "   {} - {:?} (boot: {})",
                    vm_health.vm_name, vm_health.status, vm_health.boot_completed
                );
            }
        }

        Commands::Shutdown { topology } => {
            let topology = Topology::from_file(&topology)?;
            let config = FederationConfig {
                topology,
                enable_kvm: true,
                health_check_timeout: 30,
                wait_for_healthy: false,
            };

            let mut federation = Federation::new(config);
            federation.shutdown().await?;

            println!("✅ Federation shutdown complete");
        }

        Commands::Validate { topology } => {
            let topology = Topology::from_file(&topology)?;
            println!("✅ Topology is valid!");
            println!("   Name: {}", topology.metadata.name);
            println!("   VMs: {}", topology.vm_count());
        }
    }

    Ok(())
}
