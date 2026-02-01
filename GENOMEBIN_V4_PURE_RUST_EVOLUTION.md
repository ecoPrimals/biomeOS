# genomeBin v4.0 → v4.1 Pure Rust Evolution Plan

**Date**: January 31, 2026  
**Status**: 🎯 **STRATEGIC ROADMAP**  
**Goal**: 100% Pure Rust + Universal Bootstrap Architecture

═══════════════════════════════════════════════════════════════════
🎯 STRATEGIC VISION
═══════════════════════════════════════════════════════════════════

## User Directives

> "Option 3 and 4 are the path forward"

### Option 3: Pure Rust zstd Alternative ✅ **REQUIRED**
**Rationale**: ecoBin standards mandate NO C dependencies
**Status**: Must evolve from `zstd-sys` (C binding) to pure Rust
**Impact**: Enables clean cross-compilation to ALL platforms

### Option 4: Multi-Arch Fat genomeBin ✅ **STANDARD**
**Rationale**: Already a genomeBin standard, must implement
**Status**: Embed multiple architecture extractors in single file
**Impact**: TRUE universal bootstrap - one file, native execution everywhere

### Long-Term: Universal Bootstrap Binary 🌟 **VISION**
**Rationale**: Completely architecture/hardware agnostic
**Status**: Late-term research goal
**Impact**: Run on ANY system without recompilation

═══════════════════════════════════════════════════════════════════
📋 PHASE 1: PURE RUST ZSTD (IMMEDIATE)
═══════════════════════════════════════════════════════════════════

## Goal: Eliminate C Dependency

**Current State**:
- ❌ `zstd` crate = binding to C library (`zstd-sys`)
- ❌ Cross-compile fails on ARM64 (linker errors)
- ❌ Violates ecoBin standards (C dependency)

**Target State**:
- ✅ 100% Pure Rust decompression
- ✅ Clean cross-compilation to all platforms
- ✅ ecoBin v2.0 compliant (zero C dependencies)

## Solution: `ruzstd` Pure Rust Decoder

**Crate**: `ruzstd` v0.7+  
**Type**: Pure Rust zstd decoder (no C bindings)  
**License**: MIT  
**Performance**: Comparable to C implementation

### Implementation Steps

#### Step 1: Update Dependencies (5 minutes)

**File**: `crates/biomeos-genome-extract/Cargo.toml`
```toml
[dependencies]
# BEFORE:
# zstd = "0.13"

# AFTER:
ruzstd = "0.7"  # Pure Rust decoder
```

**File**: `crates/biomeos-genomebin-v3/Cargo.toml`
```toml
[dependencies]
# BEFORE:
# zstd = "0.13"

# AFTER:
ruzstd = "0.7"  # Pure Rust encoder/decoder
```

#### Step 2: Update Extractor Code (10 minutes)

**File**: `crates/biomeos-genome-extract/src/main.rs`

**BEFORE**:
```rust
use std::io::Read;

// Decompress manifest
let mut decoder = zstd::Decoder::new(&compressed_manifest[..])?;
let mut manifest_json = Vec::new();
decoder.read_to_end(&mut manifest_json)?;
```

**AFTER**:
```rust
use ruzstd::StreamingDecoder;
use std::io::Read;

// Decompress manifest (Pure Rust)
let mut decoder = StreamingDecoder::new(&compressed_manifest[..])?;
let mut manifest_json = Vec::new();
decoder.read_to_end(&mut manifest_json)?;
```

#### Step 3: Update Creator Code (10 minutes)

**File**: `crates/biomeos-genomebin-v3/src/v4.rs`

**BEFORE**:
```rust
// Compress manifest
let compressed_manifest = zstd::encode_all(
    manifest_json.as_bytes(),
    19  // Max compression
)?;
```

**AFTER**:
```rust
use ruzstd::encoding::FrameEncoder;

// Compress manifest (Pure Rust)
let mut encoder = FrameEncoder::new(Vec::new(), 19)?;
encoder.write_all(manifest_json.as_bytes())?;
let compressed_manifest = encoder.finish()?;
```

#### Step 4: Test & Validate (15 minutes)

```bash
# Build with pure Rust
cargo build --release -p biomeos-genome-extract

# Cross-compile to ARM64 (should work now!)
cargo build --release \
  --target aarch64-unknown-linux-musl \
  -p biomeos-genome-extract

# Test on x86_64
./beardog-v4.genome info

# Test extraction
./beardog-v4.genome extract /tmp/test/

# Verify checksums
sha256sum /tmp/test/beardog-v4
```

### Success Criteria

✅ No C dependencies in dependency tree  
✅ ARM64 cross-compilation succeeds  
✅ x86_64 extraction still works  
✅ ARM64 extraction works natively  
✅ Compression ratio maintained (~40-50%)  
✅ Performance comparable (within 10% of zstd-sys)

### Validation

```bash
# Check dependency tree for C libs
cargo tree -p biomeos-genome-extract | grep -i "sys"
# Expected: No results

# Check for C compiler usage
cargo build --release -p biomeos-genome-extract -v 2>&1 | grep "gcc\|clang"
# Expected: No results (only rustc)

# Cross-compile test
cargo build --release \
  --target aarch64-unknown-linux-musl \
  -p biomeos-genome-extract
# Expected: Success!
```

## Timeline

**Total Time**: ~40 minutes  
- Dependency updates: 5 min
- Code changes: 20 min
- Testing & validation: 15 min

**Risk**: Low (ruzstd is mature, well-tested)

═══════════════════════════════════════════════════════════════════
📋 PHASE 2: MULTI-ARCH FAT GENOMEBIN (v4.1 STANDARD)
═══════════════════════════════════════════════════════════════════

## Goal: Embed Multiple Architecture Extractors

**Current State** (v4.0):
- Single extractor (e.g., x86_64)
- Works only on matching architecture
- Requires separate extractor per platform

**Target State** (v4.1):
- Multiple extractors embedded (x86_64, ARM64, RISC-V, etc.)
- Runtime architecture detection
- Native execution on ALL platforms
- **TRUE universal single-file deployment**

## Architecture

### File Structure (v4.1)

```
┌─────────────────────────────────────────────────────┐
│ BOOTSTRAP SELECTOR (Shell Script, ~1KB)            │
│ - Detects current architecture                      │
│ - Calculates offset to correct extractor           │
│ - Executes appropriate extractor                    │
├─────────────────────────────────────────────────────┤
│ EXTRACTOR TABLE (Binary, 128 bytes)                │
│ - Entry per architecture (arch, offset, size)      │
│ - Used by bootstrap selector                        │
├─────────────────────────────────────────────────────┤
│ x86_64 EXTRACTOR (Pure Rust, ~741KB)               │
│ - Native x86_64 ELF binary                          │
│ - Runs on x86_64 Linux, BSD, etc.                  │
├─────────────────────────────────────────────────────┤
│ ARM64 EXTRACTOR (Pure Rust, ~741KB)                │
│ - Native ARM64 ELF binary                           │
│ - Runs on ARM64 Linux, Android, etc.               │
├─────────────────────────────────────────────────────┤
│ RISC-V EXTRACTOR (Pure Rust, ~741KB) [OPTIONAL]    │
│ - Native RISC-V ELF binary                          │
│ - Runs on RISC-V Linux                              │
├─────────────────────────────────────────────────────┤
│ GENOME40 MAGIC (8 bytes)                            │
├─────────────────────────────────────────────────────┤
│ GENOME HEADER (60 bytes)                            │
│ - Version, offsets, fingerprint                     │
├─────────────────────────────────────────────────────┤
│ MANIFEST (Compressed JSON, ~220 bytes)             │
├─────────────────────────────────────────────────────┤
│ BINARY TABLE (64 bytes × N architectures)          │
├─────────────────────────────────────────────────────┤
│ x86_64 PAYLOAD BINARY (Compressed, zstd)           │
├─────────────────────────────────────────────────────┤
│ ARM64 PAYLOAD BINARY (Compressed, zstd)            │
├─────────────────────────────────────────────────────┤
│ RISC-V PAYLOAD BINARY (Compressed, zstd) [OPT]     │
└─────────────────────────────────────────────────────┘

Total overhead: ~1KB (bootstrap) + ~750KB per extractor arch
Example: x86_64 + ARM64 = ~1.5MB overhead per genomeBin
```

### Bootstrap Selector (Shell Script)

**File**: `crates/biomeos-genomebin-v3/bootstrap-selector.sh`

```bash
#!/bin/sh
# genomeBin v4.1 Universal Bootstrap Selector
# Detects architecture and executes correct extractor

SELF="$0"
ARCH=$(uname -m)

# Normalize architecture
case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    riscv64) ARCH="riscv64" ;;
    *) echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

# Read extractor table (128 bytes after this script)
SCRIPT_SIZE=1024  # Fixed size (padded)
TABLE_OFFSET=$SCRIPT_SIZE

# Parse table to find extractor offset
# Table format: [arch:16][offset:8][size:8] × N entries
# For simplicity, we'll use a fixed offset strategy:
#   x86_64 extractor at: SCRIPT_SIZE + TABLE_SIZE
#   ARM64 extractor at:  SCRIPT_SIZE + TABLE_SIZE + 750KB
#   etc.

TABLE_SIZE=128
EXTRACTOR_SIZE=750000  # ~750KB

case "$ARCH" in
    x86_64)
        EXTRACTOR_OFFSET=$((SCRIPT_SIZE + TABLE_SIZE))
        ;;
    aarch64)
        EXTRACTOR_OFFSET=$((SCRIPT_SIZE + TABLE_SIZE + EXTRACTOR_SIZE))
        ;;
    riscv64)
        EXTRACTOR_OFFSET=$((SCRIPT_SIZE + TABLE_SIZE + EXTRACTOR_SIZE * 2))
        ;;
esac

# Extract extractor to temp, execute with genomeBin as argument
TEMP_EXTRACTOR=$(mktemp)
trap "rm -f $TEMP_EXTRACTOR" EXIT

dd if="$SELF" bs=1 skip=$EXTRACTOR_OFFSET count=$EXTRACTOR_SIZE 2>/dev/null > "$TEMP_EXTRACTOR"
chmod +x "$TEMP_EXTRACTOR"

# Execute extractor with self as genomeBin path
exec "$TEMP_EXTRACTOR" "$SELF" "$@"
```

### Implementation Steps

#### Step 1: Build Multi-Arch Extractors (20 minutes)

```bash
# Build x86_64 extractor (musl for portability)
cargo build --release \
  --target x86_64-unknown-linux-musl \
  -p biomeos-genome-extract

# Build ARM64 extractor (musl for portability)
cargo build --release \
  --target aarch64-unknown-linux-musl \
  -p biomeos-genome-extract

# Optional: Build RISC-V extractor
cargo build --release \
  --target riscv64gc-unknown-linux-gnu \
  -p biomeos-genome-extract

# Collect extractors
mkdir -p target/extractors/
cp target/x86_64-unknown-linux-musl/release/genome-extract \
   target/extractors/genome-extract-x86_64
cp target/aarch64-unknown-linux-musl/release/genome-extract \
   target/extractors/genome-extract-aarch64
```

#### Step 2: Implement Fat genomeBin Creator (30 minutes)

**File**: `crates/biomeos-genomebin-v3/src/v4_1.rs`

```rust
use crate::{Arch, CompressedBinary, GenomeBin};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;

impl GenomeBin {
    /// Write genomeBin v4.1 - Multi-Architecture Fat Binary
    pub fn write_v4_1(
        &self,
        output: &Path,
        extractors: &[(Arch, &Path)],  // List of (arch, extractor_path)
    ) -> Result<()> {
        let mut file = File::create(output)
            .context("Failed to create genomeBin v4.1")?;

        // 1. Write bootstrap selector (padded to 1KB)
        let bootstrap = include_bytes!("../bootstrap-selector.sh");
        file.write_all(bootstrap)?;
        let padding = vec![0u8; 1024 - bootstrap.len()];
        file.write_all(&padding)?;

        // 2. Write extractor table (128 bytes)
        let table = self.build_extractor_table(extractors)?;
        file.write_all(&table)?;

        // 3. Write extractors (one per architecture)
        for (arch, extractor_path) in extractors {
            let mut extractor_data = Vec::new();
            File::open(extractor_path)?
                .read_to_end(&mut extractor_data)?;
            
            // Pad to fixed size (750KB)
            extractor_data.resize(750_000, 0);
            file.write_all(&extractor_data)?;
            
            eprintln!("✅ Embedded {} extractor ({} bytes)", 
                arch_to_string(arch), extractor_data.len());
        }

        // 4. Write GENOME40 payload (same as v4.0)
        file.write_all(MAGIC)?;
        
        // ... (rest of v4.0 implementation)
        
        Ok(())
    }
    
    fn build_extractor_table(
        &self,
        extractors: &[(Arch, &Path)],
    ) -> Result<Vec<u8>> {
        let mut table = Vec::new();
        let mut offset = 1024 + 128;  // After bootstrap + table
        
        for (arch, _) in extractors {
            // Entry: [arch:16][offset:8][size:8]
            let mut arch_bytes = [0u8; 16];
            let arch_str = arch_to_string(arch);
            arch_bytes[..arch_str.len()].copy_from_slice(arch_str.as_bytes());
            
            table.extend_from_slice(&arch_bytes);
            table.extend_from_slice(&offset.to_le_bytes());
            table.extend_from_slice(&750_000u64.to_le_bytes());
            
            offset += 750_000;
        }
        
        // Pad to 128 bytes
        table.resize(128, 0);
        Ok(table)
    }
}
```

#### Step 3: Update CLI (10 minutes)

**File**: `crates/biomeos-cli/src/commands/genome.rs`

```rust
#[derive(Parser)]
pub struct GenomeCreateArgs {
    // ... existing fields ...
    
    /// Create genomeBin v4.1 (multi-arch fat binary)
    #[arg(long)]
    pub v4_1: bool,
    
    /// Extractor architectures to embed (comma-separated)
    /// Example: x86_64,aarch64,riscv64
    #[arg(long, default_value = "x86_64,aarch64")]
    pub extractor_arches: String,
}

pub async fn create_genome(args: GenomeCreateArgs) -> Result<()> {
    // ... existing code ...
    
    if args.v4_1 {
        eprintln!("Creating genomeBin v4.1 (multi-arch fat binary)...");
        
        // Parse extractor architectures
        let arches: Vec<&str> = args.extractor_arches.split(',').collect();
        let mut extractors = Vec::new();
        
        for arch in arches {
            let arch_enum = match arch.trim() {
                "x86_64" => Arch::X86_64,
                "aarch64" => Arch::Aarch64,
                "riscv64" => Arch::Riscv64,
                _ => bail!("Unsupported extractor arch: {}", arch),
            };
            
            let extractor_path = format!(
                "target/extractors/genome-extract-{}",
                arch.trim()
            );
            
            extractors.push((arch_enum, PathBuf::from(extractor_path)));
        }
        
        genome_bin.write_v4_1(&output_path, &extractors)?;
    } else if args.v4 {
        // ... existing v4.0 logic ...
    }
    
    Ok(())
}
```

#### Step 4: Test Multi-Arch Fat genomeBin (20 minutes)

```bash
# Build extractors
./scripts/build-all-extractors.sh

# Create fat genomeBin
biomeos genome create beardog \
  --binary x86_64=target/x86_64-unknown-linux-musl/release/beardog \
  --binary aarch64=target/aarch64-unknown-linux-musl/release/beardog \
  --extractor-arches x86_64,aarch64 \
  --v4.1

# Test on x86_64
./beardog-v4.1.genome info   # Should use x86_64 extractor
./beardog-v4.1.genome extract /tmp/test/

# Deploy to Pixel (ARM64)
adb push beardog-v4.1.genome /data/local/tmp/
adb shell /data/local/tmp/beardog-v4.1.genome info  # Should use ARM64 extractor!
```

### Size Analysis

**v4.0 (Single Extractor)**:
- Extractor: 741KB
- Payload (2 arches): ~3-17 MB (depends on primal)
- Total: ~4-18 MB

**v4.1 (Multi-Arch Fat)**:
- Bootstrap: 1KB
- Extractor table: 128 bytes
- x86_64 extractor: 750KB
- ARM64 extractor: 750KB
- Payload (2 arches): ~3-17 MB (same)
- **Total: ~5-19 MB** (+750KB per additional extractor arch)

**Trade-off**: +750KB per architecture for TRUE universal execution

### Success Criteria

✅ Single genomeBin file works natively on x86_64 AND ARM64  
✅ No external tools needed (extractor is embedded)  
✅ Bootstrap selector is architecture-agnostic  
✅ Execution is native (no emulation)  
✅ Size overhead is acceptable (~750KB per arch)

## Timeline

**Total Time**: ~80 minutes  
- Build extractors: 20 min
- Implement fat genomeBin: 30 min
- CLI integration: 10 min
- Testing: 20 min

**Risk**: Medium (new format, needs validation)

═══════════════════════════════════════════════════════════════════
📋 PHASE 3: UNIVERSAL BOOTSTRAP BINARY (LONG-TERM VISION)
═══════════════════════════════════════════════════════════════════

## Goal: Architecture-Agnostic Execution

**Vision**: A truly universal binary that can execute on ANY architecture without recompilation or architecture-specific code.

### Research Areas

#### 1. Bytecode + JIT Compilation

**Concept**: Ship bytecode, JIT compile to native on first run

**Technologies**:
- WebAssembly (WASM) - Near-native performance
- LLVM IR - Optimize at runtime
- Custom bytecode - Minimal interpreter

**Example**:
```
┌─────────────────────────────────────────┐
│ Tiny Bootstrap (~10KB)                  │
│ - Detects CPU architecture              │
│ - Loads appropriate JIT compiler        │
├─────────────────────────────────────────┤
│ WASM Bytecode (~500KB)                  │
│ - Architecture-agnostic                 │
│ - JIT compiles to native                │
├─────────────────────────────────────────┤
│ GENOME40 Payload (as before)            │
└─────────────────────────────────────────┘
```

**Pros**:
- TRUE universal (one bytecode, all arches)
- Smaller file size (~500KB vs 750KB × N)
- Future-proof (new arches don't need recompilation)

**Cons**:
- JIT compiler complexity
- Cold start penalty (~100-200ms)
- Runtime dependency (WASM engine)

#### 2. Interpreted Bootstrap

**Concept**: Pure interpreter (like Python/Lua) in native code per arch

**Example**:
```
┌─────────────────────────────────────────┐
│ Shell Script Bootstrap (~2KB)           │
│ - Detects if sh/bash/python available   │
│ - Falls back to pure-binary if not      │
├─────────────────────────────────────────┤
│ Python/Lua Interpreter Code (~50KB)     │
│ - Runs on any system with python        │
├─────────────────────────────────────────┤
│ Native Extractors (fallback, 750KB×N)   │
└─────────────────────────────────────────┘
```

**Pros**:
- Works on most systems (python/sh ubiquitous)
- Simple implementation
- Gradual fallback strategy

**Cons**:
- Not truly universal (depends on system tools)
- Performance penalty
- ecoBin violation (depends on external tools)

#### 3. Self-Modifying Binary

**Concept**: Binary that rewrites itself to match target architecture

**Example**:
```
┌─────────────────────────────────────────┐
│ Polyglot Binary (~5KB)                  │
│ - Valid on x86_64, ARM64, RISC-V        │
│ - Detects architecture at runtime       │
│ - Extracts correct extractor            │
│ - Exec() into extractor                 │
├─────────────────────────────────────────┤
│ Embedded Extractors (750KB×N)           │
└─────────────────────────────────────────┘
```

**Pros**:
- True single-file execution
- No external dependencies
- Pure binary (ecoBin compliant)

**Cons**:
- Polyglot binary is complex
- Limited platform support
- Security concerns (self-modifying code)

### Recommended Path: WASM + Native Fallback

**Phase 3A** (6-12 months):
1. Implement WASM-based universal extractor
2. JIT compile on first run (cache native)
3. Fall back to embedded native extractors if WASM unavailable

**Phase 3B** (12-18 months):
1. Research polyglot binary techniques
2. Prototype truly universal bootstrap
3. Validate on exotic architectures (RISC-V, PowerPC, etc.)

**Phase 3C** (18-24 months):
1. Optimize WASM bytecode size
2. Implement ahead-of-time compilation option
3. Ship production universal bootstrap

### Success Criteria (Long-Term)

✅ Single binary works on x86_64, ARM64, RISC-V, PowerPC, etc.  
✅ No recompilation needed for new architectures  
✅ Performance within 10% of native  
✅ Size overhead < 1MB  
✅ Zero external dependencies  
✅ ecoBin v2.0 compliant

═══════════════════════════════════════════════════════════════════
📊 SUMMARY & ROADMAP
═══════════════════════════════════════════════════════════════════

## Immediate (1-2 hours) - Phase 1

**Pure Rust zstd Evolution**:
- Replace `zstd` with `ruzstd`
- Fix ARM64 cross-compilation
- Validate ecoBin compliance
- **Result**: 100% Pure Rust, zero C dependencies ✅

## Near-Term (2-4 hours) - Phase 2

**Multi-Arch Fat genomeBin v4.1**:
- Implement bootstrap selector
- Embed multiple extractors
- Test on x86_64 + ARM64
- **Result**: TRUE universal single-file deployment ✅

## Mid-Term (6-12 months) - Phase 3A

**WASM Universal Bootstrap**:
- Research WASM for extractor
- Implement JIT compilation
- Benchmark performance
- **Result**: Architecture-agnostic bytecode

## Long-Term (12-24 months) - Phase 3B/C

**Truly Universal Binary**:
- Polyglot binary research
- Exotic architecture support
- Production optimization
- **Result**: ONE binary, ALL systems

═══════════════════════════════════════════════════════════════════
🎯 IMMEDIATE NEXT STEPS
═══════════════════════════════════════════════════════════════════

## Session Goals (This Session)

1. **Replace zstd with ruzstd** (~40 min)
   - Update Cargo.toml dependencies
   - Update extractor code
   - Update creator code
   - Test & validate

2. **Start Fat genomeBin Implementation** (~40 min)
   - Create bootstrap selector script
   - Implement v4_1 module skeleton
   - Build multi-arch extractors
   - Draft CLI integration

3. **Document Architecture** (~20 min)
   - Write fat genomeBin spec
   - Document long-term vision
   - Update Deep Debt metrics

**Total**: ~100 minutes (1.5 hours)

## Deliverables

- ✅ Pure Rust zstd implementation
- ✅ ARM64 cross-compilation working
- ✅ Fat genomeBin v4.1 design complete
- ✅ Bootstrap selector implemented
- ✅ Documentation updated

═══════════════════════════════════════════════════════════════════

**Status**: 🎯 Ready to Execute  
**Priority**: HIGH (ecoBin compliance + standard feature)  
**Timeline**: Immediate (Phase 1) → Near-Term (Phase 2) → Long-Term (Phase 3)

**Let's evolve to 100% Pure Rust and TRUE universal deployment!** 🧬🦀✨

═══════════════════════════════════════════════════════════════════
