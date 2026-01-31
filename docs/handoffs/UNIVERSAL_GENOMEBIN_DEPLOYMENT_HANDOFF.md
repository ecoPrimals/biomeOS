# 🌍 Universal genomeBin Deployment Structure Handoff

**Document Version:** 1.0  
**Date:** January 30, 2026  
**Target Team:** All Primal Teams + biomeOS Architects  
**Priority:** HIGH - Standardizes universal deployment  
**Target Size:** ~1GB total  
**Status:** Ready for Implementation

---

## 🎯 **Executive Summary**

This handoff defines the **UNIVERSAL** deployment structure for **ALL primals** across **ALL platforms** (USB, Android, macOS, Windows, Linux, iOS, WASM). The goal is **ONE structure**, stored in `biomeOS/plasmidBin/`, that can deploy to ANY target with ZERO reconfiguration.

**Philosophy:**
> **"genomeBin = ecoBin + Deployment Wrapper = Universal Autonomous Organism"**

**Key Principles:**
1. **ONE source of truth:** All binaries in `plasmidBin/`
2. **Multi-arch support:** x86_64, ARM64, RISC-V, etc.
3. **Multi-platform:** Linux, Android, macOS, Windows, iOS, WASM
4. **Target size:** ~1GB total (efficient storage)
5. **Zero duplication:** Smart linking, not copying

---

## 📊 **Current Status**

### **What We Have (January 30, 2026)**

```
plasmidBin/stable/x86_64/primals/
├── beardog   (4.0M) - Platform-agnostic! 100% coverage
├── songbird  (29M)  - Network & discovery
├── nestgate  (5.0M) - Federated storage
├── toadstool (15M)  - Universal compute
└── squirrel  (6.7M) - AI routing
────────────────────
Total: 58M (ecoBin v2.0 compliant)
```

### **What We Need**

```
plasmidBin/
├── stable/
│   ├── x86_64-unknown-linux-gnu/      ecoBins for Linux x86_64
│   ├── x86_64-unknown-linux-musl/     Static ecoBins (portable)
│   ├── aarch64-unknown-linux-gnu/     ecoBins for Linux ARM64
│   ├── aarch64-linux-android/         ecoBins for Android ARM64
│   ├── x86_64-apple-darwin/           ecoBins for macOS Intel
│   ├── aarch64-apple-darwin/          ecoBins for macOS M-series
│   ├── x86_64-pc-windows-gnu/         ecoBins for Windows x86_64
│   └── wasm32-unknown-unknown/        ecoBins for WASM/browser
├── genome/                            genomeBin wrappers (deployment)
│   ├── linux/                         Linux deployment scripts
│   ├── android/                       Android deployment (APK structure)
│   ├── macos/                         macOS deployment (.app bundles)
│   ├── windows/                       Windows deployment (installers)
│   └── usb/                           USB live spore structure
├── graphs/                            Deployment graphs (NUCLEUS, Tower, Node, Nest)
└── shared/                            Shared assets (configs, certs, seeds)
```

---

## 🏗️ **Standard Structure - plasmidBin/**

### **Directory Layout**

```
biomeOS/plasmidBin/
│
├── README.md                          ← Overview and quick start
├── STRUCTURE.md                       ← This document
│
├── stable/                            ← ecoBin binaries (arch-specific)
│   ├── x86_64-unknown-linux-gnu/
│   │   └── primals/
│   │       ├── beardog
│   │       ├── songbird
│   │       ├── nestgate
│   │       ├── toadstool
│   │       └── squirrel
│   │
│   ├── x86_64-unknown-linux-musl/     ← Static (for USB, portable)
│   │   └── primals/
│   │       └── [same 5 primals]
│   │
│   ├── aarch64-unknown-linux-gnu/     ← ARM64 Linux
│   │   └── primals/
│   │       └── [same 5 primals]
│   │
│   ├── aarch64-linux-android/         ← Android ARM64
│   │   └── primals/
│   │       └── [same 5 primals]
│   │
│   ├── x86_64-apple-darwin/           ← macOS Intel
│   │   └── primals/
│   │       └── [same 5 primals]
│   │
│   ├── aarch64-apple-darwin/          ← macOS M-series
│   │   └── primals/
│   │       └── [same 5 primals]
│   │
│   ├── x86_64-pc-windows-gnu/         ← Windows x86_64
│   │   └── primals/
│   │       └── [same 5 primals].exe
│   │
│   └── wasm32-unknown-unknown/        ← WASM (browser/Wasmtime)
│       └── primals/
│           └── [same 5 primals].wasm
│
├── genome/                            ← genomeBin wrappers (deployment machinery)
│   │
│   ├── linux/                         ← Linux deployment
│   │   ├── install.sh                 ← Universal installer
│   │   ├── systemd/                   ← systemd service files
│   │   ├── openrc/                    ← OpenRC service files
│   │   └── sysvinit/                  ← SysV init scripts
│   │
│   ├── android/                       ← Android deployment
│   │   ├── AndroidManifest.xml        ← APK manifest
│   │   ├── start_nucleus.sh           ← Mobile launcher
│   │   └── README_ANDROID.md          ← Android-specific docs
│   │
│   ├── macos/                         ← macOS deployment
│   │   ├── install.sh                 ← macOS installer
│   │   ├── launchd/                   ← launchd plist files
│   │   └── app-bundle/                ← .app bundle structure
│   │
│   ├── windows/                       ← Windows deployment
│   │   ├── install.ps1                ← PowerShell installer
│   │   ├── service/                   ← Windows Service wrapper
│   │   └── nsis/                      ← NSIS installer config
│   │
│   └── usb/                           ← USB Live Spore
│       ├── make_livespore.sh          ← USB creator script
│       ├── boot/                      ← Boot configuration
│       └── README_USB.md              ← USB deployment guide
│
├── graphs/                            ← Deployment graphs (Neural API)
│   ├── nucleus_complete.toml          ← Full NUCLEUS (Tower+Node+Nest)
│   ├── tower_atomic.toml              ← Tower (BearDog+Songbird)
│   ├── node_atomic.toml               ← Node (Tower+Toadstool)
│   ├── nest_atomic.toml               ← Nest (Tower+NestGate+Squirrel)
│   └── README_GRAPHS.md               ← Graph deployment guide
│
├── shared/                            ← Shared deployment assets
│   ├── configs/                       ← Default configurations
│   │   ├── beardog.toml
│   │   ├── songbird.toml
│   │   ├── nestgate.toml
│   │   ├── toadstool.toml
│   │   └── squirrel.toml
│   │
│   ├── certs/                         ← Shared certificates (if any)
│   ├── seeds/                         ← Family seed templates
│   │   └── .family.seed.template
│   │
│   └── scripts/                       ← Utility scripts
│       ├── health_check.sh            ← System health check
│       ├── validate_binaries.sh       ← Binary validation
│       └── generate_seed.sh           ← Family seed generator
│
└── tools/                             ← Build and deployment tools
    ├── build_all.sh                   ← Build all targets
    ├── harvest.sh                     ← Harvest from primal repos
    ├── deploy_usb.sh                  ← USB deployment
    ├── deploy_android.sh              ← Android deployment (via adb)
    └── deploy_macos.sh                ← macOS deployment
```

---

## 📏 **Size Budget (~1GB Total)**

### **Per-Architecture Breakdown**

```
Target                           Size per Arch    Priority
─────────────────────────────────────────────────────────────
x86_64-unknown-linux-musl         ~70MB           HIGH (USB, portable)
x86_64-unknown-linux-gnu          ~70MB           HIGH (most Linux)
aarch64-linux-android             ~70MB           HIGH (Pixel 8a, Android)
aarch64-unknown-linux-gnu         ~70MB           MEDIUM (ARM Linux)
x86_64-apple-darwin               ~70MB           MEDIUM (macOS Intel)
aarch64-apple-darwin              ~70MB           MEDIUM (macOS M-series)
x86_64-pc-windows-gnu             ~80MB           MEDIUM (Windows)
wasm32-unknown-unknown            ~50MB           LOW (browser/WASM)
─────────────────────────────────────────────────────────────
Total ecoBin binaries:            ~590MB

genome/ (deployment wrappers)      ~50MB
graphs/ (deployment graphs)        ~5MB
shared/ (configs, scripts)         ~10MB
tools/ (build scripts)             ~5MB
documentation/                     ~5MB
─────────────────────────────────────────────────────────────
Grand Total:                       ~665MB
```

**Buffer for future growth:** ~335MB → Target fits comfortably in **1GB**! ✅

---

## 🎯 **genomeBin Compliance Matrix**

### **Current Status (January 30, 2026)**

| Primal    | UniBin | ecoBin v2.0 | genomeBin | Status |
|-----------|--------|-------------|-----------|--------|
| BearDog   | ✅     | ✅ (100%)   | 🔄        | Platform-agnostic ready! |
| Songbird  | ✅     | ✅ (TRUE)   | 🔄        | Has universal-ipc |
| NestGate  | ✅     | ✅ (Pure)   | 🔄        | 100% Pure Rust |
| Toadstool | ✅     | ✅ (A++)    | 🔄        | 120 ops, 6% CUDA parity |
| Squirrel  | ✅     | ✅ (A+)     | 🔄        | Modern discovery |

**Legend:**
- ✅ = Complete
- 🔄 = In Progress (deployment wrapper needed)
- ⚠️ = Needs Work

### **Standards Checklist**

#### **UniBin Requirements** (Foundation)
- [ ] BearDog: ✅ Single binary, multiple modes
- [ ] Songbird: ✅ Single binary, multiple modes
- [ ] NestGate: ✅ Single binary, multiple modes
- [ ] Toadstool: ✅ Single binary, multiple modes
- [ ] Squirrel: ✅ Single binary, multiple modes

#### **ecoBin v2.0 Requirements** (Portability)
- [ ] BearDog: ✅ 100% Pure Rust + Platform-agnostic IPC
- [ ] Songbird: ✅ Pure Rust (OpenSSL → Pure Rust HTTP)
- [ ] NestGate: ✅ Pure Rust (libc eliminated)
- [ ] Toadstool: ✅ Pure Rust + platform discovery
- [ ] Squirrel: ✅ Pure Rust + socket standardization

#### **genomeBin Requirements** (Autonomous Deployment)
- [ ] Deployment wrapper created (genome/)
- [ ] System detection (OS, arch, init)
- [ ] Auto-installation script
- [ ] Service integration (systemd, launchd, etc.)
- [ ] Health monitoring
- [ ] Update system
- [ ] Rollback capability
- [ ] Clean uninstall

---

## 🛠️ **Implementation Plan**

### **Phase 1: Organize Existing ecoBins** (1 hour)

**Goal:** Restructure current `plasmidBin/` to new standard

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Create new structure
mkdir -p plasmidBin/{stable,genome,graphs,shared,tools}
mkdir -p plasmidBin/stable/{x86_64-unknown-linux-{gnu,musl},aarch64-{unknown-linux-gnu,linux-android}}
mkdir -p plasmidBin/stable/{x86_64-apple-darwin,aarch64-apple-darwin}
mkdir -p plasmidBin/stable/x86_64-pc-windows-gnu
mkdir -p plasmidBin/genome/{linux,android,macos,windows,usb}
mkdir -p plasmidBin/shared/{configs,certs,seeds,scripts}

# Move existing binaries
mv plasmidBin/stable/x86_64/primals plasmidBin/stable/x86_64-unknown-linux-gnu/

# Copy to musl (for USB/portable)
cp -r plasmidBin/stable/x86_64-unknown-linux-gnu/primals \
      plasmidBin/stable/x86_64-unknown-linux-musl/

echo "✅ Structure created"
```

### **Phase 2: Build Multi-Arch ecoBins** (2-3 hours)

**Goal:** Compile all 5 primals for priority targets

**Priority 1: Linux (x86_64 musl - USB/portable)**
```bash
# BearDog
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/beardog \
   ../phase2/biomeOS/plasmidBin/stable/x86_64-unknown-linux-musl/primals/

# Repeat for Songbird, NestGate, Toadstool, Squirrel
```

**Priority 2: Android (ARM64 - Pixel 8a)**
```bash
# BearDog (with Option A: disable StrongBox)
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release --target aarch64-linux-android
cp target/aarch64-linux-android/release/beardog \
   ../phase2/biomeOS/plasmidBin/stable/aarch64-linux-android/primals/

# Repeat for all primals
```

**Priority 3: macOS (Intel + M-series)**
```bash
# For Intel Macs
cargo build --release --target x86_64-apple-darwin

# For M-series Macs
cargo build --release --target aarch64-apple-darwin
```

### **Phase 3: Create genomeBin Wrappers** (3-4 hours)

**Goal:** Add deployment machinery for each platform

**Linux genomeBin Wrapper:**
```bash
#!/bin/bash
# genome/linux/install.sh - Universal Linux Installer

set -e

echo "🦀 Installing NUCLEUS genome..."

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
  x86_64)  BIN_DIR="../stable/x86_64-unknown-linux-musl/primals" ;;
  aarch64) BIN_DIR="../stable/aarch64-unknown-linux-gnu/primals" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Detect init system
if command -v systemctl >/dev/null 2>&1; then
  INIT="systemd"
elif [ -d /etc/openrc ]; then
  INIT="openrc"
else
  INIT="sysvinit"
fi

echo "Detected: $ARCH / $INIT"

# Install binaries
sudo cp $BIN_DIR/* /usr/local/bin/
sudo chmod +x /usr/local/bin/{beardog,songbird,nestgate,toadstool,squirrel}

# Install service files
case $INIT in
  systemd)
    sudo cp systemd/*.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable beardog songbird nestgate toadstool squirrel
    ;;
  openrc)
    sudo cp openrc/* /etc/init.d/
    sudo rc-update add beardog default
    ;;
  sysvinit)
    sudo cp sysvinit/* /etc/init.d/
    sudo update-rc.d beardog defaults
    ;;
esac

# Generate family seed
sudo mkdir -p /var/lib/biomeos
sudo openssl rand 32 > /var/lib/biomeos/.family.seed
sudo chmod 600 /var/lib/biomeos/.family.seed

echo "✅ NUCLEUS genome installed successfully!"
echo ""
echo "Start services:"
echo "  sudo systemctl start beardog"
echo "  sudo systemctl start songbird"
```

**Android genomeBin Wrapper:**
```bash
#!/system/bin/sh
# genome/android/start_nucleus.sh

BIOMEOS_ROOT="/data/local/tmp/biomeos"
PRIMAL_DIR="$BIOMEOS_ROOT/primals"

# Use abstract sockets (Android-safe)
export XDG_RUNTIME_DIR="/data/local/tmp"

# Start all primals
$PRIMAL_DIR/beardog server &
$PRIMAL_DIR/songbird server &
$PRIMAL_DIR/nestgate server &
$PRIMAL_DIR/toadstool server &
$PRIMAL_DIR/squirrel server &

echo "✅ NUCLEUS operational on Android!"
```

**USB Live Spore Maker:**
```bash
#!/bin/bash
# genome/usb/make_livespore.sh

USB_MOUNT="$1"  # /media/user/USB_DRIVE

if [ -z "$USB_MOUNT" ]; then
  echo "Usage: $0 /path/to/usb"
  exit 1
fi

echo "🚀 Creating NUCLEUS Live Spore on: $USB_MOUNT"

# Copy portable binaries (musl - static linking)
mkdir -p "$USB_MOUNT/biomeOS/primals"
cp ../../stable/x86_64-unknown-linux-musl/primals/* "$USB_MOUNT/biomeOS/primals/"

# Copy deployment files
cp -r ../../graphs "$USB_MOUNT/biomeOS/"
cp -r ../../shared "$USB_MOUNT/biomeOS/"

# Copy launcher
cp ../linux/install.sh "$USB_MOUNT/biomeOS/start_nucleus.sh"
chmod +x "$USB_MOUNT/biomeOS/start_nucleus.sh"

# Generate family seed
openssl rand 32 > "$USB_MOUNT/biomeOS/.family.seed"

sync
echo "✅ Live Spore created! Size: $(du -sh $USB_MOUNT/biomeOS | cut -f1)"
```

### **Phase 4: Create Deployment Tools** (1 hour)

**Universal Harvester:**
```bash
#!/bin/bash
# tools/harvest.sh - Harvest binaries from all primal repos

PRIMALS="beardog songbird nestgate toadstool squirrel"
TARGETS="x86_64-unknown-linux-musl aarch64-linux-android"

for PRIMAL in $PRIMALS; do
  echo "🔨 Building $PRIMAL..."
  cd ../../../phase1/$PRIMAL
  
  for TARGET in $TARGETS; do
    cargo build --release --target $TARGET
    cp target/$TARGET/release/$PRIMAL \
       ../../phase2/biomeOS/plasmidBin/stable/$TARGET/primals/
  done
done

echo "✅ All primals harvested!"
```

---

## 🎯 **Validation & Testing**

### **Binary Validation**

```bash
# tools/validate_binaries.sh

for PRIMAL in beardog songbird nestgate toadstool squirrel; do
  BIN="plasmidBin/stable/x86_64-unknown-linux-musl/primals/$PRIMAL"
  
  # Check exists
  [ -f "$BIN" ] || { echo "❌ Missing: $BIN"; continue; }
  
  # Check executable
  [ -x "$BIN" ] || { echo "❌ Not executable: $BIN"; continue; }
  
  # Check size (reasonable range)
  SIZE=$(stat -f%z "$BIN" 2>/dev/null || stat -c%s "$BIN")
  [ $SIZE -gt 1000000 ] || { echo "⚠️  Small: $BIN ($SIZE bytes)"; }
  
  # Check Pure Rust (no C symbols)
  if command -v nm >/dev/null 2>&1; then
    C_SYMBOLS=$(nm -D "$BIN" 2>/dev/null | grep -c "malloc\|free\|pthread" || true)
    [ $C_SYMBOLS -eq 0 ] && echo "✅ $PRIMAL: Pure Rust" || echo "⚠️  $PRIMAL: Has C symbols ($C_SYMBOLS)"
  fi
  
  # Check runs
  timeout 2s "$BIN" --version >/dev/null 2>&1 && echo "✅ $PRIMAL: Runs" || echo "⚠️  $PRIMAL: Failed to run"
done
```

### **Deployment Testing**

**Linux:**
```bash
cd genome/linux
sudo ./install.sh
systemctl status beardog songbird nestgate toadstool squirrel
```

**Android:**
```bash
cd tools
./deploy_android.sh /path/to/plasmidBin
adb shell "cd /data/local/tmp/biomeos && ./start_nucleus.sh"
```

**USB:**
```bash
cd genome/usb
./make_livespore.sh /media/user/USB_DRIVE
# Eject USB, plug into target machine
# Mount and run: ./biomeOS/start_nucleus.sh
```

---

## 📊 **Success Metrics**

### **Structure**
- [ ] All targets have dedicated directories
- [ ] Naming is consistent (x86_64-unknown-linux-musl format)
- [ ] No duplicate binaries (use symlinks if needed)
- [ ] Total size under 1GB

### **Binaries**
- [ ] All 5 primals built for all priority targets
- [ ] All binaries are executable
- [ ] All binaries pass `--version` test
- [ ] Pure Rust validation passes (no C symbols)

### **Deployment**
- [ ] Linux installer works on Ubuntu, Fedora, Arch
- [ ] Android deployment works on Pixel 8a
- [ ] USB live spore boots and runs
- [ ] macOS .app bundle installs cleanly
- [ ] Windows installer creates Start Menu entries

### **Documentation**
- [ ] README.md explains structure
- [ ] Each platform has deployment guide
- [ ] Quick start covers common scenarios

---

## 🚀 **Next Steps**

### **Week 1: Foundation**
1. Restructure existing plasmidBin/ (1 hour)
2. Build priority targets (Linux musl, Android ARM64) (2 hours)
3. Create basic deployment wrappers (2 hours)
4. Test USB live spore (30 min)

### **Week 2: Expansion**
1. Build macOS targets (Intel + M-series) (1 hour)
2. Build Windows target (1 hour)
3. Create platform-specific installers (3 hours)
4. Documentation and polish (1 hour)

### **Week 3: genomeBin Evolution**
1. Add auto-update system
2. Add health monitoring
3. Add rollback capability
4. Achieve full genomeBin compliance!

---

## 📚 **Standards Reference**

**UniBin Standard:**
- Location: `/home/eastgate/Development/ecoPrimals/wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md`
- Requirements: One binary per primal, multiple modes

**ecoBin v2.0 Standard:**
- Location: `/home/eastgate/Development/ecoPrimals/wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md`
- Requirements: 100% Pure Rust + Platform-agnostic IPC

**genomeBin Standard:**
- Location: `/home/eastgate/Development/ecoPrimals/wateringHole/GENOMEBIN_ARCHITECTURE_STANDARD.md`
- Requirements: ecoBin + Deployment wrapper

---

## 🎊 **Vision**

**The End Goal:**

```bash
# ANY user, ANY platform, ONE command:
curl -sSf https://install.biomeos.dev/genome | sh

# Auto-detects OS, architecture, init system
# Downloads appropriate ecoBins
# Installs services
# Generates family seed
# Starts NUCLEUS

# Result: "✅ biomeOS NUCLEUS installed successfully!"
```

**This is the power of genomeBin: Universal, Autonomous, Ecological!**

---

**🌍 One Structure. All Platforms. Zero Configuration. 🚀**
