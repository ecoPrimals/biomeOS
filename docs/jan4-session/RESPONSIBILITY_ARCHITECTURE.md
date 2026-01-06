# 🏗️ Responsibility Architecture - biomeOS vs Primals

**Date**: January 4, 2026  
**Purpose**: Define clear boundaries between orchestration and primal sovereignty

---

## 🎯 Core Principle

**Primals are Sovereign** → They are independent, self-contained systems  
**biomeOS is the Orchestrator** → It coordinates but doesn't implement primal logic

---

## 📦 biomeOS (phase2/biomeOS)

**Location**: `/phase2/biomeOS/`

### Role: Operating System for Primals

**Responsibilities**:

1. **Orchestration**
   - Spawn primals (tower CLI)
   - Monitor health
   - Restart failed primals
   - Graceful shutdown

2. **Coordination Framework**
   - Capability-based routing
   - Dependency resolution
   - Wave-based concurrent startup
   - Service registry

3. **Configuration Management**
   - Load tower.toml
   - Pass environment variables to primals
   - Family seed distribution
   - Zero-configuration orchestration

4. **IPC Infrastructure**
   - Unix socket server/client helpers
   - Message passing abstractions
   - Event notification system
   - Health check protocols

5. **Testing & Validation**
   - Integration tests
   - Health monitoring
   - Performance metrics
   - USB spore packaging

### What biomeOS Does NOT Do

❌ Implement discovery protocols (Songbird's job)  
❌ Implement encryption (BearDog's job)  
❌ Implement storage (ToadStool's job)  
❌ Implement compute (Gorilla's job)

### biomeOS Components

```
biomeOS/
├── crates/
│   ├── biomeos-core/        → Orchestration engine
│   │   ├── primal_orchestrator.rs  → Spawn/manage primals
│   │   ├── capabilities.rs         → Capability-based routing
│   │   ├── primal_health.rs        → Health monitoring
│   │   └── bin/tower.rs            → CLI orchestrator
│   ├── biomeos-types/       → Shared types
│   ├── biomeos-api/         → API server (optional)
│   └── biomeos-cli/         → CLI tools
└── primals/                 → Primal binaries (copied from phase1)
```

---

## 🐕 BearDog (phase1/beardog)

**Location**: `/phase1/beardog/`

### Role: Security Primal (Sovereign)

**Responsibilities**:

1. **Encryption**
   - BTSP tunneling
   - End-to-end encryption
   - Key exchange
   - HSM management

2. **Trust Evaluation**
   - Family-based trust
   - Progressive trust levels
   - Identity verification
   - Genetic lineage validation

3. **Security Primitives**
   - Seed derivation (HKDF)
   - Signature generation/verification
   - Secure key storage
   - Audit logging

4. **Integration**
   - **Connects to Songbird via Unix socket**
   - Receives connection events from Songbird
   - Encrypts Songbird-discovered connections
   - Provides security API to other primals

### BearDog Interface

**Modes**:
- `beardog-server` → Daemon mode (what biomeOS spawns)
- `beardog` → CLI mode (commands)

**APIs**:
- HTTP API (optional, for debugging): `:9000`
- Unix socket: `/tmp/beardog-{family}.sock` (primary)
- Songbird client: Connects to `/tmp/songbird-{family}.sock`

### What BearDog Does NOT Do

❌ Discover peers (Songbird's job)  
❌ Orchestrate startup (biomeOS's job)  
❌ Manage other primals (biomeOS's job)

### BearDog Components

```
beardog/
├── crates/
│   ├── beardog-tunnel/      → BTSP encryption
│   ├── beardog-genetics/    → Seed derivation, lineage
│   ├── beardog-trust/       → Trust evaluation
│   └── beardog-api/         → HTTP API (optional)
└── primalBins/              → Compiled binaries
    └── beardog-server       → Production server binary
```

---

## 🐦 Songbird (phase1/songbird)

**Location**: `/phase1/songbird/`

### Role: Discovery Orchestrator (Sovereign)

**Responsibilities**:

1. **Peer Discovery**
   - **UDP multicast broadcasting** (224.0.0.251:5353)
   - **UDP multicast listening**
   - Peer announcement protocol
   - Peer registry management

2. **Connection Management**
   - Track discovered peers
   - Maintain peer state (alive/dead)
   - TTL expiration handling
   - Peer capability indexing

3. **Inter-Primal IPC**
   - **Unix socket server** (`/tmp/songbird-{family}.sock`)
   - JSON-RPC protocol
   - Event notifications
   - Message routing

4. **BirdSong Protocol**
   - Packet serialization/deserialization
   - Signature verification (Ed25519)
   - Family-based filtering
   - Geographic discovery

5. **Integration**
   - **Notifies BearDog** of discovered peers
   - Routes messages between primals
   - Provides connection primitives
   - Federation orchestration

### Songbird Interface

**Modes**:
- `songbird-orchestrator` → Daemon mode (what biomeOS spawns)
- `songbird` → CLI mode (commands)

**APIs**:
- UDP multicast: `224.0.0.251:5353` (primary discovery)
- Unix socket: `/tmp/songbird-{family}.sock` (IPC with primals)
- HTTP API (optional, for debugging): `:3030`

### What Songbird Does NOT Do

❌ Encrypt connections (BearDog's job)  
❌ Evaluate trust (BearDog's job)  
❌ Orchestrate startup (biomeOS's job)  
❌ Store data (ToadStool's job)

### Songbird Components

```
songbird/
├── crates/
│   ├── songbird-discovery/  → UDP multicast implementation
│   ├── songbird-protocol/   → BirdSong packet format
│   ├── songbird-registry/   → Peer registry
│   └── songbird-api/        → HTTP API (optional)
└── primalBins/              → Compiled binaries
    └── songbird-orchestrator → Production server binary
```

---

## 🔄 Collaboration Flow

### Startup Sequence

```
1. biomeOS (Tower):
   ├── Reads tower.toml
   ├── Derives family_id from seed
   └── Spawns primals:

2. Songbird spawns first:
   ├── Binds UDP multicast (224.0.0.251:5353)
   ├── Creates Unix socket (/tmp/songbird-nat0.sock)
   ├── Broadcasts presence via UDP
   └── Waits for primal registrations

3. BearDog spawns second:
   ├── Connects to Songbird Unix socket
   ├── Registers capabilities: Security, Encryption
   ├── Subscribes to: peer_discovered events
   └── Waits for peers

4. Other primals spawn:
   ├── Connect to Songbird
   ├── Register their capabilities
   └── Discover peers via Songbird
```

### Discovery Flow

```
Peer A (Songbird):
  └─→ UDP multicast: "I'm family:nat0"
       └─→ LAN broadcast

Peer B (Songbird):
  ←─→ Receives UDP packet
  └─→ Verifies signature
  └─→ Adds to peer registry
  └─→ Notifies local BearDog via Unix socket

Peer B (BearDog):
  ←─→ Receives peer_discovered event
  └─→ Evaluates: family:nat0 == my family? YES
  └─→ Trust: HIGH (same family)
  └─→ Establishes encrypted connection via Songbird
```

### Message Flow

```
Primal A → Songbird:
  "Send to capability:Storage, message:{...}"

Songbird:
  1. Lookup: capability:Storage → Peer B
  2. Check BearDog: Peer B trusted? → YES
  3. Route via encrypted channel
  
Peer B ← Songbird:
  Receives message via Unix socket callback
```

---

## 🎯 Implementation Guidelines

### Where to Implement UDP Discovery

**✅ Correct**: `phase1/songbird/crates/songbird-discovery/`

**❌ Wrong**: `phase2/biomeOS/crates/biomeos-songbird-udp/`

**Why**: Songbird is a sovereign primal. It should contain its own discovery logic.

**biomeOS Role**: Orchestrate Songbird, not implement its protocols.

### Where to Implement Unix Socket IPC

**Songbird Side** (Server):
- `phase1/songbird/crates/songbird-ipc/src/server.rs`
- Implements Unix socket server
- Handles register/discover/subscribe/send

**BearDog Side** (Client):
- `phase1/beardog/crates/beardog-ipc/src/songbird_client.rs`
- Connects to Songbird Unix socket
- Subscribes to peer events
- Requests connection establishment

**biomeOS Side** (Helpers):
- `phase2/biomeOS/crates/biomeos-core/src/ipc_helpers.rs`
- Generic Unix socket abstractions
- Health check protocols
- NOT primal-specific logic

### Where to Implement Encryption

**✅ Correct**: `phase1/beardog/crates/beardog-tunnel/`

**biomeOS Role**: Pass encrypted connections between primals, don't implement encryption.

---

## 📊 Responsibility Matrix

| Responsibility | biomeOS | BearDog | Songbird | Other Primals |
|---------------|---------|---------|----------|---------------|
| **Orchestration** | ✅ Primary | ❌ No | ❌ No | ❌ No |
| **UDP Discovery** | ❌ No | ❌ No | ✅ Primary | ❌ No |
| **Encryption** | ❌ No | ✅ Primary | ❌ No | ❌ No |
| **Trust Evaluation** | ❌ No | ✅ Primary | ❌ No | ❌ No |
| **Health Monitoring** | ✅ Primary | Helper | Helper | Helper |
| **Unix Socket Server** | Helpers | Optional | ✅ Primary | Optional |
| **Configuration** | ✅ Primary | Reads | Reads | Reads |
| **Spawn Primals** | ✅ Primary | ❌ No | ❌ No | ❌ No |

---

## 🔧 Practical Implications

### For Phase 1 Implementation

**To implement UDP discovery**:
1. ✅ Work in `phase1/songbird/`
2. ✅ Create `crates/songbird-discovery/`
3. ✅ Implement UDP multicast
4. ✅ Implement Unix socket server
5. ❌ Do NOT create `biomeos-songbird-udp` in biomeOS

**biomeOS changes needed**:
1. Update `tower.toml` schema (remove port configs)
2. Update spawn sequence (Songbird first)
3. Add Unix socket health checks
4. Update primal registry to use Songbird

### For BearDog Integration

**To integrate with Songbird**:
1. ✅ Work in `phase1/beardog/`
2. ✅ Create `crates/beardog-ipc/src/songbird_client.rs`
3. ✅ Connect to Songbird Unix socket
4. ✅ Subscribe to peer events
5. ❌ Do NOT implement discovery in BearDog

**biomeOS changes needed**:
1. Ensure BearDog spawns after Songbird
2. Pass Songbird socket path to BearDog
3. Monitor both via health checks

---

## 🎊 Key Takeaways

### Sovereignty Principle

**Each primal is independent**:
- Contains its own logic
- Runs as separate binary
- Communicates via Unix sockets
- Can be developed/tested independently

### biomeOS as Orchestrator

**biomeOS coordinates but doesn't implement**:
- Spawns primals in correct order
- Monitors health
- Facilitates communication
- Provides zero-config environment
- **Does NOT contain primal logic**

### Collaboration via IPC

**Primals collaborate via Unix sockets**:
- Songbird provides discovery
- BearDog provides encryption
- biomeOS facilitates the connection
- No tight coupling

---

## 📋 Next Steps

### Immediate Implementation

1. **Implement in Songbird**:
   ```bash
   cd phase1/songbird
   # Add UDP discovery module
   # Add Unix socket IPC server
   # Update songbird-orchestrator binary
   ```

2. **Update biomeOS**:
   ```bash
   cd phase2/biomeOS
   # Update tower spawn sequence
   # Remove port configuration
   # Add Unix socket helpers
   ```

3. **Integrate BearDog**:
   ```bash
   cd phase1/beardog
   # Add Songbird client module
   # Connect to Unix socket
   # Subscribe to peer events
   ```

---

**Status**: Architecture boundaries clearly defined. Ready to implement in correct locations.

**Key Insight**: Primals are sovereign - implement in `phase1/{primal}/`, orchestrate from `phase2/biomeOS/`.

