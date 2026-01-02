#!/bin/bash
# USB LAN Deployment Package Creator
# Creates a self-contained biomeOS deployment for tower distribution

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

USB_DEVICE="${1:-/dev/sda}"
USB_MOUNT="/media/eastgate/BEA6-BBCE"

echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  🚀 biomeOS USB LAN Deployment Package Creator${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Target USB:${NC} $USB_DEVICE (14.6GB)"
echo -e "${YELLOW}Mount Point:${NC} $USB_MOUNT"
echo ""

# Check if USB is mounted
if [ ! -d "$USB_MOUNT" ]; then
    echo -e "${RED}❌ USB not mounted at $USB_MOUNT${NC}"
    echo "Creating mount point and mounting..."
    sudo mkdir -p "$USB_MOUNT"
    sudo mount "$USB_DEVICE"1 "$USB_MOUNT"
fi

echo -e "${GREEN}✅ USB mounted and accessible${NC}"
echo ""

# Create deployment structure
echo -e "${CYAN}Creating deployment structure...${NC}"

DEPLOY_DIR="$USB_MOUNT/biomeOS-LAN-Deploy"
mkdir -p "$DEPLOY_DIR"/{biomeOS,primals,configs,scripts,docs}

echo -e "${GREEN}✅ Directory structure created${NC}"
echo ""

# Package biomeOS
echo -e "${CYAN}Packaging biomeOS...${NC}"
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Build release binary
echo "  Building release binary..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true

# Copy binary
cp target/release/biomeos "$DEPLOY_DIR/biomeOS/" 2>/dev/null || echo "  ⚠️  Binary not found, will package source"

# Copy source (for building on towers if needed)
echo "  Packaging source code..."
rsync -av --exclude target --exclude .git \
    /home/eastgate/Development/ecoPrimals/phase2/biomeOS/ \
    "$DEPLOY_DIR/biomeOS/" \
    2>&1 | grep -E "sending|sent" || true

echo -e "${GREEN}✅ biomeOS packaged${NC}"
echo ""

# Package primals
echo -e "${CYAN}Packaging primal binaries...${NC}"

# Check for primal binaries
PRIMAL_BINS="/home/eastgate/Development/ecoPrimals/primalBins"
if [ -d "$PRIMAL_BINS" ]; then
    echo "  Copying primal binaries..."
    cp -r "$PRIMAL_BINS"/* "$DEPLOY_DIR/primals/" 2>/dev/null || echo "  ⚠️  No binaries found"
else
    echo "  ⚠️  primalBins directory not found"
    echo "  Will package source for building on towers"
fi

# Package primal sources
echo "  Packaging primal sources..."
for primal in songbird beardog nestgate toadstool squirrel; do
    PRIMAL_PATH="/home/eastgate/Development/ecoPrimals/phase1/$primal"
    if [ -d "$PRIMAL_PATH" ]; then
        echo "    - Packaging $primal..."
        rsync -av --exclude target --exclude .git \
            "$PRIMAL_PATH" \
            "$DEPLOY_DIR/primals/source/" \
            2>&1 | grep -E "sending|sent" | head -1 || true
    fi
done

echo -e "${GREEN}✅ Primals packaged${NC}"
echo ""

# Create configs
echo -e "${CYAN}Creating configuration files...${NC}"

# LAN discovery config
cat > "$DEPLOY_DIR/configs/lan-discovery.toml" << 'EOF'
# biomeOS LAN Discovery Configuration
# For automatic discovery across towers

[discovery]
mode = "mdns"              # Use mDNS for LAN
enabled = true
timeout_seconds = 10

[mdns]
enabled = true
interface = "auto"         # Auto-detect network interface
domain = "local"

[network]
bind_address = "0.0.0.0"   # Listen on all interfaces
port = 8080

[primals]
# Auto-discover primals on LAN
auto_discover = true

# Fallback manual endpoints (if needed)
# songbird_endpoint = "http://tower1.local:8080"
# beardog_endpoint = "http://tower1.local:9000"
EOF

# Tower-specific configs
for tower in tower1 tower2 tower3; do
    cat > "$DEPLOY_DIR/configs/$tower.toml" << EOF
# Configuration for $tower

[metadata]
tower_id = "$tower"
role = "worker"           # tower1 could be "orchestrator"

[discovery]
mode = "mdns"
enabled = true

[network]
hostname = "${tower}.local"
bind_address = "0.0.0.0"
port = 8080

[primals]
auto_discover = true
EOF
done

echo -e "${GREEN}✅ Configurations created${NC}"
echo ""

# Create deployment scripts
echo -e "${CYAN}Creating deployment scripts...${NC}"

# Auto-deploy script
cat > "$DEPLOY_DIR/scripts/auto-deploy.sh" << 'EOFSCRIPT'
#!/bin/bash
# Auto-deployment script for LAN towers
# Run this on each tower to start biomeOS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="$(dirname "$SCRIPT_DIR")"

echo "════════════════════════════════════════════════════════"
echo "  🚀 biomeOS LAN Tower Auto-Deployment"
echo "════════════════════════════════════════════════════════"
echo ""

# Detect tower hostname
TOWER_NAME=$(hostname | cut -d. -f1)
echo "Tower: $TOWER_NAME"
echo ""

# Check for config
CONFIG_FILE="$DEPLOY_ROOT/configs/${TOWER_NAME}.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "⚠️  No config for $TOWER_NAME, using lan-discovery.toml"
    CONFIG_FILE="$DEPLOY_ROOT/configs/lan-discovery.toml"
fi

# Check if we need to build
BIOMEOS_BIN="$DEPLOY_ROOT/biomeOS/biomeos"
if [ ! -f "$BIOMEOS_BIN" ]; then
    echo "Building biomeOS..."
    cd "$DEPLOY_ROOT/biomeOS"
    cargo build --release
    cp target/release/biomeos "$BIOMEOS_BIN"
fi

# Start primals if available
echo "Starting primals..."
for primal in songbird beardog; do
    PRIMAL_BIN="$DEPLOY_ROOT/primals/$primal"
    if [ -f "$PRIMAL_BIN" ]; then
        echo "  Starting $primal..."
        $PRIMAL_BIN &
        sleep 2
    else
        echo "  ⚠️  $primal binary not found"
    fi
done

# Start biomeOS
echo ""
echo "Starting biomeOS with config: $CONFIG_FILE"
echo ""
BIOMEOS_CONFIG="$CONFIG_FILE" "$BIOMEOS_BIN"
EOFSCRIPT

chmod +x "$DEPLOY_DIR/scripts/auto-deploy.sh"

# Quick start script
cat > "$DEPLOY_DIR/scripts/quick-start.sh" << 'EOFSCRIPT'
#!/bin/bash
# Quick start script - runs biomeOS with auto-discovery

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$DEPLOY_ROOT/biomeOS"

# Use LAN discovery config
export BIOMEOS_DISCOVERY_MODE=mdns
export BIOMEOS_CONFIG="$DEPLOY_ROOT/configs/lan-discovery.toml"

./biomeos
EOFSCRIPT

chmod +x "$DEPLOY_DIR/scripts/quick-start.sh"

# Build all script
cat > "$DEPLOY_DIR/scripts/build-all.sh" << 'EOFSCRIPT'
#!/bin/bash
# Build all components on a tower

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Building biomeOS..."
cd "$DEPLOY_ROOT/biomeOS"
cargo build --release

echo ""
echo "Building primals..."
for primal in songbird beardog nestgate; do
    PRIMAL_DIR="$DEPLOY_ROOT/primals/source/$primal"
    if [ -d "$PRIMAL_DIR" ]; then
        echo "  Building $primal..."
        cd "$PRIMAL_DIR"
        cargo build --release
        cp "target/release/$primal" "$DEPLOY_ROOT/primals/"
    fi
done

echo ""
echo "✅ All components built!"
EOFSCRIPT

chmod +x "$DEPLOY_DIR/scripts/build-all.sh"

echo -e "${GREEN}✅ Deployment scripts created${NC}"
echo ""

# Copy documentation
echo -e "${CYAN}Copying documentation...${NC}"

cp /home/eastgate/Development/ecoPrimals/phase2/biomeOS/README.md "$DEPLOY_DIR/docs/"
cp /home/eastgate/Development/ecoPrimals/phase2/biomeOS/DEPLOYMENT_READINESS_ASSESSMENT.md "$DEPLOY_DIR/docs/" 2>/dev/null || true
cp /home/eastgate/Development/ecoPrimals/phase2/biomeOS/MASTER_DOCUMENTATION_INDEX.md "$DEPLOY_DIR/docs/" 2>/dev/null || true

# Create deployment guide
cat > "$DEPLOY_DIR/docs/USB_DEPLOYMENT_GUIDE.md" << 'EOF'
# USB LAN Deployment Guide

**Target**: Multi-tower LAN deployment  
**Method**: USB transfer with auto-discovery  

## Quick Start

### On Each Tower:

1. **Plug in USB**
2. **Navigate to deployment**:
   ```bash
   cd /media/*/biomeOS-LAN-Deploy
   ```

3. **Run auto-deploy**:
   ```bash
   ./scripts/auto-deploy.sh
   ```

That's it! biomeOS will:
- Auto-discover other towers via mDNS
- Start required primals
- Connect to the LAN mesh

## Manual Deployment

### Option 1: Use Pre-built Binaries

```bash
cd /media/*/biomeOS-LAN-Deploy

# Start primals
./primals/songbird &
./primals/beardog &

# Start biomeOS
./biomeOS/biomeos
```

### Option 2: Build on Tower

```bash
cd /media/*/biomeOS-LAN-Deploy

# Build everything
./scripts/build-all.sh

# Then run auto-deploy
./scripts/auto-deploy.sh
```

## Configuration

### Tower-Specific Configs

Located in `configs/`:
- `tower1.toml` - For tower1
- `tower2.toml` - For tower2
- `tower3.toml` - For tower3
- `lan-discovery.toml` - Generic LAN config

### Edit Config

```bash
# Edit your tower's config
nano configs/$(hostname).toml

# Or use generic LAN config
export BIOMEOS_CONFIG=configs/lan-discovery.toml
```

## Network Discovery

### Automatic (mDNS)

Towers automatically discover each other on the LAN:
- No manual configuration needed
- Uses mDNS for service discovery
- Works on same subnet

### Verification

```bash
# Check if other towers are discovered
# biomeOS will log discovered services
grep "Discovered" ~/.biomeos/logs/biomeos.log
```

## Troubleshooting

### Primals Not Starting

```bash
# Check if binaries exist
ls -la primals/

# If missing, build them
./scripts/build-all.sh
```

### Discovery Not Working

```bash
# Check network connectivity
ping tower1.local
ping tower2.local

# Verify mDNS is working
avahi-browse -a

# Use manual endpoints if needed
export SONGBIRD_ENDPOINT=http://tower1.local:8080
```

### Build Failures

```bash
# Ensure Rust is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Then retry build
./scripts/build-all.sh
```

## Next Steps

1. Deploy to all 3 towers
2. Verify inter-tower communication
3. Run BTSP demo to test tunnels
4. Deploy workloads across towers

## Support

See full documentation in `docs/` directory.
EOF

echo -e "${GREEN}✅ Documentation copied${NC}"
echo ""

# Create README for USB root
cat > "$DEPLOY_DIR/README.txt" << 'EOF'
═══════════════════════════════════════════════════════════════
  biomeOS LAN Tower Deployment Package
═══════════════════════════════════════════════════════════════

This USB contains everything needed to deploy biomeOS across
your LAN towers with automatic discovery.

QUICK START:
1. Plug USB into each tower
2. Run: ./scripts/auto-deploy.sh
3. biomeOS auto-discovers and connects!

WHAT'S INCLUDED:
- biomeOS orchestrator (source + binary)
- Primal services (songbird, beardog, etc.)
- Auto-discovery configs for LAN
- Deployment scripts
- Complete documentation

DEPLOYMENT MODES:
- Auto: ./scripts/auto-deploy.sh (recommended)
- Quick: ./scripts/quick-start.sh
- Manual: See docs/USB_DEPLOYMENT_GUIDE.md

DOCUMENTATION:
- USB_DEPLOYMENT_GUIDE.md - Deployment instructions
- README.md - biomeOS overview
- DEPLOYMENT_READINESS_ASSESSMENT.md - Full assessment

NETWORK:
- Auto-discovery via mDNS
- Works on same LAN/subnet
- No manual configuration needed

SUPPORT:
All documentation in docs/ directory.

═══════════════════════════════════════════════════════════════
Ready to deploy! Run ./scripts/auto-deploy.sh on each tower.
═══════════════════════════════════════════════════════════════
EOF

echo -e "${GREEN}✅ README created${NC}"
echo ""

# Create package manifest
cat > "$DEPLOY_DIR/MANIFEST.txt" << EOF
biomeOS LAN Deployment Package
Created: $(date)
Source: $(hostname)

Contents:
- biomeOS/          biomeOS orchestrator (source + binary)
- primals/          Primal services and sources
- configs/          LAN discovery configurations
- scripts/          Deployment automation scripts
- docs/             Complete documentation

Quick Start:
./scripts/auto-deploy.sh

Package Size: $(du -sh "$DEPLOY_DIR" | cut -f1)
EOF

echo -e "${GREEN}✅ Manifest created${NC}"
echo ""

# Summary
echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  ✅ USB Deployment Package Complete!${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}Package Location:${NC} $DEPLOY_DIR"
echo -e "${GREEN}Package Size:${NC} $(du -sh "$DEPLOY_DIR" | cut -f1)"
echo ""
echo -e "${YELLOW}Contents:${NC}"
ls -lh "$DEPLOY_DIR"
echo ""
echo -e "${CYAN}Next Steps:${NC}"
echo "1. Safely eject USB: sudo umount $USB_MOUNT"
echo "2. Plug into each tower"
echo "3. Run: ./scripts/auto-deploy.sh"
echo ""
echo -e "${GREEN}✅ Ready for tower deployment!${NC}"
echo ""

# Sync to ensure all writes complete
sync

echo -e "${YELLOW}Syncing filesystem...${NC}"
sleep 2
echo -e "${GREEN}✅ Sync complete!${NC}"
echo ""
echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"

EOF

