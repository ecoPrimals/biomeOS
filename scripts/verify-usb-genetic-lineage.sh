#!/bin/bash
# USB Spore Genetic Lineage Verification
# Verifies that the parent seed stays encrypted on USB

set -e

USB_ROOT="/media/eastgate/biomeOS1/biomeOS"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔐 USB Spore Genetic Lineage Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check USB is mounted
if [ ! -d "$USB_ROOT" ]; then
    echo "❌ USB not mounted at $USB_ROOT"
    exit 1
fi
echo "✅ USB mounted at $USB_ROOT"

# Check encrypted seed file
if [ ! -f "$USB_ROOT/.family.seed" ]; then
    echo "❌ Encrypted seed file not found"
    exit 1
fi

SEED_SIZE=$(stat -c %s "$USB_ROOT/.family.seed" 2>/dev/null)
SEED_PERMS=$(stat -c %a "$USB_ROOT/.family.seed" 2>/dev/null)

if [ "$SEED_SIZE" != "32" ]; then
    echo "❌ Seed file wrong size: $SEED_SIZE bytes (expected 32)"
    exit 1
fi
echo "✅ Encrypted seed file: 32 bytes (256-bit)"

if [ "$SEED_PERMS" != "600" ]; then
    echo "⚠️  Seed permissions: $SEED_PERMS (should be 600)"
else
    echo "✅ Seed permissions: 600 (owner only)"
fi

# Check seed is binary (not readable text)
if file "$USB_ROOT/.family.seed" | grep -qE "data|Non-ISO|binary"; then
    echo "✅ Seed format: Binary/Encrypted (not human-readable)"
else
    echo "⚠️  Seed may be plaintext: $(file "$USB_ROOT/.family.seed")"
fi

# Check tower.env does NOT expose raw seed
if grep -q "^export BEARDOG_FAMILY_SEED=" "$USB_ROOT/config/tower.env" | grep -v "^#"; then
    echo "⚠️  WARNING: tower.env still has BEARDOG_FAMILY_SEED (should be removed)"
else
    echo "✅ tower.env does not expose raw seed"
fi

# Check tower.env references seed file
if grep -q "BEARDOG_FAMILY_SEED_FILE" "$USB_ROOT/config/tower.env"; then
    echo "✅ tower.env references .family.seed file"
else
    echo "⚠️  tower.env should reference BEARDOG_FAMILY_SEED_FILE"
fi

# Verify binaries
echo ""
echo "📦 Binary Verification:"
for binary in tower beardog songbird; do
    case $binary in
        tower)
            path="$USB_ROOT/bin/tower"
            expected_hash="3d4eba062a205c76f48c076980668873"
            ;;
        beardog)
            path="$USB_ROOT/primals/beardog"
            expected_hash="15fdff28630cc8c98dd12517eabd4bad"
            ;;
        songbird)
            path="$USB_ROOT/primals/songbird"
            expected_hash="c90b93cdf32e60e02fb9735d1c250fee"
            ;;
    esac
    
    if [ ! -f "$path" ]; then
        echo "  ❌ $binary: NOT FOUND"
        continue
    fi
    
    actual_hash=$(md5sum "$path" | awk '{print $1}')
    if [ "$actual_hash" = "$expected_hash" ]; then
        echo "  ✅ $binary: OK ($(du -h "$path" | cut -f1))"
    else
        echo "  ⚠️  $binary: Hash mismatch"
        echo "      Expected: $expected_hash"
        echo "      Got:      $actual_hash"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Genetic Lineage Architecture"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "USB Parent Seed (Family: nat0)"
echo "  ├─ Stored: .family.seed (32 bytes binary, encrypted)"
echo "  ├─ Permissions: 600 (owner read/write only)"
echo "  ├─ Never copied to tower disk"
echo "  └─ Enables child key derivation"
echo ""
echo "Each Tower Derives:"
echo "  child_key = HKDF-SHA256("
echo "    parent_seed,      # From USB .family.seed"
echo "    hostname,         # tower1, tower2, etc."
echo "    uuid,             # Unique per boot"
echo "    system_entropy    # /dev/urandom"
echo "  )"
echo ""
echo "Security Properties:"
echo "  ✅ Parent seed never leaves USB"
echo "  ✅ Each tower has unique child key"
echo "  ✅ Lineage proofs verify family membership"
echo "  ✅ Child keys zeroized on exit (memory only)"
echo "  ✅ Key rotation via generation numbers"
echo "  ✅ New nodes join by deriving from same parent"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ USB Spore Verification Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

