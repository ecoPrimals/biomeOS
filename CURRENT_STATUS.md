# biomeOS — Current Status

**Updated**: August 4, 2026
**Version**: v4.57 (Wave 156j — Cephalization Advancing)
**Posture**: CEPHALIZATION ERA — ZERO P0/P1/P2. G64 C2 dual-socket DONE. ironGate Phase 1 OPERATIONALLY READY.
**Chain 1**: ALL 5 ITEMS COMPLETE (v4.44–v4.48)

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 8,578+ pass, 0 failures |
| Line Coverage | 88.37% (llvm-cov) |
| Clippy | 0 warnings (pedantic+nursery, --tests, -D warnings) |
| Unsafe blocks | 0 (`#![forbid(unsafe_code)]` on all 26 crates) |
| C dependencies | 0 (pure Rust stack, deny.toml enforced) |
| Largest prod file | 716 LOC |
| TODOs in prod | 0 |
| Production unwraps | 0 (workspace lint enforced) |
| Dead code | 0 |
| Dead dependencies | 0 (47 removed across sessions, cargo-machete verified) |
| cargo deny | clean (advisories, bans, licenses, sources) |
| Formatting | PASS (rustfmt clean) |
| Cross-arch | x86_64 + aarch64 + armv7 + x86_64-pc-windows-gnu |
| Workspace crates | 26 |
| Signal graphs | 30 |
| Capability domains | 27 (320+ translations) |
| Mocks in prod | 0 |
| panic! in prod | 0 (all in test modules) |
| Hardcoded primal names in prod | 0 (all capability-based) |
| Hardcoded primal names | 0 |
| Socket namespace refs (stale) | 0 (unified to membrane/) |

---

## Architecture

| Component | Status |
|-----------|--------|
| Neural API | Production — capability routing, BTSP, riboCipher, connection pooling |
| NUCLEUS Mode | Production — single-process: Neural API (JSON-RPC) + HTTP API (axum) |
| Universal IPC v3.0 | Unix + Abstract + TCP + tarpc binary escalation |
| BTSP Security | Phase 2+3 — Ed25519 handshake, ChaCha20-Poly1305 framing |
| Dark Forest Gate | HTTP sovereign mode — X-Dark-Forest-Token |
| Capability Registry | Runtime DashMap + redb persistence + 3-strike prune |
| Composition Broker | E2E validated — BTSP propagation through signal graph |
| Plasmodium | Remote compute discovery + workload dispatch |
| Lifecycle Manager | Auto-resurrection, boot_order shutdown, binary path retention |
| Socket Discovery | XDG `membrane/` standard, lazy rescan, family-scoped |
| Socket Ownership | `MEMBRANE_SOCKET_GROUP` env, chown :membrane post-bind |
| Health Ping | Dual-protocol: plain JSON-RPC first, BTSP fallback |
| Coevolution Contract | `composition.test_swap` — live validation of replacement binaries |
| **G22 Convergence** | **COMPLETE**: ALL modes unified (api, neural-api, nucleus) serve both protocols |

---

## Key Deliverables (Wave 155i–155n)

| Delivery | Version | Commit |
|----------|---------|--------|
| Graph executor riboCipher fix | v4.46 | `bd202674` |
| BTSP composition broker E2E | v4.44 | composition broker |
| Composition lifecycle (boot_order) | v4.48 | `076d4743` |
| Socket evaporation fix (persistence) | v4.46 | `bd202674` |
| Socket path unification (membrane/) | v4.46 | `bd202674` |
| Capability wipe cycle fix (3-strike) | v4.49 | `f2d4c4b3` |
| Test extraction (8 files refactored) | v4.49 | `7ae18493` |
| Dependency narrowing (futures→futures-util) | v4.49 | `80e79600` |
| Socket evaporation fix (RPC ping tolerance) | v4.50 | Wave 155m |
| Binary path retention (auto-discovery) | v4.50 | Wave 155m |
| Socket ownership (multi-user chown) | v4.51 | `0e45262f` |
| Dep pruning (14 unused deps removed) | v4.51 | `c6f83a73` |
| Registry alloc optimization (Arc reuse) | v4.51 | `744b2d17` |
| User-space binary discovery (P2 final) | v4.52 | `999044e7` |
| Permission reset fix (P3) | v4.53 | Wave 155m |
| composition.self_test sandbox endpoint (P3) | v4.53 | `c7bc2187` |
| Dep pruning round 2 (15 more dead deps) | v4.53 | `5d9374b6` |
| **P1 FIX: Respawn storm (dual-protocol health)** | v4.54 | `88785daf` |
| **P1 FIX: Socket file deletion (ownership guard)** | v4.54 | `88785daf` |
| P3 FIX: Zombie reaping (child.wait) | v4.54 | `88785daf` |
| P3 FIX: Virtual service resurrection skip | v4.54 | `88785daf` |
| P3 FIX: graphs_dir XDG fallback | v4.54 | `88785daf` |
| P3 FIX: riboCipher log level ERROR→debug | v4.54 | `88785daf` |
| P3 FIX: --version 4.54.0 (workspace synced) | v4.54 | `88785daf` |
| **P2 UNBLOCK: composition.test_swap (coevolution)** | v4.55 | Wave 155n |
| **MODE GAP: btsp_optional plain JSON-RPC** | v4.55 | `652cf8a7` |
| **G22 Step 1: NUCLEUS dual-server (HTTP+JSON-RPC)** | v4.56 | `4b48b83b` |
| **G22 Step 2: Socket namespace unified (membrane/)** | v4.56 | `bd33e17d` |
| **G22 Steps 3-5: All modes unified, neural-api deprecated** | v4.56 | `b82f0925` |
| Dep pruning round 3+4 (13 more dead deps, 47 total) | v4.56 | `4b48b83b`+`6a698078` |
| **Cell attach CLI (`biomeos nucleus attach`)** | v4.57 | `9fcca6b8` |

---

## Dependency Stack (Pure Rust)

Core: `tokio`, `serde`, `anyhow`, `thiserror`, `tracing`
Crypto: `ed25519-dalek`, `chacha20poly1305`, `blake3`, `sha2`, `hkdf`
IPC: `axum` (HTTP), `hyper` (low-level), `tokio-tungstenite` (WS)
Storage: `redb` (routing weights), `dashmap` (capability registry)
System: `rustix` (syscalls), `rtnetlink` (Linux netlink)
Config: `toml`, `serde-saphyr` (YAML), `clap`

---

## Posture

biomeOS is in the **CEPHALIZATION ERA** (G64). G22 COMPLETE. ironGate Phase 1 OPERATIONALLY READY.

**G64 Cephalization — biomeOS C2 Dual-Socket DONE (Wave 156j)**:
- **Status**: tarpc-default + dual-socket (7/15 primals now in this tier)
- biomeOS now serves `.tarpc.sock` sidecar alongside JSON-RPC socket
- `DefaultHealthService` responds to `health_check`, `health_metrics`, `version` via tarpc binary framing
- Neural API router auto-escalates to tarpc when `.tarpc.sock` sibling exists
- `ProtocolPreference` env-configurable (Auto/PreferTarpc/PreferJsonRpc/TarpcOnly/JsonRpcOnly)
- SDK helpers for primals: `biomeos_primal_sdk::tarpc_transport::{serve_tarpc_health, tarpc_socket_path}`
- Service definitions: `HealthRpc`, `DiscoveryRpc`, `SecurityRpc` traits in `biomeos-types`
- Both `run()` (standalone) and `run_with_lifecycle()` (NUCLEUS) spawn tarpc sidecar
- C1 blocker RESOLVED: all 15 primals on tarpc 0.37 (version split eliminated)

**Cell Attach CLI (Aug 4 — Wave 155v/156d)**:
- `biomeos nucleus attach <cell_graph.toml>` — closes the ops gap for live cell boot
- Pre-flight NUCLEUS health validation before graph execution
- Dry-run support (`--dry-run`), explicit socket/family overrides
- 8 unit tests covering parse, error, and connectivity paths

**Cell Deploy Graphs (Aug 4 — Wave 155u/156b)**:
- `esotericwebb_cell.toml`: First-ever live cell composition boot (ironGate Phase 1)
- `footprint_cell.toml`: GIS protist attachment (ironGate Phase 2)
- 3 garden/protist primal name constants (`ESOTERICWEBB`, `FOOTPRINT`, `TIDEGLASS`)
- Bootstrap capability hints for gaming, GIS, pharmacology domains
- 3 data federation signal graphs integrated from westGate upstream

**G22 (whitePaper API convergence) — COMPLETE**:
- All modes unified: single process serves HTTP/WebSocket + JSON-RPC
- Socket namespace unified to `membrane/`
- Standalone `neural-api` deprecated
- Single restart = full composition recovery

**Spring Dispatch Infrastructure (Aug 3)**:
- `action` field normalization, `effective_param()` unified resolution
- Shadow deploy gate validation, spring graphs v2.0.0
- hotSpring + groundSpring bootstrap hints

**Inter-gate content.get** (P1 #4): biomeOS routing infrastructure VERIFIED complete.
Gate param routing, mesh relay, nest.sync signal all functional. Needs live E2E test.

**G18 squirrel dispatch** (P1 #6): biomeOS side ready. Springs executable via signal
graphs. Squirrel needs to wire `signal.plan` → biomeOS `graph.execute`.

Coevolution (G21) COMPLETE. Both P1s GATE VALIDATED on westGate and strandGate.

Upstream items (not biomeOS code):
- C1: tarpc 0.34 → 0.37 for songBird + petalTongue (bincode 2.x migration)
- C3: coralReef JSON-RPC health shim (nestgate.io 13/13)
- C4: toadStool deploy restart on sporeGate
- C5: rustChip → Forgejo (cross-gate toadStool dev)
- O5: nestGate TCP on westGate
- O7: Inter-gate `content.get` E2E (nestGate + songBird operational)

Resume triggers:
- G64 Phase 2: biomeOS serves own `.tarpc.sock` (after vanguard primals deploy)
- D1: `biomeos nucleus attach` for tideGlass on westGate
- E2: squirrel systemd on ironGate (agent panel)
- O7: Inter-gate `content.get` E2E (operational, not code)
- Redeploy v4.57 to depot via Sovereign CI
