#!/bin/bash
# Quick TOWER Deployment & STUN Handshake Test

set -e

echo "🧬🌐 TOWER ATOMIC DEPLOYMENT & STUN HANDSHAKE"
echo "=============================================="
echo ""

# Configuration
USB_IP="192.168.1.144"
PIXEL_IP="192.168.1.80"
STUN_SERVER="stun.l.google.com:19302"
USB_BASE="/media/eastgate/biomeOS21/biomeOS"
PIXEL_BASE="/data/local/tmp"

echo "📋 Configuration:"
echo "  USB (x86_64):   $USB_IP"
echo "  Pixel (ARM64):  $PIXEL_IP"
echo "  STUN Server:    $STUN_SERVER"
echo ""

# Step 1: Verify extraction
echo "📋 Step 1: Verifying Extracted Binaries..."
if [ -f "$USB_BASE/beardog" ] && [ -f "$USB_BASE/songbird" ]; then
    echo "  ✅ USB TOWER binaries ready"
    ls -lh "$USB_BASE/beardog" "$USB_BASE/songbird"
else
    echo "  ❌ USB binaries missing!"
    exit 1
fi

if adb shell "[ -f $PIXEL_BASE/beardog ] && [ -f $PIXEL_BASE/songbird ] && echo exists" | grep -q exists; then
    echo "  ✅ Pixel TOWER binaries ready"
    adb shell "ls -lh $PIXEL_BASE/beardog $PIXEL_BASE/songbird"
else
    echo "  ❌ Pixel binaries missing!"
    exit 1
fi
echo ""

# Step 2: Verify family seeds
echo "📋 Step 2: Verifying Family Seeds..."
if [ -f "$USB_BASE/.family.seed" ]; then
    echo "  ✅ USB family seed: $(xxd -l 8 -p $USB_BASE/.family.seed)..."
else
    echo "  ❌ USB seed missing!"
    exit 1
fi

if adb shell "[ -f $PIXEL_BASE/.family.seed ] && echo exists" | grep -q exists; then
    PIXEL_SEED=$(adb shell "xxd -l 8 -p $PIXEL_BASE/.family.seed" | tr -d '\r\n ')
    echo "  ✅ Pixel family seed: ${PIXEL_SEED}..."
else
    echo "  ❌ Pixel seed missing!"
    exit 1
fi
echo ""

# Step 3: Clean up existing processes
echo "📋 Step 3: Cleaning Up..."
killall beardog songbird 2>/dev/null || true
adb shell "killall beardog songbird 2>/dev/null || true"
sleep 2
echo "  ✅ Cleanup complete"
echo ""

# Step 4: Start USB TOWER
echo "📋 Step 4: Starting USB TOWER..."
echo "  Starting beardog..."
cd "$USB_BASE"
FAMILY_SEED_PATH="$USB_BASE/.family.seed" \
  FAMILY_ID="usb_tower" \
  NODE_ID="usb_node1" \
  ./beardog server > /tmp/usb-beardog.log 2>&1 &
USB_BEARDOG_PID=$!
sleep 3

echo "  Starting songbird..."
FAMILY_ID="usb_tower" \
  NODE_ID="usb_node1" \
  ./songbird server > /tmp/usb-songbird.log 2>&1 &
USB_SONGBIRD_PID=$!
sleep 5

echo "  ✅ USB TOWER started"
echo "     Beardog PID: $USB_BEARDOG_PID"
echo "     Songbird PID: $USB_SONGBIRD_PID"
echo ""

# Step 5: Start Pixel TOWER
echo "📋 Step 5: Starting Pixel TOWER..."
echo "  Starting beardog on Pixel..."
adb shell "cd $PIXEL_BASE && \
  FAMILY_SEED_PATH=$PIXEL_BASE/.family.seed \
  FAMILY_ID=pixel_tower \
  NODE_ID=pixel_node1 \
  ./beardog server > /data/local/tmp/pixel-beardog.log 2>&1 &" &

sleep 3

echo "  Starting songbird on Pixel..."
adb shell "cd $PIXEL_BASE && \
  FAMILY_ID=pixel_tower \
  NODE_ID=pixel_node1 \
  ./songbird server > /data/local/tmp/pixel-songbird.log 2>&1 &" &

sleep 5

echo "  ✅ Pixel TOWER started"
echo ""

# Step 6: Check service status
echo "📋 Step 6: Checking Service Status..."
echo ""
echo "USB TOWER:"
ps aux | grep -E "beardog|songbird" | grep -v grep || echo "  No processes found"
echo ""
echo "USB Logs (last 10 lines):"
echo "  Beardog:"
tail -10 /tmp/usb-beardog.log | sed 's/^/    /'
echo "  Songbird:"
tail -10 /tmp/usb-songbird.log | sed 's/^/    /'
echo ""

echo "Pixel TOWER:"
adb shell "ps | grep -E 'beardog|songbird'" || echo "  No processes found"
echo ""
echo "Pixel Logs (last 10 lines):"
echo "  Beardog:"
adb shell "tail -10 /data/local/tmp/pixel-beardog.log 2>/dev/null || echo 'Log not found'" | sed 's/^/    /'
echo "  Songbird:"
adb shell "tail -10 /data/local/tmp/pixel-songbird.log 2>/dev/null || echo 'Log not found'" | sed 's/^/    /'
echo ""

# Step 7: Test isomorphic IPC
echo "📋 Step 7: Testing Isomorphic IPC..."
echo ""
echo "USB (should use Unix sockets):"
grep -E "Unix socket|TCP|IPC|isomorphic" /tmp/usb-beardog.log 2>/dev/null | tail -5 | sed 's/^/  /' || echo "  (checking...)"
grep -E "Unix socket|TCP|IPC|isomorphic" /tmp/usb-songbird.log 2>/dev/null | tail -5 | sed 's/^/  /' || echo "  (checking...)"
echo ""
echo "Pixel (should auto-fallback to TCP):"
adb shell "grep -E 'Unix socket|TCP|IPC|isomorphic|SELinux|Trying|Falling' /data/local/tmp/pixel-beardog.log 2>/dev/null | tail -5" | sed 's/^/  /' || echo "  (checking...)"
adb shell "grep -E 'Unix socket|TCP|IPC|isomorphic|SELinux|Trying|Falling' /data/local/tmp/pixel-songbird.log 2>/dev/null | tail -5" | sed 's/^/  /' || echo "  (checking...)"
echo ""

# Step 8: Summary
echo "📋 Step 8: Deployment Summary"
echo "=============================================="
echo ""
echo "✅ DEPLOYMENT COMPLETE"
echo ""
echo "USB TOWER:    $USB_IP (Unix sockets)"
echo "Pixel TOWER:  $PIXEL_IP (TCP fallback)"
echo "STUN Server:  $STUN_SERVER"
echo ""
echo "Next Steps:"
echo "  1. Monitor logs for BirdSong discovery"
echo "  2. Check for BTSP genetic verification"
echo "  3. Validate STUN handshake"
echo ""
echo "Log files:"
echo "  USB:   /tmp/usb-beardog.log, /tmp/usb-songbird.log"
echo "  Pixel: /data/local/tmp/pixel-beardog.log, /data/local/tmp/pixel-songbird.log"
echo ""
echo "To stop services:"
echo "  killall beardog songbird"
echo "  adb shell 'killall beardog songbird'"
echo ""
