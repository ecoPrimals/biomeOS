//! genomeBin v3.0 Self-Extracting Stub
//!
//! This tiny Rust binary is prepended to every genomeBin, enabling direct execution.
//!
//! Design Principles (Deep Debt):
//! - 100% Pure Rust (zero C dependencies)
//! - Zero unsafe code
//! - Tiny binary size (~50KB with opt-level="z")
//! - Self-contained (reads own binary)
//! - Clear error messages
//!
//! Architecture:
//! ```
//! genomeBin file layout:
//! [This stub binary] + [__GENOME_PAYLOAD__\n] + [bincode-serialized data]
//! ```

use anyhow::{Context, Result};
use biomeos_genomebin_v3::{GenomeManifest, Arch, CompressedBinary};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .without_time()
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    
    // Parse command
    let command = args.first().map(|s| s.as_str());
    
    match command {
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some("--extract") | Some("extract") => {
            let install_dir = if args.len() > 1 {
                PathBuf::from(&args[1])
            } else {
                default_install_dir()?
            };
            extract_genome(&install_dir)
        }
        Some("--info") | Some("info") => {
            show_info()
        }
        Some("run") => {
            // Run in-place (extract to temp, execute, cleanup)
            let run_args = &args[1..];
            run_in_place(run_args)
        }
        None => {
            // Default: Extract and install
            let install_dir = default_install_dir()?;
            extract_genome(&install_dir)
        }
        Some(unknown) => {
            eprintln!("Unknown command: {}", unknown);
            eprintln!();
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "genome".to_string());
    
    println!("genomeBin v3.0 - Self-Extracting Binary Deployment");
    println!();
    println!("USAGE:");
    println!("    {}                    Extract to default location", exe_name);
    println!("    {} extract [DIR]      Extract to specified directory", exe_name);
    println!("    {} run [ARGS...]      Run in-place (temp extraction)", exe_name);
    println!("    {} info               Show genome information", exe_name);
    println!("    {} --help             Show this help", exe_name);
    println!();
    println!("EXAMPLES:");
    println!("    ./nucleus.genome                    # Extract to ~/.local/");
    println!("    ./nucleus.genome extract /opt       # Extract to /opt/");
    println!("    ./nucleus.genome run daemon         # Run nucleus daemon");
    println!("    ./nucleus.genome info               # Show metadata");
    println!();
    println!("ENVIRONMENT:");
    println!("    RUST_LOG=debug    Enable debug logging");
}

fn show_info() -> Result<()> {
    tracing::info!("Loading genomeBin metadata");
    
    let (manifest, binaries, embedded) = load_genome_payload()?;
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("genomeBin Information");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("Name:         {}", manifest.name);
    println!("Version:      {}", manifest.version);
    if !manifest.description.is_empty() {
        println!("Description:  {}", manifest.description);
    }
    println!();
    println!("Architectures:");
    for arch in &manifest.architectures {
        if let Some(binary) = binaries.get(arch) {
            let ratio = (binary.size_compressed as f64 / binary.size_original as f64) * 100.0;
            println!("  • {:?}: {} → {} bytes ({:.1}% compressed)",
                arch,
                format_bytes(binary.size_original),
                format_bytes(binary.size_compressed),
                ratio
            );
        }
    }
    
    if !embedded.is_empty() {
        println!();
        println!("Embedded Genomes: {}", embedded.len());
        for genome_data in &embedded {
            let (emb_manifest, _, _): (GenomeManifest, HashMap<Arch, CompressedBinary>, Vec<Vec<u8>>) = 
                bincode::deserialize(genome_data)
                .context("Failed to deserialize embedded genome")?;
            println!("  • {} v{}", emb_manifest.name, emb_manifest.version);
        }
    }
    
    if !manifest.capabilities.is_empty() {
        println!();
        println!("Capabilities:");
        for cap in &manifest.capabilities {
            println!("  • {}", cap);
        }
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    
    Ok(())
}

fn extract_genome(install_dir: &Path) -> Result<()> {
    tracing::info!("Extracting genomeBin to: {}", install_dir.display());
    
    let (manifest, binaries, embedded) = load_genome_payload()?;
    
    // Detect current architecture
    let arch = Arch::detect();
    tracing::info!("Detected architecture: {:?}", arch);
    
    // Get binary for current arch
    let compressed = binaries.get(&arch)
        .with_context(|| format!("No binary available for architecture: {:?}", arch))?;
    
    // Decompress and verify
    tracing::info!("Decompressing {} ({} → {} bytes)",
        manifest.name,
        format_bytes(compressed.size_compressed),
        format_bytes(compressed.size_original)
    );
    
    let decompressed = decompress_and_verify(compressed)?;
    
    // Create install directory
    std::fs::create_dir_all(install_dir)
        .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;
    
    // Write binary
    let binary_path = install_dir.join(&manifest.name);
    std::fs::write(&binary_path, decompressed)
        .with_context(|| format!("Failed to write binary: {}", binary_path.display()))?;
    
    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = binary_path.metadata()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)?;
    }
    
    println!("✅ Extracted: {} → {}", manifest.name, binary_path.display());
    
    // Extract embedded genomes
    if !embedded.is_empty() {
        println!();
        println!("Extracting {} embedded genome(s)...", embedded.len());
        
        for genome_data in embedded {
            extract_embedded_genome(&genome_data, install_dir)?;
        }
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("✅ Extraction complete!");
    println!("═══════════════════════════════════════════════════════════════════");
    
    Ok(())
}

fn run_in_place(args: &[String]) -> Result<()> {
    tracing::info!("Running genomeBin in-place");
    
    let (manifest, binaries, _embedded) = load_genome_payload()?;
    
    // Detect current architecture
    let arch = Arch::detect();
    
    // Get binary for current arch
    let compressed = binaries.get(&arch)
        .with_context(|| format!("No binary available for architecture: {:?}", arch))?;
    
    // Decompress
    let decompressed = decompress_and_verify(compressed)?;
    
    // Create temp directory
    let temp_dir = tempfile::tempdir()
        .context("Failed to create temporary directory")?;
    
    // Write binary to temp
    let binary_path = temp_dir.path().join(&manifest.name);
    std::fs::write(&binary_path, decompressed)
        .with_context(|| format!("Failed to write temp binary: {}", binary_path.display()))?;
    
    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = binary_path.metadata()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)?;
    }
    
    tracing::info!("Executing: {} {:?}", binary_path.display(), args);
    
    // Execute
    let status = std::process::Command::new(&binary_path)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute: {}", binary_path.display()))?;
    
    if !status.success() {
        anyhow::bail!("Process exited with status: {}", status);
    }
    
    // Temp dir automatically cleaned up on drop
    Ok(())
}

fn extract_embedded_genome(genome_data: &[u8], parent_dir: &Path) -> Result<()> {
    let (manifest, binaries, nested_embedded): (GenomeManifest, HashMap<Arch, CompressedBinary>, Vec<Vec<u8>>) = 
        bincode::deserialize(genome_data)
        .context("Failed to deserialize embedded genome")?;
    
    let arch = Arch::detect();
    
    if let Some(compressed) = binaries.get(&arch) {
        let decompressed = decompress_and_verify(compressed)?;
        
        let binary_path = parent_dir.join(&manifest.name);
        std::fs::write(&binary_path, decompressed)
            .with_context(|| format!("Failed to write embedded binary: {}", binary_path.display()))?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = binary_path.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }
        
        println!("  ✅ {}", binary_path.display());
        
        // Recursively extract nested embedded genomes
        for nested_data in nested_embedded {
            extract_embedded_genome(&nested_data, parent_dir)?;
        }
    }
    
    Ok(())
}

/// Load genomeBin payload from current executable
fn load_genome_payload() -> Result<(GenomeManifest, HashMap<Arch, CompressedBinary>, Vec<Vec<u8>>)> {
    // Read self (current executable)
    let self_path = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    tracing::debug!("Reading genomeBin from: {}", self_path.display());
    
    let contents = std::fs::read(&self_path)
        .with_context(|| format!("Failed to read genomeBin: {}", self_path.display()))?;
    
    // Find payload marker - search from the END since the marker string might appear in the stub's data section
    let marker = b"__GENOME_PAYLOAD__\n";
    let marker_pos = contents.windows(marker.len())
        .rposition(|window| window == marker)  // rposition = search from end!
        .context("Payload marker not found in genomeBin - file may be corrupted")?;
    
    tracing::debug!("Found payload marker at offset: {}", marker_pos);
    
    // Deserialize payload
    let payload_start = marker_pos + marker.len();
    let (manifest, binaries, embedded): (GenomeManifest, HashMap<Arch, CompressedBinary>, Vec<Vec<u8>>) = 
        bincode::deserialize(&contents[payload_start..])
        .context("Failed to deserialize genomeBin payload")?;
    
    tracing::debug!("Loaded genome: {} v{}", manifest.name, manifest.version);
    
    Ok((manifest, binaries, embedded))
}

/// Decompress and verify binary
fn decompress_and_verify(compressed: &CompressedBinary) -> Result<Vec<u8>> {
    tracing::debug!("Decompressing {:?} binary ({} bytes)", compressed.arch, compressed.size_compressed);
    
    let decompressed = zstd::decode_all(&compressed.data[..])
        .context("Failed to decompress binary with zstd")?;
    
    // Verify size
    if decompressed.len() as u64 != compressed.size_original {
        anyhow::bail!(
            "Size mismatch after decompression: expected {}, got {}",
            compressed.size_original,
            decompressed.len()
        );
    }
    
    // Verify checksum
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&decompressed);
    let checksum: [u8; 32] = hasher.finalize().into();
    
    if checksum != compressed.checksum {
        anyhow::bail!(
            "Checksum mismatch: expected {}, got {}",
            hex::encode(compressed.checksum),
            hex::encode(checksum)
        );
    }
    
    tracing::debug!("Decompression verified: {} bytes", decompressed.len());
    Ok(decompressed)
}

/// Get default install directory for current platform
fn default_install_dir() -> Result<PathBuf> {
    // Check for Android
    if std::path::Path::new("/system/build.prop").exists() {
        tracing::debug!("Detected Android platform");
        return Ok(PathBuf::from("/data/local/tmp"));
    }
    
    // Check if root
    let is_root = std::env::var("USER").map(|u| u == "root").unwrap_or(false);
    
    if is_root {
        // Root: Use /opt
        Ok(PathBuf::from("/opt"))
    } else {
        // Non-root: Use ~/.local
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".local").join("bin"))
    }
}

/// Format bytes for human-readable display
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
