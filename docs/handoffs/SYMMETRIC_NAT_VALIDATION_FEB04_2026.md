# Symmetric NAT Validation Session - February 4, 2026

**Status**: ✅ GENOME SYNCED - Ready for Onion Testing  
**Test Environment**: Tower (x86_64) + Pixel 8a (aarch64) on mobile hotspot  
**Network**: Both devices behind symmetric NAT

---

## Executive Summary

Successfully validated the mesh.* IPC integration on Tower. Symmetric NAT connectivity test shows the infrastructure is operational but requires updated Pixel binary and completion of STUN client methods.

---

## Test Environment

### Tower (x86_64)
- **Public IP**: 162.226.225.148
- **Local IP**: 192.168.1.144 (ethernet), 192.168.1.244 (wifi)
- **NAT Type**: Symmetric (per ISP)
- **BearDog**: ✅ Running (crypto API operational)
- **Songbird**: ✅ Running with mesh.* methods
- **STUN Server**: ✅ Running on 0.0.0.0:3478
- **Relay Server**: ✅ Running on 0.0.0.0:3491

### Pixel 8a (aarch64)
- **Local IP**: 172.20.10.9 (hotspot)
- **NAT Type**: Symmetric (mobile carrier)
- **Internet**: ✅ Reachable (8.8.8.8 ping works)
- **Songbird**: ✅ Running (older version, no mesh.* methods)
- **ADB Port Forward**: ✅ tcp:19901 → tcp:9901

---

## Validated Components

### 1. Mesh IPC Methods (Tower)

| Method | Status | Notes |
|--------|--------|-------|
| `mesh.init` | ✅ Working | Initialized as "tower" node |
| `mesh.status` | ✅ Working | Returns node status |
| `mesh.find_path` | ✅ Working | Returns paths (none yet) |
| `mesh.announce` | ✅ Working | Announces as relay |
| `mesh.peers` | ✅ Working | Lists peers (empty) |
| `mesh.health_check` | ✅ Working | Health check works |
| `punch.request` | ✅ Working | Initiates punch (no coordinator) |
| `punch.status` | ✅ Working | Returns punch status |

### 2. STUN/Relay Servers (Tower)

| Service | Port | Status |
|---------|------|--------|
| STUN Server | 3478 (UDP/TCP) | ✅ Running |
| Relay Server | 3491 (TCP) | ✅ Running |

### 3. BirdSong Beacons (Tower)

```json
{
  "method": "birdsong.generate_encrypted_beacon",
  "result": {
    "beacon_size_bytes": 150,
    "encrypted_beacon": "LBeKNsDFipuRc3CnLo1+TM...",
    "family_id": "nat0",
    "node_id": "tower"
  }
}
```

### 4. Connectivity Status

| Test | Result |
|------|--------|
| Tower → Internet | ✅ 162.226.225.148 |
| Pixel → Internet | ✅ (8.8.8.8 reachable) |
| Pixel → Tower (direct) | ❌ Blocked by NAT |
| Tower → Pixel (direct) | ❌ Blocked by NAT |
| Pixel → Tower (ADB forward) | ✅ Working |

---

## Issues Identified

### 1. STUN Client Methods Not Wired

The `stun.get_public_address` and `stun.bind` methods are listed in `rpc.discover` but not implemented in the match statement.

**Impact**: Cannot programmatically discover public addresses via IPC.

**Fix Required**: Wire these methods in `service.rs`:
```rust
"stun.get_public_address" => self.handle_stun_get_public_address(params).await,
"stun.bind" => self.handle_stun_bind(params).await,
```

### 2. Pixel Has Older Songbird

The Pixel binary (from Feb 5) doesn't have the mesh.* methods. Need to cross-compile the updated version.

**Solution**: Build with Android NDK or use `cross` after fixing glibc issues.

```bash
# NDK path
/opt/android-ndk-r25c/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang

# Target
cargo build --release --target aarch64-linux-android -p songbird
```

### 3. Relay Allocation is Placeholder

The `relay.allocate` method returns a formatted request but doesn't create active sessions. The actual relay packet forwarding isn't wired to the IPC layer.

---

## Services Running

```bash
# Tower sockets
/run/user/1000/biomeos/beardog-nat0.sock   # BearDog crypto
/run/user/1000/biomeos/songbird-nat0.sock  # Songbird with mesh.*

# Tower ports
0.0.0.0:3478  # STUN server
0.0.0.0:3491  # Relay server

# Pixel via ADB
localhost:19901 → Pixel:9901 (Songbird IPC)
```

---

## Next Steps

### Immediate (Enable Full Testing)

1. **Wire STUN Client Methods** (~1 hour)
   - Implement `handle_stun_get_public_address()` in `stun_handler.rs`
   - Wire to `service.rs` match statement
   - Test with Google STUN servers

2. **Cross-Compile Songbird for Android** (~2 hours)
   - Fix NDK toolchain configuration
   - Build with mesh.* methods
   - Deploy to Pixel via ADB

### Short-Term (Complete P2P)

3. **Wire Relay Session Management** (~2-3 hours)
   - Connect relay.allocate to actual session creation
   - Implement packet forwarding callback

4. **Test Full Symmetric NAT Traversal**
   - Tower ↔ Pixel via relay
   - Verify data forwarding works

---

## Commands Reference

```bash
# Tower BearDog health
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/1000/biomeos/beardog-nat0.sock -w 2

# Tower Songbird health
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock -w 2

# Tower mesh status
echo '{"jsonrpc":"2.0","method":"mesh.status","id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock -w 2

# Pixel Songbird health (via ADB)
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc localhost 19901 -w 2

# Generate beacon
echo '{"jsonrpc":"2.0","method":"birdsong.generate_encrypted_beacon","params":{"node_id":"tower"},"id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock -w 5
```

---

## Architecture Verified

```
Tower (Symmetric NAT)              Pixel 8a (Hotspot NAT)
┌─────────────────────┐            ┌─────────────────────┐
│ BearDog (crypto)    │            │                     │
│   ↓                 │            │                     │
│ Songbird (mesh.*)   │◀──ADB──────│ Songbird (old)      │
│   ├─ STUN Server    │            │                     │
│   ├─ Relay Server   │            │                     │
│   └─ BeaconMesh     │            │                     │
└─────────────────────┘            └─────────────────────┘
        ↓                                    ↓
   162.226.225.148                    (Hotspot IP)
        ↓                                    ↓
   ══════════ Internet (Both Symmetric NAT) ══════════
        
Direct connection BLOCKED - need relay or onion
```

---

---

## Update: Genome Reharvest Complete (17:06 UTC)

### Cross-Compiled & Deployed

| Binary | Target | Location |
|--------|--------|----------|
| Songbird (aarch64) | Android | `livespore-usb/aarch64/primals/songbird` |
| Songbird (aarch64) | Android | `pixel8a-deploy/songbird` |
| Songbird (x86_64) | Linux | `livespore-usb/plasmidBin/songbird` |

### Mesh Status

**Tower**:
```json
{"node_id":"tower","reachable_peers":0,"relay_enabled":true,"uptime_seconds":703}
```

**Pixel**:
```json
{"node_id":"pixel8a","reachable_peers":0,"relay_enabled":true,"uptime_seconds":67}
```

Both devices now have mesh.* methods:
- `mesh.init`, `mesh.status`, `mesh.find_path`
- `mesh.announce`, `mesh.peers`, `mesh.health_check`
- `punch.request`, `punch.status`

---

## Sovereign Onion Gateway (Next Steps)

The sovereign onion service (`songbird-sovereign-onion`) is fully implemented and can make Tower a family gateway that bypasses symmetric NAT:

### Architecture

```
                    Internet
                       │
        ┌──────────────┴──────────────┐
        │                             │
    ═══════════                   ═══════════
    Symmetric NAT                 Hotspot NAT
    ═══════════                   ═══════════
        │                             │
   ┌─────────┐                   ┌─────────┐
   │  Tower  │                   │ Pixel8a │
   │         │                   │         │
   │ OnionSvc│◀────.onion────────│Connector│
   │ (listen)│                   │(connect)│
   └─────────┘                   └─────────┘
```

### To Enable (When Hotspot Returns)

1. **Wire onion.* methods to IPC** (not yet done):
   ```rust
   // In service.rs
   "onion.start" => self.onion_handler.handle_start(params).await,
   "onion.status" => self.onion_handler.handle_status(params).await,
   "onion.connect" => self.onion_handler.handle_connect(params).await,
   ```

2. **Start Tower as Onion Gateway**:
   ```bash
   # Tower creates .onion address
   echo '{"jsonrpc":"2.0","method":"onion.start","params":{"port":3492},"id":1}' \
     | nc -U /run/user/1000/biomeos/songbird-nat0.sock
   
   # Returns: {"onion_address": "xyz123...abc.onion:3492"}
   ```

3. **Pixel Connects via Onion**:
   ```bash
   echo '{"jsonrpc":"2.0","method":"onion.connect","params":{"address":"xyz123...abc.onion:3492"},"id":1}' \
     | nc localhost 19901
   ```

### Onion Service Implementation

The code exists in `songbird-sovereign-onion`:

- `OnionService::new_via_beardog()` - Creates service with BearDog crypto
- `OnionService::run()` - Listens for connections
- `OnionConnector::connect()` - Connects to .onion address
- `OnionConnection::send()/recv()` - Encrypted P2P communication

All crypto delegated to BearDog (TRUE PRIMAL compliant).

---

*Genome reharvest complete. Ready for onion gateway testing when hotspot returns.*
