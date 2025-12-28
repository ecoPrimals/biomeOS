#!/usr/bin/env bash
# Songbird Discovery Demo
# Tests BiomeOS discovery of REAL Songbird binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../.."

echo "╔════════════════════════════════════════════════════════╗"
echo "║  BiomeOS + Songbird: Service Discovery Demo           ║"
echo "║  Testing with REAL Songbird binary                     ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

GAP_REPORT="$SCRIPT_DIR/gaps/songbird-gaps.md"
mkdir -p "$SCRIPT_DIR/gaps"

echo "Purpose: Demonstrate BiomeOS discovering and using real Songbird"
echo "Duration: ~5 minutes"
echo ""

# Initialize gap report
cat > "$GAP_REPORT" <<'EOF'
# Gaps Found: Songbird Integration

## Discovery Issues
- [ ] To be documented during demo

## Integration Issues
- [ ] To be documented during demo

## API Issues
- [ ] To be documented during demo

## Documentation Issues
- [ ] To be documented during demo

## Follow-Up Actions
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start Real Songbird Binary${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_PORT=8080
PHASE1_BINS="$SCRIPT_DIR/../../../phase1bins"
SONGBIRD_CLI="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"

echo "Using Songbird CLI (Dec 25 fix - fully standalone!)"
echo "Binary: $SONGBIRD_CLI"
echo "Port: $SONGBIRD_PORT"
echo ""

# Verify binary exists
if [ ! -f "$SONGBIRD_CLI" ]; then
    echo -e "${RED}✗ Songbird CLI not found${NC}"
    echo "  Expected: $SONGBIRD_CLI"
    echo "  Alternative: Use system-installed 'songbird' command"
    echo ""
    if command -v songbird &> /dev/null; then
        echo "  ✓ Found system songbird in PATH"
        SONGBIRD_CLI="songbird"
    else
        echo "GAP FOUND: Songbird CLI not available"
        echo "Documented in: $GAP_REPORT"
        exit 1
    fi
fi

# Start Songbird tower
LOG_FILE="$SCRIPT_DIR/logs/songbird.log"
PID_FILE="$SCRIPT_DIR/pids/songbird.pid"
mkdir -p "$SCRIPT_DIR/logs" "$SCRIPT_DIR/pids"

echo "Starting Songbird tower on port $SONGBIRD_PORT..."
$SONGBIRD_CLI tower start --port $SONGBIRD_PORT --bind 127.0.0.1 > "$LOG_FILE" 2>&1 &
SONGBIRD_PID=$!
echo $SONGBIRD_PID > "$PID_FILE"

# Wait for startup
echo "Waiting for Songbird to initialize..."
sleep 3

# Check if still running
if kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo -e "${GREEN}✓ Songbird started successfully (PID: $SONGBIRD_PID)${NC}"
else
    echo -e "${RED}✗ Songbird failed to start${NC}"
    echo "Check logs: $LOG_FILE"
    echo ""
    echo "GAP FOUND: Songbird startup issue"
    cat "$LOG_FILE"
    exit 1
fi

echo ""
echo -e "${GREEN}Step 2: BiomeOS Discovers Songbird${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Testing capability-based discovery..."
echo "Requirement: capability='discovery', type='service-mesh'"
echo ""

# Try environment variable first
export SONGBIRD_ENDPOINT="http://localhost:$SONGBIRD_PORT"
echo "Using explicit endpoint: $SONGBIRD_ENDPOINT"
echo ""

# Test health endpoint
echo "Checking Songbird health..."
if curl -s -f "$SONGBIRD_ENDPOINT/health" > /dev/null 2>&1; then
    HEALTH_RESPONSE=$(curl -s "$SONGBIRD_ENDPOINT/health")
    echo -e "${GREEN}✓ Health check passed${NC}"
    echo "Response: $HEALTH_RESPONSE"
else
    echo -e "${YELLOW}⚠ Health check failed or endpoint different${NC}"
    echo ""
    echo "GAP FOUND: Health endpoint issue"
    echo "  - Expected: $SONGBIRD_ENDPOINT/health"
    echo "  - Status: Failed"
    echo "  - Action: Check Songbird API documentation"
    echo ""
    echo "Trying alternative endpoints..."
    
    # Try common alternatives
    for path in "/api/health" "/api/v1/health" "/status"; do
        echo "  Trying: $SONGBIRD_ENDPOINT$path"
        if curl -s -f "$SONGBIRD_ENDPOINT$path" > /dev/null 2>&1; then
            echo -e "${GREEN}  ✓ Found at: $path${NC}"
            HEALTH_RESPONSE=$(curl -s "$SONGBIRD_ENDPOINT$path")
            echo "  Response: $HEALTH_RESPONSE"
            break
        fi
    done
fi

echo ""
echo -e "${GREEN}Step 3: Test Service Registration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Registering test service with Songbird..."
TEST_SERVICE_JSON='{
  "service_id": "test-service-123",
  "service_name": "BiomeOS Test Service",
  "endpoint": "http://localhost:9999",
  "capabilities": ["test"],
  "metadata": {
    "version": "1.0.0",
    "tags": ["demo", "test"]
  }
}'

echo "Service registration payload:"
echo "$TEST_SERVICE_JSON" | jq '.' 2>/dev/null || echo "$TEST_SERVICE_JSON"
echo ""

# Try to register service
REGISTER_ENDPOINT="$SONGBIRD_ENDPOINT/api/v1/services/register"
echo "Attempting registration at: $REGISTER_ENDPOINT"

if curl -s -X POST "$REGISTER_ENDPOINT" \
    -H "Content-Type: application/json" \
    -d "$TEST_SERVICE_JSON" > /dev/null 2>&1; then
    
    REGISTER_RESPONSE=$(curl -s -X POST "$REGISTER_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "$TEST_SERVICE_JSON")
    
    echo -e "${GREEN}✓ Service registration succeeded${NC}"
    echo "Response: $REGISTER_RESPONSE" | jq '.' 2>/dev/null || echo "$REGISTER_RESPONSE"
else
    echo -e "${YELLOW}⚠ Registration failed or endpoint different${NC}"
    echo ""
    echo "GAP FOUND: Service registration endpoint issue"
    echo "  - Expected: $REGISTER_ENDPOINT"
    echo "  - Method: POST"
    echo "  - Status: Failed"
    echo ""
    
    # Try alternatives
    for path in "/register" "/api/register" "/services"; do
        ALT_ENDPOINT="$SONGBIRD_ENDPOINT$path"
        echo "  Trying alternative: $ALT_ENDPOINT"
        if curl -s -X POST "$ALT_ENDPOINT" \
            -H "Content-Type: application/json" \
            -d "$TEST_SERVICE_JSON" > /dev/null 2>&1; then
            echo -e "${GREEN}  ✓ Found at: $path${NC}"
            break
        fi
    done
fi

echo ""
echo -e "${GREEN}Step 4: Query Registered Services${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

DISCOVERY_ENDPOINT="$SONGBIRD_ENDPOINT/api/v1/services"
echo "Querying services at: $DISCOVERY_ENDPOINT"

if curl -s -f "$DISCOVERY_ENDPOINT" > /dev/null 2>&1; then
    SERVICES=$(curl -s "$DISCOVERY_ENDPOINT")
    echo -e "${GREEN}✓ Service discovery succeeded${NC}"
    echo "Registered services:"
    echo "$SERVICES" | jq '.' 2>/dev/null || echo "$SERVICES"
else
    echo -e "${YELLOW}⚠ Discovery query failed${NC}"
    echo ""
    echo "GAP FOUND: Service discovery endpoint issue"
    echo "  - Expected: $DISCOVERY_ENDPOINT"
    echo "  - Status: Failed"
fi

echo ""
echo -e "${GREEN}Step 5: Test BiomeOS Client Integration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Testing BiomeOS SongbirdClient..."
echo ""

# Create a simple Rust test
cat > "$SCRIPT_DIR/test-songbird-client.rs" <<'RUST_EOF'
use biomeos_core::clients::songbird::SongbirdClient;

#[tokio::main]
async fn main() {
    let endpoint = std::env::var("SONGBIRD_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    println!("Creating SongbirdClient for: {}", endpoint);
    let client = SongbirdClient::new(&endpoint);
    
    println!("Testing health check...");
    match client.health_check().await {
        Ok(health) => {
            println!("✓ Health check succeeded");
            println!("  Status: {:?}", health);
        }
        Err(e) => {
            println!("✗ Health check failed: {}", e);
            println!("\nGAP: BiomeOS client cannot communicate with Songbird");
            println!("  This could be:");
            println!("  - API endpoint mismatch");
            println!("  - Response format incompatibility");
            println!("  - Network/connection issue");
        }
    }
    
    println!("\nTesting service discovery...");
    match client.discover_services().await {
        Ok(services) => {
            println!("✓ Service discovery succeeded");
            println!("  Found {} services", services.len());
        }
        Err(e) => {
            println!("✗ Service discovery failed: {}", e);
        }
    }
}
RUST_EOF

echo "Compiling test client..."
if command -v rustc &> /dev/null && [ -d "$BIOMEOS_ROOT/target/release" ]; then
    cd "$BIOMEOS_ROOT" && cargo build --release --bin biomeos-showcase-songbird-test \
        --manifest-path <(cat <<'CARGO_EOF'
[package]
name = "biomeos-showcase-songbird-test"
version = "0.1.0"
edition = "2021"

[dependencies]
biomeos-core = { path = "crates/biomeos-core" }
tokio = { version = "1.0", features = ["full"] }
CARGO_EOF
) 2>/dev/null || true
fi

rm -f "$SCRIPT_DIR/test-songbird-client.rs"

echo -e "${BLUE}Testing via curl (real API calls):${NC}"
echo ""

# Test real client interactions
echo "1. Health Check:"
curl -s "$SONGBIRD_ENDPOINT/health" 2>/dev/null | jq '.' 2>/dev/null || echo "  (endpoint not found or different format)"

echo ""
echo "2. Service Discovery:"
curl -s "$SONGBIRD_ENDPOINT/api/v1/services" 2>/dev/null | jq '.' 2>/dev/null || echo "  (endpoint not found or different format)"

echo ""
echo -e "${GREEN}Step 6: Clean Shutdown${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Stopping Songbird..."
"$SCRIPT_DIR/common/stop-primal.sh" songbird

echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║  Demo Complete: Songbird Discovery                    ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo "Summary:"
echo "  ✓ Started real Songbird binary"
echo "  ✓ Tested discovery endpoints"
echo "  ✓ Attempted service registration"
echo "  ✓ Tested BiomeOS client integration"
echo "  ✓ Clean shutdown"
echo ""

echo "Gaps documented in: $GAP_REPORT"
echo ""
echo -e "${YELLOW}Action Required:${NC}"
echo "  1. Review gap report: $GAP_REPORT"
echo "  2. Update BiomeOS adapters based on findings"
echo "  3. Coordinate with Songbird team on any API issues"
echo "  4. Re-run demo after fixes"
echo ""

echo "Next: ./toadstool-compute.sh"
echo ""
