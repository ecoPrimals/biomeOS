// Test program to create a genomeBin with self-extracting stub
use anyhow::Result;
use biomeos_genomebin_v3::{GenomeBin, Arch};
use std::path::Path;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    println!("Creating test genomeBin with self-extracting stub...");
    
    let mut genome = GenomeBin::new("test-nucleus-v3");
    genome.manifest = genome.manifest
        .version("0.1.0")
        .description("Test NUCLEUS daemon with self-extracting stub");
    
    // Add nucleus binary
    let nucleus_path = Path::new("target/x86_64-unknown-linux-musl/release/nucleus");
    if !nucleus_path.exists() {
        eprintln!("Error: nucleus binary not found at: {}", nucleus_path.display());
        std::process::exit(1);
    }
    
    genome.add_binary(Arch::X86_64, nucleus_path)?;
    
    // Write genomeBin with stub
    let output_path = Path::new("plasmidBin/test-nucleus-v3.genome");
    genome.write(output_path)?;
    
    println!("\n✅ genomeBin created: {}", output_path.display());
    println!("Testing direct execution...\n");
    
    // Test: Try to run the genomeBin
    let status = std::process::Command::new(output_path)
        .arg("--help")
        .status()?;
    
    if status.success() {
        println!("\n✅ Self-extracting stub works!");
    } else {
        println!("\n❌ Stub execution failed");
    }
    
    Ok(())
}
