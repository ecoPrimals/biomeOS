#!/bin/bash
# Validate BiomeOS Federation - Using SSH keys

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Load VM IPs
if [ ! -f "$SCRIPT_DIR/vm-ips.env" ]; then
    echo "❌ VM IPs not found. Run create-test-vms-fixed.sh first."
    exit 1
fi

source "$SCRIPT_DIR/vm-ips.env"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  ✅ Validating BiomeOS Federation ✅                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Phase 1: Check Primals
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 1: Check Primal Processes"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "On tower-alpha ($ALPHA_IP):"
ALPHA_PRIMALS=$(ssh -o StrictHostKeyChecking=no biomeos@$ALPHA_IP \
    "pgrep -f 'nestgate|songbird|beardog|toadstool' | wc -l" || echo "0")
echo "  Primals running: $ALPHA_PRIMALS"

echo ""
echo "On tower-beta ($BETA_IP):"
BETA_PRIMALS=$(ssh -o StrictHostKeyChecking=no biomeos@$BETA_IP \
    "pgrep -f 'nestgate|songbird|beardog|toadstool' | wc -l" || echo "0")
echo "  Primals running: $BETA_PRIMALS"

echo ""

# Phase 2: Check mDNS Discovery
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 2: Check mDNS/Federation Discovery"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "⏳ Waiting for mDNS discovery (30s)..."
sleep 30

echo ""
echo "Checking Songbird logs on tower-alpha:"
ssh -o StrictHostKeyChecking=no biomeos@$ALPHA_IP \
    "sudo tail -20 /opt/biomeos/logs/primals/songbird.log 2>/dev/null | grep -i 'peer\|discover' || echo '  (No peer discovery logs yet)'"

echo ""
echo "Checking Songbird logs on tower-beta:"
ssh -o StrictHostKeyChecking=no biomeos@$BETA_IP \
    "sudo tail -20 /opt/biomeos/logs/primals/songbird.log 2>/dev/null | grep -i 'peer\|discover' || echo '  (No peer discovery logs yet)'"

echo ""

# Phase 3: Run E2E Tests
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 3: Run E2E Tests"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🧪 Running tests on tower-alpha..."
ssh -o StrictHostKeyChecking=no biomeos@$ALPHA_IP \
    "cd /opt/biomeos && sudo ./run-e2e-tests.sh 2>&1" | tee /tmp/alpha-results.log
ALPHA_PASS=$(grep -c "PASS" /tmp/alpha-results.log || echo "0")

echo ""
echo "🧪 Running tests on tower-beta..."
ssh -o StrictHostKeyChecking=no biomeos@$BETA_IP \
    "cd /opt/biomeos && sudo ./run-e2e-tests.sh 2>&1" | tee /tmp/beta-results.log
BETA_PASS=$(grep -c "PASS" /tmp/beta-results.log || echo "0")

echo ""

# Results
echo "═══════════════════════════════════════════════════════════"
echo "✅ VALIDATION COMPLETE"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📊 Results:"
echo "  tower-alpha: $ALPHA_PRIMALS primals, $ALPHA_PASS tests passed"
echo "  tower-beta:  $BETA_PRIMALS primals, $BETA_PASS tests passed"
echo "  Total: $((ALPHA_PRIMALS + BETA_PRIMALS)) primals, $((ALPHA_PASS + BETA_PASS)) tests passed"
echo ""

# Success criteria
SUCCESS=true

if [ "$ALPHA_PRIMALS" -lt 4 ] || [ "$BETA_PRIMALS" -lt 4 ]; then
    echo "⚠️  Less than 4 primals per VM"
    SUCCESS=false
fi

if [ "$ALPHA_PASS" -lt 10 ] || [ "$BETA_PASS" -lt 10 ]; then
    echo "⚠️  Less than 10 tests passing per VM"
    SUCCESS=false
fi

if [ "$SUCCESS" = true ]; then
    echo "🎉 SUCCESS! Federation validated!"
    echo ""
    echo "✅ Ready for NUC deployment!"
    echo ""
    echo "Next steps:"
    echo "  1. Write USB: AUTO_CONFIRM=1 ./quick-usb.sh"
    echo "  2. Boot NUC from USB"
    echo "  3. Install BiomeOS (same as VMs)"
    echo "  4. NUC will auto-discover both VMs"
    echo "  5. Result: 3-node federation!"
else
    echo "⚠️  Some criteria not met, but system may still work"
    echo ""
    echo "Debug:"
    echo "  ssh biomeos@$ALPHA_IP"
    echo "  ssh biomeos@$BETA_IP"
    echo "  Check logs: sudo tail -f /opt/biomeos/logs/primals/*.log"
fi

echo ""

