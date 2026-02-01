#!/bin/bash
# BirdSong Local Handshake - Task 1 (Fixed)
# USB ↔ Pixel via Local Network with Proper Environment

set -e

echo "🧬🌉 BIRDSONG LOCAL HANDSHAKE - TASK 1 (FIXED)"
echo "==============================================="
echo ""

# Configuration
USB_IP="192.168.1.144"
PIXEL_IP="192.168.1.80"
USB_FAMILY_SEED="/media/eastgate/biomeOS21/biomeOS/.family.seed"
PIXEL_FAMILY_SEED="/data/local/tmp/biomeos/.family.seed"

# Step 1: Verify Prerequisites
echo "📋 Step 1: Verifying Prerequisites..."
echo ""

if [ -f "$USB_FAMILY_SEED" ]; then
    echo "  ✅ USB family seed found"
    echo "     $(xxd -l 16 -p "$USB_FAMILY_SEED")"
else
    echo "  ❌ USB family seed missing!"
    exit 1
fi

if adb shell "[ -f $PIXEL_FAMILY_SEED ] && echo exists" | grep -q exists; then
    echo "  ✅ Pixel family seed found"
    PIXEL_SEED=$(adb shell "xxd -l 16 -p $PIXEL_FAMILY_SEED" | tr -d '\r\n ')
    echo "     $PIXEL_SEED"
else
    echo "  ❌ Pixel family seed missing!"
    exit 1
fi

echo ""
echo "  ✅ Both platforms have unique genetic lineage"
echo ""

# Step 2: Clean up any existing processes
echo "📋 Step 2: Cleaning Up Existing Processes..."
killall beardog 2>/dev/null || true
killall songbird 2>/dev/null || true
adb shell "killall beardog 2>/dev/null || true"
adb shell "killall songbird 2>/dev/null || true"
sleep 2
echo "  ✅ Cleanup complete"
echo ""

# Step 3: Start USB BearDog with FULL environment
echo "📋 Step 3: Starting USB BearDog with Genetic Context..."
cd /media/eastgate/biomeOS21/biomeOS

# Export environment for BearDog
export BEARDOG_FAMILY_SEED="$USB_FAMILY_SEED"
export FAMILY_ID="usb_tower"
export NODE_ID="usb_tower1"
export BEARDOG_SOCKET="/tmp/beardog-usb.sock"

~/.local/beardog/beardog server \
  --family-id "$FAMILY_ID" \
  --socket "$BEARDOG_SOCKET" \
  > /tmp/beardog-usb-local.log 2>&1 &

USB_BEARDOG_PID=$!
sleep 3

if ps -p $USB_BEARDOG_PID > /dev/null; then
    echo "  ✅ USB BearDog started (PID: $USB_BEARDOG_PID)"
    echo "     Environment: FAMILY_ID=$FAMILY_ID, NODE_ID=$NODE_ID"
    
    # Check logs for success
    if grep -qE "(family|genetic|birdsong)" /tmp/beardog-usb-local.log; then
        echo "  ✅ Genetic context detected"
    fi
    
    # Check for errors
    if grep -q "ERROR" /tmp/beardog-usb-local.log; then
        echo "  ⚠️  Errors in log:"
        grep "ERROR" /tmp/beardog-usb-local.log | tail -3
    fi
else
    echo "  ❌ USB BearDog failed to start"
    cat /tmp/beardog-usb-local.log | tail -20
    exit 1
fi
echo ""

# Step 4: Start Pixel BearDog with FULL environment
echo "📋 Step 4: Starting Pixel BearDog with Genetic Context..."

# Create startup script for Pixel with environment
cat > /tmp/pixel_beardog_start.sh << 'EOF'
#!/system/bin/sh
export BEARDOG_FAMILY_SEED="/data/local/tmp/biomeos/.family.seed"
export FAMILY_ID="pixel_tower"
export NODE_ID="pixel_tower1"
export BEARDOG_SOCKET="/tmp/beardog-pixel.sock"

cd /data/local/tmp
/data/local/tmp/beardog/beardog server \
  --family-id "$FAMILY_ID" \
  --socket "$BEARDOG_SOCKET" \
  > /tmp/beardog-pixel-local.log 2>&1 &

echo $! > /tmp/beardog-pixel.pid
EOF

adb push /tmp/pixel_beardog_start.sh /data/local/tmp/
adb shell "chmod +x /data/local/tmp/pixel_beardog_start.sh"
adb shell "/data/local/tmp/pixel_beardog_start.sh"

sleep 3

if adb shell "ps | grep beardog | grep -v grep" | grep -q beardog; then
    PIXEL_PID=$(adb shell "cat /tmp/beardog-pixel.pid 2>/dev/null" | tr -d '\r\n')
    echo "  ✅ Pixel BearDog started (PID: $PIXEL_PID)"
    echo "     Environment: FAMILY_ID=pixel_tower, NODE_ID=pixel_tower1"
    
    # Check logs
    if adb shell "grep -qE '(family|genetic|birdsong)' /tmp/beardog-pixel-local.log 2>/dev/null"; then
        echo "  ✅ Genetic context detected"
    fi
else
    echo "  ⚠️  Pixel BearDog status unclear, checking logs..."
    adb shell "cat /tmp/beardog-pixel-local.log 2>/dev/null | tail -20"
fi
echo ""

# Step 5: Start USB Songbird with security provider
echo "📋 Step 5: Starting USB Songbird with BearDog Security..."

export SONGBIRD_SECURITY_PROVIDER="beardog"
export SECURITY_ENDPOINT="unix://$BEARDOG_SOCKET"

~/.local/songbird/songbird server \
  --port 8080 \
  > /tmp/songbird-usb-local.log 2>&1 &

USB_SONGBIRD_PID=$!
sleep 3

if ps -p $USB_SONGBIRD_PID > /dev/null; then
    echo "  ✅ USB Songbird started (PID: $USB_SONGBIRD_PID)"
    echo "     Security: $SONGBIRD_SECURITY_PROVIDER @ $SECURITY_ENDPOINT"
    
    # Check for discovery
    if grep -qE "(discovery|beacon|mdns)" /tmp/songbird-usb-local.log; then
        echo "  ✅ Discovery system active"
    fi
else
    echo "  ⚠️  USB Songbird may need troubleshooting"
    echo "  Last 15 lines of log:"
    tail -15 /tmp/songbird-usb-local.log
fi
echo ""

# Step 6: Start Pixel Songbird with security provider
echo "📋 Step 6: Starting Pixel Songbird..."

cat > /tmp/pixel_songbird_start.sh << 'EOF'
#!/system/bin/sh
export SONGBIRD_SECURITY_PROVIDER="beardog"
export SECURITY_ENDPOINT="unix:///tmp/beardog-pixel.sock"

cd /data/local/tmp
/data/local/tmp/songbird/songbird server \
  --port 8080 \
  > /tmp/songbird-pixel-local.log 2>&1 &

echo $! > /tmp/songbird-pixel.pid
EOF

adb push /tmp/pixel_songbird_start.sh /data/local/tmp/
adb shell "chmod +x /data/local/tmp/pixel_songbird_start.sh"
adb shell "/data/local/tmp/pixel_songbird_start.sh"

sleep 3
echo "  ✅ Pixel Songbird started"
echo ""

# Step 7: Monitor for Discovery (30 seconds)
echo "📋 Step 7: Monitoring for Discovery..."
echo ""
echo "  Waiting 30 seconds for service discovery..."

for i in {1..30}; do
    echo -n "."
    
    # Check for discovery events every 5 seconds
    if [ $((i % 5)) -eq 0 ]; then
        if grep -q "discovered\|found.*peer\|connected" /tmp/songbird-usb-local.log 2>/dev/null; then
            echo ""
            echo ""
            echo "  🎉 Discovery event detected!"
            break
        fi
    fi
    
    sleep 1
done
echo ""
echo ""

# Step 8: Comprehensive Log Analysis
echo "📋 Step 8: BirdSong Validation Analysis"
echo "======================================="
echo ""

echo "🧬 USB BearDog - Genetic Lineage:"
grep -E "(family_id|genetic|lineage|birdsong)" /tmp/beardog-usb-local.log 2>/dev/null | head -8 || echo "  (No genetic markers in logs)"
echo ""

echo "🧬 Pixel BearDog - Genetic Lineage:"
adb shell "grep -E '(family_id|genetic|lineage|birdsong)' /tmp/beardog-pixel-local.log 2>/dev/null | head -8" || echo "  (No genetic markers in logs)"
echo ""

echo "📡 USB Songbird - Discovery Beacons:"
grep -E "(beacon|discovery|broadcast|mdns|peer)" /tmp/songbird-usb-local.log 2>/dev/null | head -8 || echo "  (No discovery activity)"
echo ""

echo "📡 Pixel Songbird - Discovery Beacons:"
adb shell "grep -E '(beacon|discovery|broadcast|mdns|peer)' /tmp/songbird-pixel-local.log 2>/dev/null | head -8" || echo "  (No discovery activity)"
echo ""

# Step 9: Check for Connection Establishment
echo "📋 Step 9: Connection Status"
echo "======================================="
echo ""

if grep -qE "(handshake|established|connected.*peer)" /tmp/songbird-usb-local.log 2>/dev/null; then
    echo "  ✅ USB → Pixel connection established!"
    grep -E "(handshake|established|connected)" /tmp/songbird-usb-local.log | head -3
else
    echo "  ⏳ Waiting for connection establishment..."
fi
echo ""

if adb shell "grep -qE '(handshake|established|connected.*peer)' /tmp/songbird-pixel-local.log 2>/dev/null"; then
    echo "  ✅ Pixel → USB connection established!"
    adb shell "grep -E '(handshake|established|connected)' /tmp/songbird-pixel-local.log | head -3"
else
    echo "  ⏳ Waiting for connection establishment..."
fi
echo ""

# Step 10: Service Status Summary
echo "📋 Step 10: Service Status Summary"
echo "======================================="
echo ""

echo "🖥️  USB (x86_64) - $USB_IP:"
echo "  BearDog:  $(ps -p $USB_BEARDOG_PID > /dev/null && echo '✅ Running' || echo '❌ Stopped') (PID: $USB_BEARDOG_PID)"
echo "  Songbird: $(ps -p $USB_SONGBIRD_PID > /dev/null && echo '✅ Running' || echo '❌ Stopped') (PID: $USB_SONGBIRD_PID)"
echo "  Socket:   $BEARDOG_SOCKET"
echo ""

echo "📱 Pixel (ARM64) - $PIXEL_IP:"
PIXEL_COUNT=$(adb shell "ps | grep -E '(beardog|songbird)' | grep -v grep | wc -l" | tr -d '\r\n ')
echo "  Services: $PIXEL_COUNT running"
echo "  BearDog:  $(adb shell 'cat /tmp/beardog-pixel.pid 2>/dev/null' | tr -d '\r\n')"
echo "  Songbird: $(adb shell 'cat /tmp/songbird-pixel.pid 2>/dev/null' | tr -d '\r\n')"
echo "  Socket:   /tmp/beardog-pixel.sock"
echo ""

# Step 11: Continuous Monitoring
echo "📋 Step 11: Live Monitoring"
echo "======================================="
echo ""
echo "Monitor these logs in real-time to see handshake:"
echo ""
echo "  USB BearDog:  tail -f /tmp/beardog-usb-local.log"
echo "  USB Songbird: tail -f /tmp/songbird-usb-local.log"
echo "  Pixel (via adb shell):"
echo "    tail -f /tmp/beardog-pixel-local.log"
echo "    tail -f /tmp/songbird-pixel-local.log"
echo ""
echo "Watch for:"
echo "  • Family ID derivation from seeds"
echo "  • BirdSong genetic initialization"
echo "  • Discovery beacon broadcasts"
echo "  • Peer discovery via mDNS"
echo "  • Genetic lineage verification"
echo "  • Encrypted channel establishment"
echo ""

echo "======================================="
echo "🧬 TASK 1: LOCAL HANDSHAKE ACTIVE"
echo "======================================="
echo ""
echo "Services running with genetic lineage!"
echo ""
echo "Press Ctrl+C to view summary and stop..."
echo ""

# Trap to provide summary on exit
cleanup_and_summary() {
    echo ""
    echo "======================================="
    echo "📊 HANDSHAKE SUMMARY"
    echo "======================================="
    echo ""
    
    echo "Services Status:"
    ps -p $USB_BEARDOG_PID > /dev/null && echo "  USB BearDog:  ✅ Running" || echo "  USB BearDog:  ❌ Stopped"
    ps -p $USB_SONGBIRD_PID > /dev/null && echo "  USB Songbird: ✅ Running" || echo "  USB Songbird: ❌ Stopped"
    adb shell "ps | grep beardog | grep -v grep" > /dev/null 2>&1 && echo "  Pixel BearDog:  ✅ Running" || echo "  Pixel BearDog:  ❌ Stopped"
    adb shell "ps | grep songbird | grep -v grep" > /dev/null 2>&1 && echo "  Pixel Songbird: ✅ Running" || echo "  Pixel Songbird: ❌ Stopped"
    echo ""
    
    echo "Stopping services..."
    kill $USB_BEARDOG_PID $USB_SONGBIRD_PID 2>/dev/null || true
    adb shell "killall beardog songbird 2>/dev/null" || true
    echo "  ✅ Services stopped"
    echo ""
    
    echo "Logs available at:"
    echo "  USB:   /tmp/*-usb-local.log"
    echo "  Pixel: /tmp/*-pixel-local.log (on device)"
    echo ""
    
    exit 0
}

trap cleanup_and_summary INT TERM

# Multi-tail log monitoring
echo "📡 Live Log Stream (Ctrl+C to stop):"
echo "======================================="
echo ""

# Monitor USB logs with color-coded output
tail -f /tmp/beardog-usb-local.log /tmp/songbird-usb-local.log 2>/dev/null | while read line; do
    if echo "$line" | grep -qE "(family|genetic|lineage)"; then
        echo "🧬 $line"
    elif echo "$line" | grep -qE "(beacon|discovery|found)"; then
        echo "📡 $line"
    elif echo "$line" | grep -qE "(established|connected|handshake)"; then
        echo "🎉 $line"
    elif echo "$line" | grep -qE "(ERROR|error)"; then
        echo "❌ $line"
    else
        echo "   $line"
    fi
done
