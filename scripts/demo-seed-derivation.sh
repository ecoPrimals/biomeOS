#!/bin/bash
# Demonstrate Genetic Seed Derivation
# Shows how Tower, Node, Nest get unique seeds from USB parent

set -euo pipefail

USB_SEED="${1:-/tmp/biomeos-lineage-test/.family.seed}"
FAMILY_ID="nat0"
DEPLOYMENT_BATCH=$(date +%Y%m%d)

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧬 Genetic Seed Derivation Demonstration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verify USB seed exists
if [ ! -f "$USB_SEED" ]; then
    echo "❌ USB seed not found: $USB_SEED"
    echo ""
    echo "Create one with:"
    echo "   ./scripts/create-test-seed.sh"
    exit 1
fi

echo "📍 Parent Seed (USB)"
echo "   Location: $USB_SEED"
PARENT_SIZE=$(stat -c %s "$USB_SEED")
PARENT_HASH=$(sha256sum "$USB_SEED" | awk '{print $1}')
PARENT_PREVIEW=$(echo "$PARENT_HASH" | head -c 32)
echo "   Size: $PARENT_SIZE bytes"
echo "   Hash: $PARENT_PREVIEW..."
echo ""

# Create output directory
OUT_DIR="/tmp/biomeos-derived-seeds"
mkdir -p "$OUT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔬 Deriving Child Seeds"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Derivation function
derive_child() {
    local node_id=$1
    local output=$2
    
    echo "   🧬 Deriving: $node_id"
    echo "      Formula: SHA256(parent || \"$node_id\" || \"$DEPLOYMENT_BATCH\")"
    
    # Perform derivation
    (cat "$USB_SEED"; echo -n "$node_id$DEPLOYMENT_BATCH") | \
        sha256sum | \
        xxd -r -p | \
        head -c 32 > "$output"
    
    chmod 600 "$output"
    
    local child_hash=$(sha256sum "$output" | awk '{print $1}')
    local child_preview=$(echo "$child_hash" | head -c 32)
    
    echo "      Output: $output"
    echo "      Hash: $child_preview..."
    echo "      ✅ Derived successfully"
    echo ""
}

# Derive Tower seed
TOWER_SEED="$OUT_DIR/tower.seed"
derive_child "tower" "$TOWER_SEED"

# Derive Node seed
NODE_SEED="$OUT_DIR/node.seed"
derive_child "node" "$NODE_SEED"

# Derive Nest seed
NEST_SEED="$OUT_DIR/nest.seed"
derive_child "nest" "$NEST_SEED"

# Analysis
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Genetic Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "🔍 Uniqueness Verification:"
echo ""

TOWER_HASH=$(sha256sum "$TOWER_SEED" | awk '{print $1}')
NODE_HASH=$(sha256sum "$NODE_SEED" | awk '{print $1}')
NEST_HASH=$(sha256sum "$NEST_SEED" | awk '{print $1}')

echo "   Tower: ${TOWER_HASH:0:32}..."
echo "   Node:  ${NODE_HASH:0:32}..."
echo "   Nest:  ${NEST_HASH:0:32}..."
echo ""

# Check all different
if [ "$TOWER_HASH" != "$NODE_HASH" ] && \
   [ "$TOWER_HASH" != "$NEST_HASH" ] && \
   [ "$NODE_HASH" != "$NEST_HASH" ]; then
    echo "   ✅ All child seeds are UNIQUE"
else
    echo "   ❌ ERROR: Duplicate seeds detected!"
    exit 1
fi

# Check all different from parent
if [ "$TOWER_HASH" != "$PARENT_HASH" ] && \
   [ "$NODE_HASH" != "$PARENT_HASH" ] && \
   [ "$NEST_HASH" != "$PARENT_HASH" ]; then
    echo "   ✅ All child seeds DIFFER from parent"
else
    echo "   ❌ ERROR: Child seed matches parent!"
    exit 1
fi

echo ""
echo "🧬 Genetic Properties:"
echo ""
echo "   • Each child seed is UNIQUE (32 bytes)"
echo "   • All derived from SAME parent seed"
echo "   • Derivation is DETERMINISTIC (same inputs → same output)"
echo "   • Derivation is ONE-WAY (cannot reverse to parent)"
echo "   • Same batch = genetic siblings"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Lineage Model"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cat << EOF
    Family: $FAMILY_ID
    Batch:  $DEPLOYMENT_BATCH
    
    Parent Seed (USB - $PARENT_PREVIEW...)
        ↓ SHA256 Genetic Mixing
        ├─→ Tower ($( echo ${TOWER_HASH:0:16})...)
        ├─→ Node  ($( echo ${NODE_HASH:0:16})...)
        └─→ Nest  ($( echo ${NEST_HASH:0:16})...)
    
    All siblings share:
      • Same parent seed (genetic heritage)
      • Same family ID ($FAMILY_ID)
      • Same deployment batch ($DEPLOYMENT_BATCH)
    
    Each sibling is unique:
      • Different node_id → different child seed
      • Unique cryptographic identity
      • Privacy preserved
    
    BearDog can verify:
      • "Do these two share a parent?" → YES/NO
      • "What's their relationship?" → siblings/parent/unrelated
      • "Can they cooperate?" → YES (same family)

EOF

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Seed Derivation Demonstration Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📁 Derived Seeds Location:"
echo "   $OUT_DIR/"
echo ""
ls -lh "$OUT_DIR/"
echo ""

echo "🔐 Security Notes:"
echo "   • Parent seed stays on USB (never transmitted)"
echo "   • Child seeds used for deployment"
echo "   • Each atomic reads its own child seed via BEARDOG_FAMILY_SEED_FILE"
echo "   • BearDog performs lineage verification via API"
echo ""

echo "🚀 Next Steps:"
echo "   1. Deploy atomics with derived seeds"
echo "   2. Test BearDog lineage verification"
echo "   3. Verify cross-atomic cooperation"
echo ""

echo "Different orders of the same architecture. 🍄🐸"
echo ""

