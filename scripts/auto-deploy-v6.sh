#!/bin/bash
# biomeOS USB v6.0 Auto-Deployment Script
# Secure by Default with USB Family Seed + Songbird Zero-Hardcoding
#
# Features:
# - USB family seed for automatic trust
# - Local entropy mixing for privacy
# - Songbird zero-hardcoding (SecurityCapabilityClient)
# - BearDog /trust/evaluate API
# - Environment-driven configuration

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="$(dirname "$SCRIPT_DIR")"

echo "══════════════════════════════════════════════════════════════"
echo "  🚀 biomeOS USB v6.0 - Secure by Default Deployment"
echo "══════════════════════════════════════════════════════════════"
echo ""

# Detect tower hostname
TOWER_NAME=$(hostname | cut -d. -f1)
echo "Tower: $TOWER_NAME"
echo ""

# ═══════════════════════════════════════════════════════════════
# 🔐 PHASE 1: USB FAMILY SEED (Genetic Lineage)
# ═══════════════════════════════════════════════════════════════

if [ -f "$DEPLOY_ROOT/secrets/family-genesis.key" ]; then
    echo "🔐 USB Family Seed Detected"
    echo "─────────────────────────────────────────────────────────"
    
    # Read USB family seed
    FAMILY_SEED=$(jq -r '.genesis_seed' "$DEPLOY_ROOT/secrets/family-genesis.key")
    FAMILY_ID=$(jq -r '.family_id' "$DEPLOY_ROOT/secrets/family-genesis.key")
    FAMILY_HASH=$(jq -r '.genesis_hash' "$DEPLOY_ROOT/secrets/family-genesis.key")
    
    echo "  Family ID:      $FAMILY_ID"
    echo "  Genesis Hash:   ${FAMILY_HASH:0:16}..."
    echo ""
    echo "  ✅ Genetic lineage will be enabled"
    echo "  ✅ Towers from this USB will auto-trust"
    echo ""
    
    GENETIC_LINEAGE_ENABLED=true
else
    echo "⚠️  No USB Family Seed Found"
    echo "─────────────────────────────────────────────────────────"
    echo "  Location: $DEPLOY_ROOT/secrets/family-genesis.key"
    echo "  Status: Not found"
    echo ""
    echo "  ⚠️  Genetic lineage disabled"
    echo "  ⚠️  Will use anonymous trust (less secure)"
    echo ""
    echo "  To enable genetic lineage:"
    echo "    ./scripts/create-usb-family-seed.sh"
    echo ""
    
    GENETIC_LINEAGE_ENABLED=false
    FAMILY_SEED=""
    FAMILY_ID=""
    FAMILY_HASH=""
fi

# ═══════════════════════════════════════════════════════════════
# 🔧 PHASE 2: ENVIRONMENT CONFIGURATION
# ═══════════════════════════════════════════════════════════════

echo "🔧 Environment Configuration"
echo "─────────────────────────────────────────────────────────"

# Collect local machine entropy for lineage mixing
LOCAL_ENTROPY=$(cat /proc/sys/kernel/random/uuid)
MACHINE_ID=$(hostnamectl status --no-pager 2>/dev/null | grep "Machine ID" | awk '{print $3}' || echo $LOCAL_ENTROPY)

echo "  Machine ID:     ${MACHINE_ID:0:16}..."
echo "  Local Entropy:  ${LOCAL_ENTROPY:0:16}..."
echo ""

# Songbird environment variables (zero-hardcoding)
export SONGBIRD_DISCOVERY_MODE=anonymous
export SONGBIRD_DISCOVERY_PORT=2300
export SONGBIRD_ORCHESTRATOR_PORT=8080
export SONGBIRD_BIND_ADDR=0.0.0.0

# BearDog environment variables
export BEARDOG_HSM_MODE=software
export BEARDOG_API_BIND_ADDR=127.0.0.1:9000  # Localhost only for security

# USB family seed (if available)
if [ "$GENETIC_LINEAGE_ENABLED" = true ]; then
    export BEARDOG_FAMILY_SEED="$FAMILY_SEED"
    export BEARDOG_LOCAL_ENTROPY="$LOCAL_ENTROPY"
    export BEARDOG_MACHINE_ID="$MACHINE_ID"
    export BEARDOG_FAMILY_ID="$FAMILY_ID"
fi

# Songbird discovers BearDog via SecurityCapabilityClient (zero-hardcoding!)
# No hardcoded "BearDog" - uses capability discovery
export SECURITY_ENDPOINT=http://localhost:9000

echo "  Environment variables set:"
echo "    SONGBIRD_DISCOVERY_PORT=$SONGBIRD_DISCOVERY_PORT"
echo "    SONGBIRD_ORCHESTRATOR_PORT=$SONGBIRD_ORCHESTRATOR_PORT"
echo "    BEARDOG_HSM_MODE=$BEARDOG_HSM_MODE"
if [ "$GENETIC_LINEAGE_ENABLED" = true ]; then
    echo "    BEARDOG_FAMILY_ID=$FAMILY_ID"
    echo "    SECURITY_ENDPOINT=$SECURITY_ENDPOINT"
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# 🐻 PHASE 3: START BEARDOG (Cryptography Layer)
# ═══════════════════════════════════════════════════════════════

cd "$DEPLOY_ROOT/primals"

if [ -f "beardog-server" ]; then
    echo "🐻 Starting BearDog (Cryptography Layer)"
    echo "─────────────────────────────────────────────────────────"
    
    # Start BearDog server
    RUST_LOG=info \
    ./beardog-server > /tmp/beardog-server.log 2>&1 &
    BEARDOG_PID=$!
    
    echo "  PID: $BEARDOG_PID"
    echo "  API: http://127.0.0.1:9000 (localhost only)"
    echo "  HSM: software (OS entropy)"
    
    if [ "$GENETIC_LINEAGE_ENABLED" = true ]; then
        echo "  Genetic Lineage: ENABLED ✨"
        echo ""
        echo "  Waiting for BearDog to initialize..."
        sleep 5
        
        # Create child lineage from USB seed + local entropy
        echo "  Creating child lineage (USB seed + local entropy)..."
        LINEAGE_RESPONSE=$(curl -s -X POST http://localhost:9000/api/v1/lineage/create \
          -H "Content-Type: application/json" \
          -d "{
            \"service_type\": \"tower\",
            \"metadata\": {
              \"family_id\": \"$FAMILY_ID\",
              \"machine_id\": \"$MACHINE_ID\",
              \"tower_name\": \"$TOWER_NAME\",
              \"deployment_source\": \"usb\"
            }
          }" 2>/dev/null || echo '{"error": "failed"}')
        
        if echo "$LINEAGE_RESPONSE" | jq -e '.lineage_id' > /dev/null 2>&1; then
            LINEAGE_ID=$(echo "$LINEAGE_RESPONSE" | jq -r '.lineage_id')
            echo "  ✅ Child lineage created: ${LINEAGE_ID:0:32}..."
            echo "  ✅ Same genesis as other towers from this USB"
            echo "  ✅ Unique identity (privacy preserved)"
        else
            echo "  ⚠️  Lineage creation pending (BearDog will auto-create)"
        fi
    else
        echo "  Genetic Lineage: DISABLED (no USB seed)"
    fi
    echo ""
    
    BEARDOG_RUNNING=true
else
    echo "⚠️  beardog-server binary not found"
    echo "─────────────────────────────────────────────────────────"
    echo "  Genetic lineage unavailable"
    echo "  Will use anonymous trust"
    echo ""
    
    BEARDOG_RUNNING=false
fi

# ═══════════════════════════════════════════════════════════════
# 🐦 PHASE 4: START SONGBIRD (Orchestration Layer)
# ═══════════════════════════════════════════════════════════════

if [ -f "songbird-orchestrator" ]; then
    echo "🐦 Starting Songbird (Orchestration Layer)"
    echo "─────────────────────────────────────────────────────────"
    
    # Start Songbird with zero-hardcoding environment
    # Songbird will use SecurityCapabilityClient (not hardcoded "BearDog"!)
    # Discovers security provider via SECURITY_ENDPOINT or mDNS
    RUST_LOG=info \
    ./songbird-orchestrator > /tmp/songbird-orchestrator.log 2>&1 &
    SONGBIRD_PID=$!
    
    echo "  PID: $SONGBIRD_PID"
    echo "  tarpc RPC: 0.0.0.0:8080"
    echo "  UDP Discovery: 224.0.0.251:2300 (multicast)"
    echo "  Discovery Mode: anonymous"
    
    if [ "$BEARDOG_RUNNING" = true ]; then
        echo "  Security Provider: BearDog (via SecurityCapabilityClient)"
        echo "  Trust Evaluation: /trust/evaluate API"
        echo "  Auto-Accept: Same family → YES ✅"
        echo "  Prompt User: Different family → CONSENT REQUIRED ⚠️"
        echo "  Reject: No lineage → DENIED ❌"
    else
        echo "  Security Provider: None (anonymous trust fallback)"
        echo "  Trust Evaluation: Anonymous (less secure)"
    fi
    echo ""
    
    sleep 3
    SONGBIRD_RUNNING=true
else
    echo "⚠️  songbird-orchestrator binary not found"
    echo ""
    SONGBIRD_RUNNING=false
fi

# ═══════════════════════════════════════════════════════════════
# ✅ PHASE 5: DEPLOYMENT COMPLETE
# ═══════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════"
echo "✅ Deployment Complete!"
echo "══════════════════════════════════════════════════════════════"
echo ""

if [ "$BEARDOG_RUNNING" = true ]; then
    echo "🐻 BearDog (Cryptography)"
    echo "  • Status: Running (PID: $BEARDOG_PID)"
    echo "  • API: http://localhost:9000"
    echo "  • HSM: software"
    if [ "$GENETIC_LINEAGE_ENABLED" = true ]; then
        echo "  • Genetic Lineage: ENABLED ✨"
        echo "  • Family: $FAMILY_ID"
        echo "  • Trust: Cryptographic (same family auto-accept)"
    else
        echo "  • Genetic Lineage: DISABLED"
    fi
    echo ""
fi

if [ "$SONGBIRD_RUNNING" = true ]; then
    echo "🐦 Songbird (Orchestration)"
    echo "  • Status: Running (PID: $SONGBIRD_PID)"
    echo "  • tarpc RPC: 0.0.0.0:8080"
    echo "  • UDP Multicast: 224.0.0.251:2300"
    echo "  • Discovery: automatic (cross-router)"
    if [ "$BEARDOG_RUNNING" = true ]; then
        echo "  • Security: BearDog trust evaluation"
        if [ "$GENETIC_LINEAGE_ENABLED" = true ]; then
            echo "  • Trust Model: Genetic lineage (secure by default)"
        else
            echo "  • Trust Model: Anonymous (fallback)"
        fi
    else
        echo "  • Security: Anonymous trust (no BearDog)"
    fi
    echo ""
fi

echo "📊 Security Status:"
if [ "$GENETIC_LINEAGE_ENABLED" = true ] && [ "$BEARDOG_RUNNING" = true ] && [ "$SONGBIRD_RUNNING" = true ]; then
    echo "  ✅ SECURE BY DEFAULT"
    echo "     • USB family seed active"
    echo "     • Genetic lineage enabled"
    echo "     • Same family → auto-trust"
    echo "     • Different family → prompt user"
    echo "     • No lineage → reject"
    echo "     • Privacy preserved (unique per tower)"
elif [ "$BEARDOG_RUNNING" = true ] || [ "$SONGBIRD_RUNNING" = true ]; then
    echo "  ⚠️  PARTIALLY SECURE"
    if [ "$GENETIC_LINEAGE_ENABLED" = false ]; then
        echo "     • No USB family seed"
        echo "     • Using anonymous trust"
        echo "     • Anyone on network can connect"
    fi
    echo "     • Run: ./scripts/create-usb-family-seed.sh"
else
    echo "  ⚠️  SERVICES NOT RUNNING"
fi

echo ""
echo "📝 Logs:"
if [ "$BEARDOG_RUNNING" = true ]; then
    echo "  • BearDog: tail -f /tmp/beardog-server.log"
fi
if [ "$SONGBIRD_RUNNING" = true ]; then
    echo "  • Songbird: tail -f /tmp/songbird-orchestrator.log"
fi

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "🎊 Tower $TOWER_NAME is ready!"
echo "══════════════════════════════════════════════════════════════"
echo ""

# ═══════════════════════════════════════════════════════════════
# 📊 OPTIONAL: VERIFY DEPLOYMENT
# ═══════════════════════════════════════════════════════════════

if [ "$BEARDOG_RUNNING" = true ]; then
    echo "🔍 Quick Verification:"
    echo ""
    echo "  BearDog Health:"
    curl -s http://localhost:9000/health | jq . 2>/dev/null || echo "  (Not responding yet, may need more time)"
    echo ""
    
    if [ "$GENETIC_LINEAGE_ENABLED" = true ]; then
        echo "  Current Lineage:"
        curl -s http://localhost:9000/api/v1/lineage/current | jq -r '.lineage_id // "Not created yet"' 2>/dev/null || echo "  (Pending)"
        echo ""
    fi
fi

echo "══════════════════════════════════════════════════════════════"
echo ""

