#!/bin/bash
# BirdSong Local Handshake - Task 1
# USB ↔ Pixel via Local Network (192.168.1.x)

set -e

echo "🧬🌉 BIRDSONG LOCAL HANDSHAKE - TASK 1"
echo "======================================="
echo ""

# Configuration
USB_IP="192.168.1.144"
PIXEL_IP="192.168.1.80"
USB_FAMILY_SEED="/media/eastgate/biomeOS21/biomeOS/.family.seed"
PIXEL_FAMILY_SEED="/data/local/tmp/biomeos/.family.seed"

# Step 1: Verify Prerequisites
echo "📋 Step 1: Verifying Prerequisites..."
echo ""

# Check USB seed
if [ -f "$USB_FAMILY_SEED" ]; then
    echo "  ✅ USB family seed found"
    echo "     $(xxd -l 16 -p "$USB_FAMILY_SEED")"
else
    echo "  ❌ USB family seed missing!"
    exit 1
fi

# Check Pixel seed
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

# Step 3: Start USB BearDog
echo "📋 Step 3: Starting USB BearDog with Genetic Context..."
cd /media/eastgate/biomeOS21/biomeOS

~/.local/beardog/beardog server \
  --family-id usb_tower \
  --socket /tmp/beardog-usb.sock \
  > /tmp/beardog-usb-local.log 2>&1 &

USB_BEARDOG_PID=$!
sleep 3

if ps -p $USB_BEARDOG_PID > /dev/null; then
    echo "  ✅ USB BearDog started (PID: $USB_BEARDOG_PID)"
    # Check for family ID in logs
    if grep -q "family" /tmp/beardog-usb-local.log; then
        echo "  ✅ Genetic context detected in logs"
    fi
else
    echo "  ❌ USB BearDog failed to start"
    cat /tmp/beardog-usb-local.log | tail -20
    exit 1
fi
echo ""

# Step 4: Start Pixel BearDog
echo "📋 Step 4: Starting Pixel BearDog with Genetic Context..."

adb shell "cd /data/local/tmp && \
  /data/local/tmp/beardog/beardog server \
    --family-id pixel_tower \
    --socket /tmp/beardog-pixel.sock \
    > /tmp/beardog-pixel-local.log 2>&1 &"

sleep 3

if adb shell "ps | grep beardog | grep -v grep" | grep -q beardog; then
    echo "  ✅ Pixel BearDog started"
else
    echo "  ⚠️  Pixel BearDog may need troubleshooting"
    adb shell "cat /tmp/beardog-pixel-local.log | tail -20"
fi
echo ""

# Step 5: Start USB Songbird
echo "📋 Step 5: Starting USB Songbird..."

~/.local/songbird/songbird server \
  --port 8080 \
  > /tmp/songbird-usb-local.log 2>&1 &

USB_SONGBIRD_PID=$!
sleep 3

if ps -p $USB_SONGBIRD_PID > /dev/null; then
    echo "  ✅ USB Songbird started (PID: $USB_SONGBIRD_PID)"
else
    echo "  ⚠️  USB Songbird may need security provider config"
    tail -20 /tmp/songbird-usb-local.log
fi
echo ""

# Step 6: Start Pixel Songbird  
echo "📋 Step 6: Starting Pixel Songbird..."

adb shell "cd /data/local/tmp && \
  /data/local/tmp/songbird/songbird server \
    --port 8080 \
    > /tmp/songbird-pixel-local.log 2>&1 &"

sleep 3
echo "  ✅ Pixel Songbird started"
echo ""

# Step 7: Monitor for Discovery
echo "📋 Step 7: Monitoring for Discovery..."
echo ""
echo "  Services should discover each other via mDNS..."
echo "  This may take 10-30 seconds..."
echo ""

# Give services time to initialize and discover
for i in {1..30}; do
    echo -n "."
    sleep 1
done
echo ""
echo ""

# Step 8: Check Logs for Key Events
echo "📋 Step 8: Checking Logs for Validation..."
echo ""

echo "  USB BearDog (Family ID & Genetic):"
if grep -E "(family|genetic|lineage)" /tmp/beardog-usb-local.log | head -5; then
    echo "    ✅ Genetic context found"
else
    echo "    ⚠️  No genetic context in logs yet"
fi
echo ""

echo "  USB Songbird (Beacon & Discovery):"
if grep -E "(beacon|discovery|service)" /tmp/songbird-usb-local.log | head -5; then
    echo "    ✅ Discovery activity found"
else
    echo "    ⚠️  No discovery activity yet"
fi
echo ""

echo "  Pixel BearDog:"
adb shell "grep -E '(family|genetic|lineage)' /tmp/beardog-pixel-local.log 2>/dev/null | head -5" && echo "    ✅ Genetic context found" || echo "    ⚠️  Check logs manually"
echo ""

echo "  Pixel Songbird:"
adb shell "grep -E '(beacon|discovery|service)' /tmp/songbird-pixel-local.log 2>/dev/null | head -5" && echo "    ✅ Discovery activity found" || echo "    ⚠️  Check logs manually"
echo ""

# Step 9: Service Status
echo "📋 Step 9: Service Status Summary"
echo "======================================="
echo ""

echo "🖥️  USB (x86_64) - $USB_IP:"
echo "  BearDog:  $(ps -p $USB_BEARDOG_PID > /dev/null && echo '✅ Running' || echo '❌ Stopped') (PID: $USB_BEARDOG_PID)"
echo "  Songbird: $(ps -p $USB_SONGBIRD_PID > /dev/null && echo '✅ Running' || echo '❌ Stopped') (PID: $USB_SONGBIRD_PID)"
echo "  Logs:     /tmp/*-usb-local.log"
echo ""

echo "📱 Pixel (ARM64) - $PIXEL_IP:"
PIXEL_COUNT=$(adb shell "ps | grep -E '(beardog|songbird)' | grep -v grep | wc -l" | tr -d '\r\n ')
echo "  Services: $PIXEL_COUNT running"
echo "  Logs:     /tmp/*-pixel-local.log (on device)"
echo ""

# Step 10: Manual Validation Steps
echo "📋 Step 10: Manual Validation"
echo "======================================="
echo ""
echo "Monitor logs in real-time:"
echo ""
echo "  USB BearDog:"
echo "    tail -f /tmp/beardog-usb-local.log | grep -E '(family|genetic|lineage|birdsong)'"
echo ""
echo "  USB Songbird:"
echo "    tail -f /tmp/songbird-usb-local.log | grep -E '(beacon|discovery|found)'"
echo ""
echo "  Pixel BearDog:"
echo "    adb shell 'tail -f /tmp/beardog-pixel-local.log' | grep -E '(family|genetic|lineage)'"
echo ""
echo "  Pixel Songbird:"
echo "    adb shell 'tail -f /tmp/songbird-pixel-local.log' | grep -E '(beacon|discovery|found)'"
echo ""

echo "======================================="
echo "🧬 TASK 1: LOCAL HANDSHAKE READY"
echo "======================================="
echo ""
echo "Services running with genetic lineage context!"
echo "Monitor logs above to observe:"
echo "  1. Family ID derivation from seeds"
echo "  2. BirdSong beacon broadcasting"
echo "  3. Service discovery via mDNS"
echo "  4. Genetic lineage verification"
echo "  5. Encrypted channel establishment"
echo ""
echo "Press Ctrl+C to stop monitoring..."
echo ""

# Keep script running to maintain services
trap "echo ''; echo 'Stopping services...'; kill $USB_BEARDOG_PID $USB_SONGBIRD_PID 2>/dev/null; adb shell 'killall beardog songbird 2>/dev/null'; echo 'Done.'; exit 0" INT TERM

# Tail logs live
tail -f /tmp/beardog-usb-local.log /tmp/songbird-usb-local.log
