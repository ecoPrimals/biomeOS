#!/bin/bash
# Temporary Genome Builder (Shell Script - Initial Solution)
# TODO: Evolve to Pure Rust in biomeos-cli/src/commands/genome.rs
#
# Usage: ./scripts/build-genome.sh <primal_name> <version> <description>
#
# This script is JELLY - it will break somewhere.
# Rust compiles to binary, shell is interpretive.
# We treat binary as a genomic solution.
# The strings of 1 and 0 are non-arbitrary, they are a fingerprint like DNA in nature.
#
# AGPL-3.0-only License

set -euo pipefail

# Arguments
PRIMAL_NAME="${1:-}"
VERSION="${2:-v1.0.0}"
DESCRIPTION="${3:-Universal Primal}"

if [ -z "$PRIMAL_NAME" ]; then
    echo "Usage: $0 <primal_name> <version> <description>"
    echo "Example: $0 songbird v3.33.0 'Songbird HTTP & Discovery'"
    exit 1
fi

# Detect project root
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PHASE1_ROOT="$(cd "$PROJECT_ROOT/../../phase1" && pwd)"

# Paths
PRIMAL_DIR="$PHASE1_ROOT/$PRIMAL_NAME"
X86_BINARY="$PRIMAL_DIR/target/release/$PRIMAL_NAME"
ARM_BINARY="$PRIMAL_DIR/target/aarch64-unknown-linux-musl/release/$PRIMAL_NAME"
OUTPUT_GENOME="$PROJECT_ROOT/plasmidBin/$PRIMAL_NAME.genome"
TEMPLATE="$SCRIPT_DIR/genomeBin-hardened-template.sh"

echo "🧬 Building genomeBin for: $PRIMAL_NAME v$VERSION"
echo "   Description: $DESCRIPTION"
echo ""

# Verify binaries exist
if [ ! -f "$X86_BINARY" ]; then
    echo "❌ x86_64 binary not found: $X86_BINARY"
    exit 1
fi

if [ ! -f "$ARM_BINARY" ]; then
    echo "❌ aarch64 binary not found: $ARM_BINARY"
    exit 1
fi

echo "✅ Found x86_64 binary: $(stat -c%s "$X86_BINARY" | numfmt --to=iec-i)"
echo "✅ Found aarch64 binary: $(stat -c%s "$ARM_BINARY" | numfmt --to=iec-i)"
echo ""

# Calculate checksums
echo "🔐 Calculating checksums..."
X86_CHECKSUM=$(sha256sum "$X86_BINARY" | awk '{print $1}')
ARM_CHECKSUM=$(sha256sum "$ARM_BINARY" | awk '{print $1}')

echo "   x86_64:  $X86_CHECKSUM"
echo "   aarch64: $ARM_CHECKSUM"
echo ""

# Create temporary directory for archive
TEMP_DIR=$(mktemp -d)
trap "rm -rf '$TEMP_DIR'" EXIT

mkdir -p "$TEMP_DIR/x86_64" "$TEMP_DIR/aarch64"
cp "$X86_BINARY" "$TEMP_DIR/x86_64/$PRIMAL_NAME"
cp "$ARM_BINARY" "$TEMP_DIR/aarch64/$PRIMAL_NAME"

echo "📦 Packaging genomeBin..."

# Create genome by combining template + tar archive
{
    # Copy template and replace placeholders
    sed -e "s/REPLACE_WITH_PRIMAL_NAME/$PRIMAL_NAME/g" \
        -e "s/REPLACE_WITH_VERSION/$VERSION/g" \
        -e "s|REPLACE_WITH_DESCRIPTION|$DESCRIPTION|g" \
        -e "s/REPLACE_WITH_SHA256 *$/SHA256_x86_64=\"$X86_CHECKSUM\"/g" \
        "$TEMPLATE" | \
    sed "s/CHECKSUM_x86_64=\"REPLACE_WITH_SHA256\"/CHECKSUM_x86_64=\"$X86_CHECKSUM\"/" | \
    sed "s/CHECKSUM_aarch64=\"REPLACE_WITH_SHA256\"/CHECKSUM_aarch64=\"$ARM_CHECKSUM\"/"
    
    # Append archive
    cd "$TEMP_DIR" && tar czf - x86_64 aarch64
} > "$OUTPUT_GENOME"

chmod +x "$OUTPUT_GENOME"

GENOME_SIZE=$(stat -c%s "$OUTPUT_GENOME" | numfmt --to=iec-i)

echo ""
echo "✅ genomeBin created: $OUTPUT_GENOME"
echo "   Size: $GENOME_SIZE"
echo "   Architectures: x86_64, aarch64"
echo "   x86_64 checksum:  $X86_CHECKSUM"
echo "   aarch64 checksum: $ARM_CHECKSUM"
echo ""
echo "🧬 genomeBin ready for deployment!"
echo ""
echo "Deploy to:"
echo "  USB:   cp $OUTPUT_GENOME /media/\$USER/biomeOS1/"
echo "  Pixel: adb push $OUTPUT_GENOME /data/local/tmp/"
echo ""
echo "TODO: Evolve to Pure Rust in biomeos-cli genome build command"
echo "      Shell is jelly script - Rust is genomic binary solution"
