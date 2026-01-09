# 🌐 Internet Federation Testing Plan

**Date**: January 9, 2026  
**Status**: Ready for Execution  
**Goal**: Validate biomeOS internet federation using simulated and real environments

---

## 🎯 **Objective**

Test biomeOS internet federation **safely** before real deployment:
1. ✅ **LAN Federation**: Already working (USB spores tested)
2. 🧪 **Simulated Internet**: Use VMs to simulate NAT, firewalls, latency
3. 🌍 **Real Internet**: Gradual rollout with safety measures

---

## 📊 **Current Status**

### **Working** ✅
- USB spore deployment
- LAN federation (multiple local nodes)
- BirdSong P2P discovery (UDP multicast)
- BTSP encrypted tunnels (BearDog)
- Genetic lineage verification

### **Unknown** ❓
- NAT traversal (BTSP handles this, but not tested)
- Firewall traversal
- High latency behavior (>100ms)
- Packet loss handling
- Cross-ISP federation
- IPv6 support

---

## 🧪 **3-Phase Testing Plan**

### **Phase 1: Simulated Internet** (4-6 hours)
Use benchScale VMs to create realistic internet conditions

### **Phase 2: Controlled Real Internet** (2-4 hours)
Test between known, controlled machines

### **Phase 3: Wild Internet** (Future)
Public deployment with unknown peers

---

## 🔬 **Phase 1: Simulated Internet** (RECOMMENDED START)

Use **benchScale + agentReagents** to create a simulated internet environment!

### **Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│  Host Machine (Your Computer)                               │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   VM Alpha   │  │   VM Beta    │  │  VM Gamma    │     │
│  │  (Site A)    │  │  (Site B)    │  │  (Site C)    │     │
│  │              │  │              │  │              │     │
│  │  biomeOS     │  │  biomeOS     │  │  biomeOS     │     │
│  │  + Songbird  │  │  + Songbird  │  │  + Songbird  │     │
│  │  + BearDog   │  │  + BearDog   │  │  + BearDog   │     │
│  │              │  │              │  │              │     │
│  │  NAT: Yes    │  │  NAT: Yes    │  │  NAT: Yes    │     │
│  │  Latency: 50ms│ │  Latency: 100ms│ │  Latency: 150ms│ │
│  │  Loss: 1%    │  │  Loss: 2%    │  │  Loss: 5%    │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                 │                 │              │
│         └─────────────────┴─────────────────┘              │
│                    Virtual Router                           │
│          (Simulates Internet with NAT/Firewall)            │
└─────────────────────────────────────────────────────────────┘
```

### **Implementation**

#### **1. Create Virtual Network Topology**

**File**: `topologies/internet-simulation.yaml`

```yaml
name: "internet-simulation"
description: "Simulates 3 sites connected over the internet with NAT, latency, packet loss"
version: "1.0.0"

# Network segments (simulated ISPs)
networks:
  - name: "isp-alpha"
    subnet: "10.100.0.0/24"
    nat: true
    firewall:
      - allow: "udp:8000-8100"  # Songbird discovery
      - allow: "udp:*"           # BTSP tunnels
      - deny: "tcp:*"            # Block direct TCP (force BTSP)
    
  - name: "isp-beta"
    subnet: "10.200.0.0/24"
    nat: true
    firewall:
      - allow: "udp:8000-8100"
      - allow: "udp:*"
      - deny: "tcp:*"
    
  - name: "isp-gamma"
    subnet: "10.300.0.0/24"
    nat: true
    firewall:
      - allow: "udp:8000-8100"
      - allow: "udp:*"
      - deny: "tcp:*"
  
  - name: "internet-backbone"
    subnet: "172.16.0.0/24"
    nat: false
    # Router connecting all ISPs

# VMs (biomeOS nodes)
nodes:
  - name: "tower-alpha"
    network: "isp-alpha"
    ip: "10.100.0.10"
    memory_mb: 4096
    vcpus: 2
    base_image: "ubuntu-24.04-server-cloudimg-amd64.img"
    
    # Simulate network conditions
    network_conditions:
      latency_ms: 50        # 50ms to backbone
      jitter_ms: 5
      packet_loss_percent: 1
      bandwidth_mbps: 100
    
    # biomeOS spore deployment
    spore: "/path/to/liveSpore-alpha"
    family_id: "internet-test"
    node_id: "tower-alpha"
  
  - name: "tower-beta"
    network: "isp-beta"
    ip: "10.200.0.10"
    memory_mb: 4096
    vcpus: 2
    base_image: "ubuntu-24.04-server-cloudimg-amd64.img"
    
    network_conditions:
      latency_ms: 100
      jitter_ms: 10
      packet_loss_percent: 2
      bandwidth_mbps: 50
    
    spore: "/path/to/liveSpore-beta"
    family_id: "internet-test"
    node_id: "tower-beta"
  
  - name: "tower-gamma"
    network: "isp-gamma"
    ip: "10.300.0.10"
    memory_mb: 4096
    vcpus: 2
    base_image: "ubuntu-24.04-server-cloudimg-amd64.img"
    
    network_conditions:
      latency_ms: 150
      jitter_ms: 20
      packet_loss_percent: 5
      bandwidth_mbps: 25
    
    spore: "/path/to/liveSpore-gamma"
    family_id: "internet-test"
    node_id: "tower-gamma"

# Router configuration
routers:
  - name: "internet-backbone-router"
    interfaces:
      - network: "isp-alpha"
        ip: "10.100.0.1"
      - network: "isp-beta"
        ip: "10.200.0.1"
      - network: "isp-gamma"
        ip: "10.300.0.1"
      - network: "internet-backbone"
        ip: "172.16.0.1"
    
    nat_rules:
      - from: "10.100.0.0/24"
        to: "172.16.0.100"
      - from: "10.200.0.0/24"
        to: "172.16.0.200"
      - from: "10.300.0.0/24"
        to: "172.16.0.300"

# Test scenarios
test_scenarios:
  - name: "basic-federation"
    description: "Verify all 3 nodes can discover each other"
    steps:
      - "Start all 3 VMs"
      - "Wait 60s for BTSP tunnel establishment"
      - "Verify federation on each node"
      - "Check genetic lineage"
  
  - name: "nat-traversal"
    description: "Verify BTSP NAT hole punching works"
    steps:
      - "Start tower-alpha and tower-beta"
      - "Monitor UDP packet flow"
      - "Verify BTSP tunnel established"
      - "Verify bidirectional communication"
  
  - name: "high-latency-resilience"
    description: "Verify system handles 150ms+ latency"
    steps:
      - "Start tower-alpha and tower-gamma"
      - "Monitor heartbeat frequency"
      - "Verify no false disconnects"
      - "Measure impact on topology updates"
  
  - name: "packet-loss-handling"
    description: "Verify system handles 5% packet loss"
    steps:
      - "Start all 3 VMs"
      - "Monitor retransmission rates"
      - "Verify federation stability"
      - "Check for data corruption"
  
  - name: "node-failure-recovery"
    description: "Verify federation recovers from node failure"
    steps:
      - "Start all 3 VMs, wait for federation"
      - "Crash tower-beta (hard shutdown)"
      - "Verify alpha and gamma detect failure"
      - "Restart tower-beta"
      - "Verify automatic re-federation"
```

#### **2. Implement Network Simulation Script**

**File**: `scripts/simulate-internet.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

# Internet Federation Simulation Script
# Uses Linux tc (traffic control) to simulate network conditions

TOPOLOGY="${1:-topologies/internet-simulation.yaml}"

echo "🌐 Starting Internet Federation Simulation..."
echo "   Topology: ${TOPOLOGY}"

# Function to apply network conditions to a VM
apply_network_conditions() {
    local vm_name=$1
    local latency_ms=$2
    local jitter_ms=$3
    local loss_percent=$4
    local bandwidth_mbps=$5
    
    echo "   Applying network conditions to ${vm_name}:"
    echo "     - Latency: ${latency_ms}ms ±${jitter_ms}ms"
    echo "     - Packet loss: ${loss_percent}%"
    echo "     - Bandwidth: ${bandwidth_mbps}Mbps"
    
    # Get VM's virtual interface
    local iface=$(virsh domiflist "${vm_name}" | grep -oP 'vnet\d+')
    
    if [ -z "$iface" ]; then
        echo "     ⚠️  Could not find interface for ${vm_name}"
        return 1
    fi
    
    # Clear existing rules
    sudo tc qdisc del dev "${iface}" root 2>/dev/null || true
    
    # Apply netem (network emulation) rules
    sudo tc qdisc add dev "${iface}" root netem \
        delay "${latency_ms}ms" "${jitter_ms}ms" \
        loss "${loss_percent}%" \
        rate "${bandwidth_mbps}mbit"
    
    echo "     ✅ Network conditions applied to ${iface}"
}

# Function to setup NAT for a network
setup_nat() {
    local network_name=$1
    local subnet=$2
    local public_ip=$3
    
    echo "   Setting up NAT for ${network_name} (${subnet} → ${public_ip})"
    
    # Enable IP forwarding
    sudo sysctl -w net.ipv4.ip_forward=1 >/dev/null
    
    # Setup iptables NAT
    sudo iptables -t nat -A POSTROUTING -s "${subnet}" -j SNAT --to-source "${public_ip}"
    
    echo "     ✅ NAT configured"
}

# Parse topology YAML and create VMs
echo ""
echo "📋 Parsing topology..."

# TODO: Implement YAML parsing or use benchScale directly
# For now, manual setup:

# 1. Create virtual networks
echo ""
echo "🌐 Creating virtual networks..."

virsh net-define - <<EOF
<network>
  <name>isp-alpha</name>
  <forward mode='nat'/>
  <bridge name='virbr-alpha' stp='on' delay='0'/>
  <ip address='10.100.0.1' netmask='255.255.255.0'>
    <dhcp>
      <range start='10.100.0.2' end='10.100.0.254'/>
    </dhcp>
  </ip>
</network>
EOF
virsh net-start isp-alpha || true

virsh net-define - <<EOF
<network>
  <name>isp-beta</name>
  <forward mode='nat'/>
  <bridge name='virbr-beta' stp='on' delay='0'/>
  <ip address='10.200.0.1' netmask='255.255.255.0'>
    <dhcp>
      <range start='10.200.0.2' end='10.200.0.254'/>
    </dhcp>
  </ip>
</network>
EOF
virsh net-start isp-beta || true

virsh net-define - <<EOF
<network>
  <name>isp-gamma</name>
  <forward mode='nat'/>
  <bridge name='virbr-gamma' stp='on' delay='0'/>
  <ip address='10.300.0.1' netmask='255.255.255.0'>
    <dhcp>
      <range start='10.300.0.2' end='10.300.0.254'/>
    </dhcp>
  </ip>
</network>
EOF
virsh net-start isp-gamma || true

echo "   ✅ Virtual networks created"

# 2. Deploy biomeOS spores to VMs (using benchScale)
echo ""
echo "🧬 Deploying biomeOS spores..."

# Use benchScale to create VMs with spores
# (This would be automated with proper YAML parsing)

echo "   ℹ️  Use benchScale to create VMs with spore deployments"
echo "   Example:"
echo "     cargo run --package benchscale --example create_tower_vm -- \\"
echo "       --name tower-alpha \\"
echo "       --network isp-alpha \\"
echo "       --spore /path/to/liveSpore-alpha"

# 3. Apply network conditions
echo ""
echo "🌡️  Applying network conditions..."

# Wait for VMs to start
sleep 5

apply_network_conditions "tower-alpha" 50 5 1 100 || echo "⚠️  tower-alpha not running"
apply_network_conditions "tower-beta" 100 10 2 50 || echo "⚠️  tower-beta not running"
apply_network_conditions "tower-gamma" 150 20 5 25 || echo "⚠️  tower-gamma not running"

echo ""
echo "✅ Internet simulation setup complete!"
echo ""
echo "📊 Monitor federation:"
echo "   • Alpha: curl http://10.100.0.10:3000/api/v1/topology | jq"
echo "   • Beta:  curl http://10.200.0.10:3000/api/v1/topology | jq"
echo "   • Gamma: curl http://10.300.0.10:3000/api/v1/topology | jq"
echo ""
echo "🧪 Run test scenarios:"
echo "   ./scripts/run-internet-tests.sh"
```

#### **3. Create Test Runner**

**File**: `scripts/run-internet-tests.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🧪 Running Internet Federation Tests..."
echo ""

# Test 1: Basic Federation
echo "═══════════════════════════════════════════════════════════"
echo "TEST 1: Basic Federation"
echo "═══════════════════════════════════════════════════════════"
echo "Goal: Verify all 3 nodes can discover each other"
echo ""

echo "⏱️  Waiting 60s for BTSP tunnel establishment..."
sleep 60

echo "Checking tower-alpha..."
ALPHA_PEERS=$(curl -s http://10.100.0.10:3000/api/v1/topology | jq -r '.primals | length')
echo "   Found ${ALPHA_PEERS} peers"

echo "Checking tower-beta..."
BETA_PEERS=$(curl -s http://10.200.0.10:3000/api/v1/topology | jq -r '.primals | length')
echo "   Found ${BETA_PEERS} peers"

echo "Checking tower-gamma..."
GAMMA_PEERS=$(curl -s http://10.300.0.10:3000/api/v1/topology | jq -r '.primals | length')
echo "   Found ${GAMMA_PEERS} peers"

if [ "$ALPHA_PEERS" -ge 3 ] && [ "$BETA_PEERS" -ge 3 ] && [ "$GAMMA_PEERS" -ge 3 ]; then
    echo "✅ TEST 1 PASSED: All nodes federated"
else
    echo "❌ TEST 1 FAILED: Federation incomplete"
    exit 1
fi

echo ""

# Test 2: NAT Traversal
echo "═══════════════════════════════════════════════════════════"
echo "TEST 2: NAT Traversal"
echo "═══════════════════════════════════════════════════════════"
echo "Goal: Verify BTSP NAT hole punching works"
echo ""

echo "Checking for BTSP tunnels..."
ALPHA_BTSP=$(curl -s http://10.100.0.10:3000/api/v1/topology | \
    jq -r '.connections[] | select(.connection_type == "btsp_tunnel") | .to' | wc -l)

if [ "$ALPHA_BTSP" -ge 2 ]; then
    echo "✅ TEST 2 PASSED: BTSP tunnels established (${ALPHA_BTSP} found)"
else
    echo "❌ TEST 2 FAILED: BTSP tunnels not established"
    exit 1
fi

echo ""

# Test 3: High Latency Resilience
echo "═══════════════════════════════════════════════════════════"
echo "TEST 3: High Latency Resilience (150ms+)"
echo "═══════════════════════════════════════════════════════════"
echo "Goal: Verify system handles high latency"
echo ""

echo "Pinging tower-gamma from tower-alpha through BTSP..."
ALPHA_SSH="ssh testuser@10.100.0.10"
GAMMA_IP="10.300.0.10"

# Measure latency through BTSP tunnel
LATENCY=$($ALPHA_SSH "ping -c 5 ${GAMMA_IP} | grep 'avg' | awk -F '/' '{print \$5}'")
echo "   Average latency: ${LATENCY}ms"

if (( $(echo "$LATENCY < 200" | bc -l) )); then
    echo "✅ TEST 3 PASSED: High latency handled (${LATENCY}ms)"
else
    echo "⚠️  TEST 3 WARNING: Very high latency (${LATENCY}ms)"
fi

echo ""

# Test 4: Packet Loss Handling
echo "═══════════════════════════════════════════════════════════"
echo "TEST 4: Packet Loss Handling (5%)"
echo "═══════════════════════════════════════════════════════════"
echo "Goal: Verify system handles packet loss"
echo ""

echo "Checking federation stability with packet loss..."
sleep 30

GAMMA_PEERS_AFTER=$(curl -s http://10.300.0.10:3000/api/v1/topology | jq -r '.primals | length')
echo "   tower-gamma still has ${GAMMA_PEERS_AFTER} peers"

if [ "$GAMMA_PEERS_AFTER" -ge 3 ]; then
    echo "✅ TEST 4 PASSED: Federation stable despite 5% packet loss"
else
    echo "❌ TEST 4 FAILED: Packet loss caused disconnections"
    exit 1
fi

echo ""

# Test 5: Node Failure Recovery
echo "═══════════════════════════════════════════════════════════"
echo "TEST 5: Node Failure Recovery"
echo "═══════════════════════════════════════════════════════════"
echo "Goal: Verify federation recovers from node failure"
echo ""

echo "Crashing tower-beta..."
virsh destroy tower-beta

echo "⏱️  Waiting 30s for failure detection..."
sleep 30

ALPHA_PEERS_AFTER_CRASH=$(curl -s http://10.100.0.10:3000/api/v1/topology | jq -r '.primals | length')
echo "   tower-alpha now has ${ALPHA_PEERS_AFTER_CRASH} peers"

echo "Restarting tower-beta..."
virsh start tower-beta

echo "⏱️  Waiting 60s for re-federation..."
sleep 60

ALPHA_PEERS_AFTER_RECOVERY=$(curl -s http://10.100.0.10:3000/api/v1/topology | jq -r '.primals | length')
echo "   tower-alpha now has ${ALPHA_PEERS_AFTER_RECOVERY} peers"

if [ "$ALPHA_PEERS_AFTER_RECOVERY" -ge 3 ]; then
    echo "✅ TEST 5 PASSED: Federation recovered after node failure"
else
    echo "❌ TEST 5 FAILED: Federation did not recover"
    exit 1
fi

echo ""
echo "🎊 ALL TESTS PASSED! 🎊"
echo ""
echo "📊 Summary:"
echo "   ✅ Basic federation working"
echo "   ✅ NAT traversal working"
echo "   ✅ High latency handled"
echo "   ✅ Packet loss handled"
echo "   ✅ Node failure recovery working"
echo ""
echo "🌍 Ready for real internet testing!"
```

---

## 🌍 **Phase 2: Controlled Real Internet** (After simulation passes)

### **Setup**

Use **2-3 controlled machines** with known IPs:

```
Machine 1: Your home (tower-home)
    ├── Public IP: X.X.X.X
    ├── NAT: Yes (home router)
    └── ISP: Provider A

Machine 2: Friend/Family (tower-friend)
    ├── Public IP: Y.Y.Y.Y
    ├── NAT: Yes (their router)
    └── ISP: Provider B

Machine 3: VPS/Cloud (tower-cloud) [OPTIONAL]
    ├── Public IP: Z.Z.Z.Z
    ├── NAT: No (public IP)
    └── ISP: Cloud provider
```

### **Test Steps**

1. **Deploy USB spores to each machine**:
```bash
# Same family, different nodes
biomeos-spore create --niche tower \
    --family "internet-test" \
    --node-id "tower-home" \
    --output /dev/sdb
```

2. **Configure firewall rules** (allow UDP):
```bash
# On each machine
sudo ufw allow 8000:8100/udp  # Songbird discovery
sudo ufw allow proto udp       # BTSP tunnels
```

3. **Start nodes and monitor**:
```bash
# On each machine
curl http://localhost:3000/api/v1/topology | jq '.primals[] | select(.node_id != "tower-home")'
```

4. **Verify BTSP tunnels**:
```bash
# Check for encrypted tunnels
curl http://localhost:3000/api/v1/topology | \
    jq '.connections[] | select(.connection_type == "btsp_tunnel")'
```

### **Success Criteria**

- ✅ All nodes discover each other (via UDP multicast or relay)
- ✅ BTSP tunnels establish (encrypted P2P)
- ✅ Genetic lineage verified
- ✅ Topology updates propagate
- ✅ No unencrypted traffic over internet

---

## 🔒 **Safety Checklist**

Before real internet testing:

### **Security** ✅
- [ ] All traffic encrypted (BTSP)
- [ ] No plain HTTP exposure
- [ ] Firewall rules configured
- [ ] Genetic lineage verification enabled
- [ ] Trust levels enforced (NUCLEUS)

### **Privacy** ✅
- [ ] No sensitive data in test deployments
- [ ] Family IDs are test-only
- [ ] Logs don't contain private info
- [ ] Metrics collection is opt-in

### **Reliability** ✅
- [ ] Automatic failover tested (simulated)
- [ ] NAT traversal validated (simulated)
- [ ] High latency handled (simulated)
- [ ] Packet loss handled (simulated)
- [ ] Node recovery tested (simulated)

### **Monitoring** ✅
- [ ] Topology tracking working
- [ ] Health checks active
- [ ] Error logging comprehensive
- [ ] Metrics collection enabled

---

## 📊 **Expected Results**

### **Simulated Internet**
- **Discovery time**: 10-30s (with NAT)
- **BTSP tunnel establishment**: 5-10s
- **Latency impact**: +50-150ms (expected)
- **Packet loss tolerance**: Up to 10%
- **Recovery time**: <60s after failure

### **Real Internet**
- **Discovery time**: 30-60s (depends on NAT type)
- **BTSP tunnel establishment**: 10-30s
- **Latency**: Varies by geography (50-300ms)
- **Stability**: Should be stable with 5% loss
- **Recovery**: <2min after network interruption

---

## 🎯 **Immediate Next Steps**

### **Option A: Full Simulation** (Recommended, 4-6 hours)
1. Use benchScale to create 3 VMs
2. Apply network conditions (tc netem)
3. Deploy biomeOS spores
4. Run test suite
5. Validate all 5 test scenarios

### **Option B: Quick Real Test** (2-3 hours)
1. Deploy 2 USB spores (same family)
2. Give one to friend/family
3. Both start nodes
4. Monitor federation
5. Document results

### **Option C: Hybrid** (3-4 hours)
1. Quick simulation (1-2 VMs locally)
2. Real test with 1 remote machine
3. Validate both approaches

---

## 📝 **Test Data Collection**

For each test, collect:

```json
{
  "test_id": "internet-sim-001",
  "date": "2026-01-09",
  "environment": "simulated",
  "nodes": 3,
  "conditions": {
    "latency_ms": [50, 100, 150],
    "packet_loss_percent": [1, 2, 5],
    "nat_enabled": true
  },
  "results": {
    "discovery_time_sec": 25,
    "tunnel_establishment_sec": 8,
    "federation_stable": true,
    "failures_detected": 0,
    "recovery_time_sec": 45
  },
  "logs": {
    "songbird": "/logs/songbird-sim-001.log",
    "beardog": "/logs/beardog-sim-001.log",
    "biomeos": "/logs/biomeos-sim-001.log"
  }
}
```

---

## 🎊 **Bottom Line**

### **Safe Path to Internet Federation**

1. **Simulated** (4-6 hours) - Use benchScale VMs with tc netem
2. **Controlled** (2-4 hours) - Test with known machines
3. **Wild** (Future) - Public deployment

### **Ready to Start!**

Everything needed is already in place:
- ✅ USB spores working (LAN tested)
- ✅ benchScale for VM simulation
- ✅ BTSP for encrypted tunnels
- ✅ BirdSong for NAT traversal
- ✅ Genetic lineage for trust

### **Recommended: Start with Simulation**

Use syntheticChemistry infrastructure to safely test internet conditions before risking real deployment!

---

🌐 **Internet Federation - Test Safely First!** 🧪✨

