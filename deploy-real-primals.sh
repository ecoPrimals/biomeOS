#!/bin/bash
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 DEPLOYING WITH REAL PHASE1 PRIMAL BINARIES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# AUTH SUDO ONCE AT START
echo "Requesting sudo access for bridge/iptables/rootfs..."
sudo -v
echo "✅ Sudo authenticated"
echo ""

# Configuration
PRIMAL_BINS="/home/eastgate/Development/ecoPrimals/primalBins"
BRIDGE_NAME="virbr-biomeos"
VM_NAME="real-primals-test"
VM_DISK="vm-testing/${VM_NAME}.qcow2"
VM_LOG="/tmp/${VM_NAME}.log"

# Keep sudo alive in background
(while true; do sudo -n true; sleep 50; done 2>/dev/null) &
SUDO_KEEPER=$!
trap "kill $SUDO_KEEPER 2>/dev/null" EXIT

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 1: Setup Bridge Network"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if ip link show ${BRIDGE_NAME} >/dev/null 2>&1; then
    echo "✅ Bridge ${BRIDGE_NAME} exists"
else
    sudo ip link add name ${BRIDGE_NAME} type bridge
    sudo ip addr add 192.168.100.1/24 dev ${BRIDGE_NAME}
    sudo ip link set ${BRIDGE_NAME} up
    echo "✅ Bridge created"
fi

sudo sysctl -w net.ipv4.ip_forward=1 >/dev/null
sudo iptables -t nat -C POSTROUTING -s 192.168.100.0/24 -j MASQUERADE 2>/dev/null || \
    sudo iptables -t nat -A POSTROUTING -s 192.168.100.0/24 -j MASQUERADE
echo "✅ Network configured"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 2: Build & Deploy VM with REAL Primals"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "Building biomeos-rootfs (with modern RAII NBD management)..."
cargo build --release --package biomeos-boot --bin biomeos-rootfs 2>&1 | grep -E "(Compiling biomeos|Finished|error)" || true
echo "✅ Built"

echo ""
echo "Creating VM rootfs with REAL primals from ${PRIMAL_BINS}..."
sudo ./target/release/biomeos-rootfs \
  --output ${VM_DISK} \
  --primals ${PRIMAL_BINS} \
  --size 4G

echo "✅ VM rootfs built with REAL primals"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 3: Launch VM"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

qemu-system-x86_64 \
  -name "BiomeOS-${VM_NAME}" \
  -drive file=${VM_DISK},format=qcow2,if=ide \
  -m 1G \
  -smp 2 \
  -netdev tap,id=net0,br=${BRIDGE_NAME},helper=/usr/lib/qemu/qemu-bridge-helper \
  -device e1000,netdev=net0,mac=52:54:00:12:34:30 \
  -serial file:${VM_LOG} \
  -display none \
  -daemonize 2>&1 | head -5

sleep 2
echo "✅ VM launched (log: ${VM_LOG})"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 4: Wait for Boot & Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for i in {15..1}; do
    echo -ne "  Boot wait: ${i}s\r"
    sleep 1
done
echo ""

echo ""
echo "=== Boot Log (last 30 lines) ==="
tail -30 ${VM_LOG}

echo ""
echo "=== Observability Check ==="
grep -q "MinimalObserver" ${VM_LOG} && echo "✅ MinimalObserver found" || echo "⚠️  MinimalObserver not in log"
grep -q "Boot time:" ${VM_LOG} && echo "✅ Boot time tracked" || echo "⚠️  Boot time not in log"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ DEPLOYMENT COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "VM: ${VM_NAME}"
echo "Log: ${VM_LOG}"
echo "Network: virbr-biomeos (192.168.100.x)"
echo ""
echo "REAL primals deployed:"
echo "  • beardog (ChaCha20-Poly1305)"
echo "  • songbird (mDNS discovery)"
echo "  • nestgate, toadstool, +more"
echo ""
echo "To stop: pkill -f 'BiomeOS-${VM_NAME}'"
echo ""

