#!/usr/bin/env bash
#
# Harvest Primals - Pull stable binaries from primal repositories
#
# This script:
# 1. Pulls latest from primal repos
# 2. Builds release binaries
# 3. Copies to plasmidBin/
# 4. Verifies integrity
# 5. Updates VERSION.txt
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ECOPRIMALS_ROOT="$(cd "$BIOMEOS_ROOT/../.." && pwd)"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║         🧬 Harvesting Primals to PlasmidBin                   ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Archive current nucleus (if exists)
if [ -f "$BIOMEOS_ROOT/plasmidBin/primals/beardog-server" ] || \
   [ -f "$BIOMEOS_ROOT/plasmidBin/primals/songbird" ]; then
    echo "📦 Archiving current nucleus..."
    DATE=$(date +%Y-%m-%d_%H-%M-%S)
    mkdir -p "$BIOMEOS_ROOT/plasmidBin/archive/$DATE"
    
    [ -f "$BIOMEOS_ROOT/plasmidBin/tower/tower" ] && \
        cp "$BIOMEOS_ROOT/plasmidBin/tower/tower" "$BIOMEOS_ROOT/plasmidBin/archive/$DATE/"
    
    [ -f "$BIOMEOS_ROOT/plasmidBin/primals/beardog-server" ] && \
        cp "$BIOMEOS_ROOT/plasmidBin/primals/beardog-server" "$BIOMEOS_ROOT/plasmidBin/archive/$DATE/"
    
    [ -f "$BIOMEOS_ROOT/plasmidBin/primals/songbird" ] && \
        cp "$BIOMEOS_ROOT/plasmidBin/primals/songbird" "$BIOMEOS_ROOT/plasmidBin/archive/$DATE/"
    
    echo "✅ Archived to: plasmidBin/archive/$DATE"
    echo ""
fi

# Build tower (biomeOS)
echo "🏗️  Building tower (biomeOS orchestrator)..."
cd "$BIOMEOS_ROOT"
cargo build --release -p biomeos-core --bin tower --quiet
TOWER_VERSION=$(git log --oneline -1 | cut -d' ' -f1)
echo "✅ tower built: $TOWER_VERSION"
echo ""

# Build BearDog
echo "🐻 Building BearDog..."
cd "$ECOPRIMALS_ROOT/phase1/beardog"
git fetch --quiet
BEARDOG_VERSION=$(git log --oneline -1 | cut -d' ' -f1)
cargo build --release -p beardog-tunnel --bin beardog-server --features btsp-api --quiet
echo "✅ beardog-server built: $BEARDOG_VERSION"
echo ""

# Build Songbird
echo "🐦 Building Songbird..."
cd "$ECOPRIMALS_ROOT/phase1/songbird"
git fetch --quiet
SONGBIRD_VERSION=$(git log --oneline -1 | cut -d' ' -f1)
cargo build --release --bin songbird-orchestrator --quiet
echo "✅ songbird-orchestrator built: $SONGBIRD_VERSION"
echo ""

# Harvest to plasmidBin
echo "🧬 Harvesting to plasmidBin..."
mkdir -p "$BIOMEOS_ROOT/plasmidBin/tower"
mkdir -p "$BIOMEOS_ROOT/plasmidBin/primals"

cp "$BIOMEOS_ROOT/target/release/tower" "$BIOMEOS_ROOT/plasmidBin/tower/"
echo "  ✅ tower → plasmidBin/tower/"

cp "$ECOPRIMALS_ROOT/phase1/beardog/target/release/beardog-server" "$BIOMEOS_ROOT/plasmidBin/primals/"
echo "  ✅ beardog-server → plasmidBin/primals/"

cp "$ECOPRIMALS_ROOT/phase1/songbird/target/release/songbird-orchestrator" "$BIOMEOS_ROOT/plasmidBin/primals/songbird"
echo "  ✅ songbird-orchestrator → plasmidBin/primals/songbird"

echo ""

# Update VERSION.txt
echo "📝 Updating VERSION.txt..."
cat > "$BIOMEOS_ROOT/plasmidBin/VERSION.txt" << EOF
# PlasmidBin Version Manifest
# Updated: $(date)

tower: git:$TOWER_VERSION
beardog-server: git:$BEARDOG_VERSION  
songbird: git:$SONGBIRD_VERSION

# Harvested with: scripts/harvest-primals.sh
EOF
echo "✅ VERSION.txt updated"
echo ""

# Verify integrity
echo "🔍 Verifying nucleus integrity..."
"$SCRIPT_DIR/verify-nucleus.sh"

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║         ✅ Harvest Complete!                                   ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "📋 Version Manifest:"
cat "$BIOMEOS_ROOT/plasmidBin/VERSION.txt"

