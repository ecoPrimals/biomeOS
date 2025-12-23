#!/bin/bash
# BiomeOS Showcase Runner
# Discovers and executes showcase demos from parent ecoPrimals

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_DIR="$(dirname "$SCRIPT_DIR")"
ECOPRIMALS_DIR="$(dirname "$(dirname "$BIOMEOS_DIR")")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║  🌱 BiomeOS Showcase Runner                                 ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo
}

# Discover showcases from parent primals
discover_showcases() {
    local primal=$1
    local showcase_dir="$ECOPRIMALS_DIR/$primal/showcase"
    
    if [[ -d "$showcase_dir" ]]; then
        echo -e "${GREEN}📂 $primal/showcase/${NC}"
        
        # Find Cargo.toml files in showcase subdirectories
        find "$showcase_dir" -name "Cargo.toml" -type f 2>/dev/null | while read cargo_file; do
            local demo_dir=$(dirname "$cargo_file")
            local demo_name=$(basename "$demo_dir")
            
            # Skip if it's the root showcase Cargo.toml (workspace)
            if [[ "$demo_dir" == "$showcase_dir" ]]; then
                continue
            fi
            
            # Get binary names from Cargo.toml
            local bins=$(grep -A1 "^\[\[bin\]\]" "$cargo_file" 2>/dev/null | grep "name" | sed 's/.*= *"\([^"]*\)".*/\1/' | head -3)
            
            if [[ -n "$bins" ]]; then
                for bin in $bins; do
                    echo "   └─ $primal:$demo_name:$bin"
                done
            else
                echo "   └─ $primal:$demo_name"
            fi
        done
    fi
}

# Run a specific showcase
run_showcase() {
    local spec=$1
    IFS=':' read -r primal demo bin <<< "$spec"
    
    local showcase_dir="$ECOPRIMALS_DIR/$primal/showcase"
    local demo_dir="$showcase_dir/$demo"
    
    if [[ ! -d "$demo_dir" ]]; then
        echo -e "${RED}❌ Demo not found: $spec${NC}"
        echo "   Expected: $demo_dir"
        return 1
    fi
    
    echo -e "${CYAN}🚀 Running: $spec${NC}"
    echo
    
    cd "$demo_dir"
    
    if [[ -n "$bin" ]]; then
        cargo run --release --bin "$bin" 2>&1 | head -100
    else
        cargo run --release 2>&1 | head -100
    fi
}

# List all available showcases
list_all() {
    echo -e "${YELLOW}📋 Available Showcases${NC}"
    echo
    
    for primal in beardog songbird toadstool nestgate squirrel; do
        discover_showcases "$primal"
    done
    
    echo
    echo -e "${BLUE}Usage: $0 run <primal:demo[:bin]>${NC}"
    echo "Example: $0 run songbird:federation"
}

# Build all showcases (for faster subsequent runs)
build_all() {
    echo -e "${YELLOW}🔨 Building all showcases...${NC}"
    
    for primal in beardog songbird toadstool nestgate; do
        local showcase_dir="$ECOPRIMALS_DIR/$primal/showcase"
        if [[ -d "$showcase_dir" ]] && [[ -f "$showcase_dir/Cargo.toml" ]]; then
            echo -e "${GREEN}Building $primal showcases...${NC}"
            (cd "$showcase_dir" && cargo build --release 2>&1 | tail -3)
        fi
    done
    
    echo -e "${GREEN}✅ All showcases built${NC}"
}

# Main
print_header

case "${1:-list}" in
    list)
        list_all
        ;;
    run)
        if [[ -z "$2" ]]; then
            echo -e "${RED}❌ Specify a showcase: $0 run <primal:demo[:bin]>${NC}"
            exit 1
        fi
        run_showcase "$2"
        ;;
    build)
        build_all
        ;;
    *)
        echo "Usage: $0 {list|run <spec>|build}"
        echo
        echo "Commands:"
        echo "  list              List all available showcases"
        echo "  run <spec>        Run a specific showcase"
        echo "  build             Build all showcases"
        echo
        echo "Spec format: primal:demo[:bin]"
        echo "Examples:"
        echo "  $0 run songbird:federation"
        echo "  $0 run toadstool:gpu-compute:demo"
        ;;
esac

