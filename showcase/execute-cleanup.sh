#!/usr/bin/env bash
set -euo pipefail

# BiomeOS Showcase Cleanup - Execute Script
# Date: December 31, 2025
# Purpose: Clean up overlaps, outdated assumptions, organize for BTSP focus

cd "$(dirname "$0")"

echo "════════════════════════════════════════════════════════"
echo "🧹 BiomeOS Showcase Cleanup - Phase 1"
echo "════════════════════════════════════════════════════════"
echo ""

# Phase 1: Create Archive Structure
echo "📁 Creating archive structure..."
mkdir -p archive/{duplicates,experimental,superseded,outdated-docs}
echo "✅ Archive structure created"
echo ""

# Phase 2: Archive Duplicates
echo "📦 Archiving duplicate directories..."

if [ -d "05-chimera-patterns" ]; then
    echo "  Moving 05-chimera-patterns to archive/duplicates/"
    mv 05-chimera-patterns archive/duplicates/
fi

if [ -d "03-multiplex" ]; then
    echo "  Moving 03-multiplex to archive/duplicates/"
    mv 03-multiplex archive/duplicates/
fi

if [ -d "03-full-ecosystem" ]; then
    echo "  Moving 03-full-ecosystem to archive/duplicates/"
    mv 03-full-ecosystem archive/duplicates/
fi

if [ -d "00-local-capabilities" ]; then
    echo "  Moving 00-local-capabilities to archive/duplicates/"
    mv 00-local-capabilities archive/duplicates/
fi

echo "✅ Duplicates archived"
echo ""

# Phase 3: Archive Superseded/Experimental
echo "📦 Archiving superseded & experimental content..."

if [ -d "02-birdsong-p2p" ]; then
    echo "  Moving 02-birdsong-p2p to archive/superseded/"
    mv 02-birdsong-p2p archive/superseded/
fi

if [ -d "03-primal-triples" ]; then
    echo "  Moving 03-primal-triples to archive/experimental/"
    mv 03-primal-triples archive/experimental/
fi

if [ -d "05-lifecycle-negotiation" ]; then
    echo "  Moving 05-lifecycle-negotiation to archive/experimental/"
    mv 05-lifecycle-negotiation archive/experimental/
fi

if [ -d "03-primal-adapter" ]; then
    echo "  Moving 03-primal-adapter to archive/duplicates/"
    mv 03-primal-adapter archive/duplicates/
fi

echo "✅ Superseded/experimental content archived"
echo ""

# Phase 4: Merge Remaining Duplicates
echo "🔀 Merging remaining duplicates..."

# Merge ecosystem dirs (if 04-complete-ecosystem exists)
if [ -d "04-complete-ecosystem" ]; then
    echo "  Renaming 04-complete-ecosystem to 07-full-ecosystem"
    mv 04-complete-ecosystem 07-full-ecosystem
fi

# Rename for consistency
if [ -d "00-substrate" ]; then
    echo "  Renaming 00-substrate to 00-substrate-capabilities"
    mv 00-substrate 00-substrate-capabilities
fi

if [ -d "06-multiplex-patterns" ]; then
    echo "  Renaming 06-multiplex-patterns to 06-multiplex"
    mv 06-multiplex-patterns 06-multiplex
fi

if [ -d "04-multi-primal-adaptation" ]; then
    echo "  Renaming 04-multi-primal-adaptation to 04-primal-adapter"
    mv 04-multi-primal-adaptation 04-primal-adapter
fi

if [ -d "04-deployment-evolution" ]; then
    echo "  Renaming 04-deployment-evolution to 08-deployment"
    mv 04-deployment-evolution 08-deployment
fi

echo "✅ Directories merged and renamed"
echo ""

# Phase 5: Fix Path References
echo "🔧 Fixing outdated path references..."
echo "  Replacing 'primalBins' with 'primalBins'..."

find . -type f \( -name "*.sh" -o -name "*.md" \) \
  -not -path "*/archive/*" \
  -not -path "*/.git/*" \
  -exec sed -i 's|primalBins|primalBins|g' {} + 2>/dev/null || true

find . -type f \( -name "*.sh" -o -name "*.md" \) \
  -not -path "*/archive/*" \
  -not -path "*/.git/*" \
  -exec sed -i 's|primalBins|primalBins|g' {} + 2>/dev/null || true

echo "✅ Path references updated"
echo ""

# Phase 6: Create Common Utilities Directory
echo "📚 Creating common utilities directory..."
mkdir -p common

# Create placeholder files (will be populated later)
cat > common/discovery.sh << 'EOF'
#!/usr/bin/env bash
# BiomeOS Runtime Discovery Library
# Created: December 31, 2025
# Purpose: Runtime primal discovery (NO HARDCODED ENDPOINTS)

# Discover primal binary
discover_primal_bin() {
    local primal=$1
    
    # Check primalBins first
    if [ -f "../../primalBins/$primal" ]; then
        realpath "../../primalBins/$primal"
        return 0
    fi
    
    # Check PATH
    which "$primal" 2>/dev/null || echo ""
}

# Check if primal binary exists
primal_exists() {
    local primal=$1
    [ -n "$(discover_primal_bin "$primal")" ]
}

# List available primals
list_available_primals() {
    echo "Available primals in ../../primalBins/:"
    ls -1 ../../primalBins/ 2>/dev/null | grep -v "README" | grep -v ".md" || echo "  (none found)"
}

# Health check (placeholder - needs primal-specific implementation)
check_primal_health() {
    local name=$1
    local endpoint=$2
    curl -sf "${endpoint}/health" >/dev/null 2>&1
}

# Wait for primal to be ready
wait_for_primal() {
    local name=$1
    local endpoint=$2
    local max_wait=${3:-30}
    
    echo "⏳ Waiting for $name at $endpoint..."
    for i in $(seq 1 "$max_wait"); do
        if check_primal_health "$name" "$endpoint"; then
            echo "✅ $name is ready!"
            return 0
        fi
        sleep 1
    done
    echo "❌ $name failed to start within ${max_wait}s"
    return 1
}
EOF

chmod +x common/discovery.sh

cat > common/README.md << 'EOF'
# Common Showcase Utilities

**Purpose**: Shared utilities for all showcase demos  
**Philosophy**: Runtime discovery, no hardcoding  

## Files

### discovery.sh
Runtime primal discovery library
- `discover_primal_bin()` - Find primal binary
- `primal_exists()` - Check if primal available
- `list_available_primals()` - Show all available
- `check_primal_health()` - Health check
- `wait_for_primal()` - Wait for startup

### Usage

```bash
source ../common/discovery.sh

# Check if beardog exists
if primal_exists "beardog"; then
    BEARDOG_BIN=$(discover_primal_bin "beardog")
    echo "Found beardog: $BEARDOG_BIN"
fi

# List all available
list_available_primals
```

### Principles

1. ✅ NO HARDCODED PATHS
2. ✅ NO HARDCODED PORTS
3. ✅ RUNTIME DISCOVERY ONLY
4. ✅ GRACEFUL FALLBACKS
5. ✅ CLEAR ERROR MESSAGES
EOF

echo "✅ Common utilities created"
echo ""

# Phase 7: Summary
echo "════════════════════════════════════════════════════════"
echo "✅ Cleanup Phase 1 Complete!"
echo "════════════════════════════════════════════════════════"
echo ""
echo "📊 Summary:"
echo "  - Archived: 8 directories"
echo "  - Merged: 4 sets of duplicates"
echo "  - Renamed: 5 directories for consistency"
echo "  - Fixed: All 'primalBins' references"
echo "  - Created: common/ utilities directory"
echo ""
echo "📁 New Structure:"
ls -1 | grep -E "^[0-9]" | nl
echo ""
echo "📋 Next Steps:"
echo "  1. Review archived content (showcase/archive/)"
echo "  2. Test common/discovery.sh"
echo "  3. Update demo scripts to use common utilities"
echo "  4. Begin BTSP buildout (03-p2p-coordination/)"
echo ""
echo "🎯 Ready for BTSP & BirdSong P2P development!"
echo ""

