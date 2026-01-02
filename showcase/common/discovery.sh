#!/usr/bin/env bash
# BiomeOS Runtime Discovery Library
# Created: December 31, 2025
# Purpose: Runtime primal discovery (NO HARDCODED ENDPOINTS)

# Discover primal binary
discover_primal_bin() {
    local primal=$1
    
    # Check multiple possible locations
    # Location 1: phase2/primalBins (from biomeOS/showcase)
    if [ -f "../../../../primalBins/$primal" ]; then
        realpath "../../../../primalBins/$primal"
        return 0
    fi
    
    # Location 2: ecoPrimals/primalBins (alternative path)
    if [ -f "../../../../../primalBins/$primal" ]; then
        realpath "../../../../../primalBins/$primal"
        return 0
    fi
    
    # Location 3: Check PATH
    which "$primal" 2>/dev/null || echo ""
}

# Check if primal binary exists
primal_exists() {
    local primal=$1
    [ -n "$(discover_primal_bin "$primal")" ]
}

# List available primals
list_available_primals() {
    echo "Available primals:"
    local found=0
    
    # Check location 1
    if [ -d "../../../../primalBins" ]; then
        echo "  From ../../../../primalBins/:"
        ls -1 ../../../../primalBins/ 2>/dev/null | grep -v "README" | grep -v ".md" | sed 's/^/    /' || true
        found=1
    fi
    
    # Check location 2
    if [ -d "../../../../../primalBins" ]; then
        echo "  From ../../../../../primalBins/:"
        ls -1 ../../../../../primalBins/ 2>/dev/null | grep -v "README" | grep -v ".md" | sed 's/^/    /' || true
        found=1
    fi
    
    if [ $found -eq 0 ]; then
        echo "  (none found - check path from: $(pwd))"
    fi
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
