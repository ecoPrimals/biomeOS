#!/bin/bash
# Minimal script - just calls benchScale!
# benchScale handles all libvirt/KVM stuff

set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🌐 Creating Songbird P2P Federation 🌐                 ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "Using benchScale to handle everything!"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHSCALE_DIR="${SCRIPT_DIR}/../../primalTools/benchscale"
TOPOLOGY="${SCRIPT_DIR}/topologies/songbird-2node.yaml"

if [ ! -d "$BENCHSCALE_DIR" ]; then
    echo "❌ benchScale not found at: $BENCHSCALE_DIR"
    exit 1
fi

if [ ! -f "$TOPOLOGY" ]; then
    echo "❌ Topology not found at: $TOPOLOGY"
    exit 1
fi

echo "📋 Configuration:"
echo "  • benchScale: $BENCHSCALE_DIR"
echo "  • Topology: $TOPOLOGY"
echo ""

# Let benchScale do all the work!
echo "🚀 Creating VMs via benchScale..."
cd "$BENCHSCALE_DIR"
cargo run --release -- create songbird-federation --topology "$TOPOLOGY"

echo ""
echo "✅ VMs created by benchScale!"
echo ""
echo "Now run the Rust binary to:"
echo "  • Deploy BiomeOS USB"
echo "  • Start Songbird P2P"
echo "  • Validate federation"
echo ""
echo "Run: cargo run --release --bin biomeos-songbird-federation"
