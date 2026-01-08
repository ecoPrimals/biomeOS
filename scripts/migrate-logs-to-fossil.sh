#!/usr/bin/env bash
# 
# Migrate existing logs to fossil record structure
#
# This script:
# 1. Identifies stale log files in /tmp/primals/
# 2. Archives them to /var/biomeos/logs/fossil/
# 3. Cleans up /tmp/primals/
#
# Usage:
#   ./scripts/migrate-logs-to-fossil.sh [--dry-run]

set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "🔍 DRY RUN MODE - No files will be moved"
    echo ""
fi

# Configuration
BIOMEOS_CLI="./target/release/biomeos"
OLD_LOG_DIR="/tmp/primals"
FOSSIL_DIR="/var/biomeos/logs/fossil/legacy"

echo "🔄 Log Migration to Fossil Record"
echo "═══════════════════════════════════════"
echo ""

# Check if biomeOS CLI exists
if [[ ! -f "$BIOMEOS_CLI" ]]; then
    echo "⚠️  biomeOS CLI not found at: $BIOMEOS_CLI"
    echo "   Building biomeOS CLI..."
    cargo build --release -p biomeos-cli --bin biomeos-cli
fi

# Check if old log directory exists
if [[ ! -d "$OLD_LOG_DIR" ]]; then
    echo "✅ No old logs found at: $OLD_LOG_DIR"
    exit 0
fi

# Count log files
LOG_COUNT=$(find "$OLD_LOG_DIR" -maxdepth 1 -name "*.log" -type f 2>/dev/null | wc -l)

if [[ "$LOG_COUNT" -eq 0 ]]; then
    echo "✅ No log files found to migrate"
    exit 0
fi

echo "Found $LOG_COUNT log file(s) to migrate"
echo ""

# Create fossil directory structure
if [[ "$DRY_RUN" == "false" ]]; then
    mkdir -p "$FOSSIL_DIR"
    echo "✅ Created fossil directory: $FOSSIL_DIR"
fi

# Migrate each log file
for log_file in "$OLD_LOG_DIR"/*.log; do
    if [[ ! -f "$log_file" ]]; then
        continue
    fi
    
    filename=$(basename "$log_file")
    dest="$FOSSIL_DIR/$filename"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  Would migrate: $filename"
    else
        mv "$log_file" "$dest"
        echo "  ✅ Migrated: $filename"
    fi
done

echo ""

if [[ "$DRY_RUN" == "false" ]]; then
    echo "✅ Migration complete!"
    echo ""
    echo "📊 Summary:"
    echo "   Migrated: $LOG_COUNT log file(s)"
    echo "   Location: $FOSSIL_DIR"
    echo ""
    echo "💡 To browse fossil logs:"
    echo "   $BIOMEOS_CLI fossil fossil --limit 10"
else
    echo "🔍 Dry run complete - run without --dry-run to execute"
fi

