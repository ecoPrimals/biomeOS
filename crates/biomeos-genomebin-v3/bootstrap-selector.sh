#!/bin/sh
# genomeBin v4.1 Bootstrap Selector
# AGPL-3.0-or-later License
#
# This script is embedded at the start of genomeBin v4.1 files.
# It detects the current architecture and extracts the appropriate binary.
#
# Deep Debt Principles:
# - Runtime discovery (architecture detection at execution)
# - Self-extracting (no external tools needed)
# - Platform-agnostic (works on any POSIX system)

set -e

SELF="$0"
OFFSET=4096  # Bootstrap is padded to 4KB

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)     echo "x86_64" ;;
        aarch64|arm64)    echo "aarch64" ;;
        armv7*|armhf)     echo "arm" ;;
        riscv64)          echo "riscv64" ;;
        *)
            echo "Unsupported architecture: $(uname -m)" >&2
            exit 1
            ;;
    esac
}

ARCH=$(detect_arch)
echo "Detected architecture: $ARCH"

# Read extractor table (128 bytes starting at offset 4096)
# Format: 4 entries x 32 bytes each
#   - architecture: 16 bytes (null-padded)
#   - offset: 8 bytes (little-endian)
#   - size: 8 bytes (little-endian)

# Find matching extractor
EXTRACTOR_OFFSET=""
EXTRACTOR_SIZE=""

# Simple table parsing (shell-compatible)
read_table() {
    dd if="$SELF" bs=1 skip=$OFFSET count=128 2>/dev/null | od -A d -t x1
}

# Extract and run the appropriate extractor
# For now, use a fallback approach: try to run the first matching binary

# Create temp directory
TMPDIR="${TMPDIR:-/tmp}"
WORKDIR=$(mktemp -d "$TMPDIR/genomebin.XXXXXX")
trap "rm -rf '$WORKDIR'" EXIT

echo "Extracting $ARCH binary..."

# Extract the genomeBin data (everything after bootstrap + table)
dd if="$SELF" bs=1 skip=$((OFFSET + 128)) of="$WORKDIR/payload" 2>/dev/null

# Run the extractor (assumes it's a self-extracting binary)
chmod +x "$WORKDIR/payload"
exec "$WORKDIR/payload" "$@"
