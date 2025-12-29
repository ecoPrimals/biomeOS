# Songbird P2P Federation Validation Plan

**Goal**: Validate Songbird P2P federation across 2 VMs, then add NUC as 3rd node

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Host Machine (benchScale + biomeOS)                        │
│  • Creates 2 VMs (VM1, VM2)                                 │
│  • Deploys BiomeOS USB packages                             │
│  • Starts Songbird orchestrate on each                      │
│  • Validates mDNS/UDP discovery                             │
└─────────────────────────────────────────────────────────────┘
                   │                    │
                   │ mDNS/UDP           │ mDNS/UDP
                   ▼                    ▼
          ┌──────────────┐      ┌──────────────┐
          │  VM1         │◄────►│  VM2         │
          │  • Songbird  │      │  • Songbird  │
          │  • mDNS      │      │  • mDNS      │
          └──────────────┘      └──────────────┘
                   ▲                    ▲
                   │                    │
                   │ mDNS/UDP           │ mDNS/UDP
                   │ (LAN)              │ (LAN)
                   │                    │
                   └────────┬───────────┘
                            │
                            ▼
                   ┌──────────────┐
                   │  NUC         │
                   │  • Songbird  │
                   │  • Discovers │
                   │    VMs auto  │
                   └──────────────┘
```

---

## Phases

### Phase 1: VM Creation ✅
**Status**: COMPLETE (VmFederationManager)

**What it does**:
- Creates 2 VMs using benchScale
- Uses agentReagents template (40x faster!)
- Validates SSH access
- Returns VM IPs

### Phase 2: Deploy BiomeOS USB
**Status**: TODO

**What it needs**:
```rust
// Deploy USB package to VMs
async fn deploy_biomeos_usb(&self, federation_name: &str, vm_ips: &[String]) -> Result<()> {
    for ip in vm_ips {
        // 1. SCP USB package to VM
        // 2. Extract to /opt/biomeos
        // 3. Set permissions
        // 4. Install systemd services (optional)
    }
}
```

**Commands**:
```bash
# On each VM:
scp biomeos-*.tar.gz biomeos@VM_IP:/tmp/
ssh biomeos@VM_IP "sudo tar -xzf /tmp/biomeos-*.tar.gz -C /opt/"
ssh biomeos@VM_IP "sudo chown -R biomeos:biomeos /opt/biomeos"
```

### Phase 3: Start Songbird P2P
**Status**: TODO

**What it needs**:
```rust
// Start Songbird on each VM
async fn start_songbird(&self, vm_ips: &[String]) -> Result<()> {
    for ip in vm_ips {
        // 1. SSH to VM
        // 2. Start Songbird orchestrate
        // 3. Verify process running
        // 4. Wait for mDNS announcement
    }
}
```

**Commands**:
```bash
# On each VM:
ssh biomeos@VM_IP "cd /opt/biomeos/primals && ./songbird orchestrate &"
```

**Expected**:
- Songbird starts
- Announces via mDNS on `_songbird._udp.local`
- Listens on UDP port (dynamic)

### Phase 4: Validate mDNS Federation
**Status**: TODO

**What it needs**:
```rust
// Validate Songbird P2P coordination
async fn validate_mdns_federation(&self, vm_ips: &[String]) -> Result<()> {
    for ip in vm_ips {
        // 1. SSH to VM
        // 2. Run avahi-browse -a
        // 3. Verify discovery of other VM's Songbird
        // 4. Check peer count
    }
}
```

**Commands**:
```bash
# On each VM:
ssh biomeos@VM_IP "avahi-browse -a | grep songbird"
ssh biomeos@VM_IP "cd /opt/biomeos/primals && ./songbird status"
```

**Expected Output**:
```
VM1 sees:
  • Self: songbird-vm1._songbird._udp.local
  • Peer: songbird-vm2._songbird._udp.local

VM2 sees:
  • Self: songbird-vm2._songbird._udp.local
  • Peer: songbird-vm1._songbird._udp.local
```

### Phase 5: Add NUC to Federation
**Status**: TODO (Hardware)

**Steps**:
1. Boot NUC from USB (biomeOS package)
2. NUC auto-starts Songbird orchestrate
3. NUC discovers VMs via mDNS
4. 3-node federation automatically forms

**Validation**:
```bash
# On NUC:
avahi-browse -a | grep songbird
# Should see VM1 and VM2

# On VM1:
avahi-browse -a | grep songbird
# Should see VM2 and NUC

# On VM2:
avahi-browse -a | grep songbird
# Should see VM1 and NUC
```

---

## Implementation Strategy

### Step 1: Extend VmFederationManager ✅
Add methods for:
- `deploy_biomeos_usb()`
- `start_songbird()`
- `validate_mdns_federation()`

### Step 2: Update Binary
Implement phases 2-4 in `biomeos-validate-federation.rs`

### Step 3: Test 2-VM Federation
Run full validation pipeline locally

### Step 4: Add NUC
Boot NUC and validate 3-node federation

---

## Key Details

### SSH Access
- User: `biomeos` (from cloud-init)
- Key: `/home/eastgate/.ssh/id_rsa` (injected via cloud-init)
- VMs accessible from host

### USB Package Contents
```
/opt/biomeos/
├── primals/
│   ├── songbird (binary)
│   ├── nestgate (binary)
│   ├── beardog (binary)
│   └── ...
├── showcases/
│   └── ... (all 20 demos)
└── README.md
```

### Songbird Requirements
- **mDNS**: Avahi daemon (should be running in template)
- **UDP**: No firewall restrictions needed (LAN)
- **Discovery**: Automatic via `_songbird._udp.local`

### Network
- VMs on libvirt bridge (192.168.122.0/24)
- NUC on LAN (same physical network)
- mDNS spans both (if properly configured)

---

## Success Criteria

### 2-VM Federation
- ✅ VMs created and SSH-accessible
- ✅ BiomeOS USB deployed to both VMs
- ✅ Songbird running on both VMs
- ✅ VMs discover each other via mDNS
- ✅ UDP communication established

### 3-Node Federation (with NUC)
- ✅ NUC boots from USB
- ✅ NUC starts Songbird automatically
- ✅ NUC discovers VMs via mDNS
- ✅ VMs discover NUC via mDNS
- ✅ 3-way UDP coordination

---

## Testing Approach

### Local First (2 VMs)
1. Create VMs with `VmFederationManager`
2. Deploy USB to VMs
3. Start Songbird on both
4. Validate discovery
5. **If this works**, proceed to NUC

### Hardware Second (NUC + 2 VMs)
1. Keep VMs running
2. Boot NUC from USB
3. Validate 3-way discovery
4. Test federation resilience (disconnect/reconnect)

---

## Why This Works

### No HTTP Needed
- Songbird uses **mDNS/UDP**, not HTTP
- No port conflicts
- No authentication needed (within LAN)
- Automatic discovery

### No Hardcoding
- VMs don't know about each other beforehand
- Discovery happens at runtime
- Works with any number of nodes

### Validated All The Way
- We validate at every step
- No assumptions
- Fail early and clearly

---

## Next Steps

1. Implement `deploy_biomeos_usb()` in VmFederationManager
2. Implement `start_songbird()` in VmFederationManager
3. Implement `validate_mdns_federation()` in VmFederationManager
4. Update binary to use new methods
5. Run full pipeline
6. **If VM federation works**, add NUC

---

**Modern Idiomatic Rust**: All the way! 🦀  
**Validation**: NOT optional ✅  
**mDNS/UDP**: The right protocol for P2P 📡  

*Let's make this federation happen!* 🌐

