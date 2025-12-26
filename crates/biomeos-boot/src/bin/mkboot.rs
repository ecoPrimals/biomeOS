//! BiomeOS Bootable Media Creator CLI
//! 
//! Command-line tool for creating bootable BiomeOS USB/ISO images.
//! Pure Rust implementation.

use anyhow::Result;
use biomeos_boot::BootableMediaBuilder;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "biomeos-mkboot")]
#[command(about = "BiomeOS Bootable Media Creator", long_about = None)]
struct Cli {
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    project_root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create bootable USB image
    Usb {
        /// Output file path (optional, auto-generated if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Create bootable ISO image
    Iso {
        /// Output file path (optional, auto-generated if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Test build initramfs only (faster iteration)
    Initramfs {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("biomeos_boot=info")
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Usb { output: _ } => {
            let builder = BootableMediaBuilder::new(cli.project_root)?;
            let image_path = builder.build_usb_image().await?;
            
            println!("\n✅ Success! Bootable USB image created:");
            println!("   {}\n", image_path.display());
            println!("To write to USB drive:");
            println!("   sudo dd if={} of=/dev/sdX bs=4M status=progress\n", 
                     image_path.display());
        }
        
        Commands::Iso { output: _ } => {
            // For now, USB and ISO are the same (hybrid image)
            let builder = BootableMediaBuilder::new(cli.project_root)?;
            let image_path = builder.build_usb_image().await?;
            
            println!("\n✅ Success! Bootable ISO image created:");
            println!("   {}\n", image_path.display());
            println!("To test in QEMU:");
            println!("   qemu-system-x86_64 -m 2048 -enable-kvm -cdrom {}\n", 
                     image_path.display());
        }
        
        Commands::Initramfs { output } => {
            use biomeos_boot::InitramfsBuilder;
            
            let work_dir = cli.project_root.join("build");
            let mut builder = InitramfsBuilder::new(&work_dir)?;
            
            builder.create_directory_structure()?;
            builder.add_biomeos_binaries(&cli.project_root)?;
            builder.install_binaries()?;
            builder.create_init_script()?;
            builder.build(&output)?;
            
            println!("\n✅ Success! Initramfs created:");
            println!("   {}\n", output.display());
        }
    }

    Ok(())
}

