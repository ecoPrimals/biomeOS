#!/usr/bin/env bash
#
# Verify Nucleus - Check integrity of plasmidBin binaries
#
# This script verifies:
# 1. All binaries exist
# 2. All binaries are executable
# 3. All binaries are valid ELF files
# 4. Checksums are recorded
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

NUCLEUS_PATH="$BIOMEOS_ROOT/plasmidBin"

# Check if plasmidBin exists
if [ ! -d "$NUCLEUS_PATH" ]; then
    echo "❌ plasmidBin/ not found!"
    exit 1
fi

echo "🔍 Verifying nucleus integrity..."
echo ""

ALL_VALID=true

# Verify tower
echo "📋 Tower:"
for bin in "$NUCLEUS_PATH/tower"/*; do
    # Skip .gitkeep
    [ "$(basename "$bin")" = ".gitkeep" ] && continue
    
    if [ -f "$bin" ]; then
        if [ -x "$bin" ]; then
            if file "$bin" | grep -q "ELF"; then
                md5=$(md5sum "$bin" | cut -d' ' -f1)
                size=$(ls -lh "$bin" | awk '{print $5}')
                echo "  ✅ $(basename "$bin") ($size) [MD5: ${md5:0:16}...]"
            else
                echo "  ❌ $(basename "$bin") - Not a valid ELF binary!"
                ALL_VALID=false
            fi
        else
            echo "  ❌ $(basename "$bin") - Not executable!"
            ALL_VALID=false
        fi
    fi
done

# Verify primals
echo ""
echo "📋 Primals:"
for bin in "$NUCLEUS_PATH/primals"/*; do
    # Skip .gitkeep
    [ "$(basename "$bin")" = ".gitkeep" ] && continue
    
    if [ -f "$bin" ]; then
        if [ -x "$bin" ]; then
            if file "$bin" | grep -q "ELF"; then
                md5=$(md5sum "$bin" | cut -d' ' -f1)
                size=$(ls -lh "$bin" | awk '{print $5}')
                echo "  ✅ $(basename "$bin") ($size) [MD5: ${md5:0:16}...]"
            else
                echo "  ❌ $(basename "$bin") - Not a valid ELF binary!"
                ALL_VALID=false
            fi
        else
            echo "  ❌ $(basename "$bin") - Not executable!"
            ALL_VALID=false
        fi
    fi
done

echo ""

if [ "$ALL_VALID" = true ]; then
    echo "✅ All binaries verified successfully!"
    exit 0
else
    echo "❌ Some binaries failed verification!"
    exit 1
fi

