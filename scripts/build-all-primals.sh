#!/bin/bash
# Build All Phase1 Primals (x86_64 + ARM64) and Create v4.1 genomeBins
# 
# Deep Debt: Automated production builds with multi-arch v4.1

set -e

PHASE1_DIR="/home/eastgate/Development/ecoPrimals/phase1"
PHASE2_DIR="/home/eastgate/Development/ecoPrimals/phase2/biomeOS"
PLASMIDBIN="$PHASE2_DIR/plasmidBin"

echo "═══════════════════════════════════════════════════════════════════"
echo "🧬 Building Phase1 Primals (x86_64 + ARM64)"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

PRIMALS=("beardog" "songbird" "toadstool" "nestgate" "squirrel")

# Step 1: Build all primals for both architectures
for primal in "${PRIMALS[@]}"; do
    PRIMAL_DIR="$PHASE1_DIR/$primal"
    
    if [ ! -d "$PRIMAL_DIR" ]; then
        echo "⏭️  Skipping $primal (directory not found)"
        continue
    fi
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Building $primal..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    cd "$PRIMAL_DIR"
    
    # Build x86_64
    echo "  🔧 Building x86_64..."
    if cargo build --release --target x86_64-unknown-linux-musl 2>&1 | tail -3; then
        X86_BIN="$PRIMAL_DIR/target/x86_64-unknown-linux-musl/release/$primal"
        if [ -f "$X86_BIN" ]; then
            echo "  ✅ x86_64: $(du -h "$X86_BIN" | cut -f1)"
        else
            echo "  ⚠️  x86_64 binary not found"
        fi
    else
        echo "  ❌ x86_64 build failed"
    fi
    
    # Build ARM64
    echo "  🔧 Building ARM64..."
    if cargo build --release --target aarch64-unknown-linux-musl 2>&1 | tail -3; then
        ARM_BIN="$PRIMAL_DIR/target/aarch64-unknown-linux-musl/release/$primal"
        if [ -f "$ARM_BIN" ]; then
            echo "  ✅ ARM64: $(du -h "$ARM_BIN" | cut -f1)"
        else
            echo "  ⚠️  ARM64 binary not found"
        fi
    else
        echo "  ❌ ARM64 build failed"
    fi
    
    echo ""
done

echo "═══════════════════════════════════════════════════════════════════"
echo "📦 Creating v4.1 genomeBins..."
echo "═══════════════════════════════════════════════════════════════════"
echo ""

# Step 2: Create v4.1 genomes for primals with both binaries
cd "$PHASE2_DIR"

for primal in "${PRIMALS[@]}"; do
    X86_BIN="$PHASE1_DIR/$primal/target/x86_64-unknown-linux-musl/release/$primal"
    ARM_BIN="$PHASE1_DIR/$primal/target/aarch64-unknown-linux-musl/release/$primal"
    
    if [ -f "$X86_BIN" ] && [ -f "$ARM_BIN" ]; then
        echo "🧬 Creating $primal.genome (v4.1 multi-arch)..."
        
        cargo run --release -p biomeos-cli --bin biomeos -- genome create "$primal" \
            --binary x86_64="$X86_BIN" \
            --binary aarch64="$ARM_BIN" \
            --extractor-arches x86_64,aarch64 \
            --version "1.0.0" \
            --description "Production $primal - Multi-Arch v4.1" \
            2>&1 | grep -E "(✅|Format|Size|Architectures|Path)"
        
        echo ""
    else
        echo "⏭️  Skipping $primal.genome (missing binaries)"
        [ ! -f "$X86_BIN" ] && echo "   Missing: x86_64"
        [ ! -f "$ARM_BIN" ] && echo "   Missing: ARM64"
        echo ""
    fi
done

echo "═══════════════════════════════════════════════════════════════════"
echo "✅ Build Complete!"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "Location: $PLASMIDBIN/"
ls -lh "$PLASMIDBIN/"*.genome 2>/dev/null | grep -E "$(date +%b.*%d)" | awk '{print "  ", $9, "-", $5}'
echo ""
echo "🧬 Ready for deployment to liveSpore USB + Pixel 8a!"
echo ""
