#!/bin/bash
# Deploy BiomeOS to Test VMs - Using SSH keys

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USB_PACKAGE="$SCRIPT_DIR/biomeos-20251228-163320.tar.gz"

# Load VM IPs
if [ ! -f "$SCRIPT_DIR/vm-ips.env" ]; then
    echo "❌ VM IPs not found. Run create-test-vms-fixed.sh first."
    exit 1
fi

source "$SCRIPT_DIR/vm-ips.env"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  📦 Deploying BiomeOS to VMs 📦                         ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "Targets:"
echo "  tower-alpha: $ALPHA_IP"
echo "  tower-beta:  $BETA_IP"
echo ""

# Test SSH connectivity
echo "🔌 Testing SSH connectivity..."
for i in {1..15}; do
    if ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 biomeos@$ALPHA_IP "echo 'alpha ok'" &>/dev/null; then
        echo "✅ tower-alpha SSH ready"
        break
    fi
    echo "⏳ Waiting for tower-alpha SSH (attempt $i/15)..."
    sleep 10
done

for i in {1..15}; do
    if ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 biomeos@$BETA_IP "echo 'beta ok'" &>/dev/null; then
        echo "✅ tower-beta SSH ready"
        break
    fi
    echo "⏳ Waiting for tower-beta SSH (attempt $i/15)..."
    sleep 10
done

echo ""

# Copy USB package to both VMs
echo "📤 Copying USB package to VMs (45MB each)..."
scp -o StrictHostKeyChecking=no "$USB_PACKAGE" biomeos@$ALPHA_IP:/tmp/ &
scp -o StrictHostKeyChecking=no "$USB_PACKAGE" biomeos@$BETA_IP:/tmp/ &
wait

echo "✅ Packages copied"
echo ""

# Deploy to tower-alpha
echo "🚀 Deploying to tower-alpha ($ALPHA_IP)..."
ssh -o StrictHostKeyChecking=no biomeos@$ALPHA_IP << 'DEPLOY_ALPHA'
set -e
cd /tmp
sudo mkdir -p /mnt/usb
sudo tar -xzf biomeos-*.tar.gz -C /mnt/usb
cd /mnt/usb/install
sudo ./install-biomeos.sh
cd /opt/biomeos
sudo -E ./deploy-real-primals.sh &
sleep 15
if curl -s http://localhost:9020/health > /dev/null 2>&1; then
    echo "✅ NestGate healthy"
else
    echo "⚠️  NestGate not responding yet (may need more time)"
fi
DEPLOY_ALPHA

echo "✅ tower-alpha deployed"
echo ""

# Deploy to tower-beta
echo "🚀 Deploying to tower-beta ($BETA_IP)..."
ssh -o StrictHostKeyChecking=no biomeos@$BETA_IP << 'DEPLOY_BETA'
set -e
cd /tmp
sudo mkdir -p /mnt/usb
sudo tar -xzf biomeos-*.tar.gz -C /mnt/usb
cd /mnt/usb/install
sudo ./install-biomeos.sh
cd /opt/biomeos
sudo -E ./deploy-real-primals.sh &
sleep 15
if curl -s http://localhost:9020/health > /dev/null 2>&1; then
    echo "✅ NestGate healthy"
else
    echo "⚠️  NestGate not responding yet (may need more time)"
fi
DEPLOY_BETA

echo "✅ tower-beta deployed"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "✅ DEPLOYMENT COMPLETE!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Next: Run validation"
echo "  ./validate-federation.sh"
echo ""

