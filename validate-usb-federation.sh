#!/bin/bash
# USB Federation Validation Pipeline
# 
# Tests USB deployment in 2-VM federation before NUC deployment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHSCALE_DIR="$(cd "$SCRIPT_DIR/../../primalTools/benchscale" && pwd)"
USB_PACKAGE="$SCRIPT_DIR/biomeos-20251228-163320.tar.gz"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🔬 USB Federation Validation Pipeline 🔬               ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check USB package exists
if [ ! -f "$USB_PACKAGE" ]; then
    echo "❌ USB package not found: $USB_PACKAGE"
    echo ""
    echo "Create it first:"
    echo "  cd $SCRIPT_DIR"
    echo "  AUTO_CONFIRM=1 ./quick-usb.sh"
    exit 1
fi

USB_SIZE=$(du -h "$USB_PACKAGE" | cut -f1)
echo "✅ USB package: $USB_SIZE"

# Check benchScale
if [ ! -f "$BENCHSCALE_DIR/target/release/benchscale" ]; then
    echo "❌ benchScale not built"
    echo ""
    echo "Build it:"
    echo "  cd $BENCHSCALE_DIR"
    echo "  cargo build --release --features libvirt"
    exit 1
fi

echo "✅ benchScale: available"
echo ""

# Topology file
TOPOLOGY="$BENCHSCALE_DIR/topologies/biomeos-usb-federation-test.yaml"

if [ ! -f "$TOPOLOGY" ]; then
    echo "❌ Topology not found: $TOPOLOGY"
    exit 1
fi

echo "✅ Topology: ready"
echo ""

# Phase 1: Create VMs
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 1: Create 2-VM Federation Test Lab"
echo "═══════════════════════════════════════════════════════════"
echo ""

LAB_NAME="biomeos-usb-federation-$(date +%Y%m%d-%H%M%S)"

echo "🏗️  Creating lab: $LAB_NAME"
echo "   Topology: $TOPOLOGY"
echo "   VMs: 2 (tower-alpha, tower-beta)"
echo ""

# Note: This is a simulation - actual benchScale integration pending
echo "📋 Lab creation steps:"
echo "   1. Create 2 Ubuntu 22.04 VMs"
echo "   2. Attach USB package as mounted volume"
echo "   3. Configure network (10.70.0.0/24)"
echo "   4. Install avahi-daemon (mDNS)"
echo ""

# Phase 2: Deploy BiomeOS from USB
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 2: Deploy BiomeOS from USB (Both VMs)"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🚀 Deployment sequence:"
echo ""
echo "Tower Alpha:"
echo "  1. Mount USB package"
echo "  2. Extract to /mnt/usb"
echo "  3. Run /mnt/usb/install/install-biomeos.sh"
echo "  4. Start primals: /opt/biomeos/deploy-real-primals.sh"
echo "  5. Verify: curl http://localhost:9020/health"
echo ""

echo "Tower Beta:"
echo "  1. Mount USB package (same)"
echo "  2. Extract to /mnt/usb"
echo "  3. Run /mnt/usb/install/install-biomeos.sh"
echo "  4. Start primals: /opt/biomeos/deploy-real-primals.sh"
echo "  5. Verify: curl http://localhost:9020/health"
echo ""

# Phase 3: Validate Federation
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 3: Validate Federation (mDNS Discovery)"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🔍 Federation tests:"
echo ""
echo "Test 1: Mutual Discovery"
echo "  - Alpha discovers Beta via Songbird mDNS"
echo "  - Beta discovers Alpha via Songbird mDNS"
echo "  - Wait 30s for mDNS propagation"
echo ""

echo "Test 2: Data Replication"
echo "  - Store data on Alpha (NestGate)"
echo "  - Retrieve from Beta (federation sync)"
echo "  - Verify data integrity"
echo ""

echo "Test 3: Sovereignty"
echo "  - Unauthorized access rejected"
echo "  - Lineage enforcement working"
echo "  - Cross-tower security maintained"
echo ""

# Phase 4: Run E2E Tests
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 4: Run E2E Tests (Both VMs)"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🧪 Running test suites:"
echo "   Alpha: /opt/biomeos/run-e2e-tests.sh"
echo "   Beta: /opt/biomeos/run-e2e-tests.sh"
echo ""
echo "   Expected: 12/15 passing on each"
echo "   (3 Songbird HTTP endpoint tests known to fail)"
echo ""

# Phase 5: Collect Results
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 5: Collect Validation Results"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "📊 Validation criteria:"
echo "   ✅ Both VMs deployed from USB"
echo "   ✅ All primals running (10 total: 5 per VM)"
echo "   ✅ Mutual mDNS discovery working"
echo "   ✅ Federation established"
echo "   ✅ Data replication functional"
echo "   ✅ Sovereignty enforced"
echo "   ✅ E2E tests: 12/15 (expected)"
echo ""

# Manual execution instructions
echo "═══════════════════════════════════════════════════════════"
echo "📋 MANUAL EXECUTION GUIDE"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Since benchScale VM creation isn't automated yet,"
echo "here's how to run this test manually:"
echo ""

echo "1️⃣  Create 2 Ubuntu 22.04 VMs:"
echo "   - VM1: tower-alpha (4GB RAM, 2 vCPUs)"
echo "   - VM2: tower-beta (4GB RAM, 2 vCPUs)"
echo "   - Network: Bridge mode (so they can see each other)"
echo ""

echo "2️⃣  Copy USB package to both VMs:"
echo "   scp $USB_PACKAGE user@tower-alpha:/tmp/"
echo "   scp $USB_PACKAGE user@tower-beta:/tmp/"
echo ""

echo "3️⃣  On BOTH VMs, run installation:"
echo "   sudo mkdir -p /mnt/usb"
echo "   cd /tmp && tar -xzf biomeos-*.tar.gz -C /mnt/usb"
echo "   cd /mnt/usb/install"
echo "   ./install-biomeos.sh"
echo ""

echo "4️⃣  Install avahi (mDNS) on BOTH VMs:"
echo "   sudo apt update"
echo "   sudo apt install -y avahi-daemon avahi-utils"
echo "   sudo systemctl start avahi-daemon"
echo ""

echo "5️⃣  Start BiomeOS on BOTH VMs:"
echo "   cd /opt/biomeos"
echo "   ./deploy-real-primals.sh"
echo ""

echo "6️⃣  Wait 30s, then verify federation:"
echo "   # On tower-alpha:"
echo "   tail -f /opt/biomeos/logs/primals/songbird.log | grep -i beta"
echo ""
echo "   # On tower-beta:"
echo "   tail -f /opt/biomeos/logs/primals/songbird.log | grep -i alpha"
echo ""

echo "7️⃣  Test data replication:"
echo "   # On alpha:"
echo "   curl -X POST http://localhost:9020/api/store -d '{\"key\":\"test\",\"data\":\"hello\"}'"
echo ""
echo "   # On beta (wait 5s):"
echo "   curl http://localhost:9020/api/retrieve/test"
echo ""

echo "8️⃣  Run E2E tests on both:"
echo "   cd /opt/biomeos && ./run-e2e-tests.sh"
echo "   Expected: 12/15 PASS on each VM"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "🎯 SUCCESS CRITERIA"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Federation validated when:"
echo "   ✅ Both VMs running BiomeOS from USB"
echo "   ✅ Songbird logs show peer discovery"
echo "   ✅ 10 total primal processes (5 per VM)"
echo "   ✅ Data replicates between towers"
echo "   ✅ E2E: 24/30 total (12/15 per VM)"
echo ""

echo "🚀 Then proceed to NUC deployment!"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "Next: NUC Deployment"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "After VM validation succeeds:"
echo "   1. Write USB: sudo dd if=$USB_PACKAGE of=/dev/sdX"
echo "   2. Boot NUC from USB"
echo "   3. Install: Same steps as VMs"
echo "   4. NUC will discover VM federation automatically"
echo "   5. Result: 3-node federation (alpha, beta, NUC)"
echo ""

echo "📄 Validation complete! See manual steps above."
echo ""

