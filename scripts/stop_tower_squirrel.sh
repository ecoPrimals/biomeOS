#!/usr/bin/env bash
#
# Stop Tower Atomic + Squirrel deployment
# Gracefully stops all services and cleans up sockets
#
# Usage: ./scripts/stop_tower_squirrel.sh [family_id]

set -euo pipefail

# Configuration
FAMILY_ID="${1:-nat0}"

# Socket paths
BEARDOG_SOCKET="/tmp/security-${FAMILY_ID}.sock"
SONGBIRD_SOCKET="/tmp/discovery-${FAMILY_ID}.sock"
NEURAL_API_SOCKET="/tmp/neural-api-${FAMILY_ID}.sock"
SQUIRREL_SOCKET="/tmp/ai-${FAMILY_ID}.sock"

# PID file directory
PID_DIR="/tmp/biomeos-${FAMILY_ID}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

stop_service() {
    local service_name=$1
    local pid_file="$PID_DIR/${service_name}.pid"
    
    if [ ! -f "$pid_file" ]; then
        log_warning "$service_name PID file not found: $pid_file"
        return 0
    fi
    
    local pid=$(cat "$pid_file")
    
    if ! kill -0 "$pid" 2>/dev/null; then
        log_warning "$service_name (PID: $pid) is not running"
        rm -f "$pid_file"
        return 0
    fi
    
    log_info "Stopping $service_name (PID: $pid)..."
    
    # Try graceful shutdown first (SIGTERM)
    if kill "$pid" 2>/dev/null; then
        # Wait up to 5 seconds for graceful shutdown
        for i in {1..5}; do
            if ! kill -0 "$pid" 2>/dev/null; then
                log_success "$service_name stopped gracefully"
                rm -f "$pid_file"
                return 0
            fi
            sleep 1
        done
        
        # Force kill if still running (SIGKILL)
        log_warning "$service_name did not stop gracefully, forcing..."
        if kill -9 "$pid" 2>/dev/null; then
            log_success "$service_name force-stopped"
        else
            log_error "Failed to stop $service_name"
        fi
    else
        log_error "Failed to send signal to $service_name"
    fi
    
    rm -f "$pid_file"
}

cleanup_sockets() {
    log_info "Cleaning up sockets..."
    
    local removed=0
    
    if [ -S "$BEARDOG_SOCKET" ]; then
        rm -f "$BEARDOG_SOCKET"
        log_info "Removed BearDog socket"
        removed=$((removed + 1))
    fi
    
    if [ -S "$SONGBIRD_SOCKET" ]; then
        rm -f "$SONGBIRD_SOCKET"
        log_info "Removed Songbird socket"
        removed=$((removed + 1))
    fi
    
    if [ -S "$NEURAL_API_SOCKET" ]; then
        rm -f "$NEURAL_API_SOCKET"
        log_info "Removed Neural API socket"
        removed=$((removed + 1))
    fi
    
    if [ -S "$SQUIRREL_SOCKET" ]; then
        rm -f "$SQUIRREL_SOCKET"
        log_info "Removed Squirrel socket"
        removed=$((removed + 1))
    fi
    
    # Clean up any other neural-api sockets for this family
    for socket in /tmp/neural-api-${FAMILY_ID}*.sock; do
        if [ -S "$socket" ]; then
            rm -f "$socket"
            log_info "Removed extra socket: $socket"
            removed=$((removed + 1))
        fi
    done
    
    if [ $removed -gt 0 ]; then
        log_success "Cleaned up $removed socket(s)"
    else
        log_info "No sockets to clean up"
    fi
}

# Main
main() {
    echo "=========================================="
    echo "Stopping Tower Atomic + Squirrel"
    echo "Family ID: $FAMILY_ID"
    echo "=========================================="
    echo ""
    
    # Stop services in reverse order
    stop_service "squirrel"
    stop_service "neural-api"
    stop_service "songbird"
    stop_service "beardog"
    
    # Clean up sockets
    cleanup_sockets
    
    # Clean up logs (optional)
    if [ -d "$PID_DIR" ]; then
        log_info "Logs are still available at: $PID_DIR/*.log"
        log_info "  To remove logs: rm -rf $PID_DIR"
    fi
    
    echo ""
    log_success "All services stopped!"
    echo ""
}

# Run main
main "$@"

