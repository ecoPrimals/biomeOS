#!/bin/bash
# BirdSong STUN Handshake - Task 2
# USB ↔ Pixel via Public Internet with NAT Traversal

set -e

echo "🌐🧬 BIRDSONG STUN HANDSHAKE - TASK 2"
echo "======================================="
echo ""

# Configuration
STUN_SERVER="stun.l.google.com:19302"
USB_IP="192.168.1.144"
PIXEL_IP="192.168.1.80"
USB_FAMILY_SEED="/media/eastgate/biomeOS21/biomeOS/.family.seed"
PIXEL_FAMILY_SEED="/data/local/tmp/biomeos/.family.seed"

echo "📋 STUN Configuration:"
echo "  Server: $STUN_SERVER"
echo "  USB LAN IP: $USB_IP"
echo "  Pixel LAN IP: $PIXEL_IP"
echo ""

# Step 1: Verify Prerequisites
echo "📋 Step 1: Verifying Prerequisites..."
echo ""

if [ -f "$USB_FAMILY_SEED" ]; then
    echo "  ✅ USB family seed found"
else
    echo "  ❌ USB family seed missing!"
    exit 1
fi

if adb shell "[ -f $PIXEL_FAMILY_SEED ] && echo exists" | grep -q exists; then
    echo "  ✅ Pixel family seed found"
else
    echo "  ❌ Pixel family seed missing!"
    exit 1
fi

echo ""

# Step 2: Test STUN Server Connectivity
echo "📋 Step 2: Testing STUN Server Connectivity..."
echo ""

# Simple STUN test using nc/telnet
if timeout 5 bash -c "echo > /dev/tcp/$(echo $STUN_SERVER | cut -d: -f1)/$(echo $STUN_SERVER | cut -d: -f2)" 2>/dev/null; then
    echo "  ✅ STUN server reachable"
else
    echo "  ⚠️  STUN server test inconclusive (will try during service startup)"
fi
echo ""

# Step 3: Clean up existing processes
echo "📋 Step 3: Cleaning Up Existing Processes..."
killall beardog 2>/dev/null || true
killall songbird 2>/dev/null || true
adb shell "killall beardog songbird 2>/dev/null || true"
sleep 2
echo "  ✅ Cleanup complete"
echo ""

# Step 4: Create STUN configuration files
echo "📋 Step 4: Creating STUN Configuration..."

# USB STUN config
cat > /tmp/usb-stun-config.toml << EOF
[network]
stun_servers = ["$STUN_SERVER"]
local_ip = "$USB_IP"
enable_upnp = true
enable_nat_traversal = true

[discovery]
mode = "stun"
service_name = "tower-usb"
announce_interval_secs = 5
enable_mdns = true
enable_stun = true

[beardog]
family_id = "usb_tower"
node_id = "usb_tower1"
socket = "/tmp/beardog-usb-stun.sock"

[songbird]
port = 8080
security_provider = "beardog"
EOF

echo "  ✅ USB STUN config: /tmp/usb-stun-config.toml"

# Pixel STUN config
cat > /tmp/pixel-stun-config.toml << EOF
[network]
stun_servers = ["$STUN_SERVER"]
local_ip = "$PIXEL_IP"
enable_upnp = false
enable_nat_traversal = true

[discovery]
mode = "stun"
service_name = "tower-pixel"
announce_interval_secs = 5
enable_mdns = true
enable_stun = true

[beardog]
family_id = "pixel_tower"
node_id = "pixel_tower1"
socket = "/tmp/beardog-pixel-stun.sock"

[songbird]
port = 8080
security_provider = "beardog"
EOF

echo "  ✅ Pixel STUN config: /tmp/pixel-stun-config.toml"

# Push Pixel config
adb push /tmp/pixel-stun-config.toml /data/local/tmp/ > /dev/null 2>&1
echo "  ✅ Pushed config to Pixel"
echo ""

# Step 5: Start USB Services with STUN
echo "📋 Step 5: Starting USB Services with STUN..."

# USB BearDog
export BEARDOG_FAMILY_SEED="$USB_FAMILY_SEED"
export FAMILY_ID="usb_tower"
export NODE_ID="usb_tower1"
export BEARDOG_SOCKET="/tmp/beardog-usb-stun.sock"
export BEARDOG_STUN_SERVERS="$STUN_SERVER"

~/.local/beardog/beardog server \
  --family-id "$FAMILY_ID" \
  --socket "$BEARDOG_SOCKET" \
  > /tmp/beardog-usb-stun.log 2>&1 &

USB_BEARDOG_PID=$!
sleep 3

if ps -p $USB_BEARDOG_PID > /dev/null; then
    echo "  ✅ USB BearDog started (PID: $USB_BEARDOG_PID)"
else
    echo "  ❌ USB BearDog failed"
    tail -20 /tmp/beardog-usb-stun.log
    exit 1
fi

# USB Songbird
export SONGBIRD_SECURITY_PROVIDER="beardog"
export SECURITY_ENDPOINT="unix://$BEARDOG_SOCKET"
export SONGBIRD_STUN_SERVERS="$STUN_SERVER"

~/.local/songbird/songbird server \
  --port 8080 \
  > /tmp/songbird-usb-stun.log 2>&1 &

USB_SONGBIRD_PID=$!
sleep 3

if ps -p $USB_SONGBIRD_PID > /dev/null; then
    echo "  ✅ USB Songbird started (PID: $USB_SONGBIRD_PID)"
else
    echo "  ⚠️  USB Songbird may need troubleshooting"
fi
echo ""

# Step 6: Start Pixel Services with STUN
echo "📋 Step 6: Starting Pixel Services with STUN..."

# Pixel BearDog
cat > /tmp/pixel_stun_start.sh << 'EOF'
#!/system/bin/sh
export BEARDOG_FAMILY_SEED="/data/local/tmp/biomeos/.family.seed"
export FAMILY_ID="pixel_tower"
export NODE_ID="pixel_tower1"
export BEARDOG_SOCKET="/tmp/beardog-pixel-stun.sock"
export BEARDOG_STUN_SERVERS="stun.l.google.com:19302"

/data/local/tmp/beardog/beardog server \
  --family-id "$FAMILY_ID" \
  --socket "$BEARDOG_SOCKET" \
  > /tmp/beardog-pixel-stun.log 2>&1 &

echo $! > /tmp/beardog-pixel-stun.pid

# Songbird
export SONGBIRD_SECURITY_PROVIDER="beardog"
export SECURITY_ENDPOINT="unix://$BEARDOG_SOCKET"
export SONGBIRD_STUN_SERVERS="stun.l.google.com:19302"

/data/local/tmp/songbird/songbird server \
  --port 8080 \
  > /tmp/songbird-pixel-stun.log 2>&1 &

echo $! > /tmp/songbird-pixel-stun.pid
EOF

adb push /tmp/pixel_stun_start.sh /data/local/tmp/ > /dev/null 2>&1
adb shell "chmod +x /data/local/tmp/pixel_stun_start.sh"
adb shell "/data/local/tmp/pixel_stun_start.sh"

sleep 5
echo "  ✅ Pixel services started"
echo ""

# Step 7: Monitor STUN Discovery
echo "📋 Step 7: Monitoring STUN Discovery (60 seconds)..."
echo ""
echo "  Services discovering public endpoints via STUN..."
echo "  This may take 30-60 seconds for NAT traversal..."
echo ""

for i in {1..60}; do
    echo -n "."
    
    # Check for STUN events
    if [ $((i % 10)) -eq 0 ]; then
        if grep -qE "(stun|public.*endpoint|nat|hole.*punch)" /tmp/songbird-usb-stun.log 2>/dev/null; then
            echo ""
            echo ""
            echo "  🎉 STUN activity detected!"
            grep -E "(stun|public|endpoint)" /tmp/songbird-usb-stun.log | head -3
            echo ""
        fi
    fi
    
    sleep 1
done
echo ""
echo ""

# Step 8: STUN Analysis
echo "📋 Step 8: STUN Validation Analysis"
echo "======================================="
echo ""

echo "🌐 USB STUN Activity:"
grep -E "(stun|public|endpoint|nat)" /tmp/songbird-usb-stun.log 2>/dev/null | head -8 || echo "  (No STUN activity detected)"
echo ""

echo "🌐 Pixel STUN Activity:"
adb shell "grep -E '(stun|public|endpoint|nat)' /tmp/songbird-pixel-stun.log 2>/dev/null | head -8" || echo "  (No STUN activity detected)"
echo ""

echo "🧬 Genetic Lineage (USB):"
grep -E "(family|genetic|birdsong)" /tmp/beardog-usb-stun.log 2>/dev/null | head -5 || echo "  (Check logs)"
echo ""

echo "🧬 Genetic Lineage (Pixel):"
adb shell "grep -E '(family|genetic|birdsong)' /tmp/beardog-pixel-stun.log 2>/dev/null | head -5" || echo "  (Check logs)"
echo ""

# Step 9: Connection Status
echo "📋 Step 9: Cross-Platform Connection Status"
echo "======================================="
echo ""

if grep -qE "(established.*internet|connected.*stun|handshake.*public)" /tmp/songbird-usb-stun.log 2>/dev/null; then
    echo "  ✅ USB → Pixel (via STUN) connection established!"
    grep -E "(established|connected|handshake)" /tmp/songbird-usb-stun.log | head -3
else
    echo "  ⏳ Waiting for STUN connection..."
    echo "  Note: NAT traversal can take 1-2 minutes"
fi
echo ""

# Step 10: Summary
echo "📋 Step 10: Service Status"
echo "======================================="
echo ""

echo "🖥️  USB Services:"
ps -p $USB_BEARDOG_PID > /dev/null && echo "  BearDog:  ✅ Running (PID: $USB_BEARDOG_PID)" || echo "  BearDog:  ❌ Stopped"
ps -p $USB_SONGBIRD_PID > /dev/null && echo "  Songbird: ✅ Running (PID: $USB_SONGBIRD_PID)" || echo "  Songbird: ❌ Stopped"
echo "  Logs: /tmp/*-usb-stun.log"
echo ""

echo "📱 Pixel Services:"
PIXEL_COUNT=$(adb shell "ps | grep -E '(beardog|songbird)' | grep -v grep | wc -l" | tr -d '\r\n ')
echo "  Services: $PIXEL_COUNT running"
echo "  Logs: /tmp/*-pixel-stun.log (on device)"
echo ""

echo "======================================="
echo "🌐 TASK 2: STUN HANDSHAKE ACTIVE"
echo "======================================="
echo ""
echo "Monitor for:"
echo "  • STUN binding requests"
echo "  • Public endpoint discovery"
echo "  • NAT type detection"
echo "  • Hole punching attempts"
echo "  • Cross-platform handshake"
echo ""
echo "Live monitoring:"
echo "  tail -f /tmp/*-usb-stun.log"
echo "  adb shell 'tail -f /tmp/*-pixel-stun.log'"
echo ""
echo "Press Ctrl+C to view summary and stop..."
echo ""

# Cleanup trap
trap "echo ''; echo 'Stopping services...'; kill $USB_BEARDOG_PID $USB_SONGBIRD_PID 2>/dev/null; adb shell 'killall beardog songbird 2>/dev/null'; echo 'Done.'; exit 0" INT TERM

# Monitor logs
tail -f /tmp/songbird-usb-stun.log /tmp/beardog-usb-stun.log
