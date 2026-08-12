# biomeOS — Current Status

**Updated**: August 12, 2026
**Version**: v4.57 (Wave 157k — Deep Debt Sweep, Routing Refactor, Topology Wiring)
**Posture**: EXEMPLAR. 5-gate gossip mesh LIVE. G72 Tier 1 COMPLETE (11/11, ~155+ crates fleet-wide). Deep debt sweep COMPLETE: routing.rs split (882→682 LOC), topology 4-tier scoring WIRED, Arc::clone normalized, hardcoded primal names eliminated, redundant_clone suppression REMOVED, dep hygiene (tokio scoping, ureq narrowed, dedup serde_json). biomeOS is the lean reference pattern — 0 P0, 0 P1, 0 P2.
**Chain 1**: ALL 5 ITEMS COMPLETE (v4.44–v4.48)

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 2,698+ (biomeos-atomic-deploy: 1606, biomeos-types: 1091) pass, 0 failures |
| Clippy | 0 warnings (pedantic+nursery, --all-targets, zero suppressed lints) |
| Unsafe blocks | 0 (`#![forbid(unsafe_code)]` on all crates) |
| C dependencies | 0 (pure Rust stack: blake3 pure, flate2 rust_backend, rustix zero-libc) |
| Largest prod file | 682 LOC (routing.rs — after orchestration extraction) |
| TODOs in prod | 0 |
| Production unwraps | 0 (workspace lint enforced) |
| Dead code | 0 (all staged code has `#[expect(dead_code, reason)]`) |
| Dead dependencies | 0 |
| Formatting | PASS (rustfmt clean) |
| Cross-arch | x86_64-unknown-linux-musl + x86_64-pc-windows-gnu + aarch64 (partial) |
| Workspace crates | 26 |
| Signal graphs | 31 (+ 47 deploy graphs = 78 total) |
| Capability domains | 27 (320+ translations) |
| Mocks in prod | 0 (all test-only) |
| panic! in prod | 0 |
| Hardcoded primal names in prod | 0 (all use primal_names constants) |
| Tokio "full" in prod | 0 (minimal features: rt-multi-thread, macros, sync, time) |
| Edition | 2024 |
| Rust version | 1.87 |

---

## Architecture

| Component | Status |
|-----------|--------|
| Neural API | Production — capability routing, BTSP, riboCipher, connection pooling |
| NUCLEUS Mode | Production — single-process: Neural API (JSON-RPC) + HTTP API (axum) |
| Universal IPC v3.0 | Unix + Abstract + TCP + tarpc binary escalation (C2 dual-socket + **G65 negotiation**) |
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
| **G64 C2 dual-socket (tarpc sidecar)** | v4.57 | `f29c38bb` |
| **tcp_only deprecated (atomic composition transport)** | v4.57 | Wave 156j |
| **Hot-path Arc\<str\> (DashMap key optimization)** | v4.57 | Wave 156j |
| **Flaky test fix (env-isolated discovery tests)** | v4.57 | Wave 156j |
| **G65 protocol negotiation (single-socket, 10 tests)** | v4.57 | Wave 156m |
| **Category shadow fix (translation→socket fallback)** | v4.57 | Wave 157i |
| **Composition lifecycle (deploy→gossip→verify pipeline)** | v4.57 | Wave 157i |
| **Composition.orchestrate (multi-tier sequencing)** | v4.57 | Wave 157a |
| **P2 FIX: skunkBat spawn leak (rapid-restart detection)** | v4.57 | Wave 157k |
| **Deep debt sweep (routing split, topology 4-tier, Arc::clone)** | v4.57 | Wave 157k |

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

biomeOS is in **STAGE 2 NEURAL API ACTIVATION** (G67). G64+G65+G66 COMPLETE (cephalization trilogy). G22 COMPLETE. G72 Tier 1 CLEAN. 5-gate gossip mesh ACTIVE. Category shadow FIXED.

**Wave 157k — POST-PANDEMIC EVOLUTION (Aug 12, 2026)**:
- **P2 FIX: skunkBat spawn leak** — rapid-restart detection in resurrection path. `last_resurrection_at` timestamp prevents spawn storms by carrying forward cumulative resurrection count when a primal crashes within 120s of its last resurrection. Previously, each Degraded transition reset `resurrection_attempts: 0`, allowing infinite spawn loops (~256/10h).
- **Pre-existing fix: `translations_with_prefix`** — missing method on `CapabilityTranslationRegistry` (used by nest_atomic handler) added.

**Wave 157i — POST-PANDEMIC CASCADE (Aug 11, 2026)**:
- **Composition lifecycle (deploy→register→gossip→verify)**: `composition.orchestrate` now wires the full atomic lifecycle — post-deploy gossip advertisement via swarmVine + composition validation via primalSpring. Both steps are best-effort/graceful (no hard failure when swarmVine or primalSpring unavailable).
- **Signal graph**: `graphs/signals/composition_lifecycle.toml` — formalizes the deploy→register→gossip→verify pipeline as a reusable signal graph for primalSpring integration testing.
- **Category shadow fix**: Translation registry now self-sufficient — explicit TOML translations (braid.verify, braid.list, etc.) route correctly without needing category registration in the capability router
- **G72 Tier 1 VERIFIED**: biomeOS already minimal Tokio, 0 dead deps, pure Rust stack
- **5-gate gossip mesh**: eastGate, sporeGate, strandGate, westGate, ironGate all ACTIVE
- **graftGate**: 15/15 primals compiled on aarch64-apple-darwin, WG LIVE at 10.13.37.13

**G67 Neural API Activation — Stage 2 (Wave 157a)**:
- **N1 DONE**: Forwarding fix — pool path for `capability.call`, outer timeout for escalation (`ffed2c5b`)
- **N2-N6 PENDING**: Live activation (primalSpring team)
- riboCipher-aware connection pooling SHIPPED (dual pool lanes: plain + `[0xEC,0x01]` prefixed)
- Bootstrap→Coordinated auto-transition watcher (15s probe, max 10min)
- TOML-driven capability translations (`config/capability_registry.toml` → compiled fallback)
- Capability-first security resolution (env priority: `BIOMEOS_SECURITY_SOCKET` > `SECURITY_PROVIDER_SOCKET` > legacy)

**G66 Transport Abstraction — COMPLETE (Wave 156z)**:
- All 15 primals: `TransportEndpoint`/`TransportStream`/`connect_transport`
- Silicon-agnostic IPC: Unix + Abstract + TCP + Named Pipes (Windows)
- Cross-arch 15/15: x86_64-linux, aarch64-linux, x86_64-windows-gnu

**G65 Protocol Negotiation — COMPLETE (Wave 156m)**:
- Single-socket dual-protocol (tarpc + JSON-RPC) — eliminates socket proliferation

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

**G65 Protocol Negotiation (Wave 156m — SHIPPED)**:
- **Status**: SHIPPED — biomeOS implements G65 independently (no shared crate — primal violation per 156m)
- G65 spec: client sends `PROTOCOLS: tarpc,jsonrpc\n`, server selects best match
- No negotiation = JSON-RPC (backward-compatible)
- Eliminates socket proliferation (dual-socket → single-socket)
- Implementation: `protocol_negotiation.rs` (server + client + 10 tests)
- Integration: `handle_connection` + `handle_with_negotiation` in `connection.rs`
- tarpc path: `handle_tarpc_stream` serves HealthRpc via LengthDelimited + Bincode on negotiated stream
- Backward-compatible: non-negotiated connections fall through to JSON-RPC (existing behavior preserved)

Upstream items (not biomeOS code):
- ~~C1: tarpc 0.34 → 0.37 for songBird + petalTongue~~ → **RESOLVED** (all 15 primals on 0.37)
- **C7: G65 protocol negotiation** (each primal implements independently — no shared crate)
- C3: coralReef JSON-RPC health shim (nestgate.io 13/13)
- C4: toadStool deploy restart on sporeGate
- ~~C5: rustChip → Forgejo~~ → **RESOLVED**
- O5: nestGate TCP on westGate
- O7: Inter-gate `content.get` E2E (nestGate + songBird operational)

Resume triggers:
- ~~G64 Phase 2: biomeOS serves own `.tarpc.sock`~~ → **DONE** (`f29c38bb`)
- ~~**G65 Phase 3**: Implement protocol negotiation~~ → **DONE** (protocol_negotiation.rs + connection.rs integration)
- ~~**G67 N1**: Forwarding fix + riboCipher pool~~ → **DONE** (Wave 157a)
- **G67 N2-N6**: Stage 2 live activation (primalSpring validates)
- D1: `biomeos nucleus attach` for tideGlass on westGate
- E2: squirrel systemd on ironGate (agent panel)
- O7: Inter-gate `content.get` E2E (operational, not code)
- G64 Phase 3 (cleanup): Remove deprecated tcp_only once all gates use Dual mode
- Redeploy v4.57+ to depot via Sovereign CI
