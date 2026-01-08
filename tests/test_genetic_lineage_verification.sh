#!/usr/bin/env bash
# 🧬 Genetic Lineage Verification Test
#
# Tests that all 5 genetic siblings can verify their family relationship
# using BearDog's cryptographic lineage API (NO custom crypto!)
#
# Prerequisites:
#   - BearDog HSM bug fixed
#   - All 5 USB spores created with unique genetic seeds
#   - BearDog server running

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                  ║${NC}"
echo -e "${BLUE}║      🧬 Genetic Lineage Verification Test (via BearDog) 🧬      ║${NC}"
echo -e "${BLUE}║                                                                  ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Configuration
BEARDOG_API="http://localhost:19000/api/v1"  # Adjust port as needed
SPORE_ROOTS=(
    "/media/eastgate/biomeOS1/biomeOS"     # node-alpha
    "/media/eastgate/biomeOS21/biomeOS"    # node-beta
    "/media/eastgate/BEA6-BBCE/biomeOS"    # node-gamma
    "/media/eastgate/BEA6-BBCE1/biomeOS"   # node-delta
    "/media/eastgate/BEA6-BBCE2/biomeOS"   # node-epsilon
)
NODE_NAMES=(
    "node-alpha"
    "node-beta"
    "node-gamma"
    "node-delta"
    "node-epsilon"
)

# Array to store lineage IDs
declare -a LINEAGE_IDS

echo "📊 Test Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "BearDog API: $BEARDOG_API"
echo "Spores: ${#SPORE_ROOTS[@]}"
echo ""

# ==============================================================================
# Phase 1: Verify all spores have unique seeds
# ==============================================================================

echo "🔍 Phase 1: Verify Genetic Uniqueness"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

declare -A seed_hashes
duplicates=0

for i in "${!SPORE_ROOTS[@]}"; do
    seed_file="${SPORE_ROOTS[$i]}/.family.seed"
    node_name="${NODE_NAMES[$i]}"
    
    if [ ! -f "$seed_file" ]; then
        echo -e "${RED}❌ Seed not found: $seed_file${NC}"
        exit 1
    fi
    
    # Get SHA256 hash of seed
    seed_hash=$(sha256sum "$seed_file" | awk '{print $1}')
    
    # Check for duplicates
    if [[ -v seed_hashes[$seed_hash] ]]; then
        echo -e "${RED}❌ DUPLICATE seed detected!${NC}"
        echo -e "   ${seed_hashes[$seed_hash]} and $node_name have identical seeds"
        duplicates=$((duplicates + 1))
    else
        seed_hashes[$seed_hash]="$node_name"
        echo -e "${GREEN}✅ $node_name: ${seed_hash:0:32}...${NC}"
    fi
done

echo ""
if [ $duplicates -gt 0 ]; then
    echo -e "${RED}❌ Test failed: Found $duplicates duplicate seeds!${NC}"
    echo -e "${YELLOW}   Genetic siblings should have UNIQUE seeds, not identical clones!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All 5 siblings have UNIQUE genetic seeds!${NC}"
echo ""

# ==============================================================================
# Phase 2: Check BearDog availability
# ==============================================================================

echo "🔍 Phase 2: Check BearDog Availability"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if ! curl -s --max-time 3 "${BEARDOG_API%/api/v1}/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ BearDog server not reachable at: ${BEARDOG_API%/api/v1}${NC}"
    echo ""
    echo -e "${YELLOW}⚠️  BearDog HSM bug still blocking!${NC}"
    echo -e "${YELLOW}   This test requires BearDog to be running.${NC}"
    echo ""
    echo "To fix:"
    echo "  1. BearDog team fixes HSM provider registration bug"
    echo "  2. Start BearDog: BEARDOG_HSM_PROVIDER=software ./beardog-server"
    echo "  3. Re-run this test"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ BearDog server is running!${NC}"
echo ""

# ==============================================================================
# Phase 3: Create lineage for each genetic sibling using BearDog
# ==============================================================================

echo "🧬 Phase 3: Register Genetic Lineages with BearDog"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# First sibling is genesis (parent)
genesis_node="${NODE_NAMES[0]}"
genesis_seed="${SPORE_ROOTS[0]}/.family.seed"

echo "Creating genesis lineage for: $genesis_node"

# Read genesis seed as base64
genesis_seed_b64=$(base64 < "$genesis_seed")

# Create genesis lineage via BearDog API
genesis_response=$(curl -s -X POST "${BEARDOG_API}/lineage/create" \
    -H "Content-Type: application/json" \
    -d "{
        \"service_type\": \"biomeOS-spore\",
        \"metadata\": {
            \"node_id\": \"$genesis_node\",
            \"seed_hash\": \"$(sha256sum "$genesis_seed" | awk '{print $1}')\",
            \"deployment_batch\": \"20260108\"
        }
    }")

genesis_lineage_id=$(echo "$genesis_response" | jq -r '.data.lineage_id')

if [ "$genesis_lineage_id" = "null" ] || [ -z "$genesis_lineage_id" ]; then
    echo -e "${RED}❌ Failed to create genesis lineage${NC}"
    echo "Response: $genesis_response"
    exit 1
fi

LINEAGE_IDS[0]="$genesis_lineage_id"
echo -e "${GREEN}✅ Genesis lineage created: $genesis_lineage_id${NC}"
echo ""

# Create lineages for remaining siblings (spawn from genesis)
for i in $(seq 1 $((${#SPORE_ROOTS[@]} - 1))); do
    node_name="${NODE_NAMES[$i]}"
    seed_file="${SPORE_ROOTS[$i]}/.family.seed"
    
    echo "Spawning lineage for sibling: $node_name"
    
    # Read seed as base64
    seed_b64=$(base64 < "$seed_file")
    
    # Spawn child lineage from genesis
    spawn_response=$(curl -s -X POST "${BEARDOG_API}/lineage/spawn" \
        -H "Content-Type: application/json" \
        -d "{
            \"parent_lineage\": \"$genesis_lineage_id\",
            \"service_type\": \"biomeOS-spore\",
            \"metadata\": {
                \"node_id\": \"$node_name\",
                \"seed_hash\": \"$(sha256sum "$seed_file" | awk '{print $1}')\",
                \"deployment_batch\": \"20260108\"
            }
        }")
    
    child_lineage_id=$(echo "$spawn_response" | jq -r '.data.lineage_id')
    
    if [ "$child_lineage_id" = "null" ] || [ -z "$child_lineage_id" ]; then
        echo -e "${RED}❌ Failed to spawn lineage for $node_name${NC}"
        echo "Response: $spawn_response"
        exit 1
    fi
    
    LINEAGE_IDS[$i]="$child_lineage_id"
    echo -e "${GREEN}✅ Sibling lineage created: $child_lineage_id${NC}"
    echo ""
done

echo -e "${GREEN}✅ All 5 lineages registered with BearDog!${NC}"
echo ""

# ==============================================================================
# Phase 4: Verify family relationships using BearDog's same_family API
# ==============================================================================

echo "🔍 Phase 4: Verify Family Relationships (BearDog Crypto)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing all pairwise relationships (should all be same_family=true):"
echo ""

total_tests=0
passed_tests=0
failed_tests=0

for i in "${!LINEAGE_IDS[@]}"; do
    for j in "${!LINEAGE_IDS[@]}"; do
        if [ $i -ge $j ]; then
            continue  # Skip self-comparison and already tested pairs
        fi
        
        lineage_a="${LINEAGE_IDS[$i]}"
        lineage_b="${LINEAGE_IDS[$j]}"
        node_a="${NODE_NAMES[$i]}"
        node_b="${NODE_NAMES[$j]}"
        
        total_tests=$((total_tests + 1))
        
        # Check if they're in the same family via BearDog API
        family_response=$(curl -s -X POST "${BEARDOG_API}/lineage/same_family" \
            -H "Content-Type: application/json" \
            -d "{
                \"lineage_a\": \"$lineage_a\",
                \"lineage_b\": \"$lineage_b\"
            }")
        
        same_family=$(echo "$family_response" | jq -r '.data.same_family')
        common_ancestor=$(echo "$family_response" | jq -r '.data.common_ancestor')
        
        if [ "$same_family" = "true" ]; then
            echo -e "${GREEN}✅ $node_a ↔ $node_b: FAMILY (ancestor: $common_ancestor)${NC}"
            passed_tests=$((passed_tests + 1))
        else
            echo -e "${RED}❌ $node_a ↔ $node_b: NOT FAMILY (unexpected!)${NC}"
            failed_tests=$((failed_tests + 1))
        fi
    done
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Test Results"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Total tests: $total_tests"
echo -e "${GREEN}Passed: $passed_tests${NC}"
if [ $failed_tests -gt 0 ]; then
    echo -e "${RED}Failed: $failed_tests${NC}"
else
    echo "Failed: 0"
fi
echo ""

if [ $failed_tests -eq 0 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                           ║${NC}"
    echo -e "${GREEN}║     🎊 ALL TESTS PASSED! 🎊                               ║${NC}"
    echo -e "${GREEN}║                                                           ║${NC}"
    echo -e "${GREEN}║  All 5 genetic siblings recognize each other as family!  ║${NC}"
    echo -e "${GREEN}║  Cryptographic lineage verification via BearDog works!   ║${NC}"
    echo -e "${GREEN}║                                                           ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                           ║${NC}"
    echo -e "${RED}║     ❌ TEST FAILED! ❌                                     ║${NC}"
    echo -e "${RED}║                                                           ║${NC}"
    echo -e "${RED}║  Some siblings did not recognize each other as family!   ║${NC}"
    echo -e "${RED}║                                                           ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi

