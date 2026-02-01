// Example: Create a self-extracting genomeBin
use biomeos_genomebin_v3::{GenomeBin, Arch};
use std::path::PathBuf;
use std::env;

fn main() -> anyhow::Result<()> {
    println!("Creating genomeBin with self-extracting stub...");
    
    // Get workspace root
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(manifest_dir).parent().unwrap().parent().unwrap().to_path_buf();
    
    println!("Workspace root: {}", workspace_root.display());
    
    let mut genome = GenomeBin::new("test-nucleus-v3");
    genome.manifest = genome.manifest
        .version("0.1.0")
        .description("Test NUCLEUS daemon");
    
    // Add binary (assumes it exists)
    let nucleus_path = workspace_root.join("target/x86_64-unknown-linux-musl/release/nucleus");
    println!("Looking for nucleus at: {}", nucleus_path.display());
    
    if !nucleus_path.exists() {
        eprintln!("Warning: nucleus binary not found!");
        eprintln!("Creating minimal test genome (no binaries)...");
    } else {
        println!("Adding nucleus binary...");
        genome.add_binary(Arch::X86_64, &nucleus_path)?;
        println!("✅ Binary added");
    }
    
    let output = workspace_root.join("plasmidBin/test-nucleus-v3.genome");
    println!("Writing to: {}", output.display());
    
    // Ensure directory exists
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    genome.write(&output)?;
    
    println!("\n✅ genomeBin created: {}", output.display());
    
    // Verify file exists
    if output.exists() {
        let metadata = std::fs::metadata(&output)?;
        println!("   Size: {} bytes ({:.2} MB)", metadata.len(), metadata.len() as f64 / 1024.0 / 1024.0);
        
        println!("\nTry running:");
        println!("  {} --help", output.display());
        println!("  {} info", output.display());
    } else {
        eprintln!("ERROR: File was not created!");
    }
    
    Ok(())
}
