#!/bin/bash
# 🧬 USB Spore Preparation Script
# Pure file operations - No deployment logic!
# Uses Tower CLI for actual orchestration

set -e

echo "════════════════════════════════════════════════════════════════════════════════"
echo "  🧬 USB Spore v14.0 Preparation"
echo "  Zero-Hardcoding Revolution - Using Tower CLI"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""

# Configuration
USB_ROOT="/media/eastgate/BEA6-BBCE"
BIOMEOS_DIR="/home/eastgate/Development/ecoPrimals/phase2/biomeOS"
BEARDOG_BIN="/home/eastgate/Development/ecoPrimals/phase1/beardog/primalBins/beardog-server-v0.15.0-with-v2-api"
SONGBIRD_BIN="/home/eastgate/Development/ecoPrimals/phase1/songbird/target/release/songbird-orchestrator"
TOWER_BIN="$BIOMEOS_DIR/target/release/tower"

# Check USB
if [ ! -d "$USB_ROOT" ]; then
    echo "❌ USB not found at $USB_ROOT"
    exit 1
fi
echo "✅ USB found"
echo ""

# Verify binaries
echo "=== Verifying Binaries ==="
for bin in "$BEARDOG_BIN" "$SONGBIRD_BIN" "$TOWER_BIN"; do
    if [ ! -f "$bin" ]; then
        echo "❌ Missing: $bin"
        exit 1
    fi
    echo "✅ $(basename $bin)"
done
echo ""

# Create structure
echo "=== Creating USB Structure ==="
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/primals"
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/scripts"
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/configs"
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/docs"
echo "✅ Directories created"
echo ""

# Copy binaries
echo "=== Copying Binaries ==="
cp "$BEARDOG_BIN" "$USB_ROOT/biomeOS-LAN-Deploy/primals/beardog-server"
cp "$SONGBIRD_BIN" "$USB_ROOT/biomeOS-LAN-Deploy/primals/songbird-orchestrator"
cp "$TOWER_BIN" "$USB_ROOT/biomeOS-LAN-Deploy/primals/tower"
chmod +x "$USB_ROOT/biomeOS-LAN-Deploy/primals/"*
echo "✅ Binaries copied"
echo ""

# Create tower.env (zero-hardcoding config)
echo "=== Creating Configuration ==="
cat > "$USB_ROOT/biomeOS-LAN-Deploy/configs/tower.env" << 'EOF'
# Tower Configuration - Zero Hardcoding!
# Adjust paths based on USB mount point

SECURITY_PROVIDER_BINARY=/media/USB/biomeOS-LAN-Deploy/primals/beardog-server
SECURITY_PROVIDER_PORT=0  # OS auto-selects!

DISCOVERY_ORCHESTRATOR_BINARY=/media/USB/biomeOS-LAN-Deploy/primals/songbird-orchestrator

# Family credentials loaded from family-seed.conf
# BEARDOG_FAMILY_ID and BEARDOG_FAMILY_SEED exported separately
EOF
echo "✅ tower.env created"

# Create family seed config (if doesn't exist)
if [ ! -f "$USB_ROOT/biomeOS-LAN-Deploy/configs/family-seed.conf" ]; then
    cat > "$USB_ROOT/biomeOS-LAN-Deploy/configs/family-seed.conf" << 'EOF'
# Family Genetic Lineage Configuration
export FAMILY_ID="iidn"
export FAMILY_SEED="V2VsbCBoZWxsbywgdGhlcmUh"
EOF
    echo "✅ family-seed.conf created"
else
    echo "✅ family-seed.conf exists (not overwriting)"
fi
echo ""

# Create minimal activation script (Tower CLI wrapper)
echo "=== Creating Activation Script ==="
cat > "$USB_ROOT/biomeOS-LAN-Deploy/scripts/activate-tower.sh" << 'EOF'
#!/bin/bash
# Minimal Tower Activation Wrapper
# Uses Tower CLI for capability-based orchestration

set -e

cd "$(dirname "$0")/.."

echo "════════════════════════════════════════════════════════════════════════════════"
echo "  🧬 Activating Tower with Zero-Hardcoding"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""

# Source configurations
if [ ! -f "configs/family-seed.conf" ]; then
    echo "❌ configs/family-seed.conf not found!"
    exit 1
fi
source configs/family-seed.conf
echo "✅ Family: $FAMILY_ID"

if [ ! -f "configs/tower.env" ]; then
    echo "❌ configs/tower.env not found!"
    exit 1
fi
source configs/tower.env
echo "✅ Configuration loaded"
echo ""

# Export for BearDog
export BEARDOG_FAMILY_ID="$FAMILY_ID"
export BEARDOG_FAMILY_SEED="$FAMILY_SEED"

echo "Starting Tower CLI with capability-based orchestration..."
echo "  Security Provider: $SECURITY_PROVIDER_BINARY"
echo "  Discovery Orchestrator: $DISCOVERY_ORCHESTRATOR_BINARY"
echo "  Port Selection: Auto (port 0)"
echo ""
echo "Press Ctrl+C to stop gracefully"
echo ""

# Run Tower CLI - it handles everything!
./primals/tower start \
  --security-binary "$SECURITY_PROVIDER_BINARY" \
  --security-port "$SECURITY_PROVIDER_PORT" \
  --discovery-binary "$DISCOVERY_ORCHESTRATOR_BINARY"

# Tower CLI handles:
# - Capability-based orchestration
# - Health monitoring
# - Retry logic
# - Graceful shutdown
EOF
chmod +x "$USB_ROOT/biomeOS-LAN-Deploy/scripts/activate-tower.sh"
echo "✅ activate-tower.sh created"
echo ""

# Copy documentation
echo "=== Copying Documentation ==="
cp "$BIOMEOS_DIR/START_HERE_ZERO_HARDCODING.md" "$USB_ROOT/biomeOS-LAN-Deploy/docs/"
cp "$BIOMEOS_DIR/ZERO_HARDCODING_COMPLETE.txt" "$USB_ROOT/biomeOS-LAN-Deploy/docs/"
cp "$BIOMEOS_DIR/docs/jan3-session/PROPER_USB_DEPLOYMENT_STRATEGY.md" "$USB_ROOT/biomeOS-LAN-Deploy/docs/"
echo "✅ Documentation copied"
echo ""

# Generate checksums
echo "=== Generating Checksums ==="
cd "$USB_ROOT/biomeOS-LAN-Deploy/primals"
sha256sum * > checksums.txt
echo "✅ Checksums generated:"
cat checksums.txt
echo ""

# Create version file
cat > "$USB_ROOT/biomeOS-LAN-Deploy/USB-V14.0-ZERO-HARDCODING.txt" << 'EOF'
════════════════════════════════════════════════════════════════════════════════
  🧬 USB Spore v14.0 - Zero-Hardcoding Revolution
════════════════════════════════════════════════════════════════════════════════

Version: 14.0
Date: January 3, 2026
Status: REVOLUTIONARY ARCHITECTURE - Using Tower CLI

WHAT'S NEW:
✅ Pure capability-based orchestration via Tower CLI
✅ Zero hardcoded ports (port 0 everywhere)
✅ Minimal bash wrapper (~30 lines)
✅ All orchestration in Rust (Tower CLI)
✅ Health monitoring, retry logic, graceful shutdown built-in

DEPLOYMENT:
$ cd /media/USB/biomeOS-LAN-Deploy
$ ./scripts/activate-tower.sh

Press Ctrl+C to stop gracefully.

Tower CLI handles ALL orchestration:
- Capability-based resolution (Security → Discovery)
- Health monitoring (automatic)
- Retry logic (exponential backoff)
- Port auto-selection (OS chooses)
- Graceful shutdown (signal handling)

NO MORE:
❌ Hardcoded ports
❌ Manual process management
❌ PID tracking
❌ Manual health checks
❌ Complex bash coordination

THIS IS THE REVOLUTION! 🚀
EOF

# Sync
echo "=== Syncing USB ==="
sync
echo "✅ USB synced"
echo ""

# Summary
echo "════════════════════════════════════════════════════════════════════════════════"
echo "  ✅ USB SPORE v14.0 PREPARED!"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""
echo "USB Contents:"
echo "  📦 Binaries: beardog-server, songbird-orchestrator, tower (CLI)"
echo "  ⚙️  Config: tower.env (zero hardcoding)"
echo "  🧬 Family: family-seed.conf"
echo "  📜 Script: activate-tower.sh (~30 lines)"
echo "  📚 Docs: Zero-hardcoding guides"
echo ""
echo "Deploy on any tower:"
echo "  $ cd /media/USB/biomeOS-LAN-Deploy"
echo "  $ ./scripts/activate-tower.sh"
echo ""
echo "Tower CLI handles everything - no hardcoded ports, no manual orchestration!"
echo ""
echo "🎊 THE FUTURE IS CAPABILITY-BASED! 🚀"
echo ""


