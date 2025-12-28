#!/usr/bin/env bash
# Common Capability-Based Discovery Functions
# Used by all BiomeOS showcases to ensure zero hardcoding

# Color codes
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export RED='\033[0;31m'
export BLUE='\033[0;34m'
export CYAN='\033[0;36m'
export NC='\033[0m'

#
#=================================================================
# discover_primal_by_capability()
#
# Discovers a primal endpoint by capability, not by name.
# Uses multiple fallback methods.
#
# Arguments:
#   $1 - Required capability (e.g., "compute", "encryption", "storage")
#   $2 - Optional capability type (e.g., "execution", "symmetric")
#
# Returns:
#   Prints discovered endpoint to stdout
#   Returns 0 on success, 1 on failure
#
# Example:
#   endpoint=$(discover_primal_by_capability "compute" "execution")
#=================================================================
discover_primal_by_capability() {
    local required_capability="$1"
    local capability_type="$2"
    
    echo -e "${CYAN}Discovering primal with capability: $required_capability${NC}" >&2
    echo "" >&2
    
    # Method 1: Environment variable (explicit)
    local env_var="$(echo $required_capability | tr '[:lower:]' '[:upper:]')_ENDPOINT"
    if [ -n "${!env_var}" ]; then
        echo -e "${GREEN}✓ Found via $env_var: ${!env_var}${NC}" >&2
        echo "${!env_var}"
        return 0
    fi
    
    # Method 2: Generic discovery endpoint
    if [ -n "$DISCOVERY_ENDPOINT" ]; then
        echo "  Querying discovery service for capability..." >&2
        local discovered=$(curl -s "$DISCOVERY_ENDPOINT/api/v1/services/query/$required_capability" 2>/dev/null | \
            jq -r '.services[0].endpoint' 2>/dev/null)
        
        if [ -n "$discovered" ] && [ "$discovered" != "null" ]; then
            echo -e "${GREEN}✓ Found via discovery service: $discovered${NC}" >&2
            echo "$discovered"
            return 0
        fi
    fi
    
    # Method 3: Network scan for capability
    echo "  Scanning network for $required_capability capability..." >&2
    local common_ports=(3000 8000 8001 8002 8080 8081 9000 9001)
    
    for port in "${common_ports[@]}"; do
        local endpoint="http://localhost:$port"
        
        # Try to get capabilities
        local caps=$(curl -s --max-time 1 "$endpoint/api/v1/info" 2>/dev/null | \
            jq -r '.capabilities[]' 2>/dev/null)
        
        if echo "$caps" | grep -q "$required_capability" 2>/dev/null; then
            echo -e "${GREEN}✓ Found $required_capability at port $port${NC}" >&2
            echo "$endpoint"
            return 0
        fi
    done
    
    echo -e "${RED}✗ Could not find primal with $required_capability capability${NC}" >&2
    return 1
}

#=================================================================
# probe_primal_interface()
#
# Discovers how to communicate with a primal by probing
# common endpoint patterns. Returns discovered endpoints.
#
# Arguments:
#   $1 - Base endpoint URL
#
# Sets global variables:
#   HEALTH_ENDPOINT
#   INFO_ENDPOINT
#   CAPABILITIES_ENDPOINT
#
# Returns:
#   0 on success, 1 if no endpoints discovered
#=================================================================
probe_primal_interface() {
    local base_endpoint="$1"
    
    echo -e "${CYAN}Probing interface patterns for: $base_endpoint${NC}" >&2
    echo "" >&2
    
    # Probe health endpoint
    export HEALTH_ENDPOINT=""
    for path in "/health" "/api/health" "/api/v1/health" "/status" "/_health"; do
        if curl -s -f --max-time 2 "$base_endpoint$path" >/dev/null 2>&1; then
            export HEALTH_ENDPOINT="$base_endpoint$path"
            echo -e "  Health: $path ${GREEN}✓${NC}" >&2
            break
        fi
    done
    
    # Probe info/capabilities endpoint
    export INFO_ENDPOINT=""
    for path in "/api/v1/info" "/api/info" "/info" "/capabilities" "/api/capabilities"; do
        if curl -s -f --max-time 2 "$base_endpoint$path" >/dev/null 2>&1; then
            export INFO_ENDPOINT="$base_endpoint$path"
            echo -e "  Info: $path ${GREEN}✓${NC}" >&2
            break
        fi
    done
    
    # Set capabilities endpoint (usually same as info)
    export CAPABILITIES_ENDPOINT="$INFO_ENDPOINT"
    
    if [ -z "$HEALTH_ENDPOINT" ] && [ -z "$INFO_ENDPOINT" ]; then
        echo -e "${RED}✗ Could not discover any standard endpoints${NC}" >&2
        return 1
    fi
    
    echo "" >&2
    return 0
}

#=================================================================
# verify_primal_capability()
#
# Verifies that a primal actually provides the required capability
#
# Arguments:
#   $1 - Endpoint URL
#   $2 - Required capability
#
# Returns:
#   0 if capability verified, 1 otherwise
#=================================================================
verify_primal_capability() {
    local endpoint="$1"
    local required_cap="$2"
    
    echo "Verifying $required_cap capability..." >&2
    
    # Probe interface if not already done
    if [ -z "$INFO_ENDPOINT" ]; then
        probe_primal_interface "$endpoint" >&2
    fi
    
    if [ -z "$INFO_ENDPOINT" ]; then
        echo -e "${RED}✗ Cannot verify capability (no info endpoint)${NC}" >&2
        return 1
    fi
    
    # Get capabilities
    local caps=$(curl -s "$INFO_ENDPOINT" 2>/dev/null | \
        jq -r '.capabilities[]' 2>/dev/null)
    
    if echo "$caps" | grep -q "$required_cap" 2>/dev/null; then
        echo -e "${GREEN}✓ Capability $required_cap verified${NC}" >&2
        return 0
    else
        echo -e "${YELLOW}⚠ Capability $required_cap not found${NC}" >&2
        echo "  Available: $caps" >&2
        return 1
    fi
}

#=================================================================
# graceful_degradation()
#
# Handles missing primal gracefully with clear messaging
#
# Arguments:
#   $1 - Missing capability
#   $2 - Suggested action
#=================================================================
graceful_degradation() {
    local capability="$1"
    local suggestion="$2"
    
    echo "" >&2
    echo -e "${YELLOW}═══ Graceful Degradation ═══${NC}" >&2
    echo "" >&2
    echo "Primal with '$capability' capability not available." >&2
    echo "" >&2
    echo "This demonstrates BiomeOS resilience:" >&2
    echo "  • System continues without hard dependencies" >&2
    echo "  • Clear error messages for operators" >&2
    echo "  • Suggested resolution paths" >&2
    echo "" >&2
    
    if [ -n "$suggestion" ]; then
        echo "Suggested action:" >&2
        echo "  $suggestion" >&2
        echo "" >&2
    fi
    
    echo "BiomeOS philosophy:" >&2
    echo "  'Primals are sovereign. Their absence is not BiomeOS failure.'" >&2
    echo "" >&2
}

#=================================================================
# start_primal_smart()
#
# Intelligently start a primal binary, searching multiple locations
#
# Arguments:
#   $1 - Primal name (e.g., "songbird", "toadstool")
#   $2 - Default port
#   $3 - Additional args (optional)
#
# Returns:
#   PID of started process, or exits on failure
#=================================================================
start_primal_smart() {
    local primal_name="$1"
    local port="$2"
    local extra_args="$3"
    
    echo -e "${CYAN}Starting $primal_name...${NC}" >&2
    
    # Search for binary
    local possible_locations=(
        "../../../../primalBins/$primal_name"
        "../../../phase1bins/$primal_name"
        "../../../phase1bins/${primal_name}-cli-dec-25-2025-standalone"
        "$(which $primal_name 2>/dev/null)"
    )
    
    local binary=""
    for location in "${possible_locations[@]}"; do
        if [ -f "$location" ] && [ -x "$location" ]; then
            binary="$location"
            break
        fi
    done
    
    if [ -z "$binary" ]; then
        echo -e "${RED}✗ $primal_name binary not found${NC}" >&2
        echo "  Searched:" >&2
        for loc in "${possible_locations[@]}"; do
            echo "    - $loc" >&2
        done
        graceful_degradation "$primal_name service" \
            "Install $primal_name binary or set ${primal_name}_ENDPOINT"
        return 1
    fi
    
    echo "  Binary: $binary" >&2
    echo "  Port: $port" >&2
    
    # Start primal
    local log_dir="$(dirname $0)/logs"
    local pid_dir="$(dirname $0)/pids"
    mkdir -p "$log_dir" "$pid_dir"
    
    local log_file="$log_dir/${primal_name}-$(date +%Y%m%d-%H%M%S).log"
    local pid_file="$pid_dir/${primal_name}.pid"
    
    $binary tower start --port $port --bind 127.0.0.1 $extra_args > "$log_file" 2>&1 &
    local pid=$!
    echo $pid > "$pid_file"
    
    echo "  PID: $pid" >&2
    echo "  Log: $log_file" >&2
    
    sleep 3
    
    if ! kill -0 $pid 2>/dev/null; then
        echo -e "${RED}✗ Failed to start${NC}" >&2
        echo "  Check log: $log_file" >&2
        return 1
    fi
    
    echo -e "${GREEN}✓ Started successfully${NC}" >&2
    echo $pid
    return 0
}

#=================================================================
# stop_primal_clean()
#
# Cleanly stop a primal and remove PID file
#
# Arguments:
#   $1 - Primal name or PID
#=================================================================
stop_primal_clean() {
    local identifier="$1"
    local pid=""
    
    # Check if it's a PID or name
    if [[ "$identifier" =~ ^[0-9]+$ ]]; then
        pid="$identifier"
    else
        local pid_file="$(dirname $0)/pids/${identifier}.pid"
        if [ -f "$pid_file" ]; then
            pid=$(cat "$pid_file")
            rm -f "$pid_file"
        fi
    fi
    
    if [ -n "$pid" ] && kill -0 $pid 2>/dev/null; then
        echo "Stopping PID $pid..." >&2
        kill $pid 2>/dev/null || true
        wait $pid 2>/dev/null || true
        echo -e "${GREEN}✓ Stopped${NC}" >&2
    fi
}

#=================================================================
# document_gap()
#
# Standardized gap documentation
#
# Arguments:
#   $1 - Gap report file
#   $2 - Gap category
#   $3 - Gap description
#=================================================================
document_gap() {
    local gap_file="$1"
    local category="$2"
    local description="$3"
    
    echo "" >> "$gap_file"
    echo "## $category" >> "$gap_file"
    echo "" >> "$gap_file"
    echo "- [x] FOUND: $description" >> "$gap_file"
    echo "- [ ] Timestamp: $(date -Iseconds)" >> "$gap_file"
    echo "" >> "$gap_file"
}

echo "BiomeOS Common Library Loaded" >&2
echo "  Zero-knowledge discovery functions available" >&2
echo "" >&2

