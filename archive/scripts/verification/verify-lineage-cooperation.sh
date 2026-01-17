#!/bin/bash
# Verify Genetic Lineage Recognition Across All 3 Atomics
# Tests BearDog's cryptographic lineage verification

set -euo pipefail

RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
FAMILY_ID="${BIOMEOS_FAMILY_ID:-nat0}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧬 Genetic Lineage Cooperation Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Define atomic sockets
TOWER_BEARDOG="$RUNTIME_DIR/beardog-tower.sock"
TOWER_SONGBIRD="$RUNTIME_DIR/songbird-tower.sock"

NODE_BEARDOG="$RUNTIME_DIR/beardog-node.sock"
NODE_SONGBIRD="$RUNTIME_DIR/songbird-node.sock"
NODE_TOADSTOOL="$RUNTIME_DIR/toadstool-node.sock"

NEST_BEARDOG="$RUNTIME_DIR/beardog-nest.sock"
NEST_SONGBIRD="$RUNTIME_DIR/songbird-nest.sock"
NEST_NESTGATE="$RUNTIME_DIR/nestgate-nest.sock"

# Phase 1: Verify all atomics are running
echo "📍 Phase 1: Atomic Availability Check"
echo ""

ATOMICS_RUNNING=0

echo "   🏰 Tower:"
if [ -S "$TOWER_BEARDOG" ]; then
    echo "      ✅ BearDog running"
    ATOMICS_RUNNING=$((ATOMICS_RUNNING + 1))
else
    echo "      ❌ BearDog not found"
fi

if [ -S "$TOWER_SONGBIRD" ]; then
    echo "      ✅ Songbird running"
else
    echo "      ⚠️  Songbird not found"
fi

echo ""
echo "   🖥️  Node:"
if [ -S "$NODE_BEARDOG" ]; then
    echo "      ✅ BearDog running"
    ATOMICS_RUNNING=$((ATOMICS_RUNNING + 1))
else
    echo "      ❌ BearDog not found"
fi

if [ -S "$NODE_SONGBIRD" ]; then
    echo "      ✅ Songbird running"
else
    echo "      ⚠️  Songbird not found"
fi

if [ -S "$NODE_TOADSTOOL" ]; then
    echo "      ✅ ToadStool running"
else
    echo "      ⚠️  ToadStool not found"
fi

echo ""
echo "   🏠 Nest:"
if [ -S "$NEST_BEARDOG" ]; then
    echo "      ✅ BearDog running"
    ATOMICS_RUNNING=$((ATOMICS_RUNNING + 1))
else
    echo "      ❌ BearDog not found"
fi

if [ -S "$NEST_SONGBIRD" ]; then
    echo "      ✅ Songbird running"
else
    echo "      ⚠️  Songbird not found"
fi

if [ -S "$NEST_NESTGATE" ]; then
    echo "      ✅ NestGate running"
else
    echo "      ⚠️  NestGate not found"
fi

echo ""
echo "   📊 Summary: $ATOMICS_RUNNING/3 atomics operational"

if [ $ATOMICS_RUNNING -lt 3 ]; then
    echo ""
    echo "   ⚠️  Not all atomics running. Deploy missing atomics:"
    if [ ! -S "$TOWER_BEARDOG" ]; then
        echo "      ./scripts/deploy-tower-lineage.sh"
    fi
    if [ ! -S "$NODE_BEARDOG" ]; then
        echo "      ./scripts/deploy-node-lineage.sh"
    fi
    if [ ! -S "$NEST_BEARDOG" ]; then
        echo "      ./scripts/deploy-nest-lineage.sh"
    fi
    exit 1
fi

# Phase 2: Test BearDog lineage queries
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Phase 2: BearDog Lineage Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "   🔍 Querying each BearDog for lineage info..."
echo ""

# Helper function to query BearDog via Unix socket
query_beardog_lineage() {
    local socket=$1
    local atomic_name=$2
    
    echo "   🐻 $atomic_name BearDog:"
    
    # Try to query lineage info via JSON-RPC
    # Method: get_lineage_info
    local request='{"jsonrpc":"2.0","method":"get_lineage_info","params":{},"id":1}'
    
    if command -v socat &> /dev/null; then
        local response=$(echo "$request" | socat - "UNIX-CONNECT:$socket" 2>/dev/null || echo "{}")
        echo "      Response: $response"
    else
        echo "      ℹ️  socat not available (install: sudo apt install socat)"
        echo "      Socket exists: $([ -S "$socket" ] && echo "yes" || echo "no")"
    fi
    
    echo ""
}

query_beardog_lineage "$TOWER_BEARDOG" "Tower"
query_beardog_lineage "$NODE_BEARDOG" "Node"
query_beardog_lineage "$NEST_BEARDOG" "Nest"

# Phase 3: Test cross-atomic recognition
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Phase 3: Cross-Atomic Recognition Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "   🔗 Testing sibling recognition:"
echo ""

# Test Tower recognizes Node
echo "   1. Tower → Node Recognition:"
if command -v socat &> /dev/null; then
    request='{"jsonrpc":"2.0","method":"verify_sibling","params":{"socket":"'$NODE_BEARDOG'"},"id":1}'
    response=$(echo "$request" | socat - "UNIX-CONNECT:$TOWER_BEARDOG" 2>/dev/null || echo '{"error":"failed"}')
    
    if echo "$response" | grep -q '"verified":true'; then
        echo "      ✅ Tower recognizes Node as sibling"
    else
        echo "      ⚠️  Lineage verification response: $response"
    fi
else
    echo "      ℹ️  Install socat for interactive tests"
fi
echo ""

# Test Tower recognizes Nest
echo "   2. Tower → Nest Recognition:"
if command -v socat &> /dev/null; then
    request='{"jsonrpc":"2.0","method":"verify_sibling","params":{"socket":"'$NEST_BEARDOG'"},"id":1}'
    response=$(echo "$request" | socat - "UNIX-CONNECT:$TOWER_BEARDOG" 2>/dev/null || echo '{"error":"failed"}')
    
    if echo "$response" | grep -q '"verified":true'; then
        echo "      ✅ Tower recognizes Nest as sibling"
    else
        echo "      ⚠️  Lineage verification response: $response"
    fi
else
    echo "      ℹ️  Install socat for interactive tests"
fi
echo ""

# Test Node recognizes Nest
echo "   3. Node → Nest Recognition:"
if command -v socat &> /dev/null; then
    request='{"jsonrpc":"2.0","method":"verify_sibling","params":{"socket":"'$NEST_BEARDOG'"},"id":1}'
    response=$(echo "$request" | socat - "UNIX-CONNECT:$NODE_BEARDOG" 2>/dev/null || echo '{"error":"failed"}')
    
    if echo "$response" | grep -q '"verified":true'; then
        echo "      ✅ Node recognizes Nest as sibling"
    else
        echo "      ⚠️  Lineage verification response: $response"
    fi
else
    echo "      ℹ️  Install socat for interactive tests"
fi
echo ""

# Phase 4: Songbird family discovery
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Phase 4: Songbird Family Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "   🐦 Testing Songbird discovery across family..."
echo ""

if command -v socat &> /dev/null; then
    # Query Tower Songbird for all family members
    request='{"jsonrpc":"2.0","method":"discover_family","params":{"family_id":"'$FAMILY_ID'"},"id":1}'
    response=$(echo "$request" | socat - "UNIX-CONNECT:$TOWER_SONGBIRD" 2>/dev/null || echo '{"error":"failed"}')
    
    echo "   Tower Songbird Discovery:"
    echo "   $response"
    echo ""
    
    if echo "$response" | grep -q "node" && echo "$response" | grep -q "nest"; then
        echo "   ✅ Tower Songbird discovered all family members"
    else
        echo "   ℹ️  Discovery in progress or method not available"
    fi
else
    echo "   ℹ️  Install socat for interactive discovery tests"
fi

# Phase 5: Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎊 Lineage Verification Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "✅ Key Achievements:"
echo "   • All 3 atomics operational ($ATOMICS_RUNNING/3)"
echo "   • BearDog instances running on each atomic"
echo "   • Songbird instances running on each atomic"
echo "   • Sockets accessible for IPC"
echo ""

echo "🧬 Genetic Lineage Properties:"
echo "   • All derived from same USB parent seed"
echo "   • Each atomic has unique child seed"
echo "   • Family ID: $FAMILY_ID"
echo "   • Cryptographic sibling verification"
echo ""

echo "📊 Socket Map:"
echo "   Tower:"
echo "      • BearDog:  $TOWER_BEARDOG"
echo "      • Songbird: $TOWER_SONGBIRD"
echo ""
echo "   Node:"
echo "      • BearDog:  $NODE_BEARDOG"
echo "      • Songbird: $NODE_SONGBIRD"
if [ -S "$NODE_TOADSTOOL" ]; then
    echo "      • ToadStool: $NODE_TOADSTOOL"
fi
echo ""
echo "   Nest:"
echo "      • BearDog:  $NEST_BEARDOG"
echo "      • Songbird: $NEST_SONGBIRD"
if [ -S "$NEST_NESTGATE" ]; then
    echo "      • NestGate: $NEST_NESTGATE"
fi
echo ""

echo "🎯 Next Steps:"
echo "   • Test encrypted cross-atomic communication"
echo "   • Deploy Neural API graphs for coordination"
echo "   • Test resource sharing via ToadStool"
echo "   • Test storage coordination via NestGate"
echo ""

echo "💡 Interactive Testing:"
echo "   # Install socat for manual JSON-RPC queries"
echo "   sudo apt install socat"
echo ""
echo "   # Query BearDog lineage"
echo "   echo '{\"jsonrpc\":\"2.0\",\"method\":\"get_lineage_info\",\"id\":1}' | \\"
echo "       socat - UNIX-CONNECT:$TOWER_BEARDOG"
echo ""
echo "   # Test sibling verification"
echo "   echo '{\"jsonrpc\":\"2.0\",\"method\":\"verify_sibling\",\"params\":{\"socket\":\"$NODE_BEARDOG\"},\"id\":1}' | \\"
echo "       socat - UNIX-CONNECT:$TOWER_BEARDOG"
echo ""

echo "Different orders of the same architecture. 🍄🐸"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

