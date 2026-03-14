> **HISTORICAL** — This handoff predates v2.37. See CURRENT_STATUS.md for latest.

# Covalent Bond Evolution Handoff - Feb 10, 2026

## Goal

Enable two NUCLEUS instances (Tower @ 192.168.1.144, gate2 @ 192.168.1.132) to discover each other via beacon, establish covalent bond via genetic lineage, and communicate via Songbird HTTP JSON-RPC without SSH. This is prerequisite for distributed AI workloads across NVIDIA/AMD GPUs on multiple computers.

## What Was Completed

### biomeOS Code Evolution (DONE)

1. **`AtomicClient::http()` transport** -- New HTTP JSON-RPC transport in `biomeos-core/src/atomic_client.rs` that POSTs to `http://{host}:{port}/jsonrpc` using pure Rust (raw `TcpStream`, zero new dependencies). Added `TransportEndpoint::HttpJsonRpc` variant to `socket_discovery/transport.rs`.

2. **Hardcoded port 3492 eliminated from Plasmodium** -- `plasmodium.rs` `query_remote_gate()` now resolves port via: `mesh.peers` address → `SONGBIRD_MESH_PORT` env → default 8080. Uses `AtomicClient::http()` instead of `AtomicClient::tcp()`.

3. **Beacon `jsonrpc_port` announcement** -- Songbird `mesh_handler.rs` `udp_multicast_discover()` now includes `jsonrpc_port` in the beacon payload (resolved from `SONGBIRD_HTTP_PORT` / `SONGBIRD_PORT` / default 8080). Peer discovery extracts this port and stores it in the endpoint address instead of the ephemeral UDP source port.

### Deployment (DONE)

- Both devices enrolled with `Blake3-Lineage-KDF` from shared `.family.seed`
- All 5 primals running on gate2 (BearDog, Songbird, NestGate, Toadstool, Squirrel)
- Cross-machine HTTP JSON-RPC verified: Tower can call `health` on gate2's Songbird via `POST http://192.168.1.132:8080/jsonrpc` and gets a healthy response

---

## Blocking Issues Found (4 root causes)

### Issue 1: Songbird IPC Socket Not Created on gate2

**Symptom**: `songbird server --socket /run/user/1000/biomeos/songbird.sock` runs but the socket file never appears. HTTP on port 8080 works fine.

**Root Cause**: The `--socket` flag triggers the IPC server in `bin_interface/server.rs` (line 133-161), but this is a **separate task** that runs in parallel with the orchestrator. The orchestrator's own `start_ipc_server()` in `app/core.rs` (line 407) creates a DIFFERENT `UnixSocketServer` that derives its socket path from env vars (`SONGBIRD_ORCHESTRATOR_SOCKET`, `SONGBIRD_SOCKET`, etc.) and **ignores the CLI `--socket` flag entirely**.

The bin_interface IPC server at line 239 does `UnixListener::bind(socket_path)`, but if the orchestrator has already bound to the same directory or there's a race condition, the bind may silently fail. More likely: the `nohup` + SSH environment may prevent the background task from running properly.

**Files**:
- `songbird-orchestrator/src/bin_interface/server.rs` lines 133-176 (bin_interface IPC task spawning)
- `songbird-orchestrator/src/bin_interface/server.rs` lines 239-335 (actual `start_ipc_server` function)
- `songbird-orchestrator/src/app/core.rs` lines 407-473 (orchestrator's SEPARATE `start_ipc_server`)
- `songbird-orchestrator/src/ipc/pure_rust_server/server.rs` lines 159-195 (`socket_path_from_env`)

**Fix Guidance**:
- The orchestrator creates its OWN socket via `socket_path_from_env()` (usually `songbird-{family_hash}.sock`). The `--socket` flag creates a SECOND server. Both should exist, OR the CLI flag should override the orchestrator's path.
- Verify that the bin_interface task (line 155) is actually completing the `start_ipc_server` call and not panicking silently in the spawned task.
- Consider consolidating to ONE IPC server that uses the CLI `--socket` path when provided.

---

### Issue 2: HTTP and IPC Have Separate Mesh State (Mesh State Split)

**Symptom**: `mesh.init` via HTTP succeeds. `mesh.auto_discover` via HTTP returns "Mesh not initialized". Meanwhile `mesh.init` and `mesh.auto_discover` via Unix socket (when available) work fine.

**Root Cause**: Three SEPARATE `IpcServiceHandler` instances are created, each with its own `MeshHandler`:

1. **HTTP gateway** (`app/http_server.rs` line 111): Creates `IpcServiceHandler::new(ipc_registry)` → creates `MeshHandler::new()`. This is wired into the Axum HTTP server's `/jsonrpc` endpoint via `JsonRpcState::with_ipc_handler()`.

2. **bin_interface IPC** (`bin_interface/server.rs` line 246): Creates another `IpcServiceHandler::new(registry)` → another `MeshHandler::new()`. This handles Unix socket connections on the `--socket` path.

3. **Orchestrator IPC** (`app/core.rs` line 418-444): Creates `UnixSocketServer` with its own `IpcHandlers` (different struct). This handles the orchestrator's own socket.

When `mesh.init` is called via HTTP, it initializes the mesh on handler #1. When `mesh.auto_discover` is called via HTTP, it uses handler #1 BUT the mesh may not be properly initialized because the actual mesh networking state (peer table, UDP sockets) is per-handler.

**Files**:
- `songbird-orchestrator/src/app/http_server.rs` lines 108-117 (HTTP gateway handler creation)
- `songbird-orchestrator/src/bin_interface/server.rs` lines 244-246 (bin_interface handler creation)
- `songbird-universal-ipc/src/service.rs` lines 155-175 (`IpcServiceHandler::new` creating `MeshHandler::new()`)
- `songbird-universal-ipc/src/handlers/mesh_handler.rs` (MeshHandler state)

**Fix Guidance**:
- Create ONE shared `IpcServiceHandler` (or at minimum one shared `MeshHandler`) and pass it via `Arc` to all three consumers (HTTP, bin_interface IPC, orchestrator IPC).
- The HTTP server already accepts an `IpcServiceHandler` via `with_ipc_handler()` -- make the bin_interface pass the SAME instance.
- Alternatively: have the bin_interface NOT create its own handler, and instead share the HTTP gateway's handler.

---

### Issue 3: UDP Multicast Discovery Protocol Bug

**Symptom**: `mesh.auto_discover` on both Tower and gate2 always returns 0 peers, even when both are running simultaneously.

**Root Cause**: The `udp_multicast_discover()` function (mesh_handler.rs line 499) binds to `0.0.0.0:0` (random ephemeral port), sends beacon to `239.255.77.77:5353` and `255.255.255.255:5353`, then listens for responses **on the ephemeral port**. But the other Songbird also does the same thing. Neither is actually listening on port 5353, so neither receives the other's beacon. They're sending to port 5353 but listening on random ports.

Additionally, port 5353 is the standard **mDNS port**, so the beacons collide with all the mDNS traffic on the network (printers, smart TVs, Chromecast, etc.), causing JSON parse failures on the receiver side.

**Files**:
- `songbird-universal-ipc/src/handlers/mesh_handler.rs` lines 498-501 (ephemeral port bind)
- `songbird-universal-ipc/src/handlers/mesh_handler.rs` lines 536-542 (send to 5353)
- `songbird-universal-ipc/src/handlers/mesh_handler.rs` lines 544-578 (listen on ephemeral port)

**Fix Guidance**:
- Bind the UDP socket to `0.0.0.0:5353` (or a dedicated Songbird port like `5377`) instead of `0.0.0.0:0`. Both sides need to be listening on the SAME port that beacons are sent to.
- Use `SO_REUSEADDR` / `SO_REUSEPORT` so multiple processes can share the port.
- Join the multicast group (`239.255.77.77`) on the listening socket so multicast packets are actually delivered.
- Consider using a Songbird-specific port (e.g., 5377 = "SB77") instead of 5353 to avoid mDNS collision.
- Add a "type" filter early in the recv loop to skip non-Songbird packets.
- The discovery should be bidirectional: when a node receives a beacon, it should send a `songbird_discovery_response` BACK to the sender's address (which is in the `recv_from` result).

---

### Issue 4: Hardcoded Port 3492 Throughout Songbird Codebase

**Symptom**: gate2's Songbird repeatedly tried to TLS-connect to `192.168.1.244:3492` (wrong IP, wrong port). Port 3492 appears as a default in many places.

**Root Cause**: Port 3492 is hardcoded as the default across multiple Songbird subsystems. It was originally the Songbird "federation port" but the actual HTTP server now defaults to 8080. The mismatch causes connection failures.

**Locations (22 occurrences across 12 files)**:

| File | Context | Severity |
|------|---------|----------|
| `songbird-orchestrator/src/server/jsonrpc_api.rs` line 65 | Comment references ":3492" | LOW (docs) |
| `songbird-universal-ipc/src/handlers/onion_handler.rs` lines 76,90,111,196,239,270 | `unwrap_or(3492)` as default onion port | HIGH |
| `songbird-universal-ipc/src/service.rs` line 583 | `unwrap_or(3492)` for onion endpoint | MEDIUM |
| `songbird-universal-ipc/src/handlers/igd_handler.rs` lines 86,100,186,267 | `unwrap_or(3492)` for IGD port mapping defaults | MEDIUM |
| `songbird-universal-ipc/src/handlers/birdsong_handler.rs` line 430 | Comment references ":3492" | LOW (docs) |
| `songbird-igd/src/soap.rs` lines 339-348 | Test uses 3492 | LOW (test) |
| `songbird-igd/src/nat_pmp.rs` line 261 | Test uses 3492 | LOW (test) |
| `songbird-igd/src/renewal.rs` lines 175-185 | Test uses 3492 | LOW (test) |
| `songbird-igd/src/gateway.rs` line 135 | Manual config instructions reference 3492 | LOW (docs) |
| `songbird-igd/src/mapping.rs` lines 196-217 | Tests use 3492 | LOW (test) |
| `songbird-igd/src/lib.rs` lines 56-57 | Doc example uses 3492 | LOW (docs) |

**Fix Guidance**:
- Replace all `unwrap_or(3492)` in production code with runtime-discoverable port: `SONGBIRD_HTTP_PORT` env → `SONGBIRD_PORT` env → 8080 default.
- The onion handler defaults are the most critical since they affect actual connections.
- Tests and docs can be updated to use 8080 for consistency, or kept as-is (they test specific values).
- Consider creating a `songbird_config::default_port()` helper that all subsystems use.

---

## Architecture Gaps for Multi-GPU Distributed AI

For the next deployment (local AI across NVIDIA/AMD GPUs, Squirrel using local + API):

### Gap A: No Graph-Based NUCLEUS Startup

We manually started primals with ad-hoc `nohup` commands. The graph executor (`biomeos atomic deploy graphs/tower_atomic_bootstrap.toml`) should handle dependency ordering, env var wiring, socket path configuration, and health verification. Current manual startup missed env vars (`NODE_ID`), used wrong flags, and required debugging each primal individually.

**Action**: Test and validate `biomeos atomic deploy` for both Tower and gate2 NUCLEUS startup.

### Gap B: No Neural API on gate2

gate2 has no Neural API running. Without it, standard `capability.call` routing (which goes through Neural API → capability registry → primal socket) won't work. Cross-gate capability routing requires Neural API on both sides.

**Action**: Deploy and start Neural API on gate2, registered with all local primal sockets.

### Gap C: Squirrel Multi-Backend Coordination

Squirrel needs to orchestrate between local GPU inference (llama.cpp on NVIDIA/AMD) and remote API endpoints. The covalent bond transport (Songbird HTTP JSON-RPC) is ready for cross-gate Squirrel-to-Squirrel communication, but the actual workload distribution logic needs testing.

**Action**: Validate Squirrel's `model.inference` capability can route to local GPU vs remote API based on availability/cost.

---

## Files Modified in This Session

### biomeOS (phase2)

| File | Change |
|------|--------|
| `crates/biomeos-core/src/socket_discovery/transport.rs` | Added `TransportEndpoint::HttpJsonRpc` variant with tier, display, parse support |
| `crates/biomeos-core/src/atomic_client.rs` | Added `AtomicClient::http()`, `call_via_http()` (pure Rust HTTP POST), updated `call_impl` dispatch |
| `crates/biomeos-core/src/plasmodium.rs` | Replaced hardcoded 3492 with runtime port discovery, switched to `AtomicClient::http()` |

### Songbird (phase1)

| File | Change |
|------|--------|
| `crates/songbird-universal-ipc/src/handlers/mesh_handler.rs` | Added `jsonrpc_port` to beacon payload, extract port from peer responses |

---

## Validation Results

| Test | Result |
|------|--------|
| `cargo check --workspace` (biomeOS) | PASS |
| `cargo clippy -p biomeos-core` | PASS (0 warnings) |
| `cargo check -p songbird` (full binary) | PASS |
| Cross-machine HTTP JSON-RPC (Tower → gate2:8080) | PASS |
| `biomeos enroll` on both machines | PASS (Blake3-Lineage-KDF) |
| All 5 primals running on gate2 | PASS |
| `mesh.auto_discover` (peer discovery) | FAIL (Issue 3: protocol bug) |
| Covalent bond beacon exchange | BLOCKED (needs discovery) |
| `biomeos plasmodium status` (collective) | BLOCKED (needs mesh.peers) |

## Priority Order for Primal Teams

1. **Songbird Team** (Issues 2, 3, 4): Fix mesh state split, fix UDP discovery protocol, normalize port defaults
2. **biomeOS Team** (Gap A): Validate graph-based NUCLEUS deployment
3. **Squirrel Team** (Gap C): Multi-backend inference routing
4. **All Teams**: Integration test with full covalent bond chain
