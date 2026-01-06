#!/usr/bin/env bash
#
# USB Spore Preparation Script (Production-Ready)
#
# This script prepares a USB drive with biomeOS tower and primal binaries
# using the modern, capability-based orchestration system.
#
# Usage: ./scripts/prepare-usb-proper.sh [usb_mount_point]
#

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

USB_MOUNT="${1:-/media/eastgate/usbSpore}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   USB Spore Preparation - Production-Ready Deployment${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Verify USB mount point exists
if [ ! -d "$USB_MOUNT" ]; then
    echo -e "${RED}❌ Error: USB mount point not found: $USB_MOUNT${NC}"
    echo -e "${YELLOW}   Please mount your USB drive or specify mount point:${NC}"
    echo -e "${YELLOW}   ./scripts/prepare-usb-proper.sh /path/to/usb${NC}"
    exit 1
fi

echo -e "${GREEN}✅ USB mount point verified: $USB_MOUNT${NC}"
echo ""

# Create directory structure
echo -e "${BLUE}📁 Creating directory structure...${NC}"
mkdir -p "$USB_MOUNT/biomeOS"/{bin,primals,config,logs}
mkdir -p "$USB_MOUNT/biomeOS/docs"
echo -e "${GREEN}✅ Directories created${NC}"
echo ""

# Copy tower binary
echo -e "${BLUE}🗼 Copying tower orchestrator...${NC}"
if [ -f "$PROJECT_ROOT/target/release/tower" ]; then
    cp "$PROJECT_ROOT/target/release/tower" "$USB_MOUNT/biomeOS/bin/"
    chmod +x "$USB_MOUNT/biomeOS/bin/tower"
    echo -e "${GREEN}✅ Tower binary copied ($(du -h "$USB_MOUNT/biomeOS/bin/tower" | cut -f1))${NC}"
else
    echo -e "${RED}❌ Error: tower binary not found. Run: cargo build --release --bin tower${NC}"
    exit 1
fi

# Copy primal binaries
echo -e "${BLUE}🐻 Copying primal binaries...${NC}"
declare -a PRIMALS=("beardog" "songbird")
PRIMALS_COPIED=0

for primal in "${PRIMALS[@]}"; do
    if [ -f "$PROJECT_ROOT/primals/$primal" ]; then
        cp "$PROJECT_ROOT/primals/$primal" "$USB_MOUNT/biomeOS/primals/"
        chmod +x "$USB_MOUNT/biomeOS/primals/$primal"
        SIZE=$(du -h "$USB_MOUNT/biomeOS/primals/$primal" | cut -f1)
        echo -e "${GREEN}   ✅ $primal ($SIZE)${NC}"
        PRIMALS_COPIED=$((PRIMALS_COPIED + 1))
    else
        echo -e "${YELLOW}   ⚠️  $primal not found in primals/ directory${NC}"
    fi
done

if [ $PRIMALS_COPIED -eq 0 ]; then
    echo -e "${RED}❌ Error: No primal binaries found${NC}"
    exit 1
fi
echo ""

# Create environment-based configuration template
echo -e "${BLUE}⚙️  Creating configuration template...${NC}"
cat > "$USB_MOUNT/biomeOS/config/tower.env" << 'EOF'
# BiomeOS Tower Configuration
# This file defines the environment for the tower and all primals
#
# Usage: source this file before running tower
#   source config/tower.env && ./bin/tower start-from-env

# =============================================================================
# Tower Configuration
# =============================================================================

# Hostname for this tower (auto-detected if not set)
export TOWER_HOSTNAME="${HOSTNAME:-tower-$(hostname -s)}"

# =============================================================================
# BearDog Configuration (Security Primal)
# =============================================================================

export PRIMAL_ID="beardog"
export PRIMAL_NAME="BearDog"
export PRIMAL_BINARY="./primals/beardog"

# Capabilities
export PRIMAL_PROVIDES="Security,Encryption,Trust"
export PRIMAL_REQUIRES=""  # BearDog has no dependencies

# BearDog-specific settings
export BEARDOG_API_BIND_ADDR="0.0.0.0:9000"
export BEARDOG_FAMILY_SEED="${BEARDOG_FAMILY_SEED:-your-secure-family-seed-here}"
export BEARDOG_FAMILY_ID="${BEARDOG_FAMILY_ID:-iidn}"

# =============================================================================
# Songbird Configuration (Discovery Orchestrator)
# =============================================================================

# Note: Songbird configuration would go here if available
# It provides Discovery capability and requires Security from BearDog

# =============================================================================
# Health Monitoring
# =============================================================================

export BIOMEOS_HEALTH_CHECK_INTERVAL="30"  # seconds
export BIOMEOS_HEALTH_TIMEOUT="5"          # seconds
export BIOMEOS_RECOVERY_ATTEMPTS="3"

# =============================================================================
# Logging
# =============================================================================

export RUST_LOG="${RUST_LOG:-info,biomeos_core=debug,tower=info}"
export BIOMEOS_LOG_DIR="./logs"
EOF

chmod 644 "$USB_MOUNT/biomeOS/config/tower.env"
echo -e "${GREEN}✅ Configuration template created${NC}"
echo ""

# Create activation script
echo -e "${BLUE}🚀 Creating activation script...${NC}"
cat > "$USB_MOUNT/biomeOS/activate-tower.sh" << 'EOF'
#!/usr/bin/env bash
#
# BiomeOS Tower Activation Script
#
# This script activates the tower on a new machine using environment-based configuration.
# No hardcoded ports, names, or dependencies - all configured via environment.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   BiomeOS Tower Activation${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check for configuration
if [ ! -f "config/tower.env" ]; then
    echo -e "${RED}❌ Error: config/tower.env not found${NC}"
    echo -e "${YELLOW}   Please create configuration from template${NC}"
    exit 1
fi

# Load configuration
echo -e "${BLUE}📋 Loading configuration from config/tower.env...${NC}"
source config/tower.env
echo -e "${GREEN}✅ Configuration loaded${NC}"
echo ""

# Verify family seed is set
if [ "$BEARDOG_FAMILY_SEED" = "your-secure-family-seed-here" ]; then
    echo -e "${RED}❌ Error: BEARDOG_FAMILY_SEED not configured${NC}"
    echo -e "${YELLOW}   Please edit config/tower.env and set a secure family seed${NC}"
    exit 1
fi

# Display configuration
echo -e "${BLUE}🔍 Configuration Summary:${NC}"
echo -e "   Tower Hostname: ${GREEN}$TOWER_HOSTNAME${NC}"
echo -e "   Family ID:      ${GREEN}$BEARDOG_FAMILY_ID${NC}"
echo -e "   BearDog Bind:   ${GREEN}$BEARDOG_API_BIND_ADDR${NC}"
echo -e "   Log Level:      ${GREEN}${RUST_LOG}${NC}"
echo ""

# Create logs directory
mkdir -p logs

# Start tower using environment-based configuration
echo -e "${GREEN}🚀 Starting Tower with capability-based orchestration...${NC}"
echo -e "${BLUE}   (Press Ctrl+C to stop)${NC}"
echo ""

exec ./bin/tower start-from-env
EOF

chmod +x "$USB_MOUNT/biomeOS/activate-tower.sh"
echo -e "${GREEN}✅ Activation script created${NC}"
echo ""

# Create README
echo -e "${BLUE}📚 Creating deployment documentation...${NC}"
cat > "$USB_MOUNT/biomeOS/README.md" << 'EOF'
# BiomeOS USB Spore - Deployment Guide

This USB drive contains a complete BiomeOS tower deployment with capability-based orchestration.

## 🎯 Zero-Hardcoding Architecture

This deployment uses the **Infant Model** where:
- Primals only know their own identity and capabilities
- No hardcoded ports, names, or dependencies
- Everything configured via environment variables
- Automatic dependency resolution via capability matching

## 📦 Contents

```
biomeOS/
├── bin/
│   └── tower                # Rust orchestrator binary
├── primals/
│   ├── beardog             # Security primal
│   └── songbird            # Discovery orchestrator (if available)
├── config/
│   └── tower.env           # Environment configuration
├── logs/                    # Runtime logs
├── activate-tower.sh       # Quick start script
└── README.md               # This file
```

## 🚀 Quick Start

1. **Edit Configuration**:
   ```bash
   nano config/tower.env
   # Set BEARDOG_FAMILY_SEED to your secure seed
   # Adjust ports if needed
   ```

2. **Activate Tower**:
   ```bash
   ./activate-tower.sh
   ```

That's it! The tower will:
- Load configuration from environment
- Start BearDog (Security capability provider)
- Start Songbird if available (Discovery capability)
- Begin health monitoring
- Handle graceful shutdown on Ctrl+C

## 📝 Configuration

Edit `config/tower.env` to customize:
- `BEARDOG_FAMILY_SEED` - **REQUIRED**: Your family's cryptographic seed
- `BEARDOG_FAMILY_ID` - Family identifier (default: "iidn")
- `BEARDOG_API_BIND_ADDR` - API bind address (default: "0.0.0.0:9000")
- `RUST_LOG` - Logging level (default: "info")

## 🔐 Security Notes

- **NEVER** commit `tower.env` with real family seeds to version control
- Keep family seeds secure and backed up separately
- Each tower can have its own family or share family membership
- Family membership enables encrypted inter-tower communication

## 🌐 Multi-Tower Deployment

To deploy across multiple towers:

1. **Same Family**: Use the same `BEARDOG_FAMILY_SEED` on all towers
   - Enables encrypted communication
   - Automatic trust within family
   - Genetic lineage tracking

2. **Different Families**: Use different seeds
   - Trust evaluation via BearDog
   - Cross-family federation support
   - Geographic distribution

## 🛠️ Troubleshooting

### Tower won't start
- Check `logs/` directory for error messages
- Verify family seed is set in `config/tower.env`
- Ensure ports are not already in use

### Primal health check failures
- Check primal logs in `logs/`
- Verify binary has execute permissions
- Check firewall rules if accessing remotely

### Discovery not working
- Ensure Songbird binary is present in `primals/`
- Check UDP multicast is not blocked by firewall
- Verify network interface supports multicast

## 📊 Monitoring

View tower status:
```bash
./bin/tower status
# Note: Requires persistent state feature (planned)
```

View logs in real-time:
```bash
tail -f logs/*.log
```

## 🔄 Updates

To update binaries:
1. Replace binaries in `bin/` and `primals/`
2. Restart tower (Ctrl+C, then `./activate-tower.sh`)
3. Configuration persists between updates

## 🆘 Support

For issues or questions:
- Check documentation in `/docs/`
- Review error logs in `/logs/`
- Consult the wateringHole knowledge base

---

**BiomeOS**: Zero-Hardcoding, Capability-Based, Production-Ready
EOF

echo -e "${GREEN}✅ Documentation created${NC}"
echo ""

# Copy key documentation
echo -e "${BLUE}📄 Copying documentation...${NC}"
if [ -d "$PROJECT_ROOT/docs/jan3-session" ]; then
    cp "$PROJECT_ROOT/docs/jan3-session/TECHNICAL_DEBT_RESOLUTION_COMPLETE.md" \
       "$USB_MOUNT/biomeOS/docs/" 2>/dev/null || true
fi
echo -e "${GREEN}✅ Documentation copied${NC}"
echo ""

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ USB Spore Preparation Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BLUE}📊 Deployment Summary:${NC}"
echo -e "   Location:     ${GREEN}$USB_MOUNT/biomeOS/${NC}"
echo -e "   Tower Binary: ${GREEN}$(du -h "$USB_MOUNT/biomeOS/bin/tower" | cut -f1)${NC}"
echo -e "   Primals:      ${GREEN}$PRIMALS_COPIED binary(ies)${NC}"
echo -e "   Config:       ${GREEN}$USB_MOUNT/biomeOS/config/tower.env${NC}"
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT: Edit config/tower.env and set BEARDOG_FAMILY_SEED${NC}"
echo ""
echo -e "${BLUE}🚀 To deploy on target tower:${NC}"
echo -e "   1. Mount USB on target machine"
echo -e "   2. cd /path/to/usb/biomeOS"
echo -e "   3. Edit config/tower.env (set family seed)"
echo -e "   4. ./activate-tower.sh"
echo ""
echo -e "${GREEN}Ready for multi-tower LAN deployment! 🌸${NC}"
echo ""

