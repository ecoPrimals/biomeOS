# 🔒 BTSP Tunnel Coordination Demo

**Status**: 📚 Gap Discovery Phase  
**Date**: December 31, 2025  
**Goal**: Document real APIs needed for BTSP tunnel coordination  

---

## What This Demo Does

This demo **discovers and documents** the actual APIs needed to establish a BTSP (BearDog Secure Tunnel Protocol) tunnel between Songbird (orchestration) and BearDog (encryption).

**Approach**: Gap discovery (not full implementation yet)
- ✅ Discovers available primals
- ✅ Documents expected APIs
- ✅ Identifies missing pieces
- ✅ Based on proven phase1 implementation

---

## Quick Start

```bash
./demo.sh
```

**Expected**: Demo will identify what's working and what gaps remain.

---

## Architecture

```
┌──────────────────┐
│ Songbird Tower   │ ← Orchestration (mDNS/UDP)
│ (orchestration)  │   Coordinates peer discovery
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼─────┐
│ Node  │ │  Node  │
│ Alice │ │   Bob  │
└───┬───┘ └───┬────┘
    └─────┬───┘
      BTSP Tunnel
     (via BearDog)
```

### Components

1. **Songbird**: Orchestration service
   - Handles node registration
   - Coordinates peer discovery
   - Uses mDNS/UDP (not HTTP!)

2. **BearDog**: Encryption service
   - Establishes BTSP tunnels
   - Provides Perfect Forward Secrecy
   - Encrypts all messages

3. **biomeOS**: Coordinator
   - Discovers primals by capability
   - Orchestrates tunnel establishment
   - Demonstrates integration

---

## Expected Flow

### 1. Discovery Phase
```bash
# Discover orchestration service (Songbird)
export PRIMAL_SONGBIRD_ENDPOINT="mdns://songbird-tower.local"
export PRIMAL_SONGBIRD_CAPABILITIES="orchestration,p2p"

# Discover encryption service (BearDog)
export PRIMAL_BEARDOG_ENDPOINT="http://localhost:9091"
export PRIMAL_BEARDOG_CAPABILITIES="encryption,btsp"
```

### 2. Registration Phase
```
Alice → Songbird: Register as "alice" with capabilities
Bob → Songbird: Register as "bob" with capabilities
```

### 3. Peer Discovery
```
Alice → Songbird: Find peer "bob"
Songbird → Alice: Bob's endpoint: 127.0.0.1:8082
```

### 4. Tunnel Establishment
```
Alice → BearDog: Establish BTSP tunnel to Bob
BearDog: Creates tunnel with Perfect Forward Secrecy
BearDog → Alice: Tunnel ID
```

### 5. Encrypted Communication
```
Alice → BearDog: Send message via tunnel
BearDog: Encrypts with ChaCha20-Poly1305
BearDog → Bob: Encrypted message
Bob → BearDog: Decrypt message
```

---

## Current Status

### ✅ What Works
- Runtime capability-based discovery
- Environment variable detection
- Gap identification and documentation

### 📚 What's Documented
- Songbird registration API (from phase1)
- BearDog BTSP API (from phase1)
- UPA (Universal Primal Adapter) pattern
- Complete API flow

### ❌ What's Missing
1. Running primal instances (Songbird, BearDog)
2. UPA client in biomeOS
3. BTSP methods in BearDog client wrapper
4. Full integration testing

---

## Gap Documentation

See `GAPS_DOCUMENTED.md` for complete API details including:
- Exact API calls from proven phase1 implementation
- UPA (Universal Primal Adapter) explanation
- Environment variable patterns
- Implementation path forward

---

## Reference Implementation

This demo is based on the **proven working** phase1 implementation:

```
phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/
```

That demo:
- ✅ Actually establishes BTSP tunnels
- ✅ Coordinates via Songbird
- ✅ Sends encrypted messages
- ✅ Uses capability-based discovery
- ✅ No hardcoded service names

---

## Running the Demo

### Prerequisites

#### Option 1: With Real Primals
```bash
# Terminal 1: Start Songbird
cd /home/eastgate/Development/ecoPrimals/primalBins/
./songbird-orchestrator

# Terminal 2: Start BearDog
./beardog-hsm --btsp-enabled

# Terminal 3: Set environment and run demo
export PRIMAL_SONGBIRD_ENDPOINT="mdns://songbird-tower.local"
export PRIMAL_SONGBIRD_CAPABILITIES="orchestration,p2p"
export PRIMAL_BEARDOG_ENDPOINT="http://localhost:9091"
export PRIMAL_BEARDOG_CAPABILITIES="encryption,btsp"

cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/showcase/03-p2p-coordination/01-btsp-tunnel-coordination/
./demo.sh
```

#### Option 2: Gap Discovery Only
```bash
# Just run the demo - it will document what's missing
./demo.sh
```

---

## Expected Output

### With Primals Running
```
🔒 BTSP Tunnel Coordination Demo
════════════════════════════════════════════════════════

Step 1: Discover Orchestration Service (Songbird)
✅ Found orchestration service
  Name: songbird
  Endpoint: mdns://songbird-tower.local
  Capabilities: orchestration,p2p

Step 2: Discover Encryption Service (BearDog)
✅ Found encryption service
  Name: beardog
  Endpoint: http://localhost:9091
  Capabilities: encryption,btsp

Step 3: Verify BTSP Capability
✅ BTSP capability confirmed

Step 4: Establish BTSP Tunnel
[API documentation and expected flow]

✨ Gap Discovery Complete
```

### Without Primals (Gap Discovery)
```
🔒 BTSP Tunnel Coordination Demo
════════════════════════════════════════════════════════

Step 1: Discover Orchestration Service (Songbird)
⚠️  No orchestration service discovered

═══ GAP IDENTIFIED ═══
📋 Songbird Discovery:
  • Expected: Songbird running with orchestration capability
  • Actual: Not found
  • Action: Start Songbird or set environment variables

[... rest of gap documentation ...]
```

---

## Next Steps

### Phase 1: Complete Gap Documentation ✅
- Study phase1 implementation
- Document actual APIs
- Understand UPA abstraction

### Phase 2: Implement UPA Client
```rust
// biomeOS/src/primal_clients/upa_client.rs
pub struct UpaClient {
    endpoint: String,
}

impl UpaClient {
    pub async fn register_node(&self, req: RegisterRequest) -> Result<String>;
    pub async fn find_peer(&self, name: &str) -> Result<PeerInfo>;
}
```

### Phase 3: Extend BearDog Client
```rust
// biomeOS/src/primal_clients/beardog_client.rs
impl BeardogClient {
    pub async fn establish_btsp_tunnel(&self, peer: PeerEndpoint) -> Result<TunnelId>;
    pub async fn send_btsp_message(&self, tunnel_id: &str, data: &[u8]) -> Result<()>;
}
```

### Phase 4: Integration Testing
- Start Songbird tower (mDNS/UDP)
- Start BearDog instances
- Run complete demo
- Send encrypted messages

---

## Files

- `demo.sh` - Main demo script (gap discovery)
- `GAPS_DOCUMENTED.md` - Complete API documentation (★ READ THIS)
- `README.md` - This file
- `../../common/discovery.sh` - Runtime discovery library

---

## Key Insights

### 1. Songbird Uses mDNS/UDP
**Not HTTP!** The phase1 implementation uses UPA (Universal Primal Adapter) to abstract the protocol.

### 2. Capability-Based Discovery Works
Environment variables `PRIMAL_*_ENDPOINT` and `PRIMAL_*_CAPABILITIES` enable runtime discovery without hardcoding.

### 3. BTSP is Real
Phase1 has a complete, working BTSP implementation. We just need to integrate it properly in biomeOS.

### 4. No Hardcoding
Following the architecture: primals only know themselves, discover others by capability at runtime.

---

## References

- `phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/` - Full working code
- `phase1/beardog/showcase/00-local-primal/06-btsp-tunnel/` - BTSP concepts
- `PRIMAL_GAPS.md` - Known integration gaps
- `specs/UNIVERSAL_ADAPTER_SPECIFICATION.md` - UPA design

---

**Status**: Gap discovery phase complete  
**Next**: Implement UPA client or HTTP wrappers  
**Estimated Time**: 2-4 hours for basic implementation  

🔍 **Understanding first, implementation second!**
