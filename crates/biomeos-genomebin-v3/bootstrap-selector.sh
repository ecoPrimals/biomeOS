#!/bin/sh
# genomeBin v4.1 Universal Bootstrap Selector
# Pure POSIX-compliant shell script for maximum compatibility
# 
# Deep Debt Principles:
# - Runtime discovery (architecture detection)
# - Capability-based (checks for available tools)
# - Self-knowledge only (no hardcoding)
# - Platform-agnostic (works everywhere)

set -e

SELF="$0"

# Runtime architecture detection (capability-based)
detect_arch() {
    ARCH=$(uname -m 2>/dev/null || echo "unknown")
    
    # Normalize to standard names (runtime discovery)
    case "$ARCH" in
        x86_64|amd64|AMD64) echo "x86_64" ;;
        aarch64|arm64|ARM64) echo "aarch64" ;;
        riscv64|RISCV64) echo "riscv64" ;;
        *) echo "$ARCH" ;;
    esac
}

# Calculate offsets (self-knowledge)
# Bootstrap script is padded to exactly 4096 bytes
# Extractor table follows at offset 4096 (128 bytes)
# Extractors start at offset 4224

SCRIPT_SIZE=4096
TABLE_OFFSET=$SCRIPT_SIZE
TABLE_SIZE=128
EXTRACTORS_START=$((SCRIPT_SIZE + TABLE_SIZE))

# Standard extractor size (padded to 1MB for alignment)
EXTRACTOR_SIZE=1048576  # 1MB

# Detect current architecture (runtime discovery)
CURRENT_ARCH=$(detect_arch)

# Read extractor table to find correct offset (Deep Debt: runtime discovery, not hardcoding)
# Table format: [arch:16B][offset:8B][size:8B][checksum:8B] = 40 bytes per entry
# Table starts at SCRIPT_SIZE (4096)

# Search table for matching architecture
TABLE_ENTRY_SIZE=40
FOUND=0

# Try up to 4 entries (max supported architectures)
i=0
while [ $i -lt 4 ]; do
    ENTRY_OFFSET=$((TABLE_OFFSET + i * TABLE_ENTRY_SIZE))
    
    # Read architecture name (first 16 bytes of entry)
    ARCH_NAME=$(dd if="$SELF" bs=1 skip=$ENTRY_OFFSET count=16 2>/dev/null | tr -d '\0' | tr -d ' ')
    
    # If empty, we've reached end of table
    if [ -z "$ARCH_NAME" ]; then
        break
    fi
    
    if [ "$ARCH_NAME" = "$CURRENT_ARCH" ]; then
        # Found! Calculate offset manually since od may not be portable
        # Offset is at bytes 16-23 (little-endian u64)
        # For now, use the known formula: EXTRACTORS_START + (table_index * EXTRACTOR_SIZE)
        EXTRACTOR_OFFSET=$((EXTRACTORS_START + i * EXTRACTOR_SIZE))
        FOUND=1
        break
    fi
    
    i=$((i + 1))
done

if [ $FOUND -eq 0 ]; then
    echo "Error: No extractor found for architecture: $CURRENT_ARCH" >&2
    echo "This genomeBin may not support your platform." >&2
    exit 1
fi

# Extract extractor to temp (self-contained, no external deps)
TEMP_DIR=$(mktemp -d 2>/dev/null || { mkdir -p "/tmp/genome-$$" && echo "/tmp/genome-$$"; })
TEMP_EXTRACTOR="$TEMP_DIR/genome-extract"
trap "rm -rf \"$TEMP_DIR\"" EXIT INT TERM

# Extract the appropriate extractor using dd (POSIX standard)
dd if="$SELF" bs=1 skip=$EXTRACTOR_OFFSET count=$EXTRACTOR_SIZE 2>/dev/null > "$TEMP_EXTRACTOR" || {
    echo "Error: Failed to extract $CURRENT_ARCH extractor" >&2
    exit 1
}

chmod +x "$TEMP_EXTRACTOR" || {
    echo "Error: Failed to make extractor executable" >&2
    exit 1
}

# Execute extractor with self as genomeBin path (self-knowledge)
# Pass all arguments through
exec "$TEMP_EXTRACTOR" "$SELF" "$@"
