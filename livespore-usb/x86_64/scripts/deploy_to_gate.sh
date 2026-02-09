#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# deploy_to_gate.sh — Deploy LiveSpore to a LAN computer and link it
#
# Usage:
#   ./deploy_to_gate.sh <target-ip> [node-id] [user]
#
# Example:
#   ./deploy_to_gate.sh 192.168.1.132 gate2
#   ./deploy_to_gate.sh 192.168.1.132 gate2 eastgate
#
# What this does:
#   1. Copies LiveSpore package (binaries, seeds, scripts) to target via scp
#   2. Derives a unique lineage.seed for the new gate via Tower BearDog
#   3. SSHs into target and runs start_tower.sh
#   4. The new gate auto-discovers and links to this Tower via Dark Forest
#
# Prerequisites:
#   - SSH access to target (password or key-based)
#   - Target is Linux x86_64 on the same LAN
#   - Tower BearDog running (for lineage derivation)
#═══════════════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USB_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"
PROJECT_ROOT="$(cd "$USB_ROOT/.." && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

# Parse args
TARGET_IP="${1:?Usage: $0 <target-ip> [node-id] [user]}"
NODE_ID="${2:-gate2}"
TARGET_USER="${3:-$(whoami)}"
DEPLOY_DIR="/tmp/biomeos-livespore"

echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}  LiveSpore Gate Deployment${NC}"
echo -e "${BOLD}  Target: ${TARGET_USER}@${TARGET_IP}  Node: ${NODE_ID}${NC}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

#───────────────────────────────────────────────────────────────────────────────
# Step 0: Verify prerequisites
#───────────────────────────────────────────────────────────────────────────────
echo -e "${BOLD}Step 0: Prerequisites${NC}"

# Target reachable?
if ! ping -c 1 -W 2 "$TARGET_IP" >/dev/null 2>&1; then
    echo -e "  ${RED}Target $TARGET_IP not reachable${NC}"
    exit 1
fi
echo -e "  ${GREEN}Target reachable${NC}"

# Family seed?
FAMILY_SEED="$USB_ROOT/.family.seed"
if [ ! -f "$FAMILY_SEED" ]; then
    FAMILY_SEED="$PROJECT_ROOT/.family.seed"
fi
if [ ! -f "$FAMILY_SEED" ]; then
    echo -e "  ${RED}No .family.seed found${NC}"
    exit 1
fi
FAMILY_ID=$(xxd -p -l 8 "$FAMILY_SEED" | tr -d '\n')
echo -e "  ${GREEN}Family ID: ${FAMILY_ID}${NC}"

# Binaries exist?
if [ ! -x "$PRIMAL_DIR/beardog" ] || [ ! -x "$PRIMAL_DIR/songbird" ]; then
    echo -e "  ${RED}Primals not found in $PRIMAL_DIR${NC}"
    exit 1
fi
echo -e "  ${GREEN}Binaries: beardog $(stat -c%s "$PRIMAL_DIR/beardog" | numfmt --to=iec), songbird $(stat -c%s "$PRIMAL_DIR/songbird" | numfmt --to=iec)${NC}"

# Tower BearDog running? (for lineage derivation)
BEARDOG_SOCK="${BEARDOG_SOCKET:-/run/user/$(id -u)/biomeos/beardog.sock}"
BD_HEALTH=$(echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U "$BEARDOG_SOCK" -w 3 -q 1 2>/dev/null || true)
if echo "$BD_HEALTH" | grep -q '"healthy"'; then
    echo -e "  ${GREEN}Tower BearDog: healthy${NC}"
else
    echo -e "  ${YELLOW}Tower BearDog: not running (lineage will be derived on target)${NC}"
fi

echo ""

#───────────────────────────────────────────────────────────────────────────────
# Step 1: Derive unique lineage for the new gate
#───────────────────────────────────────────────────────────────────────────────
echo -e "${BOLD}Step 1: Deriving lineage for ${NODE_ID}${NC}"

STAGING="/tmp/livespore-staging-${NODE_ID}"
rm -rf "$STAGING"
mkdir -p "$STAGING"

# Copy family seed
cp "$FAMILY_SEED" "$STAGING/.family.seed"

# Copy beacon seed if exists
[ -f "$USB_ROOT/.beacon.seed" ] && cp "$USB_ROOT/.beacon.seed" "$STAGING/.beacon.seed"

# Derive lineage seed via BearDog (if available)
if echo "$BD_HEALTH" | grep -q '"healthy"'; then
    # Generate entropy
    ENTROPY_B64=$(dd if=/dev/urandom bs=16 count=1 2>/dev/null | base64 | tr -d '\n')
    DEVICE_ID_B64=$(echo -n "$NODE_ID" | base64)
    
    LINEAGE_RESP=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"genetic.derive_device_seed\",\"params\":{\"family_seed\":\"$(base64 -w0 "$FAMILY_SEED")\",\"device_id\":\"$DEVICE_ID_B64\",\"entropy\":\"$ENTROPY_B64\"},\"id\":1}" | \
        nc -U "$BEARDOG_SOCK" -w 5 -q 1 2>/dev/null || true)
    
    if echo "$LINEAGE_RESP" | grep -q '"device_seed"'; then
        LINEAGE_SEED_B64=$(echo "$LINEAGE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['device_seed'])" 2>/dev/null)
        echo "$LINEAGE_SEED_B64" | base64 -d > "$STAGING/.lineage.seed"
        LINEAGE_HASH=$(echo "$LINEAGE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result'].get('seed_hash','?'))" 2>/dev/null)
        echo -e "  ${GREEN}Lineage seed derived (hash: ${LINEAGE_HASH:0:16}...)${NC}"
        
        # Create lineage.json
        python3 -c "
import json, time
d = {
    'device_id': '$NODE_ID',
    'family_id': '$FAMILY_ID',
    'lineage_seed_hash': '$LINEAGE_HASH',
    'kdf': 'beardog-genetic-derive',
    'derived_at': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
    'parent_device': 'gate',
    'generation': 2
}
with open('$STAGING/.lineage.json', 'w') as f:
    json.dump(d, f, indent=2)
print('  Lineage identity written')
"
    else
        echo -e "  ${YELLOW}BearDog derivation failed, will derive on target${NC}"
    fi
else
    echo -e "  ${YELLOW}Skipping BearDog derivation (offline)${NC}"
fi

# Copy known_beacons (with Tower endpoint)
cp "$USB_ROOT/.known_beacons.json" "$STAGING/.known_beacons.json"

# Update known_beacons for this specific gate
python3 -c "
import json
with open('$STAGING/.known_beacons.json') as f:
    d = json.load(f)
d['this_node']['node_id'] = '$NODE_ID'
# Update Tower gate LAN endpoint with current IP
d.setdefault('family_members', {}).setdefault('gate', {})['endpoints'] = {
    'lan': '$(hostname -I | awk \"{print \$1}\"):3492',
    'ipv6': '[2600:1700:b0b0:5b90::27]:3492',
    'dns': 'tower.nestgate.io',
    'onion': 'p6m5exqn44xpjtvpal6juhdzh3s7zvlpysrjcknzrxada6mny54ltiyd.onion:3492'
}
with open('$STAGING/.known_beacons.json', 'w') as f:
    json.dump(d, f, indent=2)
print('  known_beacons updated for $NODE_ID')
" 2>/dev/null

echo ""

#───────────────────────────────────────────────────────────────────────────────
# Step 2: Stage deployment package
#───────────────────────────────────────────────────────────────────────────────
echo -e "${BOLD}Step 2: Staging deployment package${NC}"

# Create deployment structure
mkdir -p "$STAGING/primals"
mkdir -p "$STAGING/scripts"
mkdir -p "$STAGING/graphs"

# Copy binaries
cp "$PRIMAL_DIR/beardog" "$STAGING/primals/"
cp "$PRIMAL_DIR/songbird" "$STAGING/primals/"
[ -x "$PRIMAL_DIR/squirrel" ] && cp "$PRIMAL_DIR/squirrel" "$STAGING/primals/"
[ -x "$PRIMAL_DIR/toadstool" ] && cp "$PRIMAL_DIR/toadstool" "$STAGING/primals/"
[ -x "$PRIMAL_DIR/nestgate" ] && cp "$PRIMAL_DIR/nestgate" "$STAGING/primals/"
echo -e "  ${GREEN}Binaries staged${NC}"

# Copy scripts
cp "$SCRIPT_DIR/start_tower.sh" "$STAGING/scripts/"
chmod +x "$STAGING/scripts/start_tower.sh"

# Copy graphs
for graph in "$SCRIPT_DIR/../graphs/"*.toml; do
    [ -f "$graph" ] && cp "$graph" "$STAGING/graphs/"
done
echo -e "  ${GREEN}Scripts and graphs staged${NC}"

# Create a simple launcher
cat > "$STAGING/start.sh" << 'LAUNCHER'
#!/bin/bash
# LiveSpore Quick Start — runs from wherever the LiveSpore is
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Accept NODE_ID from env or arg
export NODE_ID="${NODE_ID:-${1:-gate2}}"
export BIOMEOS_ROOT="$SCRIPT_DIR"

# Resolve architecture-specific paths
export PATH="$SCRIPT_DIR/primals:$PATH"
chmod +x "$SCRIPT_DIR/primals/"* 2>/dev/null

# Run the full startup
exec bash "$SCRIPT_DIR/scripts/start_tower.sh"
LAUNCHER
chmod +x "$STAGING/start.sh"
echo -e "  ${GREEN}Launcher created${NC}"

# Total package size
TOTAL_SIZE=$(du -sh "$STAGING" | awk '{print $1}')
echo -e "  ${GREEN}Package size: ${TOTAL_SIZE}${NC}"
echo ""

#───────────────────────────────────────────────────────────────────────────────
# Step 3: Deploy to target
#───────────────────────────────────────────────────────────────────────────────
echo -e "${BOLD}Step 3: Deploying to ${TARGET_USER}@${TARGET_IP}:${DEPLOY_DIR}${NC}"

# Create target directory
ssh "${TARGET_USER}@${TARGET_IP}" "mkdir -p ${DEPLOY_DIR}" 2>/dev/null

# rsync the package (fast, delta-only transfers)
if command -v rsync >/dev/null 2>&1; then
    rsync -az --progress "$STAGING/" "${TARGET_USER}@${TARGET_IP}:${DEPLOY_DIR}/"
else
    scp -r "$STAGING/"* "${TARGET_USER}@${TARGET_IP}:${DEPLOY_DIR}/"
fi
echo -e "  ${GREEN}Package deployed${NC}"

# Make binaries executable on target
ssh "${TARGET_USER}@${TARGET_IP}" "chmod +x ${DEPLOY_DIR}/primals/* ${DEPLOY_DIR}/*.sh ${DEPLOY_DIR}/scripts/*.sh" 2>/dev/null
echo ""

#───────────────────────────────────────────────────────────────────────────────
# Step 4: Start on target
#───────────────────────────────────────────────────────────────────────────────
echo -e "${BOLD}Step 4: Starting NUCLEUS on target${NC}"
echo -e "  ${YELLOW}Ensure socat is installed on target (for LAN RPC bridge):${NC}"
echo -e "  ${CYAN}ssh ${TARGET_USER}@${TARGET_IP} 'which socat || sudo apt install -y socat'${NC}"
echo ""
echo -e "  ${YELLOW}Then run on the target machine:${NC}"
echo ""
echo -e "  ${CYAN}ssh ${TARGET_USER}@${TARGET_IP}${NC}"
echo -e "  ${CYAN}cd ${DEPLOY_DIR} && NODE_ID=${NODE_ID} ./start.sh${NC}"
echo ""
echo -e "  ${YELLOW}Or from here (interactive, recommended for first run):${NC}"
echo ""
echo -e "  ${CYAN}ssh -t ${TARGET_USER}@${TARGET_IP} 'cd ${DEPLOY_DIR} && NODE_ID=${NODE_ID} bash start.sh'${NC}"
echo ""

#───────────────────────────────────────────────────────────────────────────────
# Step 5: Tower-side mesh link preparation
#───────────────────────────────────────────────────────────────────────────────
echo -e "${BOLD}Step 5: Tower mesh ready${NC}"

SONGBIRD_SOCK="${SONGBIRD_SOCKET:-/run/user/$(id -u)/biomeos/songbird.sock}"
if [ -S "$SONGBIRD_SOCK" ]; then
    # Announce this deployment to the Tower's mesh
    echo '{"jsonrpc":"2.0","method":"mesh.announce","params":{"node_id":"'$NODE_ID'","address":"'$TARGET_IP':3492","capabilities":["compute","discovery"]},"id":1}' | \
        nc -U "$SONGBIRD_SOCK" -w 5 -q 1 2>/dev/null | python3 -c "
import sys,json
r = json.load(sys.stdin)
if 'result' in r:
    print(f'  Tower mesh: announced {\"$NODE_ID\"} at ${TARGET_IP}:3492')
else:
    print(f'  Tower mesh: {r.get(\"error\",{}).get(\"message\",\"pending\")}')" 2>/dev/null
fi

echo ""
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}  LiveSpore deployment complete${NC}"
echo -e "${BOLD}  Target: ${TARGET_USER}@${TARGET_IP}  Node: ${NODE_ID}${NC}"
echo -e "${BOLD}  Path: ${DEPLOY_DIR}${NC}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "After the target starts, verify the mesh link:"
echo ""
echo "  # From Tower:"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"mesh.peers\",\"params\":{},\"id\":1}' | nc -U ${SONGBIRD_SOCK} -w 5 -q 1"
echo ""
echo "  # Quick LAN connectivity test:"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"params\":{},\"id\":1}' | nc ${TARGET_IP} 3492 -w 3"
echo ""
