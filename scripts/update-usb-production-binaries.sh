#!/usr/bin/env bash
#
# Update USB Spore with Correct Production Binaries
#

set -euo pipefail

echo "🔄 Updating USB Spore with Production Binaries..."
echo ""

USB_PATH="/media/eastgate/biomeOS/biomeOS"

if [ ! -d "$USB_PATH" ]; then
    echo "❌ USB not mounted at $USB_PATH"
    exit 1
fi

echo "✅ USB found"
echo ""

# Backup old binaries
echo "📦 Backing up old binaries..."
mkdir -p "$USB_PATH/primals/backup"
cp "$USB_PATH/primals/"* "$USB_PATH/primals/backup/" 2>/dev/null || true
echo "✅ Backed up"
echo ""

# Copy correct beardog (v0.15.0 with v2 API)
echo "🐻 Updating BearDog to v0.15.0 (v2 API)..."
cp /home/eastgate/Development/ecoPrimals/primalBins/beardog-v0.15.0-zero-hardcoding-v2api \
   "$USB_PATH/primals/beardog"
chmod +x "$USB_PATH/primals/beardog"
echo "✅ BearDog v0.15.0 installed"
echo ""

# Copy correct songbird (v3.6)
echo "🐦 Updating Songbird to v3.6..."
cp /home/eastgate/Development/ecoPrimals/primalBins/songbird-orchestrator-v3.6-api-wrapper \
   "$USB_PATH/primals/songbird"
chmod +x "$USB_PATH/primals/songbird"
echo "✅ Songbird v3.6 installed"
echo ""

# Verify
echo "🔍 Verifying binaries..."
ls -lh "$USB_PATH/primals/" | grep -E "beardog|songbird"
echo ""

echo "═══════════════════════════════════════════════════════"
echo "✅ USB Spore Updated with Production Binaries!"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Updated:"
echo "  - BearDog: v0.15.0 (v2 API, zero-hardcoding)"
echo "  - Songbird: v3.6 (API wrapper)"
echo ""
echo "🚀 Ready for deployment to Tower 2!"

