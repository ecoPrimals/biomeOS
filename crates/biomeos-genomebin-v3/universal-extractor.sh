#!/bin/sh
# Universal genomeBin Self-Extractor v1.0
# Temporary solution - evolving to Pure Rust binary
# Works on: Linux (x86_64, ARM64), macOS, BSD, Android

set -e

SELF="$0"
ARCH=$(uname -m)

# Normalize architecture
case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    riscv64) ARCH="riscv64" ;;
    *) echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

# Command handling
CMD="${1:-info}"

case "$CMD" in
    info)
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "🧬 genomeBin v3.5-universal Information"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "Current Architecture: $ARCH"
        echo ""
        # Extract first 8KB after script (manifest area) and parse JSON
        tail -c +__SCRIPT_SIZE__ "$SELF" 2>/dev/null | head -c 8192 | grep -E '(name|version|description|architectures)' | sed 's/^[ \t]*//' || {
            echo "genomeBin v3.5-universal (shell wrapper)"
            echo "Evolving to Pure Rust v4 for genomic fingerprint"
        }
        ;;
    
    extract)
        OUTPUT_DIR="${2:-.}"
        BINARY_NAME=$(basename "$SELF" .genome)
        
        mkdir -p "$OUTPUT_DIR"
        echo "Extracting $ARCH binary to: $OUTPUT_DIR/$BINARY_NAME"
        
        # Calculate payload start: script size + 8KB manifest
        SCRIPT_SIZE=__SCRIPT_SIZE__
        MANIFEST_SIZE=8192
        PAYLOAD_START=$((SCRIPT_SIZE + MANIFEST_SIZE))
        
        # Extract architecture-specific binary (simplified - uses first binary)
        # Real implementation needs manifest offset parsing
        tail -c +$PAYLOAD_START "$SELF" 2>/dev/null | zstd -d > "$OUTPUT_DIR/$BINARY_NAME" 2>/dev/null || {
            echo "Error: Extraction failed for $ARCH" >&2
            echo "This is a simplified extractor - evolving to Pure Rust v4" >&2
            exit 1
        }
        
        chmod +x "$OUTPUT_DIR/$BINARY_NAME"
        echo "✅ Extracted: $OUTPUT_DIR/$BINARY_NAME"
        ;;
    
    run)
        shift
        TEMP_DIR=$(mktemp -d)
        trap "rm -rf $TEMP_DIR" EXIT
        
        # Extract to temp and execute
        "$SELF" extract "$TEMP_DIR" > /dev/null || exit 1
        BINARY_NAME=$(basename "$SELF" .genome)
        exec "$TEMP_DIR/$BINARY_NAME" "$@"
        ;;
    
    *)
        echo "Usage: $SELF {info|extract [DIR]|run [ARGS...]}"
        exit 1
        ;;
esac

exit 0

__PAYLOAD_BEGIN__
