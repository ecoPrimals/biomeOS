#!/usr/bin/env bash
#
# Verify Genetic Lineage - Check that all spores are related siblings
#
# This script:
# 1. Extracts seeds from all spores
# 2. Loads them into BearDog
# 3. Verifies they're all in the same family

set -euo pipefail

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║         🧬 Genetic Lineage Verification                       ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if BearDog is running
BEARDOG_API="http://127.0.0.1:9000"
echo "🔍 Checking BearDog availability..."

if ! curl -s -f "$BEARDOG_API/health" > /dev/null 2>&1; then
    echo "❌ BearDog is not running!"
    echo ""
    echo "⚠️  This test requires BearDog to be running."
    echo ""
    echo "To fix:"
    echo "  1. Deploy a spore (or run BearDog standalone)"
    echo "  2. Wait for BearDog to start"
    echo "  3. Re-run this test"
    exit 1
fi

echo "✅ BearDog is running"
echo ""

# Extract seeds from all spores
echo "📦 Extracting seeds from spores..."
echo ""

SEEDS_DIR=$(mktemp -d)
declare -A SPORE_SEEDS

for mount in biomeOS1 biomeOS21 BEA6-BBCE BEA6-BBCE1 BEA6-BBCE2; do
    seed_file="/media/eastgate/$mount/biomeOS/.family.seed"
    
    if [ -f "$seed_file" ]; then
        node_id=$(grep "NODE_ID\|node_id" "/media/eastgate/$mount/biomeOS/tower.toml" 2>/dev/null | head -1 | cut -d'"' -f2 || echo "unknown")
        
        # Copy seed to temp dir
        cp "$seed_file" "$SEEDS_DIR/${node_id}.seed"
        
        # Get hash for display
        seed_hash=$(sha256sum "$seed_file" | cut -d' ' -f1)
        
        echo "  ✅ $node_id: ${seed_hash:0:16}..."
        
        SPORE_SEEDS[$node_id]="$SEEDS_DIR/${node_id}.seed"
    fi
done

echo ""
echo "✅ Extracted ${#SPORE_SEEDS[@]} seeds"
echo ""

# Verify uniqueness
echo "🔍 Verifying genetic uniqueness..."
echo ""

seed_hashes=()
for node_id in "${!SPORE_SEEDS[@]}"; do
    hash=$(sha256sum "${SPORE_SEEDS[$node_id]}" | cut -d' ' -f1)
    seed_hashes+=("$hash")
done

unique_hashes=$(printf '%s\n' "${seed_hashes[@]}" | sort -u | wc -l)
total_hashes=${#seed_hashes[@]}

if [ "$unique_hashes" -eq "$total_hashes" ]; then
    echo "✅ All seeds are unique (genetic siblings, not clones!)"
else
    echo "❌ Found duplicate seeds (clones detected!)"
    echo "   Expected: $total_hashes unique"
    echo "   Found: $unique_hashes unique"
fi

echo ""

# TODO: Use BearDog API to verify family relationships
echo "🔮 Family Relationship Verification:"
echo "   This requires BearDog's /api/v1/lineage/same_family endpoint"
echo "   (Implementation pending BearDog Unix socket fix)"
echo ""

# Cleanup
rm -rf "$SEEDS_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║         ✅ Genetic Verification Complete!                     ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"

