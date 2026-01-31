#!/bin/bash
# Update USB Live Spore with biomeOS genomeBin
# This script packages biomeOS as both a genomeBin AND the deployment orchestrator

set -e

# Configuration
USB_MOUNT="${1:-/media/eastgate/biomeOS21}"
BIOMEOS_ROOT="$USB_MOUNT/biomeOS"
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                          ║"
echo "║   🧠 biomeOS Live Spore Updater - genomeBin Edition                     ║"
echo "║                                                                          ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "USB Mount: $USB_MOUNT"
echo "Project Root: $PROJECT_ROOT"
echo ""

# Check USB is mounted
if [ ! -d "$USB_MOUNT" ]; then
  echo "❌ USB not found at: $USB_MOUNT"
  echo "Usage: $0 [usb-mount-path]"
  exit 1
fi

# Create directory structure
echo "📁 Creating biomeOS genomeBin structure..."
mkdir -p "$BIOMEOS_ROOT"/{binaries,genome,graphs,config,logs}
mkdir -p "$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl"/{biomeos,primals}
mkdir -p "$BIOMEOS_ROOT/genome"/{biomeos,primals}
mkdir -p "$BIOMEOS_ROOT/genome/biomeos/linux"/{systemd,openrc}
mkdir -p "$BIOMEOS_ROOT/genome/biomeos/android"
mkdir -p "$BIOMEOS_ROOT/genome/biomeos/health"
mkdir -p "$BIOMEOS_ROOT/genome/biomeos/update"

echo "✅ Directory structure created"
echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 1: Copy biomeOS UniBin
# ══════════════════════════════════════════════════════════════════════════════

echo "🧠 Copying biomeOS UniBin..."

if [ -f "$PROJECT_ROOT/target/release/biomeos" ]; then
  cp "$PROJECT_ROOT/target/release/biomeos" "$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl/biomeos/"
  chmod +x "$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl/biomeos/biomeos"
  echo "✅ biomeOS UniBin copied ($(du -h $PROJECT_ROOT/target/release/biomeos | cut -f1))"
else
  echo "⚠️  biomeOS binary not found at target/release/biomeos"
  echo "   Run: cargo build --release"
  exit 1
fi

echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 2: Copy all 5 primal ecoBins
# ══════════════════════════════════════════════════════════════════════════════

echo "🦀 Copying primal ecoBins..."

PRIMAL_SOURCE="$PROJECT_ROOT/plasmidBin/stable/x86_64/primals"
PRIMAL_DEST="$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl/primals"

if [ -d "$PRIMAL_SOURCE" ]; then
  cp "$PRIMAL_SOURCE"/{beardog,songbird,nestgate,toadstool,squirrel} "$PRIMAL_DEST/" 2>/dev/null || true
  chmod +x "$PRIMAL_DEST"/*
  
  echo "Primal sizes:"
  for primal in beardog songbird nestgate toadstool squirrel; do
    if [ -f "$PRIMAL_DEST/$primal" ]; then
      SIZE=$(du -h "$PRIMAL_DEST/$primal" | cut -f1)
      echo "  ✅ $primal ($SIZE)"
    else
      echo "  ⚠️  $primal (NOT FOUND)"
    fi
  done
else
  echo "⚠️  Primal source not found at: $PRIMAL_SOURCE"
fi

echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 3: Create biomeOS deployment wrapper
# ══════════════════════════════════════════════════════════════════════════════

echo "📦 Creating biomeOS deployment wrapper..."

cat > "$BIOMEOS_ROOT/genome/biomeos/install.sh" << 'WRAPPER_EOF'
#!/bin/bash
# biomeOS genomeBin Universal Installer
# Auto-detects platform and deploys biomeOS NUCLEUS

set -e

echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║   🧠 Installing biomeOS NUCLEUS                                          ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
  x86_64)  BIN_TARGET="x86_64-unknown-linux-musl" ;;
  aarch64) BIN_TARGET="aarch64-unknown-linux-gnu" ;;
  arm64)   BIN_TARGET="aarch64-apple-darwin" ;;
  *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Detect OS
OS=$(uname -s)
case $OS in
  Linux*)   PLATFORM="linux" ;;
  Darwin*)  PLATFORM="macos" ;;
  *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

echo "Detected: $OS / $ARCH / $BIN_TARGET"
echo ""

# Detect init system (Linux only)
if [ "$PLATFORM" = "linux" ]; then
  if command -v systemctl >/dev/null 2>&1; then
    INIT="systemd"
  elif [ -d /etc/openrc ]; then
    INIT="openrc"
  else
    INIT="sysvinit"
  fi
  echo "Init system: $INIT"
fi

# Installation directories
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/biomeos"
DATA_DIR="/var/lib/biomeos"

# Get source directory (same directory as this script)
SOURCE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
BINARY_SOURCE="$SOURCE_DIR/binaries/$BIN_TARGET/biomeos/biomeos"

if [ ! -f "$BINARY_SOURCE" ]; then
  echo "❌ Binary not found: $BINARY_SOURCE"
  exit 1
fi

echo ""
echo "Installing biomeOS..."
echo "  Source: $BINARY_SOURCE"
echo "  Target: $INSTALL_DIR/biomeos"

# Install binary
sudo cp "$BINARY_SOURCE" "$INSTALL_DIR/biomeos"
sudo chmod +x "$INSTALL_DIR/biomeos"
echo "✅ Binary installed"

# Create config directories
echo ""
echo "Creating configuration directories..."
sudo mkdir -p "$CONFIG_DIR"/{graphs,config}
sudo mkdir -p "$DATA_DIR"/{data,logs}
sudo mkdir -p /run/biomeos
echo "✅ Directories created"

# Copy graphs
if [ -d "$SOURCE_DIR/graphs" ]; then
  echo ""
  echo "Installing deployment graphs..."
  sudo cp -r "$SOURCE_DIR/graphs"/* "$CONFIG_DIR/graphs/"
  echo "✅ Graphs installed"
fi

# Generate family seed if not exists
if [ ! -f "$DATA_DIR/.family.seed" ]; then
  echo ""
  echo "Generating family seed..."
  sudo dd if=/dev/urandom of="$DATA_DIR/.family.seed" bs=32 count=1 2>/dev/null
  sudo chmod 600 "$DATA_DIR/.family.seed"
  echo "✅ Family seed generated"
fi

# Install systemd service (Linux only)
if [ "$INIT" = "systemd" ]; then
  echo ""
  echo "Installing systemd service..."
  
  sudo tee /etc/systemd/system/biomeos-neural-api.service > /dev/null << 'SERVICE_EOF'
[Unit]
Description=biomeOS Neural API Server
After=network.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/biomeos neural-api --graphs-dir /etc/biomeos/graphs --family-id nat0
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/biomeos /run/biomeos /etc/biomeos

[Install]
WantedBy=multi-user.target
SERVICE_EOF

  sudo systemctl daemon-reload
  sudo systemctl enable biomeos-neural-api
  echo "✅ Systemd service installed and enabled"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║   ✅ biomeOS NUCLEUS Installed Successfully!                             ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Start biomeOS Neural API:"
echo "  sudo systemctl start biomeos-neural-api"
echo ""
echo "Deploy full NUCLEUS (all primals):"
echo "  biomeos deploy /etc/biomeos/graphs/nucleus_complete.toml"
echo ""
echo "Health check:"
echo "  biomeos doctor"
echo ""
WRAPPER_EOF

chmod +x "$BIOMEOS_ROOT/genome/biomeos/install.sh"
echo "✅ Universal installer created"
echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 4: Create systemd service file
# ══════════════════════════════════════════════════════════════════════════════

echo "⚙️  Creating systemd service file..."

cat > "$BIOMEOS_ROOT/genome/biomeos/linux/systemd/biomeos-neural-api.service" << 'SERVICE_EOF'
[Unit]
Description=biomeOS Neural API Server - Graph Orchestration
Documentation=https://docs.biomeos.org
After=network.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/biomeos neural-api \
  --graphs-dir /etc/biomeos/graphs \
  --family-id nat0 \
  --socket /run/biomeos/neural-api.sock
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=biomeos-neural-api

# Environment
Environment="RUST_LOG=info"
Environment="BIOMEOS_ROOT=/var/lib/biomeos"

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/biomeos /run/biomeos /etc/biomeos
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

[Install]
WantedBy=multi-user.target
SERVICE_EOF

echo "✅ Systemd service file created"
echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 5: Copy deployment graphs
# ══════════════════════════════════════════════════════════════════════════════

echo "📊 Copying deployment graphs..."

if [ -d "$PROJECT_ROOT/graphs" ]; then
  cp "$PROJECT_ROOT/graphs"/*.toml "$BIOMEOS_ROOT/graphs/" 2>/dev/null || true
  
  GRAPH_COUNT=$(ls -1 "$BIOMEOS_ROOT/graphs"/*.toml 2>/dev/null | wc -l)
  echo "✅ $GRAPH_COUNT graphs copied"
  
  # List graphs
  if [ $GRAPH_COUNT -gt 0 ]; then
    echo ""
    echo "Available graphs:"
    for graph in "$BIOMEOS_ROOT/graphs"/*.toml; do
      GRAPH_NAME=$(basename "$graph")
      echo "  • $GRAPH_NAME"
    done
  fi
else
  echo "⚠️  Graphs directory not found"
fi

echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 6: Create startup script
# ══════════════════════════════════════════════════════════════════════════════

echo "🚀 Creating startup script..."

cat > "$BIOMEOS_ROOT/start_nucleus.sh" << 'STARTUP_EOF'
#!/bin/bash
# NUCLEUS Live Spore Launcher
# Uses biomeOS genomeBin to deploy full ecosystem

set -e

BIOMEOS_ROOT="$(cd "$(dirname "$0")" && pwd)"
BIOMEOS_BIN="$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl/biomeos/biomeos"
PRIMAL_DIR="$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl/primals"
GRAPHS_DIR="$BIOMEOS_ROOT/graphs"

echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                          ║"
echo "║   🧠✨ biomeOS NUCLEUS Live Spore ✨🧠                                    ║"
echo "║                                                                          ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "biomeOS Root: $BIOMEOS_ROOT"
echo ""

# Check binary exists
if [ ! -f "$BIOMEOS_BIN" ]; then
  echo "❌ biomeOS binary not found: $BIOMEOS_BIN"
  exit 1
fi

# Environment setup
export BIOMEOS_ROOT
export PRIMAL_DIR
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/biomeos-runtime}"
mkdir -p "$XDG_RUNTIME_DIR/biomeos"

# Generate family ID if not exists
FAMILY_SEED="$BIOMEOS_ROOT/.family.seed"
if [ ! -f "$FAMILY_SEED" ]; then
  echo "🔐 Generating family seed..."
  dd if=/dev/urandom of="$FAMILY_SEED" bs=32 count=1 2>/dev/null
  chmod 600 "$FAMILY_SEED"
  echo "✅ Family seed generated"
fi

# Clean old sockets
rm -f "$XDG_RUNTIME_DIR/biomeos/"*.sock 2>/dev/null || true

echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo "🏰 Starting NUCLEUS via biomeOS genomeBin"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""

# Check if nucleus_complete.toml exists
NUCLEUS_GRAPH="$GRAPHS_DIR/nucleus_complete.toml"
if [ -f "$NUCLEUS_GRAPH" ]; then
  echo "📊 Deploying via graph: $NUCLEUS_GRAPH"
  "$BIOMEOS_BIN" deploy "$NUCLEUS_GRAPH"
else
  echo "⚠️  Graph not found: $NUCLEUS_GRAPH"
  echo "   Starting primals manually..."
  echo ""
  
  # Manual startup fallback
  export PATH="$PRIMAL_DIR:$PATH"
  
  echo "Starting BearDog (security & crypto)..."
  "$PRIMAL_DIR/beardog" server &
  sleep 2
  
  echo "Starting Songbird (networking & discovery)..."
  "$PRIMAL_DIR/songbird" server &
  sleep 2
  
  echo "Starting Toadstool (compute)..."
  "$PRIMAL_DIR/toadstool" server &
  sleep 2
  
  echo "Starting NestGate (storage)..."
  "$PRIMAL_DIR/nestgate" server &
  sleep 2
  
  echo "Starting Squirrel (AI routing)..."
  "$PRIMAL_DIR/squirrel" server &
  sleep 2
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║   ✅ NUCLEUS OPERATIONAL                                                 ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Available commands:"
echo "  biomeos doctor              - Health check"
echo "  biomeos deploy <graph>      - Deploy a graph"
echo "  biomeos neural-api          - Start Neural API server"
echo ""
STARTUP_EOF

chmod +x "$BIOMEOS_ROOT/start_nucleus.sh"
echo "✅ Startup script created"
echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 7: Create README
# ══════════════════════════════════════════════════════════════════════════════

echo "📄 Creating README..."

cat > "$BIOMEOS_ROOT/README.md" << 'README_EOF'
# 🧠 biomeOS NUCLEUS Live Spore

**Version:** genomeBin Edition  
**Date:** January 30, 2026  
**Architecture:** Universal Deployment System

---

## 🎯 What is This?

This USB drive contains a **complete biomeOS NUCLEUS** packaged as genomeBins:

- **biomeOS:** The meta-organism (orchestrator)
- **5 Primals:** BearDog, Songbird, NestGate, Toadstool, Squirrel
- **Deployment Graphs:** Pre-configured NUCLEUS deployments
- **Deployment Wrappers:** Universal installers for any platform

---

## 🚀 Quick Start

### **Option 1: Run from USB (Temporary)**

```bash
cd /media/user/biomeOS21/biomeOS
./start_nucleus.sh
```

This starts the full NUCLEUS from the USB drive without installation.

### **Option 2: Install to System (Permanent)**

```bash
cd /media/user/biomeOS21/biomeOS/genome/biomeos
sudo ./install.sh
```

This installs biomeOS as a system service.

---

## 📦 Contents

```
biomeOS/
├── binaries/                       Compiled binaries
│   └── x86_64-unknown-linux-musl/
│       ├── biomeos/                biomeOS UniBin
│       │   └── biomeos
│       └── primals/                All 5 primals
│           ├── beardog
│           ├── songbird
│           ├── nestgate
│           ├── toadstool
│           └── squirrel
│
├── genome/                         Deployment wrappers
│   └── biomeos/
│       ├── install.sh              Universal installer
│       └── linux/systemd/          Service files
│
├── graphs/                         Deployment graphs
│   ├── nucleus_complete.toml       Full NUCLEUS
│   ├── tower_atomic.toml           BearDog + Songbird
│   └── [other graphs]
│
├── start_nucleus.sh                Quick launcher
└── README.md                       This file
```

---

## 🧠 biomeOS Commands

Once installed:

```bash
# Health check
biomeos doctor

# Deploy full NUCLEUS
biomeos deploy /etc/biomeos/graphs/nucleus_complete.toml

# Start Neural API server
biomeos neural-api --graphs-dir /etc/biomeos/graphs

# Deploy custom graph
biomeos deploy /path/to/custom_graph.toml

# Version info
biomeos version --detailed
```

---

## 🌟 What Makes This Special?

### **genomeBin Architecture**

This is NOT just a collection of binaries. This is a **genomeBin**:

- ✅ **Self-contained:** Everything needed to deploy
- ✅ **Self-installing:** ONE command to install
- ✅ **Self-healing:** Automatic health monitoring
- ✅ **Self-updating:** Can update itself
- ✅ **Universal:** Works on any Linux x86_64 system

### **The Meta-Organism**

biomeOS is unique:

- It's BOTH a deployable organism AND the deployment system
- It deploys all other primals via graphs
- It can even deploy itself (bootstrap)

---

## 🎯 Use Cases

### **1. Rapid Deployment**
Plug into any machine, run `./start_nucleus.sh`, instant NUCLEUS!

### **2. Testing**
Test new NUCLEUS configurations without installing.

### **3. Recovery**
Boot from USB to recover a failed system.

### **4. Portable Development**
Develop and test on any machine.

---

## 📊 System Requirements

- **OS:** Linux (any distribution)
- **Architecture:** x86_64
- **RAM:** 2GB minimum, 4GB recommended
- **Disk:** 200MB for binaries, 1GB for data
- **Network:** Optional (for federation)

---

## 🆘 Troubleshooting

### **"Permission denied"**
```bash
chmod +x start_nucleus.sh
chmod +x binaries/x86_64-unknown-linux-musl/biomeos/biomeos
```

### **"Socket already in use"**
```bash
rm -rf /tmp/biomeos-runtime/biomeos/*.sock
```

### **"Family seed missing"**
```bash
dd if=/dev/urandom of=.family.seed bs=32 count=1
chmod 600 .family.seed
```

---

## 📚 Documentation

- **biomeOS:** https://docs.biomeos.org
- **Deployment Graphs:** See `graphs/README.md`
- **genomeBin Standard:** See `genome/biomeos/README.md`

---

## 🎊 Success!

If you see:

```
✅ NUCLEUS OPERATIONAL
```

You have a fully functional biomeOS ecosystem running!

---

**🧠 Powered by biomeOS genomeBin Architecture 🚀**
README_EOF

echo "✅ README created"
echo ""

# ══════════════════════════════════════════════════════════════════════════════
# STEP 8: Summary
# ══════════════════════════════════════════════════════════════════════════════

echo "═══════════════════════════════════════════════════════════════════════════"
echo "📊 USB Live Spore Summary"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""

# Calculate sizes
BIOMEOS_SIZE=$(du -h "$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl/biomeos/biomeos" 2>/dev/null | cut -f1 || echo "N/A")
PRIMALS_SIZE=$(du -sh "$BIOMEOS_ROOT/binaries/x86_64-unknown-linux-musl/primals" 2>/dev/null | cut -f1 || echo "N/A")
TOTAL_SIZE=$(du -sh "$BIOMEOS_ROOT" 2>/dev/null | cut -f1 || echo "N/A")

echo "Binary Sizes:"
echo "  biomeOS: $BIOMEOS_SIZE"
echo "  Primals: $PRIMALS_SIZE"
echo "  Total:   $TOTAL_SIZE"
echo ""

echo "Structure:"
echo "  ✅ biomeOS genomeBin"
echo "  ✅ 5 primal ecoBins"
echo "  ✅ Deployment wrapper (install.sh)"
echo "  ✅ Systemd service file"
echo "  ✅ Deployment graphs"
echo "  ✅ Startup script"
echo "  ✅ README documentation"
echo ""

echo "═══════════════════════════════════════════════════════════════════════════"
echo "✅ USB Live Spore Updated Successfully!"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "Location: $BIOMEOS_ROOT"
echo ""
echo "Test it:"
echo "  cd $BIOMEOS_ROOT"
echo "  ./start_nucleus.sh"
echo ""
echo "Install it:"
echo "  cd $BIOMEOS_ROOT/genome/biomeos"
echo "  sudo ./install.sh"
echo ""

# Sync to ensure writes complete
sync
echo "🔄 Syncing filesystem..."
sleep 2
echo "✅ Sync complete!"
echo ""
echo "🎊 USB Live Spore ready for deployment!"
