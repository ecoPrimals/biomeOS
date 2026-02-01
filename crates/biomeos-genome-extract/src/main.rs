// biomeos-genome-extract/src/main.rs
// Pure Rust Universal genomeBin v4.0 Extractor
//
// Deep Debt Principles:
// - 100% Pure Rust (zero unsafe, zero C deps except libc)
// - Binary = DNA (deterministic fingerprint)
// - Platform-agnostic (runtime detection)
// - Self-contained (no external tools)

mod format;

use anyhow::{Context, Result};
use format::{BinaryEntry, GenomeHeader, GenomeManifest, MAGIC, current_arch};
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check if we're being executed as a genomeBin (argv[0] ends with .genome)
    let is_genome = args.get(0)
        .map(|s| s.ends_with(".genome"))
        .unwrap_or(false);
    
    if is_genome {
        // Running as embedded extractor in a .genome file
        run_embedded(&args)
    } else {
        // Running as standalone tool
        run_standalone(&args)
    }
}

/// Run as standalone extractor tool
fn run_standalone(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }
    
    let genome_path = &args[1];
    let command = args.get(2).map(|s| s.as_str()).unwrap_or("info");
    
    execute_command(genome_path, command, &args[2..])
}

/// Run as embedded extractor (genomeBin executed directly)
fn run_embedded(args: &[String]) -> Result<()> {
    let genome_path = &args[0];
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("info");
    
    execute_command(genome_path, command, &args[1..])
}

/// Execute command on genomeBin
fn execute_command(genome_path: &str, command: &str, args: &[String]) -> Result<()> {
    let mut file = File::open(genome_path)
        .with_context(|| format!("Failed to open genomeBin: {}", genome_path))?;
    
    // Find MAGIC marker
    let magic_offset = find_magic(&mut file)
        .context("Failed to find genomeBin v4.0 magic marker")?;
    
    // Debug output
    eprintln!("Found GENOME40 magic at offset: {}", magic_offset);
    
    // Position reader right after MAGIC
    file.seek(SeekFrom::Start(magic_offset + MAGIC.len() as u64))?;
    
    // Read header
    let header = GenomeHeader::read_from(&mut file)
        .context("Failed to read genomeBin header")?;
    
    // Debug: uncomment for troubleshooting
    // eprintln!("Header read successfully: version={}, binaries={}", 
    //     header.version, header.num_binaries);
    
    // Execute command
    match command {
        "info" => show_info(&mut file, &header, magic_offset),
        "extract" => {
            let output_dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");
            extract_binary(&mut file, &header, magic_offset, output_dir, genome_path)
        },
        "run" => {
            let binary_args = &args[1..];
            run_binary(&mut file, &header, magic_offset, binary_args, genome_path)
        },
        _ => {
            anyhow::bail!("Unknown command: {}. Use: info, extract, or run", command);
        }
    }
}

/// Find MAGIC marker in file
fn find_magic<R: Read + Seek>(reader: &mut R) -> Result<u64> {
    reader.seek(SeekFrom::Start(0))?;
    
    // For efficiency, search in larger chunks
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunks
    let mut offset = 0u64;
    let mut prev_chunk_tail = Vec::new();
    
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            if offset == 0 {
                anyhow::bail!("Empty file (not a genomeBin v4.0)");
            }
            anyhow::bail!("MAGIC marker 'GENOME40' not found (not a genomeBin v4.0 file)");
        }
        
        // Combine previous tail with current buffer to catch markers at boundaries
        let search_buffer = if !prev_chunk_tail.is_empty() {
            let mut combined = prev_chunk_tail.clone();
            combined.extend_from_slice(&buffer[..n]);
            combined
        } else {
            buffer[..n].to_vec()
        };
        
        // Search for MAGIC + version 4 (to avoid false positives in embedded strings)
        // MAGIC is "GENOME40" (8 bytes) followed by version u32 little-endian (4 bytes)
        if let Some(pos) = search_buffer.windows(12)
            .position(|window| &window[0..8] == MAGIC && window[8..12] == [0x04, 0x00, 0x00, 0x00])
        {
            let found_offset = if !prev_chunk_tail.is_empty() {
                // Position is in combined buffer, adjust for previous tail
                offset - prev_chunk_tail.len() as u64 + pos as u64
            } else {
                offset + pos as u64
            };
            
            return Ok(found_offset);
        }
        
        offset += n as u64;
        
        // Save last 11 bytes as tail for next iteration (MAGIC + version - 1)
        let tail_size = 11;
        if n >= tail_size {
            prev_chunk_tail = buffer[n - tail_size..n].to_vec();
        }
    }
}

/// Show genomeBin information
fn show_info<R: Read + Seek>(
    reader: &mut R,
    header: &GenomeHeader,
    magic_offset: u64,
) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧬 genomeBin v4.0 - Pure Rust Universal Format");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    // Calculate header start (magic_offset + MAGIC length)
    let header_offset = magic_offset + MAGIC.len() as u64;
    
    // Read manifest (offset is relative to header start)
    reader.seek(SeekFrom::Start(header_offset + header.manifest_offset))?;
    let mut manifest_compressed = vec![0u8; header.manifest_size as usize];
    reader.read_exact(&mut manifest_compressed)?;
    
    // Decompress manifest (Pure Rust ruzstd)
    let mut decoder = ruzstd::StreamingDecoder::new(&manifest_compressed[..])
        .context("Failed to create ruzstd decoder")?;
    let mut manifest_json = Vec::new();
    decoder.read_to_end(&mut manifest_json)
        .context("Failed to decompress manifest")?;
    
    let manifest: GenomeManifest = serde_json::from_slice(&manifest_json)
        .context("Failed to parse manifest")?;
    
    // Display info
    println!("Name:        {}", manifest.name);
    println!("Version:     {}", manifest.version);
    
    if let Some(desc) = &manifest.description {
        println!("Description: {}", desc);
    }
    
    println!();
    println!("Architectures: {}", manifest.architectures.join(", "));
    println!("Binaries:      {}", header.num_binaries);
    println!();
    println!("DNA Fingerprint (SHA256):");
    println!("  {}", hex::encode(header.fingerprint));
    println!();
    println!("Current System: {}", current_arch());
    println!();
    
    // Read binary entries
    // CRITICAL FIX: binaries_offset is relative to header start, not magic_offset
    let header_offset = magic_offset + MAGIC.len() as u64;
    reader.seek(SeekFrom::Start(header_offset + header.binaries_offset))?;
    
    println!("Binary Table:");
    for i in 0..header.num_binaries {
        let entry = BinaryEntry::read_from(reader)?;
        let arch = entry.architecture_str();
        
        println!("  [{i}] {arch}:");
        println!("      Compressed:   {} bytes", entry.compressed_size);
        println!("      Uncompressed: {} bytes", entry.uncompressed_size);
        
        if entry.uncompressed_size > 0 {
            println!("      Ratio:        {:.1}%", 
                (entry.compressed_size as f64 / entry.uncompressed_size as f64) * 100.0);
        } else {
            println!("      Ratio:        N/A");
        }
        
        println!("      Checksum:     {}", hex::encode(&entry.checksum[..8]));
    }
    
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Binary = DNA: Deterministic, Reproducible 🦀");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

/// Extract binary for current architecture
fn extract_binary<R: Read + Seek>(
    reader: &mut R,
    header: &GenomeHeader,
    magic_offset: u64,
    output_dir: &str,
    genome_path: &str,
) -> Result<()> {
    let arch = current_arch();
    
    // Calculate header start
    let header_offset = magic_offset + MAGIC.len() as u64;
    
    // Find binary for current architecture
    reader.seek(SeekFrom::Start(header_offset + header.binaries_offset))?;
    
    let mut entry: Option<BinaryEntry> = None;
    for _ in 0..header.num_binaries {
        let e = BinaryEntry::read_from(reader)?;
        if e.architecture_str() == arch {
            entry = Some(e);
            break;
        }
    }
    
    let entry = entry.context(format!("No binary available for architecture: {}", arch))?;
    
    // Calculate binary data offset
    // Offsets in header are relative to header start
    let header_offset = magic_offset + MAGIC.len() as u64;
    let binaries_data_start = header_offset + header.binaries_offset + 
        (header.num_binaries as u64 * 64); // Each entry is 64 bytes
    
    let binary_offset = binaries_data_start + entry.offset;
    
    // Read compressed binary
    reader.seek(SeekFrom::Start(binary_offset))?;
    let mut compressed = vec![0u8; entry.compressed_size as usize];
    reader.read_exact(&mut compressed)?;
    
    // Decompress binary (Pure Rust ruzstd)
    println!("Decompressing {} binary...", arch);
    let mut decoder = ruzstd::StreamingDecoder::new(&compressed[..])
        .context("Failed to create ruzstd decoder")?;
    let mut binary = Vec::new();
    decoder.read_to_end(&mut binary)
        .context("Failed to decompress binary")?;
    
    // Verify size
    if binary.len() != entry.uncompressed_size as usize {
        anyhow::bail!(
            "Size mismatch: expected {}, got {}",
            entry.uncompressed_size,
            binary.len()
        );
    }
    
    // Verify checksum
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&binary);
    let checksum: [u8; 32] = hasher.finalize().into();
    
    if checksum != entry.checksum {
        anyhow::bail!("Checksum mismatch! Binary may be corrupted.");
    }
    
    // Write to disk
    let output_name = Path::new(genome_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("genome-binary");
    
    let output_path = Path::new(output_dir).join(output_name);
    
    let mut file = File::create(&output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
    
    file.write_all(&binary)?;
    
    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&output_path, perms)?;
    }
    
    println!("✅ Extracted {} binary: {}", arch, output_path.display());
    println!("   Size: {} bytes", binary.len());
    println!("   Checksum verified: {}", hex::encode(&checksum[..8]));
    
    Ok(())
}

/// Run binary for current architecture
fn run_binary<R: Read + Seek>(
    reader: &mut R,
    header: &GenomeHeader,
    magic_offset: u64,
    args: &[String],
    genome_path: &str,
) -> Result<()> {
    use std::process::Command;
    
    // Extract to temp directory
    let temp_dir = std::env::temp_dir().join(format!("genome-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;
    
    extract_binary(reader, header, magic_offset, temp_dir.to_str().unwrap(), genome_path)?;
    
    let binary_name = Path::new(genome_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("genome-binary");
    
    let binary_path = temp_dir.join(binary_name);
    
    // Execute binary
    let status = Command::new(&binary_path)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute: {}", binary_path.display()))?;
    
    // Cleanup
    let _ = std::fs::remove_file(&binary_path);
    let _ = std::fs::remove_dir(&temp_dir);
    
    std::process::exit(status.code().unwrap_or(1));
}

/// Print usage information
fn print_usage() {
    eprintln!("genomeBin v4.0 Universal Extractor");
    eprintln!("Pure Rust - Binary as DNA");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  genome-extract <file.genome> info");
    eprintln!("  genome-extract <file.genome> extract [output_dir]");
    eprintln!("  genome-extract <file.genome> run [args...]");
    eprintln!();
    eprintln!("Or execute genomeBin directly:");
    eprintln!("  ./file.genome info");
    eprintln!("  ./file.genome extract");
    eprintln!("  ./file.genome run [args...]");
}
