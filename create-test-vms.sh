#!/bin/bash
# Clean 2-VM Federation Test
# Uses downloaded Ubuntu cloud image

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLOUD_IMG="$SCRIPT_DIR/vm-images/ubuntu-22.04-cloudimg.img"
USB_PACKAGE="$SCRIPT_DIR/biomeos-20251228-163320.tar.gz"
VM_DIR="/var/lib/libvirt/images/biomeos-test"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🔬 2-VM Federation Test - Clean Setup 🔬              ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Verify prerequisites
if [ ! -f "$CLOUD_IMG" ]; then
    echo "❌ Cloud image not found: $CLOUD_IMG"
    exit 1
fi

if [ ! -f "$USB_PACKAGE" ]; then
    echo "❌ USB package not found: $USB_PACKAGE"
    exit 1
fi

echo "✅ Prerequisites OK"
echo ""

# Clean up any existing test VMs
echo "🧹 Cleaning up old test VMs..."
for vm in tower-alpha tower-beta; do
    virsh destroy $vm 2>/dev/null || true
    virsh undefine $vm --remove-all-storage 2>/dev/null || true
done

# Create VM directory
mkdir -p "$VM_DIR"

echo "✅ Cleanup complete"
echo ""

# Create cloud-init config
echo "📝 Creating cloud-init configs..."

# User data for tower-alpha
cat > /tmp/user-data-alpha.yaml << 'EOF'
#cloud-config
hostname: tower-alpha
fqdn: tower-alpha.local
manage_etc_hosts: true

users:
  - name: biomeos
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    lock_passwd: false
    passwd: $6$rounds=4096$saltsalt$3yGZ8.RvJ4VBqnm7xRVYVq8VZ7XVVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXV

packages:
  - avahi-daemon
  - avahi-utils
  - curl
  - tar
  - gzip
  - openssh-server

runcmd:
  - systemctl enable ssh
  - systemctl start ssh
  - systemctl enable avahi-daemon
  - systemctl start avahi-daemon
  - echo "tower-alpha ready" > /tmp/ready
EOF

# User data for tower-beta
cat > /tmp/user-data-beta.yaml << 'EOF'
#cloud-config
hostname: tower-beta
fqdn: tower-beta.local
manage_etc_hosts: true

users:
  - name: biomeos
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    lock_passwd: false
    passwd: $6$rounds=4096$saltsalt$3yGZ8.RvJ4VBqnm7xRVYVq8VZ7XVVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXV

packages:
  - avahi-daemon
  - avahi-utils
  - curl
  - tar
  - gzip
  - openssh-server

runcmd:
  - systemctl enable ssh
  - systemctl start ssh
  - systemctl enable avahi-daemon
  - systemctl start avahi-daemon
  - echo "tower-beta ready" > /tmp/ready
EOF

# Meta-data (minimal)
echo "instance-id: iid-alpha" > /tmp/meta-data-alpha
echo "instance-id: iid-beta" > /tmp/meta-data-beta

# Create cloud-init ISOs
genisoimage -output "$VM_DIR/alpha-cidata.iso" -V cidata -r -J /tmp/user-data-alpha.yaml /tmp/meta-data-alpha 2>/dev/null
genisoimage -output "$VM_DIR/beta-cidata.iso" -V cidata -r -J /tmp/user-data-beta.yaml /tmp/meta-data-beta 2>/dev/null

echo "✅ Cloud-init configs created"
echo ""

# Create VM disk images from cloud image
echo "💾 Creating VM disks..."
cp "$CLOUD_IMG" "$VM_DIR/tower-alpha.qcow2"
cp "$CLOUD_IMG" "$VM_DIR/tower-beta.qcow2"
qemu-img resize "$VM_DIR/tower-alpha.qcow2" 30G
qemu-img resize "$VM_DIR/tower-beta.qcow2" 30G

echo "✅ VM disks created"
echo ""

# Create VMs
echo "🖥️  Creating tower-alpha VM..."
virt-install \
  --name tower-alpha \
  --ram 4096 \
  --vcpus 2 \
  --disk path="$VM_DIR/tower-alpha.qcow2",format=qcow2,bus=virtio \
  --disk path="$VM_DIR/alpha-cidata.iso",device=cdrom \
  --network network=default,model=virtio \
  --os-variant ubuntu22.04 \
  --graphics none \
  --console pty,target_type=serial \
  --import \
  --noautoconsole

echo "✅ tower-alpha created"
echo ""

echo "🖥️  Creating tower-beta VM..."
virt-install \
  --name tower-beta \
  --ram 4096 \
  --vcpus 2 \
  --disk path="$VM_DIR/tower-beta.qcow2",format=qcow2,bus=virtio \
  --disk path="$VM_DIR/beta-cidata.iso",device=cdrom \
  --network network=default,model=virtio \
  --os-variant ubuntu22.04 \
  --graphics none \
  --console pty,target_type=serial \
  --import \
  --noautoconsole

echo "✅ tower-beta created"
echo ""

# Wait for VMs to boot and get IPs
echo "⏳ Waiting for VMs to boot (90 seconds)..."
sleep 90

echo ""
echo "📡 Getting VM IP addresses..."
ALPHA_IP=$(virsh domifaddr tower-alpha | grep -oP '(\d+\.){3}\d+' | head -1)
BETA_IP=$(virsh domifaddr tower-beta | grep -oP '(\d+\.){3}\d+' | head -1)

# Retry if needed
if [ -z "$ALPHA_IP" ] || [ -z "$BETA_IP" ]; then
    echo "⏳ Waiting additional 30 seconds..."
    sleep 30
    ALPHA_IP=$(virsh domifaddr tower-alpha | grep -oP '(\d+\.){3}\d+' | head -1)
    BETA_IP=$(virsh domifaddr tower-beta | grep -oP '(\d+\.){3}\d+' | head -1)
fi

if [ -z "$ALPHA_IP" ] || [ -z "$BETA_IP" ]; then
    echo "❌ Could not get VM IPs"
    echo ""
    echo "Try manually:"
    echo "  virsh domifaddr tower-alpha"
    echo "  virsh domifaddr tower-beta"
    exit 1
fi

echo "✅ tower-alpha: $ALPHA_IP"
echo "✅ tower-beta: $BETA_IP"
echo ""

# Save IPs for later use
echo "ALPHA_IP=$ALPHA_IP" > "$SCRIPT_DIR/vm-ips.env"
echo "BETA_IP=$BETA_IP" >> "$SCRIPT_DIR/vm-ips.env"

echo "═══════════════════════════════════════════════════════════"
echo "✅ VMs READY!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "VM Access:"
echo "  tower-alpha: ssh biomeos@$ALPHA_IP"
echo "  tower-beta:  ssh biomeos@$BETA_IP"
echo "  Password: biomeos"
echo ""
echo "Next Steps:"
echo "  1. Test SSH access"
echo "  2. Run deployment script: ./deploy-to-vms.sh"
echo ""
echo "VM Management:"
echo "  View: virsh list"
echo "  Console: virsh console tower-alpha"
echo "  Stop: virsh shutdown tower-alpha tower-beta"
echo "  Delete: virsh destroy tower-alpha tower-beta && \\"
echo "          virsh undefine tower-alpha tower-beta --remove-all-storage"
echo ""

