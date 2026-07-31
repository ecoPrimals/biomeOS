# biomeOS — Current Status

**Updated**: July 31, 2026
**Version**: v4.55 (Wave 155n — Coevolution Contract)
**Posture**: STANDBY — All biomeOS-owned P0/P1/P2/P3 blockers resolved.
**Chain 1**: ALL 5 ITEMS COMPLETE (v4.44–v4.48)

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 8,570+ pass, 0 failures |
| Line Coverage | 88.37% (llvm-cov) |
| Clippy | 0 warnings (pedantic+nursery, --tests, -D warnings) |
| Unsafe blocks | 0 (`#![forbid(unsafe_code)]` on all 26 crates) |
| C dependencies | 0 (pure Rust stack, deny.toml enforced) |
| Largest prod file | 716 LOC |
| TODOs in prod | 0 |
| Production unwraps | 0 (workspace lint enforced) |
| Dead code | 0 |
| Dead dependencies | 0 (cargo-machete verified) |
| cargo deny | clean (advisories, bans, licenses, sources) |
| Formatting | PASS (rustfmt clean) |
| Cross-arch | x86_64 + aarch64 + armv7 + x86_64-pc-windows-gnu |
| Workspace crates | 26 |
| Signal graphs | 27 |
| Capability domains | 27 (320+ translations) |
| Mocks in prod | 0 |
| Hardcoded primal names | 0 |

---

## Architecture

| Component | Status |
|-----------|--------|
| Neural API | Production — capability routing, BTSP, riboCipher, connection pooling |
| NUCLEUS Mode | Production — manifest-driven launch, boot_order, lifecycle management |
| Universal IPC v3.0 | Unix + Abstract + TCP + tarpc binary escalation |
| BTSP Security | Phase 2+3 — Ed25519 handshake, ChaCha20-Poly1305 framing |
| Dark Forest Gate | HTTP sovereign mode — X-Dark-Forest-Token |
| Capability Registry | Runtime DashMap + redb persistence + 3-strike prune |
| Composition Broker | E2E validated — BTSP propagation through signal graph |
| Plasmodium | Remote compute discovery + workload dispatch |
| Lifecycle Manager | Auto-resurrection, boot_order shutdown, binary path retention |
| Socket Discovery | XDG `membrane/` standard + legacy `biomeos/` compat, lazy rescan, family-scoped |
| Socket Ownership | `MEMBRANE_SOCKET_GROUP` env, chown :membrane post-bind |
| Health Ping | Dual-protocol: plain JSON-RPC first, BTSP fallback |
| **Coevolution Contract** | `composition.test_swap` — live validation of replacement binaries |

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
| **P2 UNBLOCK: composition.test_swap (coevolution contract)** | v4.55 | Wave 155n |
| **MODE GAP: Neural API accepts plain JSON-RPC (btsp_optional)** | v4.55 | `652cf8a7` |

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

biomeOS is **STANDBY-READY**. All P0/P1/P2/P3 biomeOS-owned blockers resolved.

Key fix (v4.55): **Coevolution contract** — `composition.test_swap` allows the running
biomeOS instance to validate its own replacement binary. cellMembrane delegates validation
to the live orchestrator instead of using an isolated sandbox (which fails for broker
primals). This unblocks the sandbox P2 (J19) and enables zero-downtime deploys.

Mode gap closed: Neural API now accepts plain JSON-RPC connections when btsp_optional
is set (UDS = local trust boundary). NUCLEUS + standalone server both start permissive.

Key fix (v4.54): Health monitor speaks **dual protocol** — plain JSON-RPC first, BTSP
fallback — eliminating respawn storm. Socket deletion guarded by PID ownership.

Upstream items (not biomeOS code):
- `membrane/` vs `biomeos/` socket dir: biomeOS scans both; primals need standardization from cellMembrane
- GNU depot incomplete (4/16): sporeGate builder, not biomeOS
- cellMembrane not in sources.toml: blocks sovereign CI self-rebuild

Resume triggers:
- Redeploy biomeOS v4.55 to depot via Sovereign CI (mode gap resolved)
- Redeploy to westGate + strandGate (validates P1+P2 fix live)
- AlphaFold ~1TB ingestion through westGate Nest Atomic
- steamGate Tower deployment (user-space, gnu bins)
