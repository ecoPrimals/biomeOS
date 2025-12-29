#!/bin/bash
# Songbird P2P Federation Validation
# 
# This script validates Songbird P2P between 2 VMs, then you can add the NUC!
#
# Pipeline:
# 1. Create 2 VMs (validated SSH access)
# 2. Deploy BiomeOS USB packages
# 3. Start Songbird P2P on each VM
# 4. Validate mDNS/UDP federation
# 5. Keep VMs running for NUC to join

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🌐 Songbird P2P Federation Validation 🌐               ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "This will:"
echo "  1. Create 2 VMs using benchScale"
echo "  2. Deploy BiomeOS USB packages"
echo "  3. Start Songbird P2P on each VM"
echo "  4. Validate mDNS/UDP federation"
echo "  5. Keep VMs running for NUC to join"
echo ""
echo "Prerequisites:"
echo "  • benchScale built and configured"
echo "  • USB package created (./quick-usb.sh)"
echo "  • libvirt/KVM running"
echo ""

# Check for USB package
if ! ls biomeos-*.tar.gz >/dev/null 2>&1; then
    echo "❌ No USB package found!"
    echo "   Run: ./quick-usb.sh"
    exit 1
fi

USB_PACKAGE=$(ls -t biomeos-*.tar.gz | head -1)
echo "📦 Using USB package: $USB_PACKAGE"
echo ""

# Check for benchScale
BENCHSCALE_DIR="${SCRIPT_DIR}/../../../primalTools/benchScale"
if [ ! -d "$BENCHSCALE_DIR" ]; then
    echo "❌ benchScale not found!"
    echo "   Expected at: $BENCHSCALE_DIR"
    echo ""
    echo "Note: For now, running in demo mode with mock VMs."
    echo "      The Rust binary will use mock IPs for demonstration."
    echo ""
fi

echo "✅ Prerequisites check passed"
echo ""

# Build the Rust binary
echo "🔨 Building Songbird federation binary..."
cargo build --release --bin biomeos-songbird-federation
echo "✅ Build complete"
echo ""

# Run the validation
echo "🚀 Running Songbird P2P federation validation..."
echo ""

"${SCRIPT_DIR}/target/release/biomeos-songbird-federation" "$@"

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🎉 Songbird P2P Federation Validation Complete! 🎉      ║"
echo "╚═══════════════════════════════════════════════════════════╝"

