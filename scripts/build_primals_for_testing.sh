#!/bin/bash
# =============================================================================
# Build All Primals for NUCLEUS Integration Testing
# =============================================================================
#
# Builds all required primals in release mode:
# - BearDog (security/crypto)
# - Songbird (discovery/networking)
# - NestGate (storage/persistence)
# - Toadstool (compute/GPU)
# - Squirrel (AI/MCP)
#
# Created: January 29, 2026
# =============================================================================

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Try to locate primal directories
PRIMALS_BASE="$PROJECT_ROOT/.."
if [ -d "$PROJECT_ROOT/../../primals" ]; then
    PRIMALS_BASE="$PROJECT_ROOT/../../primals"
fi

log_section "Building Primals for NUCLEUS Integration"

log_info "Project root: $PROJECT_ROOT"
log_info "Primals base: $PRIMALS_BASE"

# List of primals to build
PRIMALS=("beardog" "songbird" "nestgate" "toadstool" "squirrel")

for PRIMAL in "${PRIMALS[@]}"; do
    log_info "Building $PRIMAL..."
    
    # Try multiple possible locations
    PRIMAL_DIR=""
    for LOCATION in "$PRIMALS_BASE/$PRIMAL" "$PROJECT_ROOT/../$PRIMAL" "$PROJECT_ROOT/../../$PRIMAL"; do
        if [ -d "$LOCATION" ] && [ -f "$LOCATION/Cargo.toml" ]; then
            PRIMAL_DIR="$LOCATION"
            break
        fi
    done
    
    if [ -z "$PRIMAL_DIR" ]; then
        log_warning "  ! $PRIMAL directory not found, skipping"
        continue
    fi
    
    log_info "  Found: $PRIMAL_DIR"
    
    # Build
    cd "$PRIMAL_DIR"
    
    log_info "  Running cargo build --release..."
    if cargo build --release --quiet 2>&1 | tail -5; then
        if [ -f "target/release/$PRIMAL" ]; then
            BINARY_SIZE=$(ls -lh "target/release/$PRIMAL" | awk '{print $5}')
            log_success "  ✓ $PRIMAL built successfully ($BINARY_SIZE)"
        else
            log_warning "  ! $PRIMAL binary not found after build"
        fi
    else
        log_warning "  ! $PRIMAL build failed"
    fi
    
    cd "$PROJECT_ROOT"
done

log_section "Build Complete"

log_success "Primals ready for integration testing"
log_info ""
log_info "Next steps:"
echo "  1. Run integration test:"
echo "     ./scripts/test_nucleus_ai_integration.sh"
echo ""
echo "  2. Or start primals manually and test:"
echo "     cd ../beardog && cargo run --release -- server &"
echo "     cd ../songbird && cargo run --release -- server &"
echo "     cd ../toadstool && cargo run --release -- server &"
