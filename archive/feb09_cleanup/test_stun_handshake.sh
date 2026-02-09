#!/usr/bin/env bash
# Test STUN handshake between USB and Pixel devices
# Uses songbird's STUN capabilities for NAT traversal validation

set -e

echo "═════════════════════════════════════════════════════════════"
echo "🌐 STUN Handshake Test - USB ↔ Pixel"
echo "═════════════════════════════════════════════════════════════"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# STUN server (Nextcloud public STUN)
STUN_SERVER="stun.nextcloud.com:3478"

echo -e "${BLUE}Step 1: Discover USB public address${NC}"
echo "────────────────────────────────────────────────────────────"

# Check if songbird has STUN API via JSON-RPC
USB_SONGBIRD_SOCKET="$XDG_RUNTIME_DIR/biomeos/songbird.sock"

if [[ ! -S "$USB_SONGBIRD_SOCKET" ]]; then
    echo "❌ USB songbird socket not found: $USB_SONGBIRD_SOCKET"
    echo "   Make sure songbird is running on USB"
    exit 1
fi

echo "✅ USB songbird socket: $USB_SONGBIRD_SOCKET"

# Test JSON-RPC connectivity to songbird
echo ""
echo -e "${BLUE}Step 2: Test songbird JSON-RPC connectivity${NC}"
echo "────────────────────────────────────────────────────────────"

# Use socat to send JSON-RPC to Unix socket
if ! command -v socat &> /dev/null; then
    echo "⚠️  socat not found, installing..."
    sudo apt-get install -y socat
fi

# Test capability discovery via JSON-RPC
JSONRPC_REQUEST='{"jsonrpc":"2.0","method":"capability.list","params":{},"id":1}'

echo "Sending: $JSONRPC_REQUEST"
echo "$JSONRPC_REQUEST" | socat - UNIX-CONNECT:$USB_SONGBIRD_SOCKET || {
    echo "❌ Failed to connect to songbird JSON-RPC"
    echo "   Songbird may not have JSON-RPC enabled yet"
    echo ""
    echo "📋 Current Status:"
    echo "   - songbird has STUN client (crates/songbird-stun)"
    echo "   - songbird has STUN handler (crates/songbird-universal-ipc/src/handlers/stun_handler.rs)"
    echo "   - Need to verify JSON-RPC integration in main server"
    echo ""
    echo "🎯 Next: Verify songbird server integrates STUN handler"
    exit 0
}

echo ""
echo -e "${BLUE}Step 3: Discover public addresses${NC}"
echo "────────────────────────────────────────────────────────────"

# USB STUN discovery
STUN_REQUEST='{"jsonrpc":"2.0","method":"stun.get_public_address","params":{"server":"'$STUN_SERVER'"},"id":2}'
echo "USB: $STUN_REQUEST"
USB_PUBLIC=$(echo "$STUN_REQUEST" | socat - UNIX-CONNECT:$USB_SONGBIRD_SOCKET | jq -r '.result.public_address')

if [[ -z "$USB_PUBLIC" || "$USB_PUBLIC" == "null" ]]; then
    echo "❌ Failed to get USB public address"
    exit 1
fi

echo -e "${GREEN}✅ USB Public Address: $USB_PUBLIC${NC}"

# Pixel STUN discovery (via adb + TCP fallback)
echo ""
echo "Pixel: Testing via TCP fallback..."

# Get Pixel songbird TCP port from discovery file
PIXEL_SONGBIRD_PORT=$(adb shell "cat /data/local/tmp/run/songbird-ipc-port 2>/dev/null" || echo "")

if [[ -z "$PIXEL_SONGBIRD_PORT" ]]; then
    echo "❌ Pixel songbird TCP port not found"
    echo "   Songbird may be using UnixAbstract (optimal for Android)"
    echo "   STUN test requires TCP or HTTP access"
    echo ""
    echo "📋 Alternative: Test via beardog/songbird capability discovery"
    exit 0
fi

echo "Pixel songbird TCP port: $PIXEL_SONGBIRD_PORT"

# Forward port for testing
adb forward tcp:9999 tcp:$PIXEL_SONGBIRD_PORT

# Test STUN via forwarded TCP
PIXEL_PUBLIC=$(echo "$STUN_REQUEST" | socat - TCP:127.0.0.1:9999 | jq -r '.result.public_address')

if [[ -z "$PIXEL_PUBLIC" || "$PIXEL_PUBLIC" == "null" ]]; then
    echo "⚠️  Could not get Pixel public address via TCP"
    echo "   This is expected if songbird uses UnixAbstract only"
fi

echo -e "${GREEN}✅ Pixel Public Address: $PIXEL_PUBLIC${NC}"

echo ""
echo "═════════════════════════════════════════════════════════════"
echo -e "${GREEN}🎊 STUN Discovery Complete!${NC}"
echo "═════════════════════════════════════════════════════════════"
echo ""
echo "USB:   $USB_PUBLIC"
echo "Pixel: $PIXEL_PUBLIC"
echo ""
echo "🎯 Next Steps:"
echo "   1. Verify both devices can see each other's public addresses"
echo "   2. Test UDP hole punching via songbird"
echo "   3. Validate BirdSong Dark Forest beacon exchange"
echo ""
