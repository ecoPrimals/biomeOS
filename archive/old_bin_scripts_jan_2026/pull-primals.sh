#!/usr/bin/env bash
# Build primal binaries from local parent directory
# Teams push updates to their primals, we build from source
#
# Structure:
#   ecoPrimals/           <- parent org directory
#   ├── beardog/          <- primal repos (parallel development)
#   ├── songbird/
#   ├── toadstool/
#   ├── nestgate/
#   ├── squirrel/
#   └── phase2/
#       └── biomeOS/      <- we are here
#           └── bin/
#               └── primals/  <- built binaries go here

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"
PHASE2_ROOT="$(dirname "$BIOMEOS_ROOT")"
ECOPRIMALS_ROOT="$(dirname "$PHASE2_ROOT")"

PRIMALS_DIR="$SCRIPT_DIR/primals"
CHIMERAS_DIR="$SCRIPT_DIR/chimeras"

# Primal source paths (relative to ecoPrimals org root)
declare -A PRIMAL_PATHS=(
    ["beardog"]="$ECOPRIMALS_ROOT/beardog"
    ["songbird"]="$ECOPRIMALS_ROOT/songbird"
    ["toadstool"]="$ECOPRIMALS_ROOT/toadstool"
    ["nestgate"]="$ECOPRIMALS_ROOT/nestgate"
    ["squirrel"]="$ECOPRIMALS_ROOT/squirrel"
)

# Primary binary names (some primals have multiple binaries)
declare -A PRIMAL_BINS=(
    ["beardog"]="beardog"
    ["songbird"]="songbird"
    ["toadstool"]="toadstool"
    ["nestgate"]="nestgate"
    ["squirrel"]="squirrel"
)

log_info()    { echo "ℹ️  $1"; }
log_success() { echo "✅ $1"; }
log_error()   { echo "❌ $1" >&2; }
log_warn()    { echo "⚠️  $1"; }
log_build()   { echo "🔨 $1"; }

build_primal() {
    local primal="$1"
    local primal_path="${PRIMAL_PATHS[$primal]:-}"
    
    if [[ -z "$primal_path" ]]; then
        log_error "Unknown primal: $primal"
        return 1
    fi
    
    if [[ ! -d "$primal_path" ]]; then
        log_error "Primal source not found: $primal_path"
        return 1
    fi
    
    log_build "Building $primal from $primal_path"
    
    # Check for Cargo.toml
    if [[ ! -f "$primal_path/Cargo.toml" ]]; then
        log_error "No Cargo.toml found in $primal_path"
        return 1
    fi
    
    # Build in release mode
    pushd "$primal_path" > /dev/null
    
    if cargo build --release 2>&1; then
        # Find binaries in target/release
        local target_dir="$primal_path/target/release"
        local found=0
        
        # Look for the primary binary
        local primary_bin="${PRIMAL_BINS[$primal]}"
        if [[ -f "$target_dir/$primary_bin" ]]; then
            cp "$target_dir/$primary_bin" "$PRIMALS_DIR/$primal"
            chmod +x "$PRIMALS_DIR/$primal"
            log_success "$primal binary installed"
            found=1
        else
            # Try to find any executable
            for bin in "$target_dir"/*; do
                if [[ -x "$bin" && -f "$bin" && ! "$bin" =~ \.(d|rlib|so|dylib)$ ]]; then
                    local bin_name=$(basename "$bin")
                    if [[ "$bin_name" != "build" && "$bin_name" != "deps" && "$bin_name" != "examples" && "$bin_name" != "incremental" ]]; then
                        cp "$bin" "$PRIMALS_DIR/${primal}-${bin_name}"
                        chmod +x "$PRIMALS_DIR/${primal}-${bin_name}"
                        log_success "Installed: ${primal}-${bin_name}"
                        found=1
                    fi
                fi
            done
        fi
        
        if [[ $found -eq 0 ]]; then
            log_warn "Build succeeded but no binaries found for $primal"
        fi
    else
        log_error "Build failed for $primal"
        popd > /dev/null
        return 1
    fi
    
    popd > /dev/null
    return 0
}

check_primal() {
    local primal="$1"
    local primal_path="${PRIMAL_PATHS[$primal]:-}"
    
    if [[ -z "$primal_path" || ! -d "$primal_path" ]]; then
        echo "  ❌ $primal (not found)"
        return 1
    fi
    
    # Get git info if available
    local git_info=""
    if [[ -d "$primal_path/.git" ]]; then
        pushd "$primal_path" > /dev/null
        local branch=$(git branch --show-current 2>/dev/null || echo "unknown")
        local commit=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
        git_info=" [${branch}@${commit}]"
        popd > /dev/null
    fi
    
    echo "  ✅ $primal$git_info"
    echo "     └─ $primal_path"
}

list_primals() {
    echo "📦 Available primals (from ecoPrimals org):"
    echo "   Source: $ECOPRIMALS_ROOT"
    echo ""
    for primal in beardog songbird toadstool nestgate squirrel; do
        check_primal "$primal"
    done
}

check_installed() {
    echo "🔧 Installed primal binaries:"
    echo "   Location: $PRIMALS_DIR"
    echo ""
    
    if [[ ! -d "$PRIMALS_DIR" ]] || [[ -z "$(ls -A "$PRIMALS_DIR" 2>/dev/null)" ]]; then
        echo "   (none installed)"
        return
    fi
    
    for bin in "$PRIMALS_DIR"/*; do
        if [[ -x "$bin" && -f "$bin" ]]; then
            local name=$(basename "$bin")
            local size=$(du -h "$bin" | cut -f1)
            local mtime=$(stat -c %y "$bin" 2>/dev/null | cut -d. -f1 || stat -f %Sm "$bin" 2>/dev/null)
            echo "  ✅ $name ($size, built $mtime)"
        fi
    done
}

build_all() {
    log_info "Building all primals from local sources..."
    echo "   Source: $ECOPRIMALS_ROOT"
    echo ""
    
    local success=0
    local failed=0
    
    for primal in beardog songbird toadstool nestgate squirrel; do
        if build_primal "$primal"; then
            ((success++))
        else
            ((failed++))
        fi
        echo ""
    done
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Results: $success succeeded, $failed failed"
}

# Ensure directories exist
mkdir -p "$PRIMALS_DIR" "$CHIMERAS_DIR"

case "${1:-help}" in
    --all|-a)
        build_all
        ;;
    --list|-l)
        list_primals
        ;;
    --check|-c)
        check_installed
        ;;
    --help|-h|help)
        cat << EOF
BiomeOS Primal Builder
======================
Build primal binaries from local ecoPrimals repositories.

Usage: $0 <command>

Commands:
  <primal>     Build a specific primal (beardog, songbird, etc.)
  --all, -a    Build all primals
  --list, -l   List available primal sources
  --check, -c  Show installed binaries
  --help, -h   Show this help

Primals: beardog, songbird, toadstool, nestgate, squirrel

Example:
  $0 beardog      # Build beardog only
  $0 --all        # Build everything
  $0 --check      # See what's installed

Source Directory: $ECOPRIMALS_ROOT
Binary Directory: $PRIMALS_DIR
EOF
        ;;
    *)
        if [[ -n "${PRIMAL_PATHS[$1]:-}" ]]; then
            build_primal "$1"
        else
            log_error "Unknown command or primal: $1"
            echo "Run '$0 --help' for usage"
            exit 1
        fi
        ;;
esac
