#!/bin/bash
# biomeOS Tower Deployment Script
# Deploys Songbird v8.14.0 and supporting primals to a remote tower

set -e

REMOTE_HOST="${1:-192.168.1.134}"
REMOTE_USER="${2:-strandgate}"
FAMILY_ID="${3:-nat0}"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              biomeOS Tower Deployment Script                     ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Target: ${REMOTE_USER}@${REMOTE_HOST}"
echo "Family: ${FAMILY_ID}"
echo ""

# Check SSH connectivity
echo "=== Checking SSH connectivity ==="
if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "${REMOTE_USER}@${REMOTE_HOST}" echo "SSH OK" 2>/dev/null; then
    echo "❌ Cannot connect to ${REMOTE_HOST}"
    echo "   Ensure SSH keys are set up: ssh-copy-id ${REMOTE_USER}@${REMOTE_HOST}"
    exit 1
fi

# Create remote directories
echo ""
echo "=== Creating remote directories ==="
ssh "${REMOTE_USER}@${REMOTE_HOST}" "mkdir -p ~/biomeOS/plasmidBin ~/biomeOS/graphs"

# Copy binaries
echo ""
echo "=== Copying binaries ==="
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
for binary in beardog songbird neural-api-server; do
    if [ -f "${SCRIPT_DIR}/plasmidBin/${binary}" ]; then
        echo "   Copying ${binary}..."
        scp "${SCRIPT_DIR}/plasmidBin/${binary}" "${REMOTE_USER}@${REMOTE_HOST}:~/biomeOS/plasmidBin/"
    else
        echo "   ⚠️  ${binary} not found, skipping"
    fi
done

# Copy deployment graphs
echo ""
echo "=== Copying deployment graphs ==="
scp "${SCRIPT_DIR}/graphs/tower_atomic_bootstrap.toml" "${REMOTE_USER}@${REMOTE_HOST}:~/biomeOS/graphs/"

# Copy bootstrap script
echo ""
echo "=== Copying bootstrap script ==="
scp "${SCRIPT_DIR}/scripts/bootstrap_tower_atomic.sh" "${REMOTE_USER}@${REMOTE_HOST}:~/biomeOS/"

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              ✅ Deployment Complete                              ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "To start Tower Atomic on remote:"
echo "  ssh ${REMOTE_USER}@${REMOTE_HOST}"
echo "  cd ~/biomeOS && FAMILY_ID=${FAMILY_ID} NODE_ID=tower2 ./bootstrap_tower_atomic.sh"
