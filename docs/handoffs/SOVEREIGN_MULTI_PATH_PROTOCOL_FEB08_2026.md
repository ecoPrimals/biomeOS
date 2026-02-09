# Sovereign Multi-Path Protocol Handoff

**Date**: February 8, 2026
**Session**: IPv6 Dual-Stack Fix + Onion Activation + Full Protocol Design
**Status**: IPv6 and Onion WORKING. Router evolution and relay wiring needed.

---

## What Was Done This Session

### 1. IPv6 Dual-Stack Fix (COMPLETE)

**Root cause**: `sovereign_socket.rs` tried IPv4 `0.0.0.0` first. Since it always succeeded, IPv6 `[::]` was never attempted. On Linux, IPv4 binding blocks IPv6 from the same port.

**Fix applied** (`phase1/songbird/crates/songbird-orchestrator/src/network/sovereign_socket.rs`):
- Reversed `bind_sovereign()` strategy order: IPv6 dual-stack first, IPv4 fallback second
- IPv6 socket uses `set_only_v6(false)` for dual-stack (serves both IPv4 and IPv6 on one socket)
- Binary rebuilt (`songbird 3.33.0`) and deployed to `livespore-usb/x86_64/primals/songbird`

**Verified**:
```
Binding: LISTEN *:3492 (dual-stack)
IPv4 localhost 127.0.0.1:3492    → OK
IPv6 localhost [::1]:3492        → OK
IPv6 global [2600:1700:b0b0:5b90::27]:3492 → OK
```

IPv6 means the Tower is globally reachable without port forwarding. The AAAA record at `tower.nestgate.io` already resolves to `2600:1700:b0b0:5b90::27`.

### 2. Sovereign Onion Service (ACTIVATED)

The `songbird-sovereign-onion` crate was already fully implemented. This session we activated it by ensuring BearDog was properly wired (`BEARDOG_SOCKET` env var).

**Onion address**: `p6m5exqn44xpjtvpal6juhdzh3s7zvlpysrjcknzrxada6mny54ltiyd.onion:3492`
**Identity persisted at**: `./data/sovereign-onion/`
**Protocol**: X25519 key exchange + ChaCha20-Poly1305 AEAD (all via BearDog)

This is NOT full Tor (no relay circuit building). It's a simplified sovereign protocol:
- Ed25519 identity → deterministic `.onion` address
- Direct TCP with X25519 handshake
- Every byte encrypted with BearDog-delegated ChaCha20-Poly1305
- Session key derived per-connection with forward secrecy

### 3. Configuration Updates

- `.known_beacons.json` (Tower and USB) updated with onion endpoint and multi-path strategy
- Connection priority updated to 7 tiers (see below)

---

## Current System State

### Running Processes (Tower/Gate)
```
BearDog:  /run/user/1000/biomeos/beardog.sock  (crypto provider)
Songbird: /run/user/1000/biomeos/songbird.sock  (network, port 3492)
```

### Songbird Capabilities (Live)
```
discovery:  peers, mdns, broadcast, scan
stun:       get_public_address, bind
http:       request, get, post
ipc:        register, resolve, discover, list
rendezvous: register, lookup
peer:       connect
birdsong:   generate_encrypted_beacon, decrypt_beacon, verify_lineage, get_lineage
```

### Active Services
```
Onion:      RUNNING (p6m5exqn44xpjtvpal6juhdzh3s7zvlpysrjcknzrxada6mny54ltiyd.onion:3492)
Mesh:       INITIALIZED (node_id: gate, relay_enabled: true)
STUN Server: Can be started via stun.serve (binds 0.0.0.0:3478)
Birdsong:   ACTIVE (family_id: 1894e909e454, encryption: chacha20_poly1305)
```

### DNS Records (nestgate.io)
```
tower.nestgate.io  A     162.226.225.148
tower.nestgate.io  AAAA  2600:1700:b0b0:5b90::27
beacon.nestgate.io TXT   v=biomeos2 (BearDog-encrypted beacon blob)
```

---

## The Full Multi-Path Protocol

### Connection Priority (7 tiers)

```
TIER 1: IPv6 Direct (via DNS)
  → tower.nestgate.io resolves AAAA → [2600:1700:b0b0:5b90::27]:3492
  → No NAT, no port forward, globally routable
  → WORKING NOW

TIER 2: Sovereign Onion Overlay
  → p6m5exqn44xpjtvpal6juhdzh3s7zvlpysrjcknzrxada6mny54ltiyd.onion:3492
  → Cryptographic identity = address, bypasses all NAT/ISP restrictions
  → Direct TCP with X25519 + ChaCha20-Poly1305
  → WORKING NOW (but needs peer to know the address — via beacon or known_beacons)

TIER 3: IPv4 Direct (via DNS)
  → tower.nestgate.io resolves A → 162.226.225.148:3492
  → REQUIRES router port forward (162.226.225.148:3492 → 192.168.1.144:3492)
  → OR: Songbird IGD evolution (see below)

TIER 4: LAN Direct
  → 192.168.1.144:3492 (same subnet only)
  → WORKING NOW

TIER 5: STUN Hole-Punch
  → Songbird STUN client races multiple servers to discover public IP:port
  → Coordinator exchanges STUN results between peers for UDP hole-punching
  → BUILT (songbird-stun), needs coordinator wiring

TIER 6: Family Relay
  → Mesh relay via family member with better connectivity
  → BUILT (mesh.init, relay_enabled: true), needs peer connections

TIER 7: DNS Beacon Discovery
  → beacon.nestgate.io TXT record contains BearDog-encrypted endpoint list
  → Family members decrypt with shared seed to discover all endpoints (including .onion)
  → WORKING NOW
```

### How a Peer Connects

```
1. Peer decrypts DNS beacon (beacon.nestgate.io TXT)
   → Gets family_id, node_id, all endpoints including .onion
   
2. Peer tries tiers in order:
   a. IPv6 direct to [2600:1700:b0b0:5b90::27]:3492
   b. Onion to p6m5exqn44xpjtvpal6juhdzh3s7zvlpysrjcknzrxada6mny54ltiyd.onion:3492
   c. IPv4 direct (if port forward or IGD configured)
   d. LAN direct (if same subnet)
   e. STUN hole-punch (if coordinator available)
   f. Family relay (if another family member is online)

3. All connections verified by Dark Forest lineage
   → Birdsong verifies family_id cryptographically
   → Non-family connections rejected at protocol level
```

---

## What Needs Building: Songbird IGD Evolution

### The Philosophy

Port forwarding is currently an external dependency — you have to manually log into the router and configure it. This is anti-sovereign. The UPnP/IGD (Internet Gateway Device) protocol allows a device on the LAN to programmatically request port forwards from the router. Songbird should speak IGD natively, turning the router from a dependency into a tool Songbird configures.

### Implementation Plan

**New crate**: `songbird-igd` (or add to `songbird-orchestrator/src/network/`)

**Protocol**: UPnP IGD (RFC 6970) — pure Rust, zero C dependencies

**Steps**:

1. **SSDP Discovery** (Simple Service Discovery Protocol)
   - Send `M-SEARCH` multicast to `239.255.255.250:1900`
   - Parse responses for `InternetGatewayDevice` or `WANIPConnection`
   - This discovers the router's UPnP control URL

2. **SOAP Control**
   - HTTP POST to the router's control URL
   - `AddPortMapping` action: request external port → internal IP:port mapping
   - `GetExternalIPAddress` action: discover public IPv4 (like STUN but from the router itself)
   - `DeletePortMapping` action: clean up on shutdown

3. **Songbird Integration**
   - New JSON-RPC methods: `igd.discover`, `igd.map_port`, `igd.unmap_port`, `igd.status`, `igd.external_ip`
   - Auto-map on startup if `SONGBIRD_IGD_ENABLED=true`
   - Auto-unmap on graceful shutdown
   - Periodic lease renewal (most IGD mappings have TTL)

4. **NAT-PMP Alternative**
   - Simpler protocol (Apple's alternative to UPnP IGD)
   - Binary UDP protocol, send to gateway:5351
   - Some routers support NAT-PMP but not UPnP
   - Songbird tries IGD first, falls back to NAT-PMP

### Rust Ecosystem

- `igd-next` crate: async UPnP IGD client (MIT licensed, pure Rust)
- However, for TRUE PRIMAL purity, implement directly using Songbird's existing HTTP client
- The SSDP discovery is just UDP multicast + HTTP parsing — Songbird already has both

### New JSON-RPC Methods

```json
// Discover router IGD capabilities
{"method": "igd.discover", "params": {}}
→ {"gateway_ip": "192.168.1.1", "control_url": "http://192.168.1.1:5000/ctl/IPConn", "external_ip": "162.226.225.148"}

// Request port mapping
{"method": "igd.map_port", "params": {"external_port": 3492, "internal_port": 3492, "protocol": "TCP", "description": "Songbird sovereign beacon", "ttl": 86400}}
→ {"mapped": true, "external": "162.226.225.148:3492", "internal": "192.168.1.144:3492", "ttl": 86400}

// Check mapping status
{"method": "igd.status", "params": {}}
→ {"mappings": [{"external": 3492, "internal": 3492, "protocol": "TCP", "ttl_remaining": 85200}], "external_ip": "162.226.225.148"}

// Remove mapping
{"method": "igd.unmap_port", "params": {"external_port": 3492, "protocol": "TCP"}}
→ {"unmapped": true}
```

### Startup Integration

In `start_nucleus.sh` (or equivalent):
```bash
# After Songbird starts, request IGD port mapping
echo '{"jsonrpc":"2.0","method":"igd.discover","params":{},"id":1}' | nc -U $SONGBIRD_SOCKET -w 5
echo '{"jsonrpc":"2.0","method":"igd.map_port","params":{"external_port":3492,"internal_port":3492,"protocol":"TCP","description":"Songbird sovereign beacon","ttl":86400},"id":2}' | nc -U $SONGBIRD_SOCKET -w 5
```

Or better: Songbird auto-configures at startup when `SONGBIRD_IGD_ENABLED=true`.

---

## What Needs Building: Hole-Punch Coordinator

### Current State

- `songbird-stun` crate: COMPLETE (client + server, concurrent racing)
- `punch.request` RPC method: EXISTS but returns `"hole_punch_coordinator_not_initialized"`
- STUN server: Can be started via `stun.serve` (binds UDP 3478)

### What's Missing

The **coordinator** that exchanges STUN results between two peers so they can punch through NAT simultaneously:

1. **Peer A** does STUN, learns its public IP:port → tells coordinator
2. **Peer B** does STUN, learns its public IP:port → tells coordinator
3. Coordinator tells both peers each other's public IP:port
4. Both peers simultaneously send UDP packets to each other's public IP:port
5. NAT sees outbound packet, creates mapping, inbound packet from peer arrives

### Implementation

- Use the existing `rendezvous.register` / `rendezvous.lookup` to exchange STUN results
- The Tower (gate) acts as the rendezvous server (it has a public IP via IPv6)
- `punch.request` should: do STUN → register result at rendezvous → lookup peer → punch

---

## What Needs Building: Full Tor Relay Integration

### Current State

`songbird-tor-protocol` has:
- Directory protocol (fetch consensus, select relays) — scaffolded with TODOs
- Circuit protocol (ntor handshake, extend) — scaffolded with TODOs
- Onion service (descriptor, introduction, rendezvous) — scaffolded with TODOs
- Connection (TLS link protocol) — scaffolded
- All crypto delegated to BearDog

### What This Would Enable

Instead of direct TCP connections to .onion addresses (current sovereign-onion), traffic would route through actual Tor relay circuits. This provides:
- **True anonymity**: ISP sees encrypted Tor traffic, not the destination
- **Censorship resistance**: Even if ISP blocks destination IP, traffic routes through relays
- **No direct IP exposure**: Neither peer reveals their IP to each other

### Priority

**LOW** for the family mesh use case. The current sovereign-onion provides:
- Encrypted connections (ChaCha20-Poly1305)
- Cryptographic identity (.onion addresses)
- BearDog-delegated crypto (no embedded secrets)

Full Tor is only needed if the ISP actively blocks the Tower's IP or if anonymity between family members matters (it doesn't — they're family).

---

## What Needs Building: Beacon DNS Auto-Update

### Current State

`scripts/beacon_dns_updater.sh` manually pushes encrypted beacon to Porkbun DNS. It should be automated:

1. Songbird generates the encrypted beacon via `birdsong.generate_encrypted_beacon`
2. Songbird uses `http.post` to update the Porkbun API
3. Runs periodically (every 6 hours or on IP change)
4. New method: `beacon.publish_dns` that does all of this

This keeps the DNS beacon always current with the latest endpoints (including the .onion address).

---

## Files Changed This Session

### Modified
- `phase1/songbird/crates/songbird-orchestrator/src/network/sovereign_socket.rs` — IPv6-first binding
- `livespore-usb/x86_64/primals/songbird` — rebuilt binary (v3.33.0)
- `.known_beacons.json` (Tower) — added onion endpoint, multi-path strategy
- `livespore-usb/.known_beacons.json` (USB) — added onion endpoint, multi-path strategy

### Not Yet Updated (Need Sync)
- `livespore-usb/aarch64/primals/songbird` — needs cross-compile with IPv6 fix
- `pixel8a-deploy/primals/songbird` — needs cross-compile with IPv6 fix
- `pixel8a-deploy/.known_beacons.json` — needs onion endpoint added
- `scripts/start_nucleus.sh` — should auto-start onion + mesh after Songbird
- `scripts/beacon_dns_updater.sh` — should include .onion in beacon payload

---

## Quick Start Commands

### Start the full stack on Tower
```bash
# BearDog (if not running)
FAMILY_ID=1894e909e454 NODE_ID=gate BIOMEOS_ROOT=/home/eastgate/Development/ecoPrimals/phase2/biomeOS \
  /path/to/beardog server --socket /run/user/1000/biomeos/beardog.sock &

# Songbird (IPv6 dual-stack + BearDog wired)
FAMILY_ID=1894e909e454 NODE_ID=gate BIOMEOS_BIND_ALL=true \
  BEARDOG_SOCKET=/run/user/1000/biomeos/beardog.sock \
  SONGBIRD_SECURITY_PROVIDER=/run/user/1000/biomeos/beardog.sock \
  BIOMEOS_ROOT=/home/eastgate/Development/ecoPrimals/phase2/biomeOS \
  /path/to/songbird server --port 3492 --socket /run/user/1000/biomeos/songbird.sock --verbose &

# Activate onion + mesh (via IPC)
echo '{"jsonrpc":"2.0","method":"onion.start","params":{"port":3492},"id":1}' | nc -U /run/user/1000/biomeos/songbird.sock -w 10
echo '{"jsonrpc":"2.0","method":"mesh.init","params":{"family_id":"1894e909e454","node_id":"gate"},"id":2}' | nc -U /run/user/1000/biomeos/songbird.sock -w 5
```

### Verify
```bash
# IPv6 dual-stack
curl -sk http://[::1]:3492/health        # IPv6 localhost
curl -sk http://127.0.0.1:3492/health    # IPv4 localhost

# Onion status
echo '{"jsonrpc":"2.0","method":"onion.status","params":{},"id":1}' | nc -U /run/user/1000/biomeos/songbird.sock -w 5

# Birdsong lineage
echo '{"jsonrpc":"2.0","method":"birdsong.get_lineage","params":{},"id":1}' | nc -U /run/user/1000/biomeos/songbird.sock -w 5
```

---

## Priority Order for Next Session

1. **IGD/UPnP evolution** — Turn router port forwarding into a Songbird tool
2. **Auto-start onion + mesh in start_nucleus.sh** — Make the full stack come up automatically
3. **Hole-punch coordinator wiring** — Connect STUN + rendezvous + punch.request
4. **Cross-compile aarch64 Songbird** — Bring IPv6 fix to USB and Pixel
5. **Beacon DNS auto-update** — Include .onion in beacon, auto-refresh
6. **Tor relay circuits** (LOW priority) — Full anonymity routing

---

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    TOWER (gate)                              │
│                                                             │
│  BearDog ──── Songbird ──── Mesh                            │
│  (crypto)     (network)     (coordination)                  │
│     │            │              │                            │
│     │     ┌──────┼──────┐      │                            │
│     │     │      │      │      │                            │
│     ▼     ▼      ▼      ▼      ▼                            │
│  [::]:3492    .onion    STUN   Relay                        │
│  IPv6+IPv4    Overlay   Server  Node                        │
│                                                             │
│  ┌─────────────────────────────────────┐                    │
│  │ DNS Beacon (beacon.nestgate.io TXT) │                    │
│  │ Encrypted: family, endpoints, .onion│                    │
│  └─────────────────────────────────────┘                    │
│                                                             │
│  ┌─────────────────────────────────────┐  ← NEXT: BUILD    │
│  │ IGD/UPnP (igd.map_port)            │                    │
│  │ Router becomes a tool, not a dep   │                    │
│  └─────────────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
         │           │          │         │
    IPv6 Direct   Onion     STUN Punch  Relay
         │           │          │         │
         ▼           ▼          ▼         ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   USB (usb)  │  │ Pixel (pixel)│  │  Future Node │
│              │  │              │  │              │
│ BearDog      │  │ BearDog      │  │ BearDog      │
│ Songbird     │  │ Songbird     │  │ Songbird     │
│ Dark Forest  │  │ Dark Forest  │  │ Dark Forest  │
└──────────────┘  └──────────────┘  └──────────────┘

ZERO external dependencies.
Pure Rust ecoPrimals throughout.
BearDog crypto | Songbird networking | Dark Forest gating.
```
