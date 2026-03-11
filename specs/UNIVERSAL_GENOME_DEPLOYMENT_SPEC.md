# Universal Genome Deployment Specification
**Version**: 1.0.0
**Status**: EVOLUTION (from NUC deployment learnings)
**Updated**: February 13, 2026

---

## Vision

A USB spore plugged into **ANY device** (iPhone, Windows PC, Pixel, Raspberry Pi, server) should be able to deploy biomeOS or at minimum establish communication with the family mesh.

**Philosophy**: Start UNIVERSAL, specialize downstream. The genome is the most robust, widely-deployable artifact.

---

## Current State vs Target

### Current (Feb 2026)
```
Architectures: x86_64-linux-musl, aarch64-linux-musl
Platforms:     Linux only
Deployment:    Manual binary copy
Issue:         Non-PIE musl binaries segfault on ASLR systems
```

### Target (Evolution)
```
Architectures: x86_64, aarch64, arm32, riscv64, wasm32
Platforms:     Linux, macOS, Windows, Android, FreeBSD, (iOS limited)
Deployment:    Self-detecting, auto-extracting
Robustness:    PIE + fallbacks + verification
```

---

## Universal Genome Structure

```
genome.bin (self-extracting archive)
├── manifest.toml           # Describes contents, checksums
├── deploy.sh               # POSIX shell bootstrap (Linux/macOS/BSD)
├── deploy.ps1              # PowerShell bootstrap (Windows)
├── deploy.wasm             # WebAssembly bootstrap (browser/deno)
│
├── x86_64-linux/
│   ├── beardog             # PIE + static
│   ├── songbird
│   ├── nestgate
│   ├── toadstool
│   ├── squirrel
│   └── biomeos
│
├── aarch64-linux/
│   └── (same 6 binaries)
│
├── x86_64-darwin/          # macOS Intel
│   └── (same 6 binaries)
│
├── aarch64-darwin/         # macOS Apple Silicon
│   └── (same 6 binaries)
│
├── x86_64-windows/
│   └── *.exe binaries
│
├── aarch64-android/        # Termux-compatible
│   └── (same 6 binaries)
│
├── wasm32/                 # Browser/Deno fallback
│   └── *.wasm modules
│
└── genetics/
    ├── .family.seed        # Mito beacon (PASSED)
    └── .lineage.template   # For deriving child lineage
```

---

## Platform Detection Algorithm

The genome's deploy script should detect and adapt:

```bash
#!/bin/sh
# deploy.sh - Universal POSIX bootstrap

detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    # Normalize architecture names
    case "$ARCH" in
        x86_64|amd64)     ARCH="x86_64" ;;
        aarch64|arm64)    ARCH="aarch64" ;;
        armv7l|armhf)     ARCH="arm32" ;;
        riscv64)          ARCH="riscv64" ;;
    esac
    
    # Normalize OS names
    case "$OS" in
        linux)            OS="linux" ;;
        darwin)           OS="darwin" ;;
        freebsd|openbsd)  OS="bsd" ;;
        mingw*|msys*)     OS="windows" ;;
    esac
    
    # Check for Android (Linux + Android env)
    if [ "$OS" = "linux" ] && [ -n "$ANDROID_ROOT" ]; then
        OS="android"
    fi
    
    echo "${ARCH}-${OS}"
}

PLATFORM=$(detect_platform)
BINARY_DIR="./${PLATFORM}"

if [ ! -d "$BINARY_DIR" ]; then
    echo "Platform $PLATFORM not supported in this genome"
    echo "Attempting WASM fallback..."
    # ... wasm fallback logic
fi
```

---

## Binary Build Requirements

### All Platforms Must Have

1. **PIE Enabled** - Position Independent Executable
2. **Static Linking** - No external dependencies
3. **Verified Checksum** - SHA256 in manifest
4. **Signed** - Ed25519 signature (future)

### Build Commands by Target

```bash
# Linux x86_64 (PIE + static)
RUSTFLAGS="-C target-feature=+crt-static -C relocation-model=pie" \
  cargo build --release --target x86_64-unknown-linux-musl

# Linux aarch64 (cross-compile)
RUSTFLAGS="-C target-feature=+crt-static -C relocation-model=pie" \
  cargo build --release --target aarch64-unknown-linux-musl

# macOS (universal binary possible)
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-gnu

# WebAssembly (subset of features)
cargo build --release --target wasm32-unknown-unknown
```

---

## Primal Deployment Capabilities

Each primal should learn these deployment skills:

### NestGate (Storage)
- **Detect storage backend**: filesystem, SQLite, memory, cloud
- **Auto-configure**: Find writable paths, create directories
- **Persist genetics**: Store family seed securely
- **Cache binaries**: Store genome for offline re-deployment

### Songbird (Network)
- **Detect network type**: LAN, WiFi, cellular, VPN
- **Find family**: mDNS, broadcast, beacon rendezvous
- **Relay deployment**: Fetch genome from family member
- **NAT detection**: Determine best connection strategy

### BearDog (Crypto)
- **Verify signatures**: Ed25519 genome signatures
- **Derive lineage**: Mix parent lineage with local entropy
- **Encrypt genetics**: Protect seeds at rest
- **Platform entropy**: Use best available RNG

### Toadstool (Compute)
- **Detect capabilities**: CPU, GPU, TPU, NPU
- **Benchmark**: Quick capability assessment
- **Resource limits**: Respect host constraints
- **Sandboxing**: Run safely on untrusted hosts

### Squirrel (AI/Orchestration)
- **Discover AI backends**: Local (Ollama), cloud (API keys)
- **Capability routing**: Find best execution path
- **Fallback logic**: Degrade gracefully

---

## Deployment Tiers

### Tier 1: Full NUCLEUS (Linux x86_64/aarch64)
All 5 primals + orchestrator, full capabilities.

### Tier 2: Node Deployment (macOS, Windows)
Core primals (BearDog, Songbird, NestGate), compute optional.

### Tier 3: Gateway (Android/Termux)
Songbird + BearDog only, relay to full nodes.

### Tier 4: Beacon (iOS, Browser)
WASM Songbird, beacon-only, family discovery.

### Tier 5: Seed (Any USB-readable device)
Genetics + instructions for manual bootstrap.

---

## Self-Healing Mechanisms

### Binary Verification
```rust
fn verify_binary(path: &Path, expected_hash: &str) -> Result<()> {
    let actual = sha256_file(path)?;
    if actual != expected_hash {
        // Attempt recovery
        if let Some(nestgate) = find_family_nestgate() {
            let fresh = nestgate.fetch_binary(path.file_name())?;
            std::fs::write(path, fresh)?;
        }
    }
    Ok(())
}
```

### Corruption Recovery
1. Check local NestGate cache
2. Request from family mesh via Songbird
3. Fetch from nestgate.io (if beacon verified)
4. Fallback to read-only mode

---

## Evolution Priorities

### Wave 1 (Current - Feb 2026)
- [x] x86_64-linux-musl (PIE fixed)
- [x] aarch64-linux-musl
- [ ] Verify all binaries are PIE
- [ ] Add deploy.sh bootstrap

### Wave 2 (Q1 2026)
- [ ] x86_64-darwin (macOS Intel)
- [ ] aarch64-darwin (Apple Silicon)
- [ ] arm32-linux (Raspberry Pi)

### Wave 3 (Q2 2026)
- [ ] x86_64-windows
- [ ] aarch64-android (Termux)
- [ ] WASM beacon fallback

### Wave 4 (Future)
- [ ] riscv64-linux
- [ ] iOS beacon (limited)
- [ ] Browser-based enrollment

---

## Manifest Evolution

```toml
# manifest.toml v2.0

[genome]
version = "2.0.0"
format = "universal"
min_deploy_version = "1.0.0"

[platforms]
supported = [
    "x86_64-linux",
    "aarch64-linux", 
    "x86_64-darwin",
    "aarch64-darwin",
    "x86_64-windows",
    "aarch64-android",
    "wasm32"
]

fallback_order = [
    "native",      # Try exact platform match
    "compatible",  # Try compatible platform (e.g., x86_64 on amd64)
    "wasm",        # WASM fallback
    "beacon"       # Beacon-only mode
]

[primals.beardog]
tiers = [1, 2, 3]  # Available in tiers 1, 2, 3
required = true

[primals.songbird]
tiers = [1, 2, 3, 4]
required = true

[primals.nestgate]
tiers = [1, 2]
required = false  # Optional in gateway mode

[primals.toadstool]
tiers = [1]
required = false

[primals.squirrel]
tiers = [1]
required = false

[verification]
algorithm = "sha256"
signature = "ed25519"
public_key = "..." # Family public key
```

---

## Success Criteria

A universal genome is successful when:

1. **USB Insert → Deploy**: No manual steps on supported platforms
2. **Verification**: All binaries verified before execution
3. **Family Discovery**: Finds existing family within 30 seconds
4. **Graceful Degradation**: Works partially if full deployment fails
5. **Self-Documenting**: Clear instructions for unsupported platforms

---

## Lessons from NUC Deployment

The segfault taught us:

1. **Test on target**: Always verify binaries execute on actual targets
2. **PIE is mandatory**: ASLR is default on modern systems
3. **Platform variance**: Same "Linux" can have different requirements
4. **Fallbacks matter**: One failure shouldn't block all deployment

This is an **evolution step**, not a bug. Each deployment teaches the genome to be more robust.

