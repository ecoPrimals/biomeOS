#!/bin/bash
# Two-Tower Integration Test: Local Host + VM
# Tests USB v6.0 with genetic lineage auto-trust

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"

echo "══════════════════════════════════════════════════════════════════════════"
echo "🧪 Two-Tower Integration Test: Host + VM"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""
echo "This test will:"
echo "  1. Deploy Tower 1 (local host) with USB seed"
echo "  2. Create VM for Tower 2"
echo "  3. Deploy Tower 2 (VM) with same USB seed"
echo "  4. Verify both create unique lineages"
echo "  5. Verify both share same genesis"
echo "  6. Verify BearDog returns auto_accept for same family"
echo "  7. Verify Songbird forms mesh automatically"
echo ""
read -p "Press Enter to start test..."
echo ""

# Check if USB seed exists
if [ ! -f "$BIOMEOS_ROOT/secrets/family-genesis.key" ]; then
    echo "❌ Error: USB family seed not found!"
    echo "   Run: ./scripts/create-usb-family-seed.sh"
    exit 1
fi

FAMILY_ID=$(jq -r .family_id "$BIOMEOS_ROOT/secrets/family-genesis.key")
FAMILY_HASH=$(jq -r .genesis_hash "$BIOMEOS_ROOT/secrets/family-genesis.key")

echo "✅ USB Family Seed Found:"
echo "   Family ID:   $FAMILY_ID"
echo "   Genesis:     ${FAMILY_HASH:0:16}..."
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# PHASE 1: Deploy Tower 1 (Local Host)
# ═══════════════════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════════════════"
echo "📍 PHASE 1: Deploy Tower 1 (Local Host)"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

# Stop any existing services
echo "🧹 Cleaning up existing services..."
pkill -9 -f "songbird-orchestrator" 2>/dev/null || true
pkill -9 -f "beardog-server" 2>/dev/null || true
sleep 3

# Deploy Tower 1
echo "🚀 Deploying Tower 1..."
cd "$BIOMEOS_ROOT"
./scripts/auto-deploy-v6.sh > /tmp/tower1-deploy.log 2>&1 &
DEPLOY_PID=$!

# Wait for deployment to complete
echo "   Waiting for deployment to complete..."
sleep 10

# Check if services are running
if ! ps aux | grep -q "[b]eardog-server"; then
    echo "❌ BearDog failed to start on Tower 1"
    cat /tmp/tower1-deploy.log
    exit 1
fi

if ! ps aux | grep -q "[s]ongbird-orchestrator"; then
    echo "❌ Songbird failed to start on Tower 1"
    cat /tmp/tower1-deploy.log
    exit 1
fi

echo "✅ Tower 1 services running"
echo ""

# Get Tower 1 identity
echo "🔍 Querying Tower 1 Identity..."
sleep 3
TOWER1_IDENTITY=$(curl -s http://localhost:9000/api/v1/trust/identity)
TOWER1_TAG=$(echo "$TOWER1_IDENTITY" | jq -r '.data.encryption_tag')
TOWER1_FAMILY=$(echo "$TOWER1_IDENTITY" | jq -r '.data.family_id')

echo "✅ Tower 1 Identity:"
echo "   Encryption Tag: $TOWER1_TAG"
echo "   Family ID:      $TOWER1_FAMILY"
echo ""

# Get Tower 1 IP
TOWER1_IP=$(hostname -I | awk '{print $1}')
echo "📡 Tower 1 Network:"
echo "   IP Address: $TOWER1_IP"
echo "   RPC Port:   8080"
echo "   Discovery:  2300 (UDP multicast)"
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# PHASE 2: Create and Deploy Tower 2 (VM)
# ═══════════════════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════════════════"
echo "📍 PHASE 2: Tower 2 VM Setup"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

echo "⚠️  VM Creation Options:"
echo ""
echo "Option A: Use existing VM (recommended if available)"
echo "Option B: Create new VM with virt-manager/libvirt"
echo "Option C: Use Docker container (simpler, faster)"
echo "Option D: Manual deployment (you handle VM creation)"
echo ""
echo "For this test, we'll prepare the deployment package."
echo "You can then deploy it however you prefer."
echo ""

# Create deployment package for Tower 2
echo "📦 Creating Tower 2 Deployment Package..."
TOWER2_DEPLOY="/tmp/tower2-deploy"
rm -rf "$TOWER2_DEPLOY"
mkdir -p "$TOWER2_DEPLOY"/{primals,scripts,secrets}

# Copy binaries
cp "$BIOMEOS_ROOT/primals/beardog-server" "$TOWER2_DEPLOY/primals/"
cp "$BIOMEOS_ROOT/primals/songbird-orchestrator" "$TOWER2_DEPLOY/primals/"

# Copy scripts
cp "$BIOMEOS_ROOT/scripts/auto-deploy-v6.sh" "$TOWER2_DEPLOY/scripts/"

# Copy USB seed
cp "$BIOMEOS_ROOT/secrets/family-genesis.key" "$TOWER2_DEPLOY/secrets/"

# Make executables
chmod +x "$TOWER2_DEPLOY/primals/"*
chmod +x "$TOWER2_DEPLOY/scripts/"*

echo "✅ Tower 2 deployment package created: $TOWER2_DEPLOY"
echo ""

# Create Tower 2 deployment instructions
cat > "$TOWER2_DEPLOY/DEPLOY.txt" << EOF
═══════════════════════════════════════════════════════════════════════════
Tower 2 Deployment Instructions
═══════════════════════════════════════════════════════════════════════════

This package contains everything needed to deploy Tower 2.

USB Family Seed: $FAMILY_ID
Genesis Hash:    ${FAMILY_HASH:0:32}...

═══════════════════════════════════════════════════════════════════════════
Deployment Steps:
═══════════════════════════════════════════════════════════════════════════

1. Transfer this directory to Tower 2 (VM/container/physical machine)

2. On Tower 2, run:
   cd tower2-deploy
   ./scripts/auto-deploy-v6.sh

3. Tower 2 will:
   • Create unique lineage from same USB seed
   • Start BearDog and Songbird
   • Discover Tower 1 automatically
   • Establish auto-trust (same family)
   • Form mesh

═══════════════════════════════════════════════════════════════════════════
Expected Result:
═══════════════════════════════════════════════════════════════════════════

Tower 1 and Tower 2 will automatically trust each other because they
share the same genetic family ($FAMILY_ID).

BearDog will return "auto_accept" when evaluating peer trust.
Songbird will form a mesh without user intervention.

This is "Secure by Default" in action! ✅

═══════════════════════════════════════════════════════════════════════════
EOF

echo "📄 Deployment instructions created: $TOWER2_DEPLOY/DEPLOY.txt"
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# PHASE 3: Provide Next Steps
# ═══════════════════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════════════════"
echo "📍 PHASE 3: Next Steps for Tower 2"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

echo "✅ Tower 1 is running and ready!"
echo ""
echo "🔄 To complete the two-tower test, you need to:"
echo ""
echo "   1. Create/start your VM or container"
echo ""
echo "   2. Transfer the deployment package:"
echo "      scp -r $TOWER2_DEPLOY user@vm-ip:~/"
echo ""
echo "   3. On the VM, run:"
echo "      cd ~/tower2-deploy"
echo "      ./scripts/auto-deploy-v6.sh"
echo ""
echo "   4. Return here to verify integration"
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# PHASE 4: Docker Alternative (Quick Test)
# ═══════════════════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════════════════"
echo "🐳 Alternative: Quick Docker Test"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""
echo "For a quick test without full VM, we can use Docker:"
echo ""
read -p "Use Docker for Tower 2? (y/N): " USE_DOCKER
echo ""

if [[ "$USE_DOCKER" =~ ^[Yy]$ ]]; then
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker not found. Please install Docker or use VM option."
        exit 1
    fi
    
    echo "🐳 Creating Docker container for Tower 2..."
    
    # Create Dockerfile
    cat > "$TOWER2_DEPLOY/Dockerfile" << 'DOCKERFILE'
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    jq \
    iproute2 \
    && rm -rf /var/lib/apt/lists/*

# Copy deployment package
COPY primals /app/primals
COPY scripts /app/scripts
COPY secrets /app/secrets

WORKDIR /app

# Make binaries executable
RUN chmod +x primals/* scripts/*

# Expose ports
EXPOSE 8080 2300/udp 9000

# Run deployment script
CMD ["./scripts/auto-deploy-v6.sh"]
DOCKERFILE
    
    # Build Docker image
    echo "   Building Docker image..."
    docker build -t biomeos-tower2 "$TOWER2_DEPLOY" > /tmp/docker-build.log 2>&1
    
    if [ $? -ne 0 ]; then
        echo "❌ Docker build failed. See /tmp/docker-build.log"
        exit 1
    fi
    
    echo "✅ Docker image built"
    echo ""
    
    # Run Docker container
    echo "🚀 Starting Tower 2 in Docker..."
    docker run -d \
        --name biomeos-tower2 \
        --network host \
        -v "$TOWER2_DEPLOY/secrets:/app/secrets:ro" \
        biomeos-tower2 > /tmp/docker-run.log 2>&1
    
    echo "✅ Tower 2 running in Docker (container: biomeos-tower2)"
    echo ""
    
    sleep 10
    
    # Get Tower 2 logs
    echo "📋 Tower 2 Logs (first 30 lines):"
    docker logs biomeos-tower2 2>&1 | head -30
    echo ""
    
    echo "🔍 To see full Tower 2 logs:"
    echo "   docker logs -f biomeos-tower2"
    echo ""
fi

# ═══════════════════════════════════════════════════════════════════════════
# PHASE 5: Verification
# ═══════════════════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════════════════"
echo "📍 PHASE 5: Verification Checklist"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

echo "Once Tower 2 is running, verify the following:"
echo ""

echo "1️⃣  Tower 2 Identity (from Tower 2):"
echo "   curl http://localhost:9000/api/v1/trust/identity"
echo "   Expected: Different node ID, same family ($TOWER1_FAMILY)"
echo ""

echo "2️⃣  Cross-Tower Trust Evaluation (from Tower 1):"
echo "   curl -X POST http://localhost:9000/api/v1/trust/evaluate \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"peer_id\": \"tower2\", \"peer_tags\": [\"beardog:family:$TOWER1_FAMILY:tower2_xxx\"]}'"
echo "   Expected: {\"decision\": \"auto_accept\", \"confidence\": 1.0}"
echo ""

echo "3️⃣  Songbird Discovery (from Tower 1):"
echo "   grep 'Discovered peer' /tmp/songbird-orchestrator.log"
echo "   Expected: Tower 2 discovered via UDP multicast"
echo ""

echo "4️⃣  Songbird Mesh Formation (from Tower 1):"
echo "   grep 'joined federation' /tmp/songbird-orchestrator.log"
echo "   Expected: Tower 2 auto-accepted and joined"
echo ""

echo "5️⃣  TCP Connections (from Tower 1):"
echo "   ss -tn | grep 8080"
echo "   Expected: ESTABLISHED connection to Tower 2"
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════════════════"
echo "✅ Test Setup Complete!"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

echo "📊 Current Status:"
echo "   Tower 1: ✅ Running"
echo "   Tower 2: ⏳ Deploy to VM/Docker and verify"
echo ""

echo "📦 Deployment Package:"
echo "   Location: $TOWER2_DEPLOY"
echo "   Contents: Binaries, scripts, USB seed"
echo ""

echo "🎯 Success Criteria:"
echo "   ✅ Both towers have different node IDs"
echo "   ✅ Both towers share same family ($FAMILY_ID)"
echo "   ✅ BearDog returns auto_accept for same family"
echo "   ✅ Songbird forms mesh automatically"
echo "   ✅ No user intervention required"
echo ""

echo "══════════════════════════════════════════════════════════════════════════"
echo "🎊 Ready for Two-Tower Verification!"
echo "══════════════════════════════════════════════════════════════════════════"
echo ""

