#!/usr/bin/env bash
# BiomeOS Runtime Discovery Utilities
# Zero hardcoding - Discover what's actually available

set -e

# ANSI colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Discovery functions

# Discover available primals
discover_primals() {
    echo -e "${BLUE}🔍 Discovering available primals...${NC}"
    
    local primals_dir="${PRIMALS_DIR:-../../primals}"
    local discovered=()
    
    if [ -d "$primals_dir" ]; then
        for binary in "$primals_dir"/*; do
            if [ -x "$binary" ] && [ -f "$binary" ]; then
                local name=$(basename "$binary")
                if [ "$name" != "README.md" ]; then
                    discovered+=("$name")
                fi
            fi
        done
    fi
    
    if [ ${#discovered[@]} -eq 0 ]; then
        echo -e "${YELLOW}⚠ No primals discovered${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Discovered ${#discovered[@]} primals:${NC}"
    for primal in "${discovered[@]}"; do
        echo "  - $primal"
    done
    
    # Export for use by other functions
    export DISCOVERED_PRIMALS="${discovered[*]}"
}

# Check if a primal is available
primal_available() {
    local primal_name="$1"
    local primals_dir="${PRIMALS_DIR:-../../primals}"
    
    if [ -x "$primals_dir/$primal_name" ]; then
        return 0
    else
        return 1
    fi
}

# Discover primal capabilities (REST API check)
discover_rest_api() {
    local endpoint="$1"
    local timeout="${2:-2}"
    
    # Try /health endpoint
    if curl -sf -m "$timeout" "$endpoint/health" > /dev/null 2>&1; then
        echo "rest_api"
        return 0
    fi
    
    # Try /api/health
    if curl -sf -m "$timeout" "$endpoint/api/health" > /dev/null 2>&1; then
        echo "rest_api"
        return 0
    fi
    
    # Not a REST API (or not running)
    return 1
}

# Discover primal type (server, CLI, library)
discover_primal_type() {
    local primal_name="$1"
    local primals_dir="${PRIMALS_DIR:-../../primals}"
    local binary="$primals_dir/$primal_name"
    
    if [ ! -x "$binary" ]; then
        echo "unknown"
        return 1
    fi
    
    # Run with --help and analyze output
    local help_output
    help_output=$("$binary" --help 2>&1 | head -20 || true)
    
    # Check for server indicators
    if echo "$help_output" | grep -qi "service start\|server\|daemon"; then
        echo "server"
        return 0
    fi
    
    # Check for CLI indicators
    if echo "$help_output" | grep -qi "Commands:\|Usage:"; then
        echo "cli"
        return 0
    fi
    
    echo "unknown"
    return 0
}

# Check service health
check_health() {
    local endpoint="$1"
    local name="${2:-service}"
    
    local response
    response=$(curl -sf -m 2 "$endpoint/health" 2>&1)
    local status=$?
    
    if [ $status -eq 0 ]; then
        echo -e "${GREEN}✅ $name healthy${NC}"
        echo "$response" | jq . 2>/dev/null || echo "$response"
        return 0
    else
        # Try alternate endpoint
        response=$(curl -sf -m 2 "$endpoint/api/health" 2>&1)
        status=$?
        
        if [ $status -eq 0 ]; then
            echo -e "${GREEN}✅ $name healthy${NC}"
            echo "$response" | jq . 2>/dev/null || echo "$response"
            return 0
        else
            echo -e "${RED}✗ $name not responding${NC}"
            return 1
        fi
    fi
}

# Discover capability by type
discover_capability() {
    local capability_type="$1"
    
    echo -e "${BLUE}🔍 Discovering capability: $capability_type${NC}"
    
    case "$capability_type" in
        storage)
            # Check for NestGate
            if check_health "http://localhost:9020" "nestgate" 2>/dev/null; then
                echo "http://localhost:9020"
                return 0
            fi
            
            # Check environment variable
            if [ -n "$NESTGATE_ENDPOINT" ]; then
                if check_health "$NESTGATE_ENDPOINT" "nestgate" 2>/dev/null; then
                    echo "$NESTGATE_ENDPOINT"
                    return 0
                fi
            fi
            
            echo -e "${YELLOW}⚠ Storage capability not available${NC}"
            return 1
            ;;
            
        orchestration)
            # Check for Songbird via UDP discovery (port 2300)
            # Songbird broadcasts its HTTPS endpoint via mDNS
            echo -e "${BLUE}  Searching for Songbird via discovery...${NC}" >&2
            
            # Try common Songbird ports
            for port in 8080 9000 3000; do
                if check_health "http://localhost:$port" "songbird" 2>/dev/null; then
                    echo "http://localhost:$port"
                    return 0
                fi
                if check_health "https://localhost:$port" "songbird" 2>/dev/null; then
                    echo "https://localhost:$port"
                    return 0
                fi
            done
            
            # Check environment variable
            if [ -n "$SONGBIRD_ENDPOINT" ]; then
                if check_health "$SONGBIRD_ENDPOINT" "songbird" 2>/dev/null; then
                    echo "$SONGBIRD_ENDPOINT"
                    return 0
                fi
            fi
            
            # Check if Songbird process is running and extract port
            if songbird_pid=$(pgrep -f songbird-orchestrator 2>/dev/null); then
                echo -e "${BLUE}  Found Songbird process (PID: $songbird_pid)${NC}" >&2
                # Try to get port from lsof
                if command -v lsof &> /dev/null; then
                    songbird_port=$(sudo lsof -i -P -n 2>/dev/null | grep "$songbird_pid" | grep TCP | grep LISTEN | head -1 | awk '{print $9}' | cut -d':' -f2 || echo "")
                    if [ -n "$songbird_port" ]; then
                        echo -e "${GREEN}  Detected Songbird on port $songbird_port${NC}" >&2
                        echo "https://localhost:$songbird_port"
                        return 0
                    fi
                fi
            fi
            
            echo -e "${YELLOW}⚠ Orchestration capability not available${NC}" >&2
            echo -e "${YELLOW}  Songbird uses mDNS/UDP discovery (port 2300)${NC}" >&2
            echo -e "${YELLOW}  It may be running but port not yet bound${NC}" >&2
            return 1
            ;;
            
        encryption)
            # Check for BearDog binary
            local primals_dir="${PRIMALS_DIR:-../../primals}"
            if [ -x "$primals_dir/beardog" ]; then
                echo "$primals_dir/beardog"
                return 0
            fi
            
            echo -e "${YELLOW}⚠ Encryption capability not available${NC}"
            return 1
            ;;
            
        compute)
            # Check for Toadstool
            local primals_dir="${PRIMALS_DIR:-../../primals}"
            if [ -x "$primals_dir/toadstool" ]; then
                echo "$primals_dir/toadstool"
                return 0
            fi
            
            echo -e "${YELLOW}⚠ Compute capability not available${NC}"
            return 1
            ;;
            
        *)
            echo -e "${RED}✗ Unknown capability type: $capability_type${NC}"
            return 1
            ;;
    esac
}

# Query primal API
query_primal() {
    local endpoint="$1"
    local path="$2"
    local method="${3:-GET}"
    
    case "$method" in
        GET)
            curl -sf -m 5 "$endpoint$path" | jq . 2>/dev/null || curl -sf -m 5 "$endpoint$path"
            ;;
        POST)
            curl -sf -m 5 -X POST "$endpoint$path" "${@:4}" | jq . 2>/dev/null || curl -sf -m 5 -X POST "$endpoint$path" "${@:4}"
            ;;
        *)
            echo -e "${RED}✗ Unsupported method: $method${NC}"
            return 1
            ;;
    esac
}

# Execute CLI tool
execute_cli() {
    local binary="$1"
    shift
    
    if [ ! -x "$binary" ]; then
        echo -e "${RED}✗ Binary not executable: $binary${NC}"
        return 1
    fi
    
    echo -e "${BLUE}▶ Executing: $binary $@${NC}"
    "$binary" "$@"
}

# Discover all available services
discover_all() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  BiomeOS Runtime Discovery             ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""
    
    # Discover primals
    discover_primals
    echo ""
    
    # Check capabilities
    echo -e "${BLUE}🔍 Checking capabilities...${NC}"
    
    # Storage
    if storage_endpoint=$(discover_capability "storage" 2>&1 | grep "http"); then
        echo -e "${GREEN}✅ Storage: $storage_endpoint${NC}"
    else
        echo -e "${YELLOW}⚠  Storage: Not available${NC}"
    fi
    
    # Orchestration
    if orch_endpoint=$(discover_capability "orchestration" 2>&1 | grep "http"); then
        echo -e "${GREEN}✅ Orchestration: $orch_endpoint${NC}"
    else
        echo -e "${YELLOW}⚠  Orchestration: Not available${NC}"
    fi
    
    # Encryption
    if crypto_binary=$(discover_capability "encryption" 2>&1 | grep "/"); then
        echo -e "${GREEN}✅ Encryption: $crypto_binary${NC}"
    else
        echo -e "${YELLOW}⚠  Encryption: Not available${NC}"
    fi
    
    # Compute
    if compute_binary=$(discover_capability "compute" 2>&1 | grep "/"); then
        echo -e "${GREEN}✅ Compute: $compute_binary${NC}"
    else
        echo -e "${YELLOW}⚠  Compute: Not available${NC}"
    fi
    
    echo ""
    echo -e "${GREEN}✅ Discovery complete${NC}"
}

# Export functions for use in other scripts
export -f discover_primals
export -f primal_available
export -f discover_rest_api
export -f discover_primal_type
export -f check_health
export -f discover_capability
export -f query_primal
export -f execute_cli
export -f discover_all

# If run directly, perform discovery
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    discover_all
fi

