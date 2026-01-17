#!/usr/bin/env bash
# Enable multi_thread flavor on all tokio::test annotations
# This enables true concurrent testing across the codebase

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 Enabling concurrent testing across biomeOS..."
echo ""

# Count before
BEFORE=$(grep -r "^#\[tokio::test\]$" "$PROJECT_ROOT/crates" "$PROJECT_ROOT/tests" 2>/dev/null | wc -l || echo 0)
echo "📊 Tests to convert: $BEFORE"

# Find all Rust files with simple #[tokio::test] and convert them
# Exclude archive, target, and already-converted files
find "$PROJECT_ROOT/crates" "$PROJECT_ROOT/tests" -name "*.rs" -type f \
    -not -path "*/target/*" \
    -not -path "*/archive/*" \
    2>/dev/null | while read -r file; do
    
    # Check if file has simple tokio::test annotations
    if grep -q "^#\[tokio::test\]$" "$file" 2>/dev/null; then
        echo "  🔧 Converting: ${file#$PROJECT_ROOT/}"
        
        # Use sed to replace simple #[tokio::test] with multi_thread version
        # -i.bak creates backup, then we remove it
        sed -i.bak 's/^#\[tokio::test\]$/#[tokio::test(flavor = "multi_thread", worker_threads = 4)]/' "$file"
        rm -f "$file.bak"
    fi
done

echo ""

# Count after
AFTER=$(grep -r "^#\[tokio::test\]$" "$PROJECT_ROOT/crates" "$PROJECT_ROOT/tests" 2>/dev/null | wc -l || true)
AFTER=${AFTER:-0}
CONVERTED=$((BEFORE - AFTER))

echo "✅ Conversion complete!"
echo "   Before: $BEFORE"
echo "   After:  $AFTER"
echo "   Converted: $CONVERTED"
echo ""

# Verify it compiles
echo "🔨 Verifying workspace builds..."
cd "$PROJECT_ROOT"
if cargo build --workspace --tests 2>&1 | tail -5; then
    echo ""
    echo "✅ Workspace builds successfully with concurrent tests!"
else
    echo ""
    echo "⚠️  Build issues detected - review errors above"
    exit 1
fi

echo ""
echo "🎯 Next: Run test suite with 'cargo test --workspace'"

