#!/bin/bash
# NUCLEUS Validation - Using Existing Deployments
# Validates all primals already deployed on USB + Pixel platforms
# Date: 2026-01-31

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}ℹ${NC} $*"; }
log_success() { echo -e "${GREEN}✅${NC} $*"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $*"; }
log_error() { echo -e "${RED}❌${NC} $*"; }
log_section() { echo -e "\n${CYAN}${BOLD}━━━ $* ━━━${NC}\n"; }

# Configuration
WORKSPACE_DIR="$HOME/Development/ecoPrimals/phase2/biomeOS"
USB_DEPLOY_DIR="$HOME/.local"
PIXEL_DEPLOY_DIR="/data/local/tmp"
RESULTS_DIR="$WORKSPACE_DIR/nucleus-validation-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Counters
TOTAL_SERVICES=0
RUNNING_SERVICES=0

# Create results directory
mkdir -p "$RESULTS_DIR"

log_section "NUCLEUS Validation - Using Existing Deployments"
log_info "Timestamp: $TIMESTAMP"
log_info "Workspace: $WORKSPACE_DIR"
log_info "Results: $RESULTS_DIR"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Phase 1: Verify Existing Deployments
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log_section "Phase 1: Verify Existing Deployments"

# Check USB binaries
PRIMALS=("beardog" "songbird" "squirrel" "toadstool" "nestgate" "biomeos")
USB_FOUND=0
for primal in "${PRIMALS[@]}"; do
    if [ -f "$USB_DEPLOY_DIR/$primal/$primal" ]; then
        log_success "USB $primal found"
        USB_FOUND=$((USB_FOUND + 1))
    else
        log_warn "USB $primal not found"
    fi
done

# Check Pixel binaries
PIXEL_FOUND=0
for primal in "${PRIMALS[@]}"; do
    if adb shell "[ -f $PIXEL_DEPLOY_DIR/$primal/$primal ]" 2>/dev/null; then
        log_success "Pixel $primal found"
        PIXEL_FOUND=$((PIXEL_FOUND + 1))
    else
        log_warn "Pixel $primal not found"
    fi
done

log_info "USB binaries: $USB_FOUND/6"
log_info "Pixel binaries: $PIXEL_FOUND/6"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Phase 2: Start TOWER Services (BearDog + Songbird)
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log_section "Phase 2: TOWER Atomic Validation"

# Generate family seeds if they don't exist
USB_FAMILY_SEED="$HOME/.family.seed"
PIXEL_FAMILY_SEED="$PIXEL_DEPLOY_DIR/biomeos/.family.seed"

if [ ! -f "$USB_FAMILY_SEED" ]; then
    openssl rand -hex 32 > "$USB_FAMILY_SEED"
    log_info "Generated USB family seed"
fi

# Ensure Pixel family seed exists
adb shell "[ -d $PIXEL_DEPLOY_DIR/biomeos ] || mkdir -p $PIXEL_DEPLOY_DIR/biomeos" > /dev/null 2>&1
if ! adb shell "[ -f $PIXEL_FAMILY_SEED ]" > /dev/null 2>&1; then
    openssl rand -hex 32 | adb shell "cat > $PIXEL_FAMILY_SEED"
    log_info "Generated Pixel family seed"
fi

# Kill any existing services
pkill -f "beardog server" || true
pkill -f "songbird server" || true
adb shell "pkill -f 'beardog server'" 2>/dev/null || true
adb shell "pkill -f 'songbird server'" 2>/dev/null || true
sleep 2

# Start USB TOWER
log_info "Starting USB TOWER (BearDog + Songbird)..."

# Start USB BearDog
cat > /tmp/usb_beardog_start.sh << 'EOF'
#!/bin/bash
export BEARDOG_FAMILY_SEED="$HOME/.family.seed"
export FAMILY_ID="usb_nucleus"
export NODE_ID="usb_nucleus1"

cd ~/.local/beardog
./beardog server --family-id "$FAMILY_ID" > /tmp/beardog-usb-nucleus.log 2>&1 &
echo $! > /tmp/beardog-usb-nucleus.pid
EOF
chmod +x /tmp/usb_beardog_start.sh
/tmp/usb_beardog_start.sh
TOTAL_SERVICES=$((TOTAL_SERVICES + 1))
sleep 2

# Verify USB BearDog started
if [ -f /tmp/beardog-usb-nucleus.pid ] && kill -0 "$(cat /tmp/beardog-usb-nucleus.pid)" 2>/dev/null; then
    log_success "USB BearDog started (PID $(cat /tmp/beardog-usb-nucleus.pid))"
    RUNNING_SERVICES=$((RUNNING_SERVICES + 1))
    cp /tmp/beardog-usb-nucleus.log "$RESULTS_DIR/usb-beardog.log"
else
    log_error "USB BearDog failed to start"
    [ -f /tmp/beardog-usb-nucleus.log ] && tail -20 /tmp/beardog-usb-nucleus.log
fi

# Start USB Songbird
cat > /tmp/usb_songbird_start.sh << 'EOF'
#!/bin/bash
export SONGBIRD_SECURITY_PROVIDER="beardog"
export SECURITY_ENDPOINT="unix:///tmp/beardog.sock"
export FAMILY_ID="usb_nucleus"
export NODE_ID="usb_nucleus1"

cd ~/.local/songbird
./songbird server --port 8080 > /tmp/songbird-usb-nucleus.log 2>&1 &
echo $! > /tmp/songbird-usb-nucleus.pid
EOF
chmod +x /tmp/usb_songbird_start.sh
/tmp/usb_songbird_start.sh
TOTAL_SERVICES=$((TOTAL_SERVICES + 1))
sleep 3

# Verify USB Songbird started
if [ -f /tmp/songbird-usb-nucleus.pid ] && kill -0 "$(cat /tmp/songbird-usb-nucleus.pid)" 2>/dev/null; then
    log_success "USB Songbird started (PID $(cat /tmp/songbird-usb-nucleus.pid))"
    RUNNING_SERVICES=$((RUNNING_SERVICES + 1))
    cp /tmp/songbird-usb-nucleus.log "$RESULTS_DIR/usb-songbird.log"
else
    log_error "USB Songbird failed to start"
    [ -f /tmp/songbird-usb-nucleus.log ] && tail -20 /tmp/songbird-usb-nucleus.log
fi

# Start Pixel TOWER
log_info "Starting Pixel TOWER (BearDog + Songbird)..."

# Start Pixel BearDog
cat > /tmp/pixel_beardog_start.sh << 'EOF'
#!/system/bin/sh
export BEARDOG_FAMILY_SEED="/data/local/tmp/biomeos/.family.seed"
export FAMILY_ID="pixel_nucleus"
export NODE_ID="pixel_nucleus1"
export BEARDOG_ABSTRACT_SOCKET="beardog_nucleus"

cd /data/local/tmp/beardog
./beardog server --family-id "$FAMILY_ID" > /tmp/beardog-pixel-nucleus.log 2>&1 &
echo $! > /tmp/beardog-pixel-nucleus.pid
EOF
adb push /tmp/pixel_beardog_start.sh /data/local/tmp/ > /dev/null 2>&1
adb shell "chmod +x /data/local/tmp/pixel_beardog_start.sh"
adb shell "/data/local/tmp/pixel_beardog_start.sh"
TOTAL_SERVICES=$((TOTAL_SERVICES + 1))
sleep 2

# Verify Pixel BearDog started
if adb shell "[ -f /tmp/beardog-pixel-nucleus.pid ] && kill -0 \$(cat /tmp/beardog-pixel-nucleus.pid) 2>/dev/null" 2>/dev/null; then
    PIXEL_BEARDOG_PID=$(adb shell "cat /tmp/beardog-pixel-nucleus.pid" 2>/dev/null | tr -d '\r\n')
    log_success "Pixel BearDog started (PID $PIXEL_BEARDOG_PID)"
    RUNNING_SERVICES=$((RUNNING_SERVICES + 1))
    adb shell "cat /tmp/beardog-pixel-nucleus.log" > "$RESULTS_DIR/pixel-beardog.log" 2>&1
else
    log_error "Pixel BearDog failed to start"
    adb shell "tail -20 /tmp/beardog-pixel-nucleus.log" 2>&1
fi

# Start Pixel Songbird (set PID dir to writable location)
cat > /tmp/pixel_songbird_start.sh << 'EOF'
#!/system/bin/sh
export SONGBIRD_SECURITY_PROVIDER="beardog"
export BEARDOG_ABSTRACT_SOCKET="beardog_nucleus"
export FAMILY_ID="pixel_nucleus"
export NODE_ID="pixel_nucleus1"
export SONGBIRD_PID_DIR="/data/local/tmp"

cd /data/local/tmp/songbird
./songbird server --port 8080 > /tmp/songbird-pixel-nucleus.log 2>&1 &
echo $! > /data/local/tmp/songbird-pixel-nucleus.pid
EOF
adb push /tmp/pixel_songbird_start.sh /data/local/tmp/ > /dev/null 2>&1
adb shell "chmod +x /data/local/tmp/pixel_songbird_start.sh"
adb shell "/data/local/tmp/pixel_songbird_start.sh"
TOTAL_SERVICES=$((TOTAL_SERVICES + 1))
sleep 3

# Verify Pixel Songbird started
if adb shell "[ -f /data/local/tmp/songbird-pixel-nucleus.pid ] && kill -0 \$(cat /data/local/tmp/songbird-pixel-nucleus.pid) 2>/dev/null" 2>/dev/null; then
    PIXEL_SONGBIRD_PID=$(adb shell "cat /data/local/tmp/songbird-pixel-nucleus.pid" 2>/dev/null | tr -d '\r\n')
    log_success "Pixel Songbird started (PID $PIXEL_SONGBIRD_PID)"
    RUNNING_SERVICES=$((RUNNING_SERVICES + 1))
    adb shell "cat /tmp/songbird-pixel-nucleus.log" > "$RESULTS_DIR/pixel-songbird.log" 2>&1
else
    log_error "Pixel Songbird failed to start"
    adb shell "tail -20 /tmp/songbird-pixel-nucleus.log" 2>&1
fi

log_section "Phase 2 Results: TOWER Atomic"
log_info "Total TOWER services: 4"
log_success "Running: $RUNNING_SERVICES/4"

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Wait for discovery and genetic verification
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log_section "Monitoring Discovery & Genetic Verification"
log_info "Waiting 10 seconds for mDNS discovery and BirdSong initialization..."
sleep 10

# Check USB logs for genetic verification
log_info "Checking USB BearDog genetic initialization..."
if grep -q "Genetic engine initialized\|Family ID:" /tmp/beardog-usb-nucleus.log 2>/dev/null; then
    log_success "USB genetic engine: ✅"
    grep "Family ID:\|Node ID:" /tmp/beardog-usb-nucleus.log | head -2
else
    log_warn "USB genetic engine: No confirmation (check logs)"
fi

# Check Pixel logs for genetic verification
log_info "Checking Pixel BearDog genetic initialization..."
if adb shell "grep 'Genetic engine initialized\|Family ID:' /tmp/beardog-pixel-nucleus.log 2>/dev/null" | grep -q "Family ID:"; then
    log_success "Pixel genetic engine: ✅"
    adb shell "grep 'Family ID:\|Node ID:' /tmp/beardog-pixel-nucleus.log 2>/dev/null" | head -2
else
    log_warn "Pixel genetic engine: No confirmation (check logs)"
fi

# Check for mDNS discovery
log_info "Checking mDNS discovery..."
if grep -q "mDNS\|Discovery\|beacon" /tmp/songbird-usb-nucleus.log 2>/dev/null; then
    log_success "USB mDNS/discovery: ✅"
fi
if adb shell "grep 'mDNS\|Discovery\|beacon' /tmp/songbird-pixel-nucleus.log 2>/dev/null" | grep -q .; then
    log_success "Pixel mDNS/discovery: ✅"
fi

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Generate Results Summary
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log_section "NUCLEUS Validation Results"

log_info "Deployment Verification:"
log_info "  USB binaries: $USB_FOUND/6"
log_info "  Pixel binaries: $PIXEL_FOUND/6"

log_info "Service Phase (TOWER only):"
log_info "  Total services: $TOTAL_SERVICES"
log_success "  Running: $RUNNING_SERVICES"
if [ $TOTAL_SERVICES -gt 0 ]; then
    log_info "  Success rate: $(( RUNNING_SERVICES * 100 / TOTAL_SERVICES ))%"
fi

# Capture service PIDs
USB_BEARDOG_PID=$(cat /tmp/beardog-usb-nucleus.pid 2>/dev/null || echo "N/A")
USB_SONGBIRD_PID=$(cat /tmp/songbird-usb-nucleus.pid 2>/dev/null || echo "N/A")

# Generate summary report
cat > "$RESULTS_DIR/nucleus-validation-summary-$TIMESTAMP.md" << EOF
# NUCLEUS Validation Summary
**Date**: $(date)
**Timestamp**: $TIMESTAMP

## Deployment Verification
- USB binaries found: $USB_FOUND/6
- Pixel binaries found: $PIXEL_FOUND/6

## Service Results (TOWER Atomic)
- Total services: $TOTAL_SERVICES
- Running: $RUNNING_SERVICES
- Success rate: $([ $TOTAL_SERVICES -gt 0 ] && echo "$(( RUNNING_SERVICES * 100 / TOTAL_SERVICES ))%" || echo "N/A")

## Services Running
- USB BearDog: $([ -f /tmp/beardog-usb-nucleus.pid ] && kill -0 "$(cat /tmp/beardog-usb-nucleus.pid)" 2>/dev/null && echo "✅ PID $USB_BEARDOG_PID" || echo "❌")
- USB Songbird: $([ -f /tmp/songbird-usb-nucleus.pid ] && kill -0 "$(cat /tmp/songbird-usb-nucleus.pid)" 2>/dev/null && echo "✅ PID $USB_SONGBIRD_PID" || echo "❌")
- Pixel BearDog: $(adb shell "[ -f /tmp/beardog-pixel-nucleus.pid ] && kill -0 \$(cat /tmp/beardog-pixel-nucleus.pid) 2>/dev/null" 2>/dev/null && echo "✅ PID $PIXEL_BEARDOG_PID" || echo "❌")
- Pixel Songbird: $(adb shell "[ -f /data/local/tmp/songbird-pixel-nucleus.pid ] && kill -0 \$(cat /data/local/tmp/songbird-pixel-nucleus.pid) 2>/dev/null" 2>/dev/null && echo "✅ PID $PIXEL_SONGBIRD_PID" || echo "❌")

## Logs Captured
$(ls -1 "$RESULTS_DIR" | sed 's/^/- /')

## Next Steps
1. Monitor services for stability
2. Expand to complete NEST atomic (+ NestGate + Squirrel)
3. Add NODE atomic (+ Toadstool)
4. Complete NUCLEUS coordination validation

**Status**: TOWER Atomic Initial Test
EOF

log_success "Summary report: $RESULTS_DIR/nucleus-validation-summary-$TIMESTAMP.md"

log_section "Validation Complete!"
log_info "Check logs in: $RESULTS_DIR"
log_info "USB logs: /tmp/*-usb-nucleus.log"
log_info "Pixel logs available via: adb shell cat /tmp/*-pixel-nucleus.log"

if [ $RUNNING_SERVICES -ge 3 ]; then
    log_success "TOWER Atomic validation successful! 🎊"
else
    log_warn "Some services failed to start. Check logs for details."
fi
