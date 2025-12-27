# BiomeOS Multi-VM Federation Plan

**Goal:** Deploy multiple BiomeOS VMs and validate P2P coordination before physical hardware

**Date:** December 27, 2025

---

## Architecture

### VM Topology

```
┌─────────────────────────────────────────────────────────┐
│ Host System (Development Machine)                      │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  BiomeOS-VM1 │  │  BiomeOS-VM2 │  │  BiomeOS-VM3 │ │
│  │  (Tower 1)   │  │  (Tower 2)   │  │  (Tower 3)   │ │
│  │              │  │              │  │              │ │
│  │  192.168.100 │  │  192.168.100 │  │  192.168.100 │ │
│  │      .10     │  │      .20     │  │      .30     │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
│         │                 │                 │          │
│         └─────────────────┴─────────────────┘          │
│                    │                                    │
│              ┌─────┴──────┐                            │
│              │  bridge0   │                            │
│              │ 192.168.100│                            │
│              │     .1     │                            │
│              └────────────┘                            │
└─────────────────────────────────────────────────────────┘
```

### Network Configuration
- **Network:** `192.168.100.0/24`
- **Bridge:** `virbr-biomeos` (192.168.100.1)
- **VM1:** 192.168.100.10 (Tower 1 - Primary)
- **VM2:** 192.168.100.20 (Tower 2 - Secondary)
- **VM3:** 192.168.100.30 (Tower 3 - Discovery)

---

## Implementation Steps

### Phase 1: Single VM Validation ✅
- [x] Boot single VM successfully
- [x] Verify shell access
- [x] Confirm init system operational
- [ ] Test primal execution
- [ ] Validate BYOB config loading

### Phase 2: Multi-VM Infrastructure
- [ ] Create network bridge
- [ ] Create 3 VM disk images
- [ ] Configure VM network adapters
- [ ] Create VM launch scripts
- [ ] Setup serial console logging per VM

### Phase 3: P2P Coordination
- [ ] Deploy BirdSong to each VM
- [ ] Configure discovery mode
- [ ] Validate mDNS discovery
- [ ] Test inter-VM communication
- [ ] Verify mesh formation

### Phase 4: Primal Deployment
- [ ] Deploy BearDog to VM1
- [ ] Deploy Songbird to VM2
- [ ] Deploy Sett to VM3
- [ ] Configure cross-VM coordination
- [ ] Test full BYOB scenario

### Phase 5: Federation Validation
- [ ] Create federated tower configuration
- [ ] Test lineage-gated communication
- [ ] Validate security policies
- [ ] Stress test with traffic
- [ ] Performance benchmarking

---

## VM Configuration

### Disk Images
```bash
# VM1 - Primary Tower
qemu-img create -f qcow2 vm-testing/vm1-tower1.qcow2 2G

# VM2 - Secondary Tower
qemu-img create -f qcow2 vm-testing/vm2-tower2.qcow2 2G

# VM3 - Discovery Node
qemu-img create -f qcow2 vm-testing/vm3-discovery.qcow2 2G
```

### QEMU Launch Parameters
```bash
# VM1
qemu-system-x86_64 \
  -name "BiomeOS-Tower1" \
  -cdrom dist/biomeos-latest.iso \
  -drive file=vm-testing/vm1-tower1.qcow2,format=qcow2,if=ide \
  -m 512 \
  -net nic,model=virtio,macaddr=52:54:00:12:34:10 \
  -net bridge,br=virbr-biomeos \
  -serial file:/tmp/vm1-serial.log \
  -display gtk &

# VM2
qemu-system-x86_64 \
  -name "BiomeOS-Tower2" \
  -cdrom dist/biomeos-latest.iso \
  -drive file=vm-testing/vm2-tower2.qcow2,format=qcow2,if=ide \
  -m 512 \
  -net nic,model=virtio,macaddr=52:54:00:12:34:20 \
  -net bridge,br=virbr-biomeos \
  -serial file:/tmp/vm2-serial.log \
  -display none \
  -daemonize

# VM3
qemu-system-x86_64 \
  -name "BiomeOS-Tower3" \
  -cdrom dist/biomeos-latest.iso \
  -drive file=vm-testing/vm3-discovery.qcow2,format=qcow2,if=ide \
  -m 512 \
  -net nic,model=virtio,macaddr=52:54:00:12:34:30 \
  -net bridge,br=virbr-biomeos \
  -serial file:/tmp/vm3-serial.log \
  -display none \
  -daemonize
```

---

## Network Setup Script

```bash
#!/usr/bin/env bash
# scripts/setup-vm-network.sh

set -euo pipefail

echo "🌐 Setting up BiomeOS VM network..."

# Create bridge
sudo ip link add virbr-biomeos type bridge
sudo ip addr add 192.168.100.1/24 dev virbr-biomeos
sudo ip link set virbr-biomeos up

# Enable IP forwarding
sudo sysctl -w net.ipv4.ip_forward=1

# Setup NAT for internet access
sudo iptables -t nat -A POSTROUTING -s 192.168.100.0/24 ! -d 192.168.100.0/24 -j MASQUERADE

echo "✅ Network ready: virbr-biomeos (192.168.100.1/24)"
```

---

## BYOB Configurations

### tower1.biome.yaml
```yaml
lineage:
  name: "Tower1-Primary"
  id: "tower1-001"
  network: "192.168.100.0/24"

primals:
  - type: BearDog
    port: 8080
    mode: primary
    peers:
      - "192.168.100.20:8080"  # Tower2
      - "192.168.100.30:8080"  # Tower3
  
  - type: BirdSong
    discovery: true
    broadcast: true

security:
  lineage_gate: true
  trust_anchors:
    - "tower2-002"
    - "tower3-003"
```

### tower2.biome.yaml
```yaml
lineage:
  name: "Tower2-Secondary"
  id: "tower2-002"
  network: "192.168.100.0/24"

primals:
  - type: Songbird
    port: 8080
    relay_to: "192.168.100.10:8080"  # Tower1
  
  - type: BirdSong
    discovery: true
```

### tower3.biome.yaml
```yaml
lineage:
  name: "Tower3-Discovery"
  id: "tower3-003"
  network: "192.168.100.0/24"

primals:
  - type: BirdSong
    discovery: true
    mode: scan_only
  
  - type: Sett
    port: 8081
    data_dir: "/biomeos/data"
```

---

## Testing Scenarios

### Scenario 1: Discovery
```bash
# On VM1
/biomeos/primals/birdsong --discover

# Expected: Should find VM2 and VM3
```

### Scenario 2: P2P Communication
```bash
# On VM1
echo "test message" | /biomeos/primals/beardog send --to 192.168.100.20

# On VM2
/biomeos/primals/songbird receive

# Expected: Message arrives encrypted and authenticated
```

### Scenario 3: Federation
```bash
# On VM1
/biomeos/primals/beardog tower --start --config /biomeos/configs/tower1.biome.yaml

# On VM2
/biomeos/primals/songbird relay --connect 192.168.100.10:8080

# Expected: Federated mesh forms with lineage validation
```

---

## Monitoring & Debugging

### Serial Console Monitoring
```bash
# Watch all VMs
tail -f /tmp/vm{1,2,3}-serial.log

# Individual VM
tail -f /tmp/vm1-serial.log | grep -E "ERROR|BiomeOS"
```

### Network Verification
```bash
# From host
ping 192.168.100.10
ping 192.168.100.20
ping 192.168.100.30

# Inside VM (via QEMU monitor)
# Press Ctrl-Alt-2 for monitor
info network
```

### Process Verification
```bash
# Check VMs running
ps aux | grep qemu-system-x86_64

# Check bridge
ip link show virbr-biomeos
bridge link show
```

---

## Success Criteria

### Single VM
- [x] Boots to shell
- [ ] Can execute binaries
- [ ] Network interface up
- [ ] Can reach host
- [ ] Can load primal

### Multi-VM
- [ ] All 3 VMs boot successfully
- [ ] All can ping each other
- [ ] All can ping host
- [ ] mDNS discovery works
- [ ] Primals can communicate

### Federation
- [ ] Tower1 accepts connections
- [ ] Tower2 relays to Tower1
- [ ] Tower3 discovers both
- [ ] Lineage validation works
- [ ] Encrypted traffic flows
- [ ] No dropped packets

---

## Next Steps

1. **Immediate:** Verify current VM still boots with refactored code
2. **Short-term:** Create 3 VM disk images and network bridge
3. **Medium-term:** Deploy primals to VMs and test coordination
4. **Long-term:** Full federation validation before NUC deployment

---

**Status:** Planning Complete - Ready for Implementation  
**Blocker:** Must verify refactored code boots before multi-VM setup

