#!/bin/bash
# Simple Songbird P2P Federation Test
# Uses agentReagents template for fast deployment

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🎵 Songbird P2P Federation Test 🎵                      ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Configuration
TEMPLATE="/home/eastgate/Development/ecoPrimals/primalTools/agentReagents/images/templates/rustdesk-ubuntu-22.04-template.qcow2"
BIOMEOS_PKG="/home/eastgate/Development/ecoPrimals/phase2/biomeOS/biomeos-20251228-181049.tar.gz"

# Check prerequisites
if [ ! -f "$TEMPLATE" ]; then
    echo "❌ Template not found: $TEMPLATE"
    exit 1
fi

if [ ! -f "$BIOMEOS_PKG" ]; then
    echo "❌ BiomeOS package not found: $BIOMEOS_PKG"
    exit 1
fi

echo "✅ Prerequisites OK"
echo ""
echo "This script will:"
echo "  1. Create 2 VMs from template (FAST!)"
echo "  2. Deploy biomeOS to both"
echo "  3. Start Songbird P2P"
echo "  4. Test mDNS federation"
echo ""
echo "Ready to proceed!"
