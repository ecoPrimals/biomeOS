#!/bin/bash
# Quick test: Create 1 VM from template

set -e

TEMPLATE="/home/eastgate/Development/ecoPrimals/primalTools/agentReagents/images/templates/rustdesk-ubuntu-22.04-template.qcow2"
VM_NAME="biomeos-test-alpha"

echo "🚀 Quick VM Test from Template"
echo ""

# 1. Create CoW disk
echo "1. Creating CoW disk..."
DISK_PATH="/var/lib/libvirt/images/${VM_NAME}.qcow2"
qemu-img create -f qcow2 -F qcow2 -b "$TEMPLATE" "$DISK_PATH" 20G

# 2. Create minimal cloud-init
echo "2. Creating cloud-init..."
CIDATA_DIR="/tmp/${VM_NAME}-cidata"
mkdir -p "$CIDATA_DIR"

cat > "$CIDATA_DIR/meta-data" << EOF
instance-id: $VM_NAME
local-hostname: $VM_NAME
EOF

cat > "$CIDATA_DIR/user-data" << EOF
#cloud-config
hostname: $VM_NAME
EOF

# Create ISO
CIDATA_ISO="/var/lib/libvirt/images/${VM_NAME}-cidata.iso"
genisoimage -output "$CIDATA_ISO" -volid cidata -joliet -rock "$CIDATA_DIR/user-data" "$CIDATA_DIR/meta-data" > /dev/null 2>&1

# 3. Create VM
echo "3. Creating VM..."
virt-install \
    --name "$VM_NAME" \
    --memory 2048 \
    --vcpus 2 \
    --disk path="$DISK_PATH",format=qcow2 \
    --disk path="$CIDATA_ISO",device=cdrom \
    --os-variant ubuntu22.04 \
    --network network=default \
    --graphics vnc \
    --noautoconsole \
    --import

echo ""
echo "✅ VM created! Waiting for IP..."
sleep 10

VM_IP=$(virsh domifaddr "$VM_NAME" | grep -oP '(\d+\.){3}\d+' | head -1)
echo "VM IP: $VM_IP"
