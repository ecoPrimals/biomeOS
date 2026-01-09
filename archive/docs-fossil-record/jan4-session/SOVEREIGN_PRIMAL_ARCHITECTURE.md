# 🏛️ Sovereign Primal Architecture - Implementation Plan

**Date**: January 4, 2026  
**Vision**: Zero-configuration, UDP-based, fractal-scaling primal ecosystem  
**Goal**: Implement true BirdSong/Songbird collaboration model

---

## 🎯 Core Principles

### Sovereign Primals

**Each primal is sovereign**:
- Self-contained
- No hardcoded dependencies
- Discovers capabilities dynamically
- Scales fractally

**Collaboration, not coordination**:
- No central authority
- Peer-to-peer discovery
- Family-based trust
- UDP multicast communication

### Zero Port Management

**Songbird eliminates port configuration**:
- UDP multicast for discovery (224.0.0.x)
- Dynamic connection establishment
- No fixed HTTP ports
- Infinite instances possible

**BearDog secures connections**:
- Encrypts Songbird-discovered peers
- Family-based automatic trust
- No pre-configured endpoints
- Fractal security model

---

## 🧬 Architecture Components

### 1. Songbird - Discovery Orchestrator

**Role**: Universal discovery and connection management

**Capabilities**:
```rust
pub struct Songbird {
    // UDP multicast discovery
    multicast_addr: SocketAddr,  // e.g., 224.0.0.251:5353
    
    // Discovered peers
    peers: HashMap<PeerId, PeerInfo>,
    
    // Capability registry
    capabilities: HashMap<Capability, Vec<PeerId>>,
    
    // Connection primitives
    connections: ConnectionManager,
}
```

**Operations**:
- Broadcast presence via UDP multicast
- Listen for peer announcements
- Maintain peer registry
- Provide connection primitives to other primals
- **NO HTTP REQUIRED**

### 2. BearDog - Security Provider

**Role**: Encryption and trust for discovered connections

**Capabilities**:
```rust
pub struct BearDog {
    // Family identity
    family_id: FamilyId,
    encryption_tag: String,
    
    // Trust evaluation
    trust_evaluator: TrustEvaluator,
    
    // Encryption for Songbird connections
    encryptor: ConnectionEncryptor,
    
    // Integration with Songbird
    discovery: SongbirdClient,
}
```

**Operations**:
- Register with Songbird (via IPC/Unix socket)
- Receive connection events from Songbird
- Evaluate trust based on family
- Encrypt connections
- **NO FIXED HTTP PORT**

### 3. Tower - Orchestrator

**Role**: Minimal bootstrap, then hands off to primals

**Capabilities**:
```rust
pub struct Tower {
    // Spawns primals
    primals: Vec<PrimalProcess>,
    
    // Zero configuration
    config: MinimalConfig,  // Only: family_id, binary paths
    
    // Monitors health
    health: HealthMonitor,
}
```

**Operations**:
- Spawn Songbird (UDP only)
- Spawn BearDog (registers with Songbird)
- Spawn other primals as needed
- Monitor health
- **NO PORT CONFIGURATION**

---

## 🌊 Communication Flow

### Startup Sequence

```
T+0s: Tower starts
  ├── Reads family.seed (encrypted)
  └── Derives family_id

T+1s: Songbird spawns
  ├── Binds UDP multicast (224.0.0.251:5353)
  ├── Creates Unix socket (/tmp/songbird.sock)
  └── Broadcasts: "family:nat0, capabilities:Discovery"

T+2s: BearDog spawns
  ├── Connects to Songbird via Unix socket
  ├── Registers: "capabilities:Security,Encryption"
  └── Requests: Connection events

T+3s: Other primals spawn
  ├── Each connects to Songbird
  ├── Registers capabilities
  └── Discovers peers via Songbird

T+4s: Discovery happens
  ├── Peer announces via UDP
  ├── Songbird notifies BearDog
  ├── BearDog evaluates trust
  └── Encrypted connection established
```

### Discovery Protocol

**Phase 1: UDP Announcement**
```
Songbird broadcasts:
{
  "family_id": "nat0",
  "node_id": "beardog:family:nat0:host_uuid",
  "capabilities": ["Discovery", "Security"],
  "timestamp": 1704326400
}
```

**Phase 2: Peer Response**
```
Other Songbird receives and broadcasts back:
{
  "family_id": "nat0",  // SAME FAMILY!
  "node_id": "beardog:family:nat0:host_uuid2",
  "capabilities": ["Discovery", "Security"],
  "timestamp": 1704326401
}
```

**Phase 3: Connection Establishment**
```
Songbird notifies local BearDog:
  "Peer discovered: family:nat0, ip:192.168.1.100"

BearDog evaluates:
  My family: nat0
  Peer family: nat0
  Trust: HIGH → Auto-accept

BearDog establishes encrypted connection via Songbird
```

### Inter-Primal Communication

**Songbird as Message Bus**:
```
Primal A → Songbird (Unix socket)
  "Send to: capability:Storage, message: {data}"

Songbird:
  1. Looks up capability:Storage → Peer B
  2. Checks BearDog trust → HIGH
  3. Routes message via encrypted connection
  4. Delivers to Peer B

Peer B receives via Songbird callback
```

**No direct connections between primals!**
- All communication via Songbird
- BearDog encrypts transparently
- Zero configuration

---

## 🛠️ Implementation Phases

### Phase 1: Songbird UDP Foundation (Priority 1)

**Goal**: Pure UDP multicast discovery

**Tasks**:
1. Implement UDP multicast listener/broadcaster
2. Peer announcement protocol
3. Capability registry
4. Unix socket IPC for local primals
5. Remove HTTP dependency

**Deliverables**:
- Songbird discovers peers via UDP
- No HTTP ports needed
- Unix socket for primal integration

### Phase 2: BearDog Integration (Priority 2)

**Goal**: Security via Songbird connections

**Tasks**:
1. Unix socket client for Songbird
2. Connection event handling
3. Trust evaluation for discovered peers
4. Encryption layer for Songbird connections
5. Remove HTTP API (or make optional/debug only)

**Deliverables**:
- BearDog registers with Songbird
- Encrypts all discovered connections
- Family-based automatic trust
- No fixed HTTP port

### Phase 3: Tower Zero-Config (Priority 3)

**Goal**: Minimal bootstrap orchestration

**Tasks**:
1. Remove port configuration from tower.toml
2. Spawn Songbird first (UDP only)
3. Spawn BearDog (connects to Songbird)
4. Spawn other primals (connect to Songbird)
5. Health monitoring via Songbird

**Deliverables**:
- tower.toml: NO port configuration
- Primals discover each other via Songbird
- Automatic scaling (spawn N instances)

### Phase 4: Fractal Scaling (Priority 4)

**Goal**: Dynamic primal spawning

**Tasks**:
1. Tower detects resource needs
2. Spawns specialized instances
3. All discover via Songbird automatically
4. BearDog secures all connections
5. Zero configuration required

**Deliverables**:
- Spawn N beardog instances → No conflicts
- Spawn N storage instances → Auto-discover
- Spawn N compute instances → Fractal scale
- True sovereign primal ecosystem

---

## 📊 Success Criteria

### Minimum Viable Implementation

**Phase 1 Complete**:
- ✅ Songbird uses UDP multicast only
- ✅ Peers discover each other automatically
- ✅ Unix socket IPC working
- ✅ No HTTP ports required

**Phase 2 Complete**:
- ✅ BearDog connects via Unix socket
- ✅ Encrypts Songbird connections
- ✅ Family-based trust working
- ✅ No fixed HTTP port

### Full Sovereign Architecture

**All Phases Complete**:
- ✅ Zero port configuration
- ✅ Automatic discovery
- ✅ Fractal scaling proven
- ✅ Spawn N instances without conflicts
- ✅ Dynamic capability-based routing
- ✅ True zero-hardcoding achieved

---

## 🔧 Technical Details

### Songbird UDP Multicast

**Address**: `224.0.0.251:5353` (mDNS-like)  
**Protocol**: JSON over UDP  
**Broadcast Interval**: 30 seconds  
**TTL**: 255 (LAN-wide)

**Packet Format**:
```json
{
  "version": "1.0",
  "family_id": "nat0",
  "node_id": "beardog:family:nat0:hostname_uuid",
  "capabilities": ["Security", "Encryption"],
  "endpoints": {
    "unix_socket": "/tmp/beardog.sock",
    "debug_http": "127.0.0.1:9000"  // Optional
  },
  "timestamp": 1704326400,
  "ttl": 90
}
```

### Songbird Unix Socket IPC

**Path**: `/tmp/songbird-{family_id}.sock`  
**Protocol**: JSON-RPC over Unix domain socket  
**Operations**:
- `register(capabilities, endpoints)`
- `discover(capability) → Vec<Peer>`
- `send(peer_id, message)`
- `subscribe(event_type, callback)`

**Example**:
```rust
let socket = UnixStream::connect("/tmp/songbird-nat0.sock")?;
let request = json!({
    "method": "register",
    "params": {
        "capabilities": ["Security", "Encryption"],
        "endpoints": { "unix_socket": "/tmp/beardog.sock" }
    }
});
socket.write_all(&request.to_string().as_bytes())?;
```

### BearDog Encryption Layer

**Connection Flow**:
```
1. Songbird discovers Peer X
2. Songbird notifies BearDog: "peer_discovered(X)"
3. BearDog evaluates trust(X)
4. If trusted: BearDog establishes encrypted channel
5. Songbird uses encrypted channel for all traffic to X
```

**Transparent to other primals**:
- Other primals just use Songbird
- Don't know about encryption
- BearDog handles it automatically

---

## 🎯 Implementation Priority

### Week 1: Songbird UDP Foundation
- UDP multicast implementation
- Unix socket IPC
- Basic peer registry
- Remove HTTP from Songbird

### Week 2: BearDog Integration
- Unix socket client
- Connection event handling
- Encryption layer
- Trust evaluation

### Week 3: Tower Zero-Config
- Remove port configuration
- Update tower.toml schema
- Dynamic primal spawning
- Health monitoring via Songbird

### Week 4: Testing & Validation
- USB spore clone testing (no port conflicts!)
- Multi-tower federation
- Fractal scaling tests
- Performance benchmarks

---

## 📝 Next Immediate Steps

1. **Design Songbird UDP protocol** (detailed spec)
2. **Design Unix socket IPC** (API specification)
3. **Implement Songbird UDP** (pure UDP, no HTTP)
4. **Test local discovery** (2+ instances on same machine)
5. **Implement BearDog integration** (Unix socket client)
6. **Test encryption layer** (transparent to other primals)
7. **Update tower.toml** (remove all port config)
8. **Test USB spore clones** (N instances, zero conflicts)

---

## 🎊 Vision Realized

**When complete**:
- Spawn 10 beardog instances → All discover each other
- Spawn 10 songbird instances → Automatic peer mesh
- Spawn 10 toadstool instances → Fractal storage network
- **ZERO port configuration**
- **ZERO manual discovery**
- **INFINITE scale**

**This is the sovereign primal architecture!**

---

**Status**: Architecture documented. Ready to implement Phase 1: Songbird UDP Foundation.

🚀 **Let's build the future!**

