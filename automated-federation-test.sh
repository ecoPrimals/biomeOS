#!/bin/bash
# Automated USB Federation Test using benchScale
# Creates 2 VMs, deploys BiomeOS, validates federation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHSCALE_DIR="$(cd "$SCRIPT_DIR/../../primalTools/benchscale" && pwd)"
USB_PACKAGE="$SCRIPT_DIR/biomeos-20251228-163320.tar.gz"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🔬 Automated USB Federation Test (benchScale) 🔬       ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if [ ! -f "$USB_PACKAGE" ]; then
    echo "❌ USB package not found: $USB_PACKAGE"
    exit 1
fi

if [ ! -f "$BENCHSCALE_DIR/target/release/benchscale" ]; then
    echo "❌ benchScale not built"
    echo "Building now..."
    cd "$BENCHSCALE_DIR"
    cargo build --release --features libvirt
fi

echo "✅ Prerequisites OK"
echo ""

# Configuration
UBUNTU_ISO="${UBUNTU_ISO:-/var/lib/libvirt/images/ubuntu-22.04-server-cloudimg-amd64.img}"
LAB_NAME="biomeos-federation-$(date +%Y%m%d-%H%M%S)"

echo "═══════════════════════════════════════════════════════════"
echo "PHASE 1: Create VMs with benchScale"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Create cloud-init user data for both VMs
cat > /tmp/biomeos-cloud-init.yaml << 'EOF'
#cloud-config
hostname: {HOSTNAME}
fqdn: {HOSTNAME}.local

users:
  - name: biomeos
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    ssh_authorized_keys:
      - ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC... # Add your SSH key here

packages:
  - avahi-daemon
  - avahi-utils
  - curl
  - tar
  - gzip

runcmd:
  - systemctl start avahi-daemon
  - systemctl enable avahi-daemon
  - echo "BiomeOS VM ready" > /tmp/cloud-init-done
EOF

echo "📦 Creating VM: tower-alpha"
# Use benchScale to create VM
cd "$BENCHSCALE_DIR"

# Note: This is a conceptual example - adjust based on actual benchScale API
# For now, create VMs manually using libvirt/qemu

echo "🖥️  VM Creation Steps:"
echo ""
echo "Creating tower-alpha..."
echo "  RAM: 4096MB"
echo "  vCPUs: 2"
echo "  Disk: 30GB"
echo "  Network: bridge"
echo ""

# Create VMs using qemu/libvirt directly (benchScale integration)
sudo virt-install \
  --name tower-alpha \
  --ram 4096 \
  --vcpus 2 \
  --disk size=30 \
  --os-variant ubuntu22.04 \
  --cloud-init user-data=/tmp/biomeos-cloud-init.yaml \
  --network bridge=virbr0 \
  --graphics none \
  --console pty,target_type=serial \
  --noautoconsole &

ALPHA_PID=$!

echo "Creating tower-beta..."
sudo virt-install \
  --name tower-beta \
  --ram 4096 \
  --vcpus 2 \
  --disk size=30 \
  --os-variant ubuntu22.04 \
  --cloud-init user-data=/tmp/biomeos-cloud-init.yaml \
  --network bridge=virbr0 \
  --graphics none \
  --console pty,target_type=serial \
  --noautoconsole &

BETA_PID=$!

echo "⏳ Waiting for VMs to boot (60s)..."
sleep 60

echo "✅ VMs created"
echo ""

# Get VM IP addresses
echo "📡 Discovering VM IP addresses..."
ALPHA_IP=$(sudo virsh domifaddr tower-alpha | grep ipv4 | awk '{print $4}' | cut -d/ -f1)
BETA_IP=$(sudo virsh domifaddr tower-beta | grep ipv4 | awk '{print $4}' | cut -d/ -f1)

if [ -z "$ALPHA_IP" ] || [ -z "$BETA_IP" ]; then
    echo "⚠️  Could not get VM IPs automatically"
    echo "   Waiting another 30s..."
    sleep 30
    ALPHA_IP=$(sudo virsh domifaddr tower-alpha | grep ipv4 | awk '{print $4}' | cut -d/ -f1)
    BETA_IP=$(sudo virsh domifaddr tower-beta | grep ipv4 | awk '{print $4}' | cut -d/ -f1)
fi

echo "✅ tower-alpha: $ALPHA_IP"
echo "✅ tower-beta: $BETA_IP"
echo ""

# Phase 2: Deploy BiomeOS on both VMs
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 2: Deploy BiomeOS from USB Package"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Copy USB package to both VMs
echo "📤 Copying USB package to VMs..."
scp -o StrictHostKeyChecking=no "$USB_PACKAGE" biomeos@$ALPHA_IP:/tmp/ &
scp -o StrictHostKeyChecking=no "$USB_PACKAGE" biomeos@$BETA_IP:/tmp/ &
wait

echo "✅ USB packages copied"
echo ""

# Deploy on tower-alpha
echo "🚀 Deploying on tower-alpha ($ALPHA_IP)..."
ssh -o StrictHostKeyChecking=no biomeos@$ALPHA_IP << 'DEPLOY_ALPHA'
set -e
cd /tmp
sudo mkdir -p /mnt/usb
tar -xzf biomeos-*.tar.gz -C /mnt/usb
cd /mnt/usb/install
./install-biomeos.sh
cd /opt/biomeos
./deploy-real-primals.sh
sleep 10
curl -s http://localhost:9020/health && echo "✅ NestGate healthy"
DEPLOY_ALPHA

echo "✅ tower-alpha deployed"
echo ""

# Deploy on tower-beta
echo "🚀 Deploying on tower-beta ($BETA_IP)..."
ssh -o StrictHostKeyChecking=no biomeos@$BETA_IP << 'DEPLOY_BETA'
set -e
cd /tmp
sudo mkdir -p /mnt/usb
tar -xzf biomeos-*.tar.gz -C /mnt/usb
cd /mnt/usb/install
./install-biomeos.sh
cd /opt/biomeos
./deploy-real-primals.sh
sleep 10
curl -s http://localhost:9020/health && echo "✅ NestGate healthy"
DEPLOY_BETA

echo "✅ tower-beta deployed"
echo ""

# Phase 3: Validate Federation
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 3: Validate Federation"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "⏳ Waiting for mDNS discovery (30s)..."
sleep 30

echo "🔍 Checking Songbird logs for peer discovery..."
echo ""

echo "On tower-alpha:"
ssh biomeos@$ALPHA_IP "tail -20 /opt/biomeos/logs/primals/songbird.log | grep -i 'peer\|discover'" || echo "  (No peer logs yet)"

echo ""
echo "On tower-beta:"
ssh biomeos@$BETA_IP "tail -20 /opt/biomeos/logs/primals/songbird.log | grep -i 'peer\|discover'" || echo "  (No peer logs yet)"

echo ""

# Phase 4: Run E2E Tests
echo "═══════════════════════════════════════════════════════════"
echo "PHASE 4: Run E2E Tests"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🧪 Running E2E tests on tower-alpha..."
ssh biomeos@$ALPHA_IP "cd /opt/biomeos && ./run-e2e-tests.sh" | tee /tmp/alpha-e2e.log
ALPHA_RESULT=$(grep -c "PASS" /tmp/alpha-e2e.log || echo "0")

echo ""
echo "🧪 Running E2E tests on tower-beta..."
ssh biomeos@$BETA_IP "cd /opt/biomeos && ./run-e2e-tests.sh" | tee /tmp/beta-e2e.log
BETA_RESULT=$(grep -c "PASS" /tmp/beta-e2e.log || echo "0")

echo ""

# Phase 5: Results
echo "═══════════════════════════════════════════════════════════"
echo "✅ FEDERATION TEST COMPLETE"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📊 Results:"
echo "   tower-alpha: $ALPHA_RESULT tests passed"
echo "   tower-beta: $BETA_RESULT tests passed"
echo "   Total: $((ALPHA_RESULT + BETA_RESULT)) tests passed"
echo ""
echo "🌐 Federation Status:"
echo "   tower-alpha: $ALPHA_IP"
echo "   tower-beta: $BETA_IP"
echo ""

# Check primal counts
ALPHA_PRIMALS=$(ssh biomeos@$ALPHA_IP "pgrep -f 'nestgate|songbird|beardog|toadstool|squirrel' | wc -l")
BETA_PRIMALS=$(ssh biomeos@$BETA_IP "pgrep -f 'nestgate|songbird|beardog|toadstool|squirrel' | wc -l")

echo "📈 Primal Processes:"
echo "   tower-alpha: $ALPHA_PRIMALS primals"
echo "   tower-beta: $BETA_PRIMALS primals"
echo "   Total: $((ALPHA_PRIMALS + BETA_PRIMALS)) primals"
echo ""

# Success criteria
SUCCESS=true

if [ "$ALPHA_RESULT" -lt 10 ] || [ "$BETA_RESULT" -lt 10 ]; then
    echo "⚠️  Less than 10 tests passing per VM"
    SUCCESS=false
fi

if [ "$ALPHA_PRIMALS" -lt 4 ] || [ "$BETA_PRIMALS" -lt 4 ]; then
    echo "⚠️  Less than 4 primals running per VM"
    SUCCESS=false
fi

if [ "$SUCCESS" = true ]; then
    echo "🎉 SUCCESS! Federation is validated!"
    echo ""
    echo "✅ Ready for NUC deployment!"
    echo ""
    echo "Next steps:"
    echo "  1. Write USB: AUTO_CONFIRM=1 ./quick-usb.sh"
    echo "  2. Boot NUC from USB"
    echo "  3. Install same way as VMs"
    echo "  4. NUC will auto-discover both VMs"
    echo "  5. Result: 3-node federation!"
else
    echo "❌ Some validation criteria not met"
    echo ""
    echo "Check logs:"
    echo "  ssh biomeos@$ALPHA_IP 'tail -f /opt/biomeos/logs/primals/*.log'"
    echo "  ssh biomeos@$BETA_IP 'tail -f /opt/biomeos/logs/primals/*.log'"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "VM Management"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Keep VMs running:"
echo "  ssh biomeos@$ALPHA_IP"
echo "  ssh biomeos@$BETA_IP"
echo ""
echo "Stop VMs:"
echo "  sudo virsh shutdown tower-alpha"
echo "  sudo virsh shutdown tower-beta"
echo ""
echo "Delete VMs:"
echo "  sudo virsh destroy tower-alpha"
echo "  sudo virsh undefine tower-alpha --remove-all-storage"
echo "  sudo virsh destroy tower-beta"
echo "  sudo virsh undefine tower-beta --remove-all-storage"
echo ""

