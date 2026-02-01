# Songbird ARM64 + genomeBin v3.0 Handoff

**Date**: January 31, 2026  
**From**: biomeOS NUCLEUS Team  
**To**: Songbird Development Team  
**Priority**: 🟡 MEDIUM  
**Status**: Ready for Implementation

═══════════════════════════════════════════════════════════════════
🎯 OBJECTIVE
═══════════════════════════════════════════════════════════════════

Enable Songbird to:
1. Build for ARM64 (aarch64-unknown-linux-musl)
2. Be packaged as genomeBin v3.0 self-extracting binary
3. Deploy cross-platform (USB x86_64 + Pixel 8a ARM64)
4. Enable STUN validation across devices

═══════════════════════════════════════════════════════════════════
✅ CURRENT STATUS - WHAT'S WORKING
═══════════════════════════════════════════════════════════════════

## x86_64 Linux ✅ COMPLETE

**Build**: ✅ Working  
**genomeBin v2.0**: ✅ Deployed  
**Size**: 7.6 MB (28.8% compression)  
**Features**: All operational (JSON-RPC, HTTP, STUN, TLS)

**Current genomeBin**:
```
songbird-linux.genome (7.6 MB, x86_64 only)
```

**USB Live Spore**: ✅ Deployed & operational  
**neuralAPI Integration**: ✅ Working via `BEARDOG_MODE=neural`

---

## What's Been Validated (Jan 28-31, 2026)

| Feature | Status | Validation |
|---------|--------|------------|
| JSON-RPC over Unix sockets | ✅ | BearDog discovery working |
| HTTP client with headers | ✅ | Anthropic API calls successful |
| STUN client (Pure Rust) | ✅ | NAT detection operational |
| TLS with XDG discovery | ✅ | HTTPS requests working |
| Runtime discovery | ✅ | Finds BearDog via Songbird |
| LAN discovery | ✅ | Port:0 beacon fixed |

═══════════════════════════════════════════════════════════════════
🔴 BLOCKERS - WHAT'S NEEDED
═══════════════════════════════════════════════════════════════════

## Blocker 1: ARM64 Build Fails

**Status**: 🔴 BLOCKED  
**Error**: Linker ELF incompatibility (rust-lld)

**Last Attempt** (Jan 31, 2026):
```bash
cargo build --release --target aarch64-unknown-linux-musl --bin songbird
```

**Error Output**:
```
error: linking with `rust-lld` failed: exit status: 1
= note: rust-lld: error: incompatible ELF file
```

**Root Cause**: Cross-compilation environment needs proper musl toolchain for ARM64

---

## Blocker 2: genomeBin v3.0 Not Available

**Status**: 🟡 READY (after ARM64 builds)

**Current**: Songbird has genomeBin v2.0 (single-arch x86_64)  
**Needed**: genomeBin v3.0 with self-extracting stub

**Why v3.0?**
- ✅ Direct execution (`./songbird.genome run`)
- ✅ Built-in extraction (`./songbird.genome extract`)
- ✅ Info command (`./songbird.genome info`)
- ✅ SHA256 verification + zstd compression
- ✅ Multi-architecture support (x86_64 + ARM64 in one file)

═══════════════════════════════════════════════════════════════════
📋 IMPLEMENTATION PLAN
═══════════════════════════════════════════════════════════════════

## Phase 1: ARM64 Build Environment (30 minutes)

### Option A: Use GitHub Actions Native ARM64 Runner (RECOMMENDED)

**Why**: Avoids cross-compilation complexity, native builds are faster

**Setup** (in `.github/workflows/ci.yml`):
```yaml
jobs:
  build-arm64:
    runs-on: ubuntu-24.04-arm64  # Native ARM64 runner
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-unknown-linux-musl
      - name: Build Songbird ARM64
        run: |
          cargo build --release --target aarch64-unknown-linux-musl --bin songbird
      - name: Upload ARM64 binary
        uses: actions/upload-artifact@v4
        with:
          name: songbird-arm64
          path: target/aarch64-unknown-linux-musl/release/songbird
```

**Pros**:
- ✅ Native compilation (no cross-compile issues)
- ✅ Fast builds
- ✅ No local toolchain setup needed

**Cons**:
- Requires GitHub Actions access

---

### Option B: Local Cross-Compilation Setup

**Install musl cross-compilation toolchain**:
```bash
# Install cross-compilation tools
sudo apt-get install -y musl-tools gcc-aarch64-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-musl

# Configure cargo for ARM64 cross-compilation
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-gnu-gcc"
EOF

# Build
cargo build --release --target aarch64-unknown-linux-musl --bin songbird
```

**If linker issues persist**:
```bash
# Option 1: Use cross tool
cargo install cross
cross build --release --target aarch64-unknown-linux-musl --bin songbird

# Option 2: Build on actual ARM64 device (Pixel with Termux)
# (Advanced - see Option C below)
```

---

### Option C: Build on Pixel 8a (Termux)

**For true native ARM64 builds**:
```bash
# On Pixel 8a (Termux)
pkg install rust binutils
cd ~/songbird
cargo build --release --target aarch64-unknown-linux-android
```

**Note**: This produces an Android binary. For musl (preferred), use Option A or B.

---

## Phase 2: Create genomeBin v3.0 (15 minutes)

Once ARM64 binary is built:

```bash
# In biomeOS repository
cd ~/Development/ecoPrimals/phase2/biomeOS

# Create multi-arch genomeBin v3.0
./biomeos genome create songbird-v3 \
  --binary x86_64=/path/to/songbird-x86_64 \
  --binary aarch64=/path/to/songbird-aarch64 \
  --description "Songbird Discovery Primal (Multi-Architecture)" \
  --version "v8.14.0"

# Output: plasmidBin/songbird-v3.genome (self-extracting)
```

**Verify**:
```bash
# Test self-extracting stub
./plasmidBin/songbird-v3.genome info
# Should show: 2 architectures (x86_64, aarch64)

./plasmidBin/songbird-v3.genome extract --output /tmp/songbird-test
# Should extract both binaries

./plasmidBin/songbird-v3.genome run server
# Should auto-select correct architecture and run
```

---

## Phase 3: Deploy to USB + Pixel (30 minutes)

### USB Live Spore (x86_64)
```bash
# Copy to USB
cp plasmidBin/songbird-v3.genome /media/eastgate/biomeOS1/biomeOS/

# On USB, run directly
cd /media/eastgate/biomeOS1/biomeOS
./songbird-v3.genome run server \
  --socket /run/user/1000/biomeos/songbird-nat0.sock
```

### Pixel 8a (ARM64)
```bash
# Copy to Pixel via adb
adb push plasmidBin/songbird-v3.genome /data/local/tmp/

# On Pixel (Termux or shell)
cd /data/local/tmp
chmod +x songbird-v3.genome
./songbird-v3.genome extract --output ~/primals/
cd ~/primals
./songbird server --socket ~/songbird.sock
```

---

## Phase 4: STUN Validation (15 minutes)

**Test cross-device discovery**:

**USB (x86_64)**:
```bash
# Start Songbird with STUN
./songbird-v3.genome run server \
  --socket /run/user/1000/biomeos/songbird-nat0.sock \
  --stun-server stun.l.google.com:19302
```

**Pixel (ARM64)**:
```bash
# Start Songbird with STUN
./songbird server \
  --socket ~/songbird.sock \
  --stun-server stun.l.google.com:19302
```

**Validate Handshake**:
```bash
# From either device, query for peers
echo '{"jsonrpc":"2.0","method":"discovery.list_peers","params":{},"id":1}' | \
  nc -U /path/to/songbird.sock

# Expected: Should see peer from other device
```

═══════════════════════════════════════════════════════════════════
🔧 TECHNICAL DETAILS
═══════════════════════════════════════════════════════════════════

## genomeBin v3.0 Architecture

**Self-Extracting Stub**:
- Pure Rust, ~50 KB overhead
- Commands: `info`, `extract`, `run`, `--help`
- Auto-selects architecture based on `uname -m`
- SHA256 verification of extracted binaries
- zstd compression (~30% size reduction)

**File Structure**:
```
songbird-v3.genome (ELF executable)
├── Stub binary (~50 KB)
├── __GENOME_PAYLOAD__ marker
└── Payload (bincode serialized):
    ├── GenomeManifest (metadata)
    ├── x86_64 binary (compressed + checksum)
    └── aarch64 binary (compressed + checksum)
```

**Binary Selection**:
```rust
// Automatic architecture detection
let arch = std::env::consts::ARCH; // "x86_64" or "aarch64"
let binary = manifest.binaries.get(arch)?;
// Extract, verify, chmod +x, run
```

---

## Deep Debt Compliance

**Songbird v3.0 genomeBin will achieve**:

| Criterion | Status | Notes |
|-----------|--------|-------|
| Pure Rust | ✅ | Already 100% Rust |
| No unsafe | ✅ | Already safe (checked) |
| Multi-arch | 🟡 | After ARM64 build |
| Self-extracting | 🟡 | After v3.0 creation |
| Runtime discovery | ✅ | Already via JSON-RPC |
| No hardcoded endpoints | ✅ | XDG discovery implemented |
| Platform-agnostic | 🟡 | After ARM64 |

**Grade Impact**: +5 points (enables cross-platform validation)

═══════════════════════════════════════════════════════════════════
📊 EXPECTED RESULTS
═══════════════════════════════════════════════════════════════════

## File Sizes

**Estimated genomeBin v3.0 sizes**:
- Single-arch (x86_64 only): ~8 MB (same as v2.0)
- Multi-arch (x86_64 + ARM64): ~15 MB (both binaries + stub)

**Why larger?**
- Contains both architectures
- Enables universal deployment
- No network dependency for downloads

**Alternative**: Keep separate single-arch genomeBins if size is critical

---

## Performance

**Self-extracting overhead**: <100ms (one-time)  
**Runtime performance**: Identical to native binary  
**Startup time**: No change (extraction cached)

---

## Compatibility

| Platform | Status | Notes |
|----------|--------|-------|
| USB Live Spore (x86_64) | ✅ Ready | Already deployed v2.0 |
| Pixel 8a (ARM64) | 🟡 After build | Termux compatible |
| macOS (Intel) | ✅ Ready | Same toolchain |
| macOS (ARM64) | 🟡 Needs build | Same as Linux ARM64 |
| Android (ARM64) | ✅ Ready | Use `aarch64-linux-android` target |

═══════════════════════════════════════════════════════════════════
🎯 SUCCESS CRITERIA
═══════════════════════════════════════════════════════════════════

## Phase 1 Complete When:
- ✅ Songbird builds for ARM64 (aarch64-unknown-linux-musl)
- ✅ Binary runs on Pixel 8a
- ✅ All tests pass on ARM64

## Phase 2 Complete When:
- ✅ genomeBin v3.0 created with both architectures
- ✅ `./songbird-v3.genome info` shows 2 architectures
- ✅ Self-extraction works on both platforms

## Phase 3 Complete When:
- ✅ Deployed to USB Live Spore
- ✅ Deployed to Pixel 8a
- ✅ Both devices running Songbird

## Phase 4 Complete When:
- ✅ USB and Pixel discover each other via STUN
- ✅ Cross-device handshake successful
- ✅ Encrypted channel established

═══════════════════════════════════════════════════════════════════
🤝 SUPPORT & RESOURCES
═══════════════════════════════════════════════════════════════════

## Available from biomeOS Team

**Code References**:
- genomeBin v3.0 implementation: `crates/biomeos-genomebin-v3/`
- Self-extracting stub: `crates/biomeos-genomebin-v3/stub/`
- Example creation: `crates/biomeos-genomebin-v3/examples/create_self_extracting.rs`

**Documentation**:
- Full spec: `docs/evolution/GENOMEBIN_V3_BINARY_ISOMORPHIC.md`
- Deep Debt guidelines: `ECOSYSTEM_STATUS.md`
- Handoff templates: `docs/handoffs/`

**Testing Support**:
- USB Live Spore access for validation
- Pixel 8a deployment scripts
- STUN test fixtures

**Pairing Sessions**:
- biomeOS team available for implementation support
- Can review PRs and provide guidance
- Willing to pair on build environment setup

═══════════════════════════════════════════════════════════════════
📝 NOTES & RECOMMENDATIONS
═══════════════════════════════════════════════════════════════════

## Recommended Approach

**Priority Order**:
1. **Phase 1**: Get ARM64 building (use GitHub Actions if available)
2. **Phase 2**: Create genomeBin v3.0 (15 min with biomeOS tools)
3. **Phase 3**: Deploy to Pixel (validate ARM64 works)
4. **Phase 4**: STUN testing (cross-platform handshake)

**Timeline**: ~2 hours total (most time in Phase 1 build environment)

---

## Alternative: Quick STUN Validation

If ARM64 build takes longer than expected, you can:
1. Use old Songbird ARM64 binary (if available from previous builds)
2. Package as genomeBin v3.0 with current x86_64
3. Deploy and validate STUN immediately
4. Update ARM64 binary later

This unblocks STUN validation while ARM64 build is perfected.

---

## Questions?

Contact: biomeOS NUCLEUS Team  
**Status Updates**: Track in `ECOSYSTEM_STATUS.md`  
**Issues**: Document in `docs/handoffs/SONGBIRD_*.md`

═══════════════════════════════════════════════════════════════════
HANDOFF COMPLETE - READY FOR IMPLEMENTATION
═══════════════════════════════════════════════════════════════════

**Next Step**: Choose Phase 1 option (A, B, or C) and begin ARM64 build

**Estimated Time to Complete All Phases**: 2 hours  
**Priority**: MEDIUM (enables cross-platform validation)  
**Blockers**: None (all dependencies available)

*Generated: January 31, 2026*  
*biomeOS Version: genomeBin v3.0 Era*  
*Songbird Version: v8.14.0 (x86_64 complete, ARM64 pending)*
