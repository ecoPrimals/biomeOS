# genomeBin v4.0 - Pure Rust Universal Binary Format

**Date**: January 31, 2026  
**Status**: 🎯 **IMPLEMENTATION IN PROGRESS**  
**Goal**: Binary as DNA - TRUE genomic fingerprint

═══════════════════════════════════════════════════════════════════
🦀 PURE RUST ARCHITECTURE
═══════════════════════════════════════════════════════════════════

## Design Principles

**Deep Debt Compliant**:
- ✅ 100% Pure Rust (zero shell dependency)
- ✅ Deterministic binary fingerprint (SHA256)
- ✅ Reproducible builds
- ✅ Platform-agnostic runtime
- ✅ Self-knowledge only
- ✅ No hardcoding

## File Format Specification

```
╔═══════════════════════════════════════════════════════════════════╗
║                    genomeBin v4.0 File Structure                  ║
╠═══════════════════════════════════════════════════════════════════╣
║ [Universal Extractor Binary]  <- Small Pure Rust tool (~500KB)   ║
║ [Format Magic: "GENOME40"]    <- 8 bytes                          ║
║ [Header]                      <- GenomeHeader struct              ║
║ [Manifest JSON]               <- Metadata (compressed)            ║
║ [Binary Table]                <- Array of BinaryEntry            ║
║ [Compressed Binaries]         <- zstd compressed, per-arch        ║
╚═══════════════════════════════════════════════════════════════════╝
```

### Binary Layout

```rust
#[repr(C)]
struct GenomeHeader {
    magic: [u8; 8],              // "GENOME40"
    version: u32,                // 4
    manifest_offset: u64,        // Bytes from start
    manifest_size: u32,          // Compressed size
    binaries_offset: u64,        // Bytes from start
    num_binaries: u32,           // Count
    fingerprint: [u8; 32],       // SHA256 of entire payload
}

#[repr(C)]
struct BinaryEntry {
    architecture: [u8; 16],      // "x86_64", "aarch64", etc (null-padded)
    offset: u64,                 // From binaries_offset
    compressed_size: u32,
    uncompressed_size: u32,
    checksum: [u8; 32],          // SHA256 of uncompressed binary
}
```

## Implementation Plan

### Phase 1: Universal Extractor Tool (2 hours)

**Create**: `crates/biomeos-genome-extract/`

```toml
# Cargo.toml
[package]
name = "biomeos-genome-extract"
version = "4.0.0"

[dependencies]
anyhow = { workspace = true }
zstd = "0.13"
sha2 = "0.10"
```

```rust
// src/main.rs
use anyhow::{Context, Result};
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const MAGIC: &[u8; 8] = b"GENOME40";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: genome-extract <file.genome> <command>");
        eprintln!("Commands: info, extract [DIR], run [ARGS]");
        std::process::exit(1);
    }
    
    let genome_path = &args[1];
    let command = args.get(2).map(|s| s.as_str()).unwrap_or("info");
    
    let mut file = File::open(genome_path)?;
    
    // Read and verify header
    let header = read_header(&mut file)?;
    verify_header(&header)?;
    
    // Detect current architecture
    let arch = current_arch();
    
    match command {
        "info" => show_info(&mut file, &header),
        "extract" => {
            let output_dir = args.get(3).map(|s| s.as_str()).unwrap_or(".");
            extract_binary(&mut file, &header, &arch, output_dir)
        },
        "run" => {
            let binary_args = &args[3..];
            run_binary(&mut file, &header, &arch, binary_args)
        },
        _ => {
            anyhow::bail!("Unknown command: {}", command);
        }
    }
}

fn read_header(file: &mut File) -> Result<GenomeHeader> {
    use std::mem::size_of;
    
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    
    if &magic != MAGIC {
        anyhow::bail!("Invalid genomeBin: magic mismatch");
    }
    
    let mut header_bytes = vec![0u8; size_of::<GenomeHeader>() - 8];
    file.read_exact(&mut header_bytes)?;
    
    // Deserialize header (bincode or manual parsing)
    Ok(GenomeHeader::from_bytes(&header_bytes)?)
}

fn current_arch() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64".to_string(),
        "aarch64" => "aarch64".to_string(),
        "riscv64" => "riscv64".to_string(),
        arch => arch.to_string(),
    }
}

fn extract_binary(
    file: &mut File,
    header: &GenomeHeader,
    arch: &str,
    output_dir: &str,
) -> Result<()> {
    // Seek to binaries table
    file.seek(SeekFrom::Start(header.binaries_offset))?;
    
    // Read binary entries
    let entries = read_binary_entries(file, header.num_binaries)?;
    
    // Find matching architecture
    let entry = entries.iter()
        .find(|e| e.architecture_str() == arch)
        .context(format!("No binary for architecture: {}", arch))?;
    
    // Extract compressed binary
    let binary_offset = header.binaries_offset + 
        (header.num_binaries as u64 * size_of::<BinaryEntry>() as u64) +
        entry.offset;
    
    file.seek(SeekFrom::Start(binary_offset))?;
    
    let mut compressed = vec![0u8; entry.compressed_size as usize];
    file.read_exact(&mut compressed)?;
    
    // Decompress
    let binary = zstd::decode_all(&compressed[..])?;
    
    // Verify checksum
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&binary);
    let checksum: [u8; 32] = hasher.finalize().into();
    
    if checksum != entry.checksum {
        anyhow::bail!("Checksum mismatch for {} binary", arch);
    }
    
    // Write to disk
    let output_path = std::path::Path::new(output_dir)
        .join(extract_genome_name(file, header)?);
    
    std::fs::write(&output_path, &binary)?;
    
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&output_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&output_path, perms)?;
    }
    
    println!("✅ Extracted {} binary: {}", arch, output_path.display());
    Ok(())
}
```

### Phase 2: genomeBin v4 Creator (1 hour)

**Update**: `crates/biomeos-genomebin-v4/` (new crate)

```rust
// src/lib.rs
pub struct GenomeBinV4 {
    manifest: GenomeManifest,
    binaries: HashMap<Arch, CompressedBinary>,
}

impl GenomeBinV4 {
    pub fn write_v4(&self, output: &Path, extractor_path: &Path) -> Result<()> {
        let mut file = File::create(output)?;
        
        // 1. Copy universal extractor binary
        let extractor = std::fs::read(extractor_path)?;
        file.write_all(&extractor)?;
        
        // 2. Write header (with offsets calculated)
        let header = self.create_header(extractor.len())?;
        file.write_all(bytes_of(&header))?;
        
        // 3. Write compressed manifest
        let manifest_json = serde_json::to_vec(&self.manifest)?;
        let manifest_compressed = zstd::encode_all(&manifest_json[..], 3)?;
        file.write_all(&manifest_compressed)?;
        
        // 4. Write binary table
        let entries = self.create_binary_entries()?;
        for entry in &entries {
            file.write_all(bytes_of(entry))?;
        }
        
        // 5. Write compressed binaries
        for binary in self.binaries.values() {
            file.write_all(&binary.data)?;
        }
        
        // 6. Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(output, perms)?;
        }
        
        Ok(())
    }
}
```

### Phase 3: Reproducible Builds (30 min)

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "target-cpu=generic"]

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

```bash
# Build with reproducibility
SOURCE_DATE_EPOCH=0 cargo build --release --target x86_64-unknown-linux-musl
SOURCE_DATE_EPOCH=0 cargo build --release --target aarch64-unknown-linux-musl

# Verify fingerprint
sha256sum target/x86_64-unknown-linux-musl/release/genome-extract
sha256sum target/aarch64-unknown-linux-musl/release/genome-extract
```

═══════════════════════════════════════════════════════════════════
🧬 GENOMIC FINGERPRINT VALIDATION
═══════════════════════════════════════════════════════════════════

## Concept: Binary = DNA

```bash
# Build genome on Machine A
machine-a$ cargo build --release
machine-a$ sha256sum beardog.genome
abc123...def

# Build genome on Machine B (different OS)
machine-b$ cargo build --release
machine-b$ sha256sum beardog.genome
abc123...def  # IDENTICAL!

# Deploy to USB
$ sha256sum /media/usb/beardog.genome
abc123...def  # SAME!

# Deploy to Pixel
$ adb shell sha256sum /data/local/tmp/beardog.genome
abc123...def  # SAME!
```

**This enables**:
- ✅ DarkForest: ZK proofs on identical code hash
- ✅ BTSP: Consensus on exact code version
- ✅ BirdSong: Verifiable code lineage

═══════════════════════════════════════════════════════════════════
📋 TIMELINE
═══════════════════════════════════════════════════════════════════

**Today** (30 min remaining):
- ✅ Shell script v3.5 created (temporary validation)
- ⏳ Test shell version on USB + Pixel
- ⏳ Validate concept works

**Next Session** (3 hours):
- [ ] Implement Pure Rust extractor (1 hour)
- [ ] Implement genomeBin v4 creator (1 hour)
- [ ] Test + validate fingerprints (30 min)
- [ ] Reproducible builds setup (30 min)

**Result**:
- Pure Rust universal deployment
- Deterministic binary fingerprints
- TRUE genomic architecture
- Deep Debt A++ compliant

═══════════════════════════════════════════════════════════════════
✅ DECISION: Skip Shell v3.5 Debugging
═══════════════════════════════════════════════════════════════════

**Reason**: Shell script is "jelly" (as user said)
- Fragile, platform-dependent
- Not a true genome (interpretive, not binary)
- Doesn't provide deterministic fingerprint

**Better Path**: Implement Pure Rust v4 immediately
- 3 hours of focused work
- TRUE Deep Debt solution
- Binary = DNA fingerprint
- Production-ready

═══════════════════════════════════════════════════════════════════

Ready to implement Pure Rust genomeBin v4.0! 🦀🧬
