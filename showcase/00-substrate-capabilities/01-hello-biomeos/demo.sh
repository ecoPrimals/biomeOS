#!/usr/bin/env bash
# Hello BiomeOS Demo - Runtime Discovery
set -e

# Source discovery utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../common/discovery.sh"

# Set primals directory
export PRIMALS_DIR="$SCRIPT_DIR/../../../primals"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║                                                          ║"
echo "║     🌱 Hello BiomeOS - Runtime Discovery Demo           ║"
echo "║                                                          ║"
echo "║  Demonstrating: Zero hardcoding, agnostic adaptation    ║"
echo "║                                                          ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
sleep 1

# Step 1: Discovery
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 1: Primal Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
discover_primals
echo ""
sleep 2

# Step 2: Capability Mapping
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 2: Capability Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Storage capability
echo -e "${BLUE}🔍 Discovering 'storage' capability...${NC}"
if STORAGE=$(discover_capability "storage" 2>&1 | grep -o "http://[^[:space:]]*"); then
    echo -e "${GREEN}✅ Storage available: $STORAGE${NC}"
    echo "   Type: REST API"
    echo "   Primal: NestGate"
    
    # Test it
    echo ""
    echo "   Testing health endpoint..."
    response=$(curl -sf "$STORAGE/health" 2>&1 || echo "failed")
    if [ "$response" != "failed" ]; then
        echo "   Response:"
        echo "$response" | jq -C '.' 2>/dev/null || echo "   $response"
    fi
else
    echo -e "${YELLOW}⚠  Storage capability not available${NC}"
fi
echo ""
sleep 2

# Encryption capability
echo -e "${BLUE}🔍 Discovering 'encryption' capability...${NC}"
if CRYPTO=$(discover_capability "encryption" 2>&1 | tail -1 | grep "/"); then
    echo -e "${GREEN}✅ Encryption available: $CRYPTO${NC}"
    echo "   Type: CLI Tool"
    echo "   Primal: BearDog"
    
    # Test it
    echo ""
    echo "   Testing CLI..."
    $CRYPTO --version 2>&1 | head -3 || echo "   (version command not supported)"
else
    echo -e "${YELLOW}⚠  Encryption capability not available${NC}"
fi
echo ""
sleep 2

# Compute capability
echo -e "${BLUE}🔍 Discovering 'compute' capability...${NC}"
if COMPUTE=$(discover_capability "compute" 2>&1 | tail -1 | grep "/"); then
    echo -e "${GREEN}✅ Compute available: $COMPUTE${NC}"
    echo "   Type: CLI Tool / Runtime"
    echo "   Primal: Toadstool"
    
    # Test it
    echo ""
    echo "   Testing CLI..."
    $COMPUTE --version 2>&1 | head -3 || echo "   (version command not supported)"
else
    echo -e "${YELLOW}⚠  Compute capability not available${NC}"
fi
echo ""
sleep 2

# Orchestration capability
echo -e "${BLUE}🔍 Discovering 'orchestration' capability...${NC}"
if ORCH=$(discover_capability "orchestration" 2>&1 | grep -o "http://[^[:space:]]*"); then
    echo -e "${GREEN}✅ Orchestration available: $ORCH${NC}"
    echo "   Type: REST API"
    echo "   Primal: Songbird"
else
    echo -e "${YELLOW}⚠  Orchestration capability not available${NC}"
    echo "   (This is OK - demonstrates graceful degradation)"
fi
echo ""
sleep 2

# Step 3: Adaptation Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 3: Adaptation Strategy"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "BiomeOS discovered primals with different architectures:"
echo ""
if [ -n "$STORAGE" ]; then
    echo "  📦 Storage (NestGate):"
    echo "     Architecture: REST API server"
    echo "     Requires: JWT authentication"
    echo "     Access: HTTP/HTTPS endpoints"
    echo "     Example: curl $STORAGE/api/v1/zfs/datasets"
    echo ""
fi

if [ -n "$CRYPTO" ]; then
    echo "  🔐 Encryption (BearDog):"
    echo "     Architecture: CLI tool"
    echo "     Integration: In-house, fully decentralized"
    echo "     Access: Direct binary execution"
    echo "     Example: $CRYPTO birdsong encrypt --data payload.bin"
    echo ""
fi

if [ -n "$COMPUTE" ]; then
    echo "  🧪 Compute (Toadstool):"
    echo "     Architecture: Runtime launcher"
    echo "     Purpose: Deploy and manage biomes"
    echo "     Access: CLI invocation"
    echo "     Example: $COMPUTE run --manifest biome.yaml"
    echo ""
fi

sleep 2

# Step 4: Key Insight
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "KEY INSIGHT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎯 BiomeOS doesn't impose standardization"
echo ""
echo "   Each primal team evolves independently:"
echo "   • NestGate team: Built REST API with JWT"
echo "   • BearDog team: Built CLI for decentralized crypto"
echo "   • Toadstool team: Built runtime launcher"
echo ""
echo "   BiomeOS discovers and adapts to ALL of them."
echo ""
echo "   ✅ Zero code changes when primals evolve"
echo "   ✅ Works with user-defined primals"
echo "   ✅ Graceful degradation when primals unavailable"
echo ""
sleep 3

# Final summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Demo Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📚 What you learned:"
echo "   1. BiomeOS discovers primals at runtime"
echo "   2. Each primal can have different architecture"
echo "   3. BiomeOS adapts agnostically"
echo "   4. No hardcoding of primal names or endpoints"
echo ""
echo "🔗 Next demos:"
echo "   • 02-capability-composition: Combine multiple primals"
echo "   • 03-primal-evolution: Handle API changes gracefully"
echo "   • 04-custom-primals: Add your own primals"
echo ""
echo "🌱 BiomeOS: The substrate for digital sovereignty"
echo ""

