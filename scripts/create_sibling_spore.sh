#!/bin/bash
# Create Sibling Spore - Deploy a genetically linked LiveSpore
#
# This script creates a new LiveSpore that is genetically linked to an existing
# parent spore via BearDog's lineage system.
#
# The new spore will:
# 1. Have a UNIQUE seed (derived from parent, not cloned)
# 2. Be able to prove genetic relationship via BearDog
# 3. Receive automatic trust from siblings via Songbird federation
#
# Usage:
#   ./scripts/create_sibling_spore.sh /media/parent/biomeOS /media/newusb node-beta
#
# Arguments:
#   $1 - Parent spore root (must have .family.seed)
#   $2 - Target path for new spore (USB mount point or directory)
#   $3 - Node ID for new spore (unique identifier)
#
# Environment:
#   DEPLOYMENT_BATCH - Optional batch ID (default: today's date)
#   FAMILY_ID - Optional family ID (default: 1894e909e454)
#
# Author: biomeOS Team
# Date: 2026-01-27

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# Validate arguments
if [[ $# -lt 3 ]]; then
    echo "Usage: $0 <parent_spore_root> <target_path> <node_id>"
    echo ""
    echo "Example:"
    echo "  $0 /media/user/USB1/biomeOS /media/user/USB2 node-beta"
    exit 1
fi

PARENT_ROOT="$1"
TARGET_PATH="$2"
NODE_ID="$3"
DEPLOYMENT_BATCH="${DEPLOYMENT_BATCH:-$(date +%Y%m%d)}"
FAMILY_ID="${FAMILY_ID:-1894e909e454}"

PARENT_SEED="$PARENT_ROOT/.family.seed"
SPORE_ROOT="$TARGET_PATH/biomeOS"

log_info "Creating sibling spore: $NODE_ID"
log_info "  Parent: $PARENT_ROOT"
log_info "  Target: $SPORE_ROOT"
log_info "  Batch:  $DEPLOYMENT_BATCH"

# ============================================================================
# Validation
# ============================================================================

log_info "Validating parent spore..."

if [[ ! -f "$PARENT_SEED" ]]; then
    log_error "Parent seed not found: $PARENT_SEED"
    exit 1
fi

SEED_SIZE=$(stat -c%s "$PARENT_SEED" 2>/dev/null || stat -f%z "$PARENT_SEED")
if [[ "$SEED_SIZE" -ne 32 ]]; then
    log_error "Parent seed has wrong size: $SEED_SIZE bytes (expected 32)"
    exit 1
fi

log_success "Parent seed validated: $PARENT_SEED"

if [[ ! -w "$TARGET_PATH" ]]; then
    log_error "Target path not writable: $TARGET_PATH"
    exit 1
fi

log_success "Target path is writable"

# ============================================================================
# Create Directory Structure
# ============================================================================

log_info "Creating spore directory structure..."

mkdir -p "$SPORE_ROOT"/{bin,config,graphs,logs,certs,secrets,primals/bin,metrics}
log_success "Directory structure created"

# ============================================================================
# Derive Sibling Seed (SHA256 genetic mixing)
# ============================================================================

log_info "Deriving sibling seed for $NODE_ID..."

# This is the biomeOS seed derivation formula:
# child_seed = SHA256(parent_seed || node_id || deployment_batch)
#
# We use openssl for portable SHA256

CHILD_SEED="$SPORE_ROOT/.family.seed"

# Create input for hashing: parent_seed || node_id || batch
{
    cat "$PARENT_SEED"
    printf '%s' "$NODE_ID"
    printf '%s' "$DEPLOYMENT_BATCH"
} | openssl dgst -sha256 -binary > "$CHILD_SEED"

# Set secure permissions
chmod 600 "$CHILD_SEED"

log_success "Derived unique sibling seed"

# Verify the derivation
CHILD_SIZE=$(stat -c%s "$CHILD_SEED" 2>/dev/null || stat -f%z "$CHILD_SEED")
if [[ "$CHILD_SIZE" -ne 32 ]]; then
    log_error "Derived seed has wrong size: $CHILD_SIZE bytes"
    exit 1
fi

# Show seed hashes for verification (not the actual seeds!)
PARENT_HASH=$(sha256sum "$PARENT_SEED" | cut -c1-16)
CHILD_HASH=$(sha256sum "$CHILD_SEED" | cut -c1-16)
log_info "  Parent hash (first 16): $PARENT_HASH..."
log_info "  Child hash (first 16):  $CHILD_HASH..."

if [[ "$PARENT_HASH" == "$CHILD_HASH" ]]; then
    log_error "CRITICAL: Child seed matches parent! Derivation failed."
    exit 1
fi

log_success "Seed uniqueness verified (hashes differ)"

# ============================================================================
# Copy Primal Binaries
# ============================================================================

log_info "Copying primal binaries..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"
# Try plasmidBin/primals, plasmidBin/, then target/release (actual layout varies)
PLASMID_PRIMALS="$BIOMEOS_ROOT/plasmidBin/primals"
PLASMID_ROOT="$BIOMEOS_ROOT/plasmidBin"
TARGET_RELEASE="$BIOMEOS_ROOT/target/release"

for binary in beardog songbird nestgate toadstool; do
    src=""
    for dir in "$PLASMID_PRIMALS" "$PLASMID_ROOT"; do
        if [[ -f "$dir/$binary" ]]; then
            src="$dir/$binary"
            break
        elif [[ "$binary" == "beardog" ]] && [[ -f "$dir/beardog-server" ]]; then
            src="$dir/beardog-server"
            break
        fi
    done
    if [[ -z "$src" ]] && [[ -d "$TARGET_RELEASE" ]]; then
        [[ -f "$TARGET_RELEASE/$binary" ]] && src="$TARGET_RELEASE/$binary"
        [[ -z "$src" ]] && [[ "$binary" == "beardog" ]] && [[ -f "$TARGET_RELEASE/beardog-server" ]] && src="$TARGET_RELEASE/beardog-server"
    fi
    if [[ -n "$src" ]]; then
        dest_name="$binary"
        cp "$src" "$SPORE_ROOT/primals/bin/$dest_name"
        chmod +x "$SPORE_ROOT/primals/bin/$dest_name"
        log_success "Copied $binary"
    fi
done

if ! ls "$SPORE_ROOT/primals/bin/"* 1>/dev/null 2>&1; then
    log_warn "No primal binaries found in plasmidBin/ or target/release"
    log_warn "You'll need to copy binaries manually (e.g. cargo build -p biomeos)"
fi

# ============================================================================
# Copy Deployment Graphs
# ============================================================================

log_info "Copying deployment graphs..."

GRAPHS_DIR="$BIOMEOS_ROOT/graphs"
if [[ -d "$GRAPHS_DIR" ]]; then
    cp "$GRAPHS_DIR"/*.toml "$SPORE_ROOT/graphs/" 2>/dev/null || true
    log_success "Copied deployment graphs"
else
    log_warn "Graphs directory not found - skipping"
fi

# ============================================================================
# Create Configuration
# ============================================================================

log_info "Creating spore configuration..."

# .spore.json - Spore metadata
cat > "$SPORE_ROOT/.spore.json" << EOF
{
  "version": "1.0.0",
  "node_id": "$NODE_ID",
  "family_id": "$FAMILY_ID",
  "lineage_mode": "sibling",
  "parent_hash": "$PARENT_HASH",
  "created_at": "$(date -Iseconds)",
  "deployment_batch": "$DEPLOYMENT_BATCH",
  "spore_type": "live",
  "capabilities": ["tower_atomic", "neural_api", "federation"]
}
EOF
log_success "Created .spore.json"

# tower.toml - Tower Atomic configuration
cat > "$SPORE_ROOT/tower.toml" << EOF
# Tower Atomic Configuration
# Generated by create_sibling_spore.sh
# Node: $NODE_ID
# Created: $(date)

[tower]
node_id = "$NODE_ID"
family_id = "$FAMILY_ID"
deployment_mode = "live_spore"
lineage_mode = "sibling"

[tower.sockets]
beardog = "/tmp/beardog-$NODE_ID.sock"
songbird = "/tmp/songbird-$NODE_ID.sock"
neural_api = "/tmp/neural-api-$NODE_ID.sock"

[tower.security]
require_btsp = true
allow_federation = true
lineage_verification = "genetic_blake3"

[tower.federation]
trust_siblings = true
trust_level = "family"
require_seed_verification = true
# BearDog will verify genetic lineage via Blake3 proofs
EOF
log_success "Created tower.toml"

# ============================================================================
# Create Deployment Script
# ============================================================================

log_info "Creating deployment script..."

cat > "$SPORE_ROOT/deploy.sh" << 'DEPLOY_SCRIPT'
#!/bin/bash
# LiveSpore Deployment Script
# Generated by create_sibling_spore.sh

set -euo pipefail

SPORE_ROOT="$(cd "$(dirname "$0")" && pwd)"
NODE_ID="__NODE_ID__"
FAMILY_ID="__FAMILY_ID__"

# Family credentials
export FAMILY_SEED_FILE="$SPORE_ROOT/.family.seed"
export FAMILY_ID="$FAMILY_ID"
export NODE_ID="$NODE_ID"

# Socket paths (per-node to allow multiple spores)
export NEURAL_API_SOCKET="/tmp/neural-api-$NODE_ID.sock"
export BEARDOG_SOCKET="/tmp/beardog-$NODE_ID.sock"
export SONGBIRD_SOCKET="/tmp/songbird-$NODE_ID.sock"

# Security endpoints
export SECURITY_ENDPOINT="unix://$NEURAL_API_SOCKET"
export CAPABILITY_SECURITY_ENDPOINT="unix://$NEURAL_API_SOCKET"
export BEARDOG_MODE="neural"

echo "🚀 Starting LiveSpore: $NODE_ID (family: $FAMILY_ID)"
echo "   Seed: $FAMILY_SEED_FILE"
echo "   Sockets: /tmp/*-$NODE_ID.sock"

# Cleanup any existing sockets
rm -f "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET" "$NEURAL_API_SOCKET"

# Start BearDog first (security primal)
if [[ -f "$SPORE_ROOT/primals/bin/beardog" ]]; then
    "$SPORE_ROOT/primals/bin/beardog" server --socket "$BEARDOG_SOCKET" &
    BEARDOG_PID=$!
    echo "   BearDog PID: $BEARDOG_PID"
    sleep 1
    
    # Verify BearDog is running
    if ! kill -0 $BEARDOG_PID 2>/dev/null; then
        echo "❌ BearDog failed to start"
        exit 1
    fi
else
    echo "⚠️  BearDog binary not found - skipping"
fi

# Start Songbird (HTTP/TLS)
if [[ -f "$SPORE_ROOT/primals/bin/songbird" ]]; then
    "$SPORE_ROOT/primals/bin/songbird" serve --socket "$SONGBIRD_SOCKET" &
    SONGBIRD_PID=$!
    echo "   Songbird PID: $SONGBIRD_PID"
    sleep 1
else
    echo "⚠️  Songbird binary not found - skipping"
fi

echo ""
echo "✅ LiveSpore $NODE_ID is running"
echo ""
echo "Federation info:"
echo "   BearDog socket:  $BEARDOG_SOCKET"
echo "   Songbird socket: $SONGBIRD_SOCKET"
echo ""
# NOTE: federation_verify_lineage.toml does not exist; use biomeos verify-lineage or gossip_federation.toml
# echo "To verify lineage with a sibling:"
# echo "   biomeos verify-lineage <sibling_spore_path>"
echo ""
echo "Press Ctrl+C to stop..."
wait
DEPLOY_SCRIPT

# Replace placeholders
sed -i "s/__NODE_ID__/$NODE_ID/g" "$SPORE_ROOT/deploy.sh"
sed -i "s/__FAMILY_ID__/$FAMILY_ID/g" "$SPORE_ROOT/deploy.sh"
chmod +x "$SPORE_ROOT/deploy.sh"

log_success "Created deploy.sh"

# ============================================================================
# Create README
# ============================================================================

cat > "$SPORE_ROOT/README.md" << EOF
# LiveSpore: $NODE_ID

Genetically linked sibling spore created from parent lineage.

## Lineage Information

- **Node ID**: $NODE_ID
- **Family ID**: $FAMILY_ID
- **Lineage Mode**: sibling
- **Parent Hash**: $PARENT_HASH...
- **Deployment Batch**: $DEPLOYMENT_BATCH
- **Created**: $(date)

## Genetic Verification

This spore's seed was derived from a parent seed using:

\`\`\`
child_seed = SHA256(parent_seed || node_id || deployment_batch)
\`\`\`

This ensures:
- **Uniqueness**: Each sibling has a distinct cryptographic identity
- **Relationship**: BearDog can verify genetic relationship with siblings
- **Trust**: Siblings can establish mutual trust via federation

## Usage

### Start the Spore

\`\`\`bash
./deploy.sh
\`\`\`

### Verify Lineage with Sibling

\`\`\`bash
# Use biomeos verify-lineage (federation_verify_lineage.toml graph does not exist)
biomeos verify-lineage <sibling_spore_path> --detailed
\`\`\`

## Security

- \`.family.seed\` has 0600 permissions (owner read/write only)
- Seed is 32 bytes of cryptographically derived entropy
- BearDog uses Blake3 for lineage proof generation/verification
- Songbird only trusts peers verified via genetic lineage

## Files

- \`.family.seed\` - Cryptographic family seed (DO NOT SHARE)
- \`.spore.json\` - Spore metadata
- \`tower.toml\` - Tower Atomic configuration
- \`deploy.sh\` - Deployment script
- \`primals/bin/\` - Primal binaries
- \`graphs/\` - Deployment graphs
EOF

log_success "Created README.md"

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
log_success "Sibling spore created successfully!"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "  Node ID:     $NODE_ID"
echo "  Family:      $FAMILY_ID"
echo "  Location:    $SPORE_ROOT"
echo "  Seed Hash:   $CHILD_HASH..."
echo ""
echo "  To start this spore:"
echo "    cd $SPORE_ROOT && ./deploy.sh"
echo ""
echo "  To verify lineage with parent:"
echo "    biomeos verify-lineage <sibling_path> --detailed"
echo ""

