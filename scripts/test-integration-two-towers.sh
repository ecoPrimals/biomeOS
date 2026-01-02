#!/bin/bash
# Two-Tower Integration Test - Generic Trust API
# Tests: Same family auto-trust with genetic lineage

set -euo pipefail

echo "══════════════════════════════════════════════════════════════════════════"
echo "🧪 Two-Tower Integration Test - Generic Trust API"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

# Stop all services
echo "🧹 Stopping all services..."
pkill -9 -f "beardog" 2>/dev/null || true
pkill -9 -f "songbird" 2>/dev/null || true
sleep 3
echo "   ✅ All stopped"
echo ""

# Deploy fresh from USB
echo "📦 Deploying fresh package from USB..."
rm -rf ~/biomeOS-Deploy
mkdir -p ~/biomeOS-Deploy
cp -r /media/eastgate/BEA6-BBCE/biomeOS-LAN-Deploy/* ~/biomeOS-Deploy/
cd ~/biomeOS-Deploy
chmod +x scripts/*.sh primals/* 2>/dev/null || true
echo "   ✅ Package ready"
echo ""

# Start deployment
echo "🚀 Starting Tower 1 (Local)..."
echo "─────────────────────────────────────────────────────────────────────────"
./scripts/auto-deploy-v6.sh > /tmp/tower1-deploy.log 2>&1 &
DEPLOY_PID=$!
echo "   Deployment started (PID: $DEPLOY_PID)"
echo ""

echo "⏳ Waiting for services to initialize (30 seconds)..."
sleep 30

echo ""
echo "══════════════════════════════════════════════════════════════════════════"
echo "📊 Verification Phase"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

echo "1️⃣  BearDog Health Check:"
echo "─────────────────────────────────────────────────────────────────────────"
BEARDOG_HEALTH=$(curl -s http://localhost:9000/api/v1/health 2>/dev/null || echo '{"error": "not responding"}')
echo "$BEARDOG_HEALTH" | python3 -m json.tool || echo "   ❌ Not responding"
echo ""

echo "2️⃣  BearDog Identity (Generic Trust API):"
echo "─────────────────────────────────────────────────────────────────────────"
BEARDOG_IDENTITY=$(curl -s http://localhost:9000/api/v1/trust/identity 2>/dev/null || echo '{"error": "not responding"}')
echo "$BEARDOG_IDENTITY" | python3 -m json.tool || echo "   ❌ Not responding"

# Extract key fields
if echo "$BEARDOG_IDENTITY" | grep -q "encryption_tag"; then
    ENCRYPTION_TAG=$(echo "$BEARDOG_IDENTITY" | jq -r '.data.encryption_tag // .encryption_tag // "unknown"')
    FAMILY_ID=$(echo "$BEARDOG_IDENTITY" | jq -r '.data.family_id // .family_id // "unknown"')
    echo ""
    echo "   ✅ Identity Retrieved:"
    echo "      Encryption Tag: $ENCRYPTION_TAG"
    echo "      Family ID:      $FAMILY_ID"
fi
echo ""

echo "3️⃣  Songbird Process:"
echo "─────────────────────────────────────────────────────────────────────────"
SONGBIRD_PID=$(ps aux | grep songbird-orchestrator | grep -v grep | awk '{print $2}' | head -1)
if [ -n "$SONGBIRD_PID" ]; then
    echo "   ✅ Running (PID: $SONGBIRD_PID)"
else
    echo "   ❌ Not running"
fi
echo ""

echo "4️⃣  BearDog Process:"
echo "─────────────────────────────────────────────────────────────────────────"
BEARDOG_PID=$(ps aux | grep beardog-server | grep -v grep | awk '{print $2}' | head -1)
if [ -n "$BEARDOG_PID" ]; then
    echo "   ✅ Running (PID: $BEARDOG_PID)"
else
    echo "   ❌ Not running"
fi
echo ""

echo "5️⃣  Ports Listening:"
echo "─────────────────────────────────────────────────────────────────────────"
PORT_9000=$(ss -tulpn 2>/dev/null | grep ':9000' || echo "")
PORT_8080=$(ss -tulpn 2>/dev/null | grep ':8080' || echo "")
PORT_2300=$(ss -tulpn 2>/dev/null | grep ':2300' || echo "")

if [ -n "$PORT_9000" ]; then
    echo "   ✅ 9000 (BearDog)"
else
    echo "   ❌ 9000 (BearDog) - NOT LISTENING"
fi

if [ -n "$PORT_8080" ]; then
    echo "   ✅ 8080 (Songbird RPC)"
else
    echo "   ❌ 8080 (Songbird RPC) - NOT LISTENING"
fi

if [ -n "$PORT_2300" ]; then
    echo "   ✅ 2300 (Songbird UDP Discovery)"
else
    echo "   ❌ 2300 (Songbird UDP Discovery) - NOT LISTENING"
fi
echo ""

echo "6️⃣  Songbird Discovery Activity:"
echo "─────────────────────────────────────────────────────────────────────────"
if [ -f /tmp/songbird-orchestrator.log ]; then
    DISCOVERY_LINES=$(grep -i "discover\|attestation\|identity" /tmp/songbird-orchestrator.log | tail -5 || echo "")
    if [ -n "$DISCOVERY_LINES" ]; then
        echo "$DISCOVERY_LINES"
    else
        echo "   No discovery activity yet (this is normal for single tower)"
    fi
else
    echo "   No Songbird logs yet"
fi
echo ""

echo "7️⃣  BearDog Trust Evaluation Activity:"
echo "─────────────────────────────────────────────────────────────────────────"
if [ -f /tmp/beardog-server.log ]; then
    TRUST_LINES=$(grep -i "trust\|evaluate\|lineage" /tmp/beardog-server.log | tail -5 || echo "")
    if [ -n "$TRUST_LINES" ]; then
        echo "$TRUST_LINES"
    else
        echo "   No trust evaluation activity yet (waiting for peer discovery)"
    fi
else
    echo "   No BearDog logs yet"
fi
echo ""

echo "══════════════════════════════════════════════════════════════════════════"
echo "📋 Full Logs"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

echo "🐻 BearDog Logs (last 30 lines):"
echo "─────────────────────────────────────────────────────────────────────────"
if [ -f /tmp/beardog-server.log ]; then
    tail -30 /tmp/beardog-server.log
else
    echo "   No logs found"
fi
echo ""

echo "🐦 Songbird Logs (last 30 lines):"
echo "─────────────────────────────────────────────────────────────────────────"
if [ -f /tmp/songbird-orchestrator.log ]; then
    tail -30 /tmp/songbird-orchestrator.log
else
    echo "   No logs found"
fi
echo ""

echo "══════════════════════════════════════════════════════════════════════════"
echo "✅ Tower 1 Verification Complete!"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""
echo "📊 Summary:"
echo "   BearDog:  $([ -n "$BEARDOG_PID" ] && echo "✅ Running" || echo "❌ Not Running")"
echo "   Songbird: $([ -n "$SONGBIRD_PID" ] && echo "✅ Running" || echo "❌ Not Running")"
echo "   Identity: $([ -n "$ENCRYPTION_TAG" ] && [ "$ENCRYPTION_TAG" != "unknown" ] && echo "✅ Retrieved" || echo "❌ Not Retrieved")"
echo "   Family:   $FAMILY_ID"
echo ""
echo "🚀 Next Steps:"
echo "   1. Eject USB"
echo "   2. Plug into Tower 2"
echo "   3. Run the same auto-deploy script"
echo "   4. Verify automatic mesh formation (watch logs)"
echo ""
echo "Expected: Mesh forms automatically in 30-60 seconds! 🎊"
echo ""

