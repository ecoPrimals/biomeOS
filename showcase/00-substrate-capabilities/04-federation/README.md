# 04 - Federation: Multi-Tower Coordination

**Duration**: 3 minutes  
**Prerequisites**: Songbird running (for federation)

---

## Overview

This demo shows BiomeOS **federating multiple towers** to create a distributed sovereign network.

**What it demonstrates**:
- Multi-tower discovery via mDNS
- Automatic peer coordination
- Capability sharing across towers
- Zero manual configuration
- Trust escalation

---

## The Power of Federation

### Traditional Approach (Manual Mesh)
```bash
# ❌ Manual peer configuration
# On Tower 1:
PEERS="tower2:9000,tower3:9000,tower4:9000" ./start

# On Tower 2:
PEERS="tower1:9000,tower3:9000,tower4:9000" ./start

# Breaks when:
# - IPs change
# - Ports change
# - New towers added
# - Towers go offline
```

### BiomeOS Approach (Auto-Federation)
```bash
# ✅ On every tower:
biomeOS start

# That's it! Towers discover each other automatically via mDNS
# Federation forms without any configuration
```

---

## Run the Demo

```bash
cd showcase/00-substrate/04-federation
./demo.sh
```

---

## What You'll See

### Phase 1: Tower Discovery
```
🔍 Discovering towers in federation...

Local Tower:
  Name: biomeOS-pop-os
  ID: a1b2c3d4
  Capabilities: [storage, encryption]

Discovered Peers:
  ✅ pop-os (192.168.1.185:8080)
     Capabilities: [orchestration, federation]
     Trust Level: Anonymous
     Last Seen: 2s ago
```

### Phase 2: Capability Federation
```
🌐 Federated capabilities across 2 towers:

Storage Providers (1):
  • biomeOS-pop-os: NestGate (local)

Orchestration Providers (1):
  • pop-os: Songbird (peer)

Encryption Providers (1):
  • biomeOS-pop-os: BearDog (local)

Total Capabilities: 3 across 2 towers
```

### Phase 3: Cross-Tower Operations
```
🔄 Testing cross-tower operations...

Operation 1: Store data locally, orchestrate remotely
  ✓ Data stored on local NestGate
  ✓ Operation coordinated by peer Songbird
  ✓ Federation metadata recorded

Operation 2: Distributed capability query
  ✓ Query sent to all towers
  ✓ Responses aggregated
  ✓ Best provider selected

✅ Federation is fully operational!
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│         BiomeOS Federation              │
│                                         │
│  mDNS/UDP Discovery (port 2300)         │
│  ├─ Broadcast presence                  │
│  ├─ Listen for peers                    │
│  └─ Maintain peer list                  │
└─────────────────────────────────────────┘
         │           │          │
         ▼           ▼          ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │Tower 1 │  │Tower 2 │  │Tower 3 │
    │Storage │  │Compute │  │Security│
    │NestGate│  │Toadstool│ │BearDog │
    └────────┘  └────────┘  └────────┘
         │           │          │
         └───────────┴──────────┘
              Federation
         (Auto-discovered mesh)
```

---

## Federation Concepts

### 1. Automatic Discovery
Towers find each other via:
- **mDNS**: Service advertising
- **UDP Broadcast**: Presence announcements
- **Peer Exchange**: Known peers share other peers

### 2. Trust Escalation
```
Level 0: Anonymous
  ↓ (Reachability verified)
Level 1: Capability
  ↓ (Capabilities validated)
Level 2: Identity
  ↓ (Lineage verified via BearDog)
Level 3: Hardware
  (Physical genesis ceremony)
```

### 3. Capability Aggregation
```yaml
# Local tower capabilities:
local:
  storage: nestgate
  encryption: beardog

# Peer tower capabilities:
peer_1:
  orchestration: songbird
  federation: songbird

peer_2:
  compute: toadstool
  storage: nestgate

# Federated view (all capabilities):
federation:
  storage: [local.nestgate, peer_2.nestgate]
  encryption: [local.beardog]
  orchestration: [peer_1.songbird]
  federation: [peer_1.songbird]
  compute: [peer_2.toadstool]
```

### 4. Load Balancing
When multiple providers exist:
```rust
// Request storage capability
let storage = federation.request("storage", LoadBalancing::RoundRobin);
// Returns: local.nestgate or peer_2.nestgate (alternating)

let storage = federation.request("storage", LoadBalancing::Closest);
// Returns: local.nestgate (lowest latency)

let storage = federation.request("storage", LoadBalancing::LeastLoaded);
// Returns: Provider with most available resources
```

---

## Use Cases

### Use Case 1: Research Lab Federation
```
University Network:
- Building A: Storage tower (500TB NestGate)
- Building B: Compute tower (GPU cluster Toadstool)
- Building C: Security tower (HSM-backed BearDog)

Result: Any researcher can use any capability from any building
```

### Use Case 2: Personal Sovereign Network
```
Personal Devices:
- Desktop: Primary tower (all capabilities)
- Laptop: Mobile tower (storage + encryption)
- NUC: Remote tower (backup storage)

Result: Seamless capability access across all devices
```

### Use Case 3: Community Mesh
```
Community Network:
- Alice: Provides storage (NestGate)
- Bob: Provides compute (Toadstool)
- Carol: Provides orchestration (Songbird)

Result: Community members share capabilities via federation
```

---

## Federation Policies

### Privacy by Default
```yaml
# Default: Share capabilities, not data
federation:
  share_capabilities: true  # What you can do
  share_data: false         # What you have
  
  # Data sharing requires explicit lineage permission
  data_sharing:
    require_lineage: true
    require_consent: true
```

### Selective Federation
```yaml
# Only federate specific capabilities
federation:
  advertise:
    - storage         # Share storage
    - compute         # Share compute
  
  do_not_advertise:
    - encryption      # Keep local only
```

### Geographic Boundaries
```yaml
# Limit federation to specific networks
federation:
  scope: local_network  # Only LAN
  # scope: vpn_network  # Only VPN
  # scope: global       # Entire internet
```

---

## Technical Details

### Discovery Protocol
```
1. Tower starts
2. Broadcast UDP announcement (port 2300)
   Message: {
     tower_id: "abc123",
     capabilities: ["storage", "encryption"],
     endpoint: "https://192.168.1.100:8080"
   }
3. Listen for peer announcements
4. Connect to discovered peers (HTTPS)
5. Verify capabilities
6. Add to federation
7. Maintain heartbeat (every 30s)
```

### Failure Handling
```
Peer becomes unreachable:
  1. Mark as "suspected"
  2. Retry connection (3 attempts)
  3. If still unreachable, mark as "down"
  4. Remove from active pool
  5. Continue checking (every 5 min)
  6. Re-add when returns
```

### Split Brain Prevention
```
Two networks merge:
  1. Detect duplicate tower IDs
  2. Compare lineage proofs
  3. Keep tower with older lineage
  4. Reject impostor
```

---

## Security Considerations

### Trust Verification
- **Anonymous**: Anyone can join (capability-only)
- **Lineage**: BearDog lineage proof required
- **Hardware**: Physical ceremony required

### Attack Prevention
- **Sybil Attack**: Limited by lineage proof
- **DDoS**: Rate limiting per peer
- **Data Exfiltration**: Lineage-gated access
- **Man-in-Middle**: TLS + lineage verification

---

## Success Criteria

✅ **Auto-Discovery**: Towers find each other without configuration  
✅ **Capability Sharing**: All capabilities accessible across federation  
✅ **Fault Tolerance**: Federation continues when towers offline  
✅ **Zero Config**: No manual peer management  
✅ **Privacy Preserved**: Data sharing requires lineage  

---

## Real-World Examples

### Example 1: Academic Research Network
```
5 Universities federated:
- Total storage: 2.5 PB (aggregated)
- Total compute: 1,200 GPUs (aggregated)
- Total users: 5,000 researchers

Each researcher can access any capability from any institution
Data stays local unless explicitly shared via lineage
```

### Example 2: Personal Mesh
```
3 Personal devices federated:
- Home server: Always-on storage
- Laptop: Mobile capabilities
- Cloud VPS: Remote backup

Capabilities follow you across devices
Data syncs via encrypted federation
```

---

## Monitoring Federation

### Health Check
```bash
biomeOS federation status

Output:
Federation: Healthy
  Active Peers: 2
  Total Capabilities: 5
  Network Latency: 12ms avg
  Uptime: 2d 3h 15m
```

### Peer List
```bash
biomeOS federation peers

Output:
Active Peers (2):
  1. pop-os (192.168.1.185)
     Trust: Anonymous
     Capabilities: orchestration, federation
     Latency: 8ms
     
  2. tower-west (192.168.1.200)
     Trust: Capability
     Capabilities: compute, storage
     Latency: 15ms
```

---

## Next Steps

After this demo:
- **05-custom-primals**: Add your own primal to federation
- **06-benchscale**: Deploy multi-tower federation with benchScale
- **07-birdsong-p2p**: Secure P2P tunnels across federation

---

**Philosophy**: *"Federation without configuration. Discovery without hardcoding. Sovereignty without isolation."*

