# biomeOS - Current Status

**Updated**: March 31, 2026 (v2.81: `#[serial]` elimination, BM-04/05 wired, TCP-only CLI, gate routing, `mem::forget` elimination, hardcoded UID/primal evolution, probe timeout centralization)
**Version**: 2.81
**Status**: PRODUCTION READY - Multi-Computer Federation Validated - Zero Blocking Debt - Fully Concurrent Testing - BM-04/BM-05 Resolved

---

## Quick Summary

| Metric | Status |
|--------|--------|
| **genomeBins** | 7/7 components ready (7 primals + orchestrator; barraCuda + coralReef added) |
| **Cross-Arch** | x86_64 + aarch64 (USB + Pixel) |
| **IPC Standard** | Universal IPC v3.0 + HTTP JSON-RPC (inter-gate) |
| **Security Grade** | A++ (TRUE PRIMAL + Security Headers + Dark Forest Gate) |
| **Security Score** | 100/100 (HSTS, X-Frame, CSP, Referrer-Policy, Cache-Control) |
| **Code Quality** | A++ (Pure Rust, Edition 2024 all crates, ecoBin v3.0, fully concurrent, zero warnings, full doc coverage, sovereignty audit) |
| **Lint hardening** | `deny` on unwrap_used/expect_used, workspace lints inherited by all 26 workspace crates |
| **Tests Passing** | 7,212 lib + bin + doc + proptest (0 failures, fully concurrent `RUST_TEST_THREADS=16`) |
| **Test Coverage** | 90%+ (llvm-cov workspace-wide verified) — all three metrics above 90% target |
| **Unsafe Code** | 0 production (`#[forbid(unsafe_code)]` on all crate roots, `mem::forget` eliminated) |
| **Clippy** | PASS (0 warnings, pedantic+nursery, `-D warnings`, all crates via `[lints] workspace = true`) |
| **Formatting** | PASS (rustfmt.toml enforced, `cargo fmt --check` clean) |
| **C dependencies** | 0 (zstd-sys eliminated → lz4_flex, deny.toml enforced) |
| **Continuous Systems** | ContinuousExecutor (60Hz tick), GraphEventBroadcaster, SensorEventBus |
| **XR/VR Types** | StereoConfig, Pose6DoF, TrackingFrame, HapticCommand, MotionCaptureAdapter |
| **Surgical Domain** | SurgicalProcedure, TissueMaterial, AnatomyModel, PkModelParams |
| **Capability Domains** | 26 domains (+ health cross-cutting, genetic/lineage added), 290+ translations |
| **Deploy Graphs** | 40 (+ 2 Pipeline coordination graphs, all parseable via unified schema) |
| **Niche Templates** | 20 (+ rootpulse-branch, rootpulse-merge, rootpulse-diff, rootpulse-federate, soil-microbiome) |
| **Genetic Model** | EVOLVED - Mitochondrial + Nuclear DNA |
| **BirdSong Discovery** | Encrypted, shared beacon model |
| **Discovery Model** | 5-tier capability-first protocol (centralized) + taxonomy + manifest fallback |
| **NAT Traversal** | 4-tier strategy (LAN/punch/coordinated/relay) |
| **P2P Sovereign Onion** | PRODUCTION READY |
| **Deep Debt Resolution + Standards Compliance (Mar 28)** | `CONTEXT.md` created (PUBLIC_SURFACE_STANDARD compliance), README "Part of ecoPrimals" footer added, version footer v2.68→v2.70 reconciled, `forwarding.rs` split from 1001→357 LOC (integration tests extracted to `forwarding_routing_tests.rs`), `#[allow(clippy::cast_possible_wrap)]` → `#[expect(..., reason)]` in tower_orchestration.rs, `deployment_graph.rs` `to_toml()` stub evolved to real TOML serialization via `toml::to_string_pretty`, chimera builder codegen evolved from stub error → capability-based IPC forwarding pattern (+ `FusionEndpoint.capability` field), `generate_api_endpoints()` extracted for clippy `too_many_lines` compliance, 2 new tests (to_toml roundtrip, node structure preservation), full audit confirmed: 0 mocks in production (all `#[cfg(test)]`), 0 `.unwrap()` in production types, 0 files >1000 LOC, 0 C-sys deps (only `linux-raw-sys`/`netlink-sys` kernel interfaces), 0 TODO/FIXME/HACK |
| **Multi-Transport IPC Evolution (Mar 28)** | Neural router evolved from Unix-socket-only to universal transport: `RegisteredCapability.socket_path: PathBuf` → `RegisteredCapability.endpoint: TransportEndpoint`, `DiscoveredPrimal.socket_path` → `DiscoveredPrimal.endpoint`, `DiscoveredAtomic.primary_socket` → `DiscoveredAtomic.primary_endpoint`, `forward_request` routes via `AtomicClient::from_endpoint()` (Unix/abstract/TCP/HTTP), health checks evolved from `Path::exists()` + manual `UnixStream` to `AtomicClient`-based transport-aware probing, `capability.register` JSON-RPC handler parses transport strings (`@abstract`, `tcp://`, `http://`, `/path.sock`), `TransportEndpoint` gains `Serialize`/`Deserialize` (tagged JSON: `{"transport":"TcpSocket","address":{"host":"...","port":9001}}`), `register_capability_unix()` convenience method for backward compat, 5 new tests (TCP/abstract/HTTP endpoint registration, primal label extraction, TCP tarpc policy), all primalSpring P0+P1 gaps resolved (cross-gate routing foundation complete) |
| **Zero-Copy + Dep Governance (Mar 28)** | `Value::take()` zero-copy evolution on Songbird discovery + provider hot paths (eliminates JSON subtree duplication), tokio workspace unification (11 crates: biomeos-types, biomeos-system, neural-api-client, biomeos-api, biomeos-deploy, biomeos-cli, biomeos-boot, biomeos-atomic-deploy, biomeos-ui, root biomeos deps+dev-deps), base64 0.21→0.22 unified, `deny.toml` cleaned (MPL-2.0/Unicode-DFS-2016/Zlib unused allowances removed), 25 new tests (vm_federation: benchscale_create_argv, subcommand_argv, topology_path_for_cli, validate_ssh_probe success+failure, collect_ips_for_vm_names mock/error/empty, wait_for_vm_ssh_ready success+retry-exceeded; trust: all-variant serde, copy semantics, comprehensive ord; constants: 14 env-driven port/bind/path tests), SPDX headers on 7 test modules, 2 rustdoc warnings fixed, test port `ports::TEST_DEFAULT` + `endpoints::production_bind_address()` centralized |
| **Deep Debt Session (Mar 18)** | Full audit execution: 18 crates migrated to Edition 2024, tarpc sidecar wired, Google/Cloudflare STUN removed (sovereignty), zero-copy fixes, 39 new tests, workspace lint inheritance for all 26 crates, scyBorg license trio (ORC + CC-BY-SA), large files refactored (963→835/899), capability-based discovery evolution |
| **Ecosystem Absorption (Mar 18)** | IpcErrorPhase + extract_rpc_result (5+ springs), OrExit trait (groundSpring/loamSpine), cast module (airSpring), proptest IPC fuzzing (8 fuzz tests), capability.list cost_estimates+operation_dependencies (Squirrel Pathway Learner), socket-registry.json discovery (Squirrel), MCP tool definitions (healthSpring/airSpring/wetSpring), ValidationSink (rhizoCrypt/airSpring), primal_names::display (neuralSpring), primal capability routing types (relay.authorize, compute.dispatch, model.*, sourDough lifecycle), deny.toml expanded to 15 C-dep bans |
| **Deep Debt Audit (Mar 20)** | Zero-copy `JsonRpcVersion` marker type (eliminates String alloc per request/response), 5 production files >1000 LOC refactored into submodules (nucleus/client, plasmodium, fossil, monitor, rendering), `#[allow]`→`#[expect(reason)]` migration across workspace, BUILD_TIMESTAMP evolved from hardcoded placeholder to `build.rs`-injected, flaky tests fixed (beardog mock flush+shutdown, spore CWD→env-based `discover_plasmid_dir()`), SPDX header gap closed (692/692), deprecated `capability_from_primal_name`→`bootstrap_capability_hint_for_primal_name`, dead_code→`#[cfg(test)]` |
| **Deep Resilience (Mar 20)** | TOCTOU fix in federation `discover_unix_sockets()` (non-fatal `read_dir`), `SocketNucleation::assign_socket()` ensures parent dir exists, 10 fossil tests serialized (`#[serial]`), 4 large test modules extracted to files (capabilities 946→377, handlers/discovery 908→293, vm_federation 929→470, UBM/discovery 923→462), orphan `biomeos-genome-extract` crate removed, `neural-api-client` identified as non-workspace dep (used by biomeos-api) |
| **Deep Debt Evolution (Mar 20b)** | `capability_taxonomy/helpers.rs` evolved: hardcoded `match primal_name` → taxonomy-driven `representative_for_category()` + `default_primal_with()` resolution (zero hardcoded primal names in helper), `fossil/tests.rs` split into `format_tests.rs` (pure logic) + `integration_tests.rs` (serial/env) with thin index module (1006→30 lines), `env_helpers.rs` hardened with `static ENV_MUTEX` serializing all env mutations (unsafe still required by Rust 2024 but now mutex-protected), `realtime_tests.rs` expanded with 8 new tests (SSE derivation, event variant coverage, multi-type handler), `#[allow]`→`#[expect(reason)]` consistency pass, `Copyright 2025`→`Copyright 2025-2026` across all 692 .rs files, zero-copy audit confirmed idiomatic clone patterns in circuit-breaker closures and graph algorithms, mock audit confirmed zero production mocks (all `MockDiscovery`/`spawn_*_mock` strictly `#[cfg(test)]`) |
| **Deep Audit v2.61 (Mar 21)** | Comprehensive codebase audit against wateringHole standards: `serde_yaml`→`serde_yml` (deprecated dep evolved, 9 Cargo.toml updated via package rename, zero source changes), 3 files >1000 LOC refactored (metrics.rs 1056→metrics/mod.rs 509 + metrics/tests.rs 548, lib.rs 1055→lib.rs 424 + lib_tests.rs 596, websocket.rs 1038→websocket.rs 411 + websocket_tests.rs 673), federation `query_primal_info` hardened with flush+shutdown+BufReader (fixes flaky `test_discover_unix_socket_mock_primal_jsonrpc`), `handle_websocket` decomposed via `dispatch_ws_method` (eliminates `#[allow(clippy::too_many_lines)]`), `create_app_with_transport` decomposed into `register_api_routes` + `apply_security_headers`, zero-copy: WebSocket subscription IDs→`Arc<str>`, filters→`Arc<SubscriptionFilter>`, unused imports cleaned (verify_lineage.rs), `stable_sort_primitive` lint fixed (discovery/tests.rs), unix_socket_client tests hardened (expect→unwrap under `#[expect]`), realtime_tests.rs Mutex drop ordering fixed |
| **Coverage Push v2.62 (Mar 21)** | 80+ new tests across 15 files pushing all three coverage metrics above 90% target: neural-api-client-sync (full socket round-trip + `resolve_socket_with` tiers + `parse_response` edge cases), model_cache (`show_status_with` mesh/HF branches, `resolve_model_with` Local/Remote, `import_huggingface_with`), checks_config (`check_binary_health_inner` extraction + error paths), realtime (`process_events` channel-close, `parse_event`/`parse_sse_event` edge cases), verify_lineage (missing path, file-not-directory, invalid UTF-8, empty primals, empty directory warnings), haptic_feedback/motion_capture/xr_rendering (discovery, command dispatch, calibration, session lifecycle), action_handler (assignment fallback, refresh sources, assign-device flow, Squirrel accept/dismiss), device_management discovery/provider (human_size, statvfs, resolve_provider, validate_niche), suggestions/manager (`probe_ai_capability` mock socket tests replacing flaky env-var tests), rendezvous (post_beacon/check_peer success via Neural API mock), beacon_genetics (default lineage, bad seed_hex, short peer_beacon_id, missing crypto mock), manifest (validate_service, ManifestAnalyzer), forwarding (`parse_security_bytes_param` branches); discovery env var race eliminated: `discover_unix_sockets` refactored to `discover_unix_sockets_in(path)` (test no longer depends on `XDG_RUNTIME_DIR`) |
| **Coverage Push (Mar 20)** | 6 large test files (1039–1309 LOC) split into domain submodules, `tui/types.rs` split into types/ submodules, 3 remaining `RestoreCwd` patterns evolved to env-based discovery (verify.rs, niche.rs, chimera.rs with `BIOMEOS_NICHE_TEMPLATES_DIR`, `BIOMEOS_CHIMERA_DEFINITIONS_DIR`, etc.), all beardog/federation mock tests hardened against timing races (case-insensitive error matching, flush+shutdown), health.rs/spore.rs test extraction, ~600 new test lines across vm_federation, neural_executor, graph handlers, capability_registry, beacon_verification, family_credentials, deployment_mode, socket discovery, model cache, fossil, monitor, network; coverage pushed from ~89% to 90.01% line / 90.95% function (llvm-cov verified, v2.60) |
| **Capability-First Discovery (Mar 18)** | Capability-named sockets (security.sock, compute.sock), `mcp.tools.list` aggregation (Squirrel alpha.13), Provenance metadata type (primalSpring v0.3.0), capability_registry.toml sync tests, 3 new primals registered (petalTongue, skunkBat, sourDough) |
| **External C deps** | 0 (nix→rustix, sysinfo→/proc, libc removed, dirs→etcetera, sudo ip→rtnetlink) |
| **ecoBin v3.0** | COMPLIANT (pure Rust: rustix for POSIX, /proc for metrics, rtnetlink for networking, zero -sys crates, zero shell-outs) |
| **Capability constants** | `capability` module: CRYPTO, MESH_NETWORKING, TLS, STORAGE, GATEWAY, NAT_TRAVERSAL, etc. |
| **Files >1000 LOC** | 0 (all production AND test files under 1000 lines, max 949); metrics.rs→metrics/, lib.rs→lib+lib_tests, websocket.rs→websocket+websocket_tests, plus earlier splits: nucleus/client, plasmodium, fossil, monitor, rendering, health, spore, all 6 large test files, tui/types, fossil/tests |
| **JSON-RPC types** | `JSONRPC_VERSION` const + zero-alloc `JsonRpcVersion` marker type (was `String`), `JsonRpcRequest::new()` builder everywhere, `JsonRpcResponse::success()`/`error()` builders |
| **Zero-copy** | `JsonRpcVersion` (zero-size, zero-alloc serde), `bytes::Bytes` for binary payloads (`SecurityRpc`, P2P, compute, genomeBin, HTTP client, primal SDK IPC); `Arc<str>` for identifiers + `PrimalManifest` + `PrimalConnections` keys + `OptimizationType` graph nodes + WebSocket subscription IDs; `Arc<SubscriptionFilter>` for subscriptions; `Value::take()` on Songbird discovery + provider hot paths (eliminates subtree clone); `TransportEndpoint` (tagged enum, zero `PathBuf` allocation for abstract/TCP/HTTP transports) |
| **Safe casts** | 0 truncation `as` casts — PID casts use `i32::try_from().unwrap_or(-1)`, duration use `u32::try_from().unwrap_or(MAX)` |
| **Dep policy** | `deny.toml` (cargo-deny 0.19) bans openssl-sys, ring, aws-lc-sys, native-tls, zstd-sys, dirs-sys; `serde_yaml`→`serde_yml` (deprecated dep evolved via Cargo package rename) |
| **Plasmodium** | HTTP JSON-RPC collective (runtime port, SSH legacy removed) |
| **Model Cache** | NUCLEUS-integrated, HuggingFace import, NestGate fallback |
| **AI Bridge** | Squirrel -> Songbird -> Cloud/Local AI (validated) |
| **Neural API** | 290+ capability translations, JSON-RPC 2.0 batch + notifications, runtime TOML registry, proxy_http, capability.call, graph.start_continuous, graph.execute_pipeline, graph.suggest_optimizations, circuit-breaker protected RPC |
| **Lifecycle** | Deep health monitoring, auto-resurrection, coordinated shutdown |
| **SystemPaths** | All paths XDG-compliant via centralized `SystemPaths` (production `/tmp/` eliminated) |
| **Hardcoded `/tmp`** | 0 in production code (rootpulse, neural_api, continuous, enroll evolved to SystemPaths) |
| **Hardcoded Primals** | 0 in routing code (all via capability-based discovery + `DISCOVERY_PROVIDER`/`SECURITY_PROVIDER` env) |
| **Hardcoded plasmidBin** | 0 (evolved to `discover_plasmid_dir()` with `BIOMEOS_PLASMID_DIR` env override) |
| **Hardcoded user paths** | 0 (tools evolved to runtime workspace discovery) |
| **Production unwrap()** | 0 (all replaced with `expect()` + context) |
| **Federation** | api.nestgate.io via Cloudflare Tunnel (QUIC, 4x HA) |
| **External Access** | LAN + Cloudflare (ISP-invisible, Tor-blocked workaround) |

---

## Validated Systems (Feb 12, 2026)

### 0. Federation - External Access via Cloudflare Tunnel (NEW)

Permanent tunnel established for external beacon rendezvous:

| Component | Status |
|-----------|--------|
| Tunnel ID | `ea845ed5-3722-4473-8344-79a4f3757c7b` |
| Endpoint | `https://api.nestgate.io` |
| Protocol | QUIC (4x HA connections) |
| Latency | ~160ms (Cloudflare edge) |
| LAN Access | `http://192.168.1.144:3492` (direct) |
| Pixel Hotspot | ✅ Reachable via Cloudflare |
| Security Audit | 100/100 (0 metadata leaks) |

**Traffic Flow:**
```
Pixel (any network) → HTTPS → Cloudflare → QUIC Tunnel → Tower:3492
                               ↑
                         ISP sees normal HTTPS
                         (cannot block/sniff)
```

**Security Headers (all responses):**
- `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`
- `Referrer-Policy: no-referrer`
- `Cache-Control: no-store, no-cache, must-revalidate`

---

### 0.1 Pixel Hotspot ↔ LAN Transition (VALIDATED)

Dynamic address book synchronization tested:

| Network | Pixel IP | Access Method | Status |
|---------|----------|---------------|--------|
| **Hotspot** | 172.20.10.x | `api.nestgate.io` (Cloudflare) | ✅ Validated |
| **Home WiFi** | 192.168.1.114 | Direct LAN HTTP | ✅ Validated |

**Transition Flow:**
```
1. Pixel on hotspot → uses api.nestgate.io → beacon exchange ✅
2. Pixel switches to home WiFi → detects new IP (192.168.1.114)
3. Address book updated via NestGate storage → ✅
4. Direct LAN HTTP test → 0% packet loss, 141ms latency
5. Bidirectional beacon exchange → family verified ✅
```

**Validated Operations:**
- Tower → Pixel (LAN): HTTP JSON-RPC, beacon encrypt/decrypt
- Pixel → Tower (LAN): HTTP 200 OK with security headers
- Address book persistence: `storage.store`/`retrieve` via NestGate
- Lineage verification: BirdSong family ID match

---

### 0.2 NUC Federation - Multi-Computer NUCLEUS (NEW Feb 13)

First multi-computer federated cluster established:

| Node | IP | Gen | Role | Primals |
|------|----|-----|------|---------|
| **Tower** | 192.168.1.144 | 0 | Parent/Orchestrator | biomeos-api |
| **NUC** | 192.168.1.190 | 2 | Gate/Compute | All 5 primals |

**NUC Hardware:**
- CPU: Ryzen 5 6600H (6 cores)
- RAM: 28GB
- OS: Pop!_OS 22.04
- Deployment: SSH + LiveSpore

**Verified Primals on NUC:**
| Atomic | Primal | Version | Status |
|--------|--------|---------|--------|
| Tower | BearDog | 0.9.0 | ✅ healthy |
| Tower | Songbird | 0.1.0 | ✅ healthy |
| Node | Toadstool | 0.1.0 | ✅ working |
| Node | Squirrel | 0.1.0 | ✅ working |
| Nest | NestGate | 2.1.0 | ✅ healthy |

**Cross-Node Communication:**
```bash
# Tower → NUC (via SSH tunnel)
ssh nuc 'echo "{...}" | nc -U /run/user/1000/biomeos/beardog-8ff3b864a4bc589a.sock'
# Result: {"jsonrpc":"2.0","result":{"status":"healthy"},...}
```

**Binary Evolution Discovery:**
During deployment, NestGate segfaulted due to non-PIE musl binary. Fixed by using PIE-enabled build. See `specs/BINARY_BUILD_TARGETS_SPEC.md` for details.

---

## Validated Systems (Feb 10, 2026)

### 1. AI Bridge - Local + Cloud AI via Capability Routing

The Squirrel AI bridge works end-to-end through capability-based discovery:

```
Squirrel (query_ai)
  -> discovers http.request via HTTP_REQUEST_PROVIDER_SOCKET env var
  -> Songbird socket (http.request JSON-RPC)
  -> BearDog TLS 1.3 crypto (for HTTPS)
  -> External API (Anthropic Claude, OpenAI GPT)
  -> Response in ~786ms

Songbird (http.request)
  -> HTTP -> localhost:11434 (Ollama)
  -> phi3/tinyllama/llama3.2 inference
  -> Response in ~2s
```

| Chain | Validated | Latency |
|-------|-----------|---------|
| Squirrel -> Anthropic Claude Haiku | Yes | 786ms |
| Neural API proxy_http -> Anthropic | Yes | 756ms |
| Songbird -> Ollama (phi3) | Yes | 2s |
| Songbird -> Ollama (tinyllama) | Yes | 2.4s |

### 2. Covalent Bond Transport - Inter-NUCLEUS HTTP JSON-RPC (NEW)

Cross-machine communication via Songbird HTTP JSON-RPC gateway (no SSH):

| Test | Result |
|------|--------|
| Tower → gate2:8080 `health` | PASS (HTTP POST /jsonrpc) |
| AtomicClient::http() transport | PASS (pure Rust, zero deps) |
| Device enrollment (Blake3-Lineage-KDF) | PASS (both machines) |
| Shared `.family.seed` | PASS (identical on both) |
| Beacon auto-discovery | BLOCKED (Songbird Issue 3) |
| Covalent bond chain | BLOCKED (needs beacon fix) |

### 3. Plasmodium - Distributed Slime Mold Collective

HTTP JSON-RPC collective with runtime port discovery (hardcoded 3492 eliminated):

| Gate | GPU | VRAM | RAM | CPU | Primals |
|------|-----|------|-----|-----|---------|
| Tower | RTX 4070 | 12 GB | 31 GB | 24 | BearDog, Songbird, NestGate, Toadstool, Squirrel, Neural API |
| gate2 | RTX 3090 | 24 GB | 251 GB | 128 | BearDog, Songbird, NestGate, Toadstool, Squirrel |
| **Total** | **2 GPUs** | **36 GB** | **282 GB** | **152** | |

### 4. Nest Atomic - Distributed Storage

| Gate | Backend | Features | Status |
|------|---------|----------|--------|
| Tower | Filesystem | storage.exists/store/retrieve | Validated |
| gate2 | ZFS | + snapshots, dedup, compression | Validated |

### 5. Neural API - Semantic Capability Routing

- 290+ capability translations across 26 domains
- `capability.call` routes semantic names to provider-specific methods
- `proxy_http` delegates HTTPS through Songbird + BearDog TLS
- Capability domains: crypto, security, http, mesh, stun, relay, onion, compute, storage, ai, inference, ephemeral_workspace (rhizoCrypt), permanent_storage (LoamSpine), attribution (sweetGrass), game, medical
- Provenance trio: `dag.*` → rhizoCrypt, `commit.*` → LoamSpine, `provenance.*` → sweetGrass

### 6. Tower Atomic - Crypto + Network Foundation

| Component | Method | Status |
|-----------|--------|--------|
| BearDog | health, crypto.sign, jwt.provision | Validated |
| Songbird | http.request, discovery.peers, relay.*, stun.* | Validated |
| BearDog TLS | HTTPS via Songbird HTTP client | Validated |

### 7. Lifecycle Management

- LifecycleManager tracks primal state: Incubating -> Active -> Degraded -> Stopped
- Deep JSON-RPC health checks for Active primals (not just socket existence)
- Auto-resurrection of degraded primals with exponential backoff
- Coordinated dependency-aware shutdown via `LifecycleManager::shutdown_all()`
- Bootstrap vs. coordinated mode auto-detection on startup

---

## Completed Evolution Items (biomeOS Team)

### Deep Audit + DI Evolution + Cleanup — v2.77 (Mar 28, 2026)

| Category | Change |
|----------|--------|
| **Flaky test fix** | `capability_registry_tests2` socket tests evolved from env-var (`XDG_RUNTIME_DIR`) mutation to `CapabilityRegistry::with_socket_path()` dependency injection — eliminates parallel test races permanently |
| **Commented code cleanup** | Removed legacy `/* ... */` blocks from `universal_biomeos_manager/{ai,runtime,service}.rs` (ToadStool `ClientRegistry` references); git history preserves intent |
| **Cargo.toml hygiene** | Stale `exclude = ["validation"]` → accurate `exclude = ["tools", "tools/harvest"]`; commented showcase members preserved as fossil record |
| **Infallible error handling** | `biomeos-federation` `Capability::from_str` / `from_tags`: `.expect()` → `match never {}` exhaustive match on `Infallible` |
| **Hardcoded primal names** | `trust.rs`, `beardog.rs`, `primal_spawner.rs`, `orchestrator.rs`: string literals → `primal_names::*` constants |
| **Doc-tests** | New doctests on `identifiers.rs`, `error/core.rs`, `paths.rs`, `config/mod.rs`, `transport.rs`, `atomic_client.rs`, `capability.rs` |
| **Deployments doc** | `basement-hpc/README.md`: hardcoded `/home/eastgate/...` → `$BIOMEOS_REPO` |
| **Tests** | 7,209 passing (0 failures), 135 ignored, 0 Clippy warnings |

### Deep Audit + Hardcoding Evolution — v2.68 (Mar 27, 2026)

Comprehensive audit against all wateringHole standards + systematic evolution execution:

| Category | Change |
|----------|--------|
| **Formatting** | `cargo fmt --check` regression fixed (10 diffs across 5 files: `checks_primal.rs`, `server_lifecycle.rs`, `discovery/mod.rs`, `discovery.rs`, `identity.rs`, `capability_discovery.rs`) |
| **Blocking-in-async** | `probe_live_sockets()` evolved from `Handle::block_on` + `std::thread::scope` hack to native `async fn` with `.await` — eliminates potential deadlock in single-threaded runtime |
| **Hardcoded `/tmp`** | 4 production sites centralized: `capability_discovery.rs` tier 4, `tower_orchestration.rs` pid/socket fallbacks, `node_handlers.rs` Neural API fallback, `subfederation/beardog.rs` Neural API fallback → all use `constants::runtime_paths::FALLBACK_RUNTIME_BASE` + `fallback_runtime_dir()` helper |
| **Hardcoded IPs** | 6 production sites evolved: `strategy.rs` TCP fallback (2×), `stun_extension.rs`, `federation/config.rs`, `config/network.rs`, `system/network.rs` → all use `endpoints::DEFAULT_LOCALHOST` / `PRODUCTION_BIND_ADDRESS` constants |
| **New constants** | `biomeos-types::constants::runtime_paths` module: `FALLBACK_RUNTIME_BASE`, `SOCKET_SUBDIR`, `BIOMEOS_SUBDIR`, `fallback_runtime_dir(family_id)` |
| **License** | `LICENSE-CC-BY-SA` reconciled: `AGPL-3.0-or-later` → `AGPL-3.0-only` (matches Cargo.toml + SPDX headers) |
| **llvm-cov** | Stale profdata cleaned (529 spurious warnings from old `phase2/biomeOS/` paths) |
| **Dep audit** | `blake3`+`cc` acceptable (perf-critical genome hashing), `tokio-process` 0.2 legacy identified in `biomeos-deploy`, `bincode` v1 RUSTSEC-2025-0141 documented (blocked by tarpc) |
| **Mock audit** | Zero production mocks confirmed (274 hits all test-gated: `#[cfg(test)]`, `*_tests.rs`, `biomeos-test-utils`) |
| **Tests** | 6 discovery probe tests evolved to `#[tokio::test] async fn` (from sync `#[test]`) |
| **Clippy** | PASS (0 warnings, pedantic+nursery, `-D warnings`, all 26 workspace crates) |
| **Formatting** | PASS (`cargo fmt --check` clean after all changes) |

### Coverage push + flaky/cwd test hardening — v2.55 (Mar 20, 2026)

| Category | Change |
|----------|--------|
| **Region coverage** | 83.84% → 89.07% (+5.23pp, llvm-cov verified) |
| **Function coverage** | 90.21% (over 90% target) |
| **Test count** | 6,169 → 6,760 (+485 new tests in coverage push, all passing) |
| **Flaky test fixes** | Env-var races: `serial_test::serial` + `tokio::sync::Mutex`; "Text file busy" race fixed; hanging pipeline test wrapped with timeout |
| **cwd-sensitive tests** | ~20 marked `#[ignore]` with instructions to run `cargo test --ignored --test-threads=1` |
| **Quality gates** | fmt, clippy (pedantic+nursery, `-D warnings`), doc, cargo-deny — all passing |

### Concurrency Evolution + Coverage Push — v2.54 (Mar 19, 2026)

Deep evolution to fully concurrent, modern idiomatic Rust. Eliminated all test sleeps and serial constraints — test issues are production issues.

| Category | Change |
|----------|--------|
| **Test count** | 5,340 → 6,169 (829 new tests across all crates) |
| **Line coverage** | 78.32% → 83.62% (5,386 more lines covered) |
| **Sleep-before-connect** | Eliminated — all socket tests use `ReadySender`/`ReadyReceiver` (biomeos-test-utils) |
| **Wall-clock sleeps** | Eliminated — `TickClock`, circuit breaker, cooldown, cache TTL all use `tokio::time::Instant` + `start_paused = true` + `advance()` |
| **`#[ignore]` removed** | 10 tests evolved from `#[ignore]` to concurrent — env-var tests use DI overrides (`with_xdg_override`) instead of global mutation |
| **Production sleeps** | All configurable — `DEFAULT_POLL_INTERVAL`, `DEFAULT_RETRY_INTERVAL`, `DEFAULT_SOCKET_POLL_INTERVAL` consts with injectable durations |
| **New test infra** | `biomeos_test_utils::ready_signal` (oneshot readiness), `MockJsonRpcServer` (zero-sleep mock), `server_ready_signal` convenience |
| **Env var safety** | Combined concurrent-hostile env tests into single test with `TestEnvGuard` (RAII restoration) |
| **Hanging tests** | Pipeline redirect test evolved to use `tokio::time::timeout` |
| **File extraction** | `genome_tests.rs`, `neural_executor_async_tests.rs`, `main_tests.rs` (CLI) extracted — all files under 1000 lines |
| **Time types** | `std::time::Instant` → `tokio::time::Instant` in cache, cooldown, circuit breaker, tick clock, health service |

### Deep Audit Execution — v2.53 (Mar 19, 2026)

Comprehensive audit against all wateringHole standards + systematic evolution execution:

| Category | Change |
|----------|--------|
| **Bypasses resolved** | All 3 active bypasses evolved: HTTP_REQUEST_PROVIDER_SOCKET → capability discovery, NestGate boolean → omit flag, Squirrel model → AI_DEFAULT_MODEL env passthrough |
| **engine.rs refactor** | Registry query logic extracted to `registry_queries.rs` (1023→871 lines, under 1000 limit) |
| **Hardcoded primal names** | Production code evolved to `primal_names::*` constants in engine.rs, paths.rs, provider.rs, discovery.rs |
| **Zero-copy evolution** | primal-sdk IPC: `Vec<u8>` → `bytes::Bytes` for crypto_sign/hash/storage_get; neural_router forwarding → Bytes; `PrimalManifest` primal/socket → `Arc<str>`; `PrimalConnections` keys → `Arc<str>`; `OptimizationType` nodes → `Arc<str>` |
| **Shell-out elimination** | `sudo ip link/addr/set` → `rtnetlink` crate (pure Rust Netlink); 0 remaining shell-outs in production |
| **Unsafe code evolution** | verify_lineage.rs: raw `unsafe { set_var/remove_var }` → `biomeos_test_utils::TestEnvGuard` RAII pattern |
| **Infallible unwrap** | unix_server.rs: `.expect("never Err")` → `match infallible {}` (exhaustive match on Infallible) |
| **forbid(unsafe_code)** | Added to `biomeos-genome-deploy/src/main.rs`; biomeos-test-utils kept at `deny` (has legitimate #[allow] functions) |
| **Doc-tests** | 25 new doc-tests in biomeos-types (JsonRpc, FamilyId, PrimalId, ValidationSink, BufferSink) |
| **Property-based tests** | New `proptest_types.rs`: FamilyId roundtrip, JsonRpcRequest serde, PrimalId validation, is_known_primal constants |
| **Coverage push** | device_management_server: 37%→88% (+19 tests); api.rs: 73%→96% (+3 tests); model_cache: 73%→79% (+11 tests); nucleus: 68%→69% (+5 tests); realtime +3, orchestrator +2, action_handler +6, enroll +2, suggestions +3 |
| **Formatting** | `cargo fmt --all` clean after all changes |
| **Clippy** | PASS (0 warnings, pedantic+nursery, -D warnings) |
| **Tests** | 5,279 → 5,340+ (+61 new tests), 0 failures |
| **Coverage** | 77.83% → 78.36% line (llvm-cov verified) |

### Spring Absorption Deep Debt — v2.40 (Mar 15, 2026)

Absorbed spring capabilities, eliminated deep debt across 9 phases: BYOB graph deployment, JSON-RPC 2.0 batch, compute dispatch, runtime TOML registry, real capability querying, 50 `#[ignore]` removed via DI, hardcoded primal name constants, dead code cleanup, semantic health alignment.

| Category | Change |
|----------|--------|
| **BYOB evolution** | Redefined from "Bring Your Own Beardog" to "Build Your Own Biome" — graph-based niche deployment via Neural API; deleted orphaned `byob/manager.rs`; `NicheDeployment` spawns processes via `which` + `std::process::Command`, kills via `rustix` (pure Rust) |
| **JSON-RPC 2.0 batch** | `JsonRpcInput` enum (Single/Batch) with concurrent batch processing via `futures::future::join_all` in Neural API connection handler |
| **Compute dispatch translations** | 6 new translations: `compute.dispatch.submit/status/cancel`, `compute.hardware.observe/distill/apply` for barraCuda integration |
| **Runtime TOML registry** | Neural API loads `config/capability_registry.toml` at startup, overlaying hardcoded defaults; three-layer loading: defaults → TOML → graph |
| **Real capability querying** | `query_primal_capabilities()` connects to primal sockets via `capability.list` JSON-RPC, replacing stub |
| **DI for env-var tests** | 50 `#[ignore]` annotations removed across `network_config.rs`, `defaults.rs`, `env_config.rs`, `engine_tests.rs` — all use `_with` variants with explicit `HashMap` |
| **Primal name constants** | Hardcoded primal names in `primal_discovery.rs` → `biomeos_types::primal_names::is_known_primal()` (case-insensitive) |
| **Dead code cleanup** | `#[allow(dead_code)]` resolved: `#[serde(rename)]` for wire fields, `#[cfg(test)]` for planned utilities |
| **Health alignment** | `health.ping` and `health.status` aliases translate to canonical `health.check` |
| **Pure Rust process mgmt** | `libc::kill` → `rustix::process::kill_process` for SIGTERM; `which` crate for binary discovery |
| **Tests** | 4,885 → 4,946 (+61), ignored 181 → 131 (-50), 0 failures |

### Concurrency Evolution — Fully Concurrent Test Suite (Mar 15, 2026)

Systematic elimination of global state dependencies to achieve fully concurrent test execution. All non-chaos/E2E tests now run in parallel.

| Category | Change |
|----------|--------|
| **Dependency injection** | 30+ functions evolved with `_with` / `_in` variants accepting explicit config params instead of reading env vars |
| **Env var races eliminated** | `std::env::set_var` / `remove_var` removed from all unit/integration tests — tests pass config directly |
| **CWD races eliminated** | `std::env::set_current_dir` removed from all tests — functions accept explicit base paths via `SporeConfig.plasmid_bin_dir` |
| **#[serial] removed** | 13 `#[serial_test::serial]` annotations removed from non-chaos tests (biomeos-core, biomeos-spore, biomeos-api, continuous, enroll) |
| **#[ignore] removed** | 22 `#[ignore]` annotations removed — tests now run with explicit config (nucleus, model_cache, doctor, paths, identifiers, defaults, discovery_bootstrap, neural-api-client-sync, capability_taxonomy) |
| **Config structs** | `DiscoveryConfig`, `FamilyDiscoveryConfig` introduced; `SporeConfig.plasmid_bin_dir` added for explicit path injection |
| **serial_test dep removed** | Removed from `biomeos-core` and `biomeos-spore` Cargo.toml (only E2E/chaos tests in `tests/atomics/` retain it) |
| **Test total** | 4,728 → 4,885 (+157), ignored 203 → 181 (-22), 0 failures |
| **Concurrency** | All 4,885 tests run fully concurrent — race conditions are production pitfalls, not test artifacts |

### Deep Debt Evolution — Modern Idiomatic Rust (Mar 14, 2026)

Comprehensive evolution pass: zero-copy binary payloads, capability-based discovery, async-first tests, smart module refactoring.

| Category | Change |
|----------|--------|
| **Zero-copy (Bytes)** | 22 `Vec<u8>` sites migrated to `bytes::Bytes` across 13 files (cryptographic keys, payloads, signatures, entropy) |
| **Primal name constants** | 9 production files evolved from hardcoded string literals to `primal_names::*` constants; `PROVENANCE_PRIMALS` slice for arrays |
| **SystemPaths** | `neural-api-client` fallback and `biomeos-federation` discovery evolved from `/tmp/` to XDG-aware `SystemPaths` / `socket_path()` |
| **Async-first tests** | ~70 sleep-based synchronization sites replaced with proper async primitives: `wait_for_socket()`, `wait_for_health()`, oneshot readiness, `Notify`, `watch` channels, `yield_now()` |
| **Smart refactoring** | `capability_translation.rs` (985→302+191+28), `provider.rs` (944→407+494), `concurrent_startup.rs` (931→210+672) — split at logical boundaries, not arbitrary lines |
| **Error handling** | `concurrent_startup.rs` `expect` → `unwrap_or` for malformed dependency graphs (no panic) |
| **Doc collision** | Root `[lib] doc = false` eliminates `biomeos/index.html` collision between workspace root and `crates/biomeos` |
| **SPDX headers** | 619/619 `.rs` files now have `SPDX-License-Identifier: AGPL-3.0-only` |
| **Coverage expansion** | ~25 new tests for `checks_config`, `checks_primal`, `model_cache`, `rootpulse`, `main.rs`, `neural-api-client-sync` |
| **Test total** | 4,383 → 4,728 (+345), 0 failures, 203 ignored |
| **Coverage** | 75.38% → 76.15% line; per-file: rootpulse 45→67%, model_cache 47→54%, main 38→44% |

### Deep Debt Audit + Zero-Copy + JSON-RPC Builders + Safe Casts + SystemPaths (Mar 14, 2026)

Comprehensive codebase audit against all wateringHole standards, followed by systematic evolution pass.

| Category | Change |
|----------|--------|
| **JSON-RPC builders** | New `JSONRPC_VERSION` constant, `JsonRpcRequest::new()` + `::notification()`, `JsonRpcResponse::success()` + `::error()` builders; 30+ manual JSON construction sites evolved across 15 crates |
| **Zero-copy (Bytes)** | `SecurityRpc` sign/verify, `LineageProof`, `TunnelRequest`, `BroadcastKeys`, `EncryptedDiscoveryConfig`, `Workload.code`, `CompressedBinary.data`, `fetch_binary()` all evolved from `Vec<u8>` to `bytes::Bytes` with base64 serde helpers |
| **Primal name constants** | `capability_translation.rs`, `definition.rs`, `primal_client.rs` evolved from hardcoded string literals to `primal_names::` constants |
| **SystemPaths** | Production `/tmp/` paths eliminated in `rootpulse.rs`, `neural_api.rs`, `continuous.rs`, `enroll.rs` — all evolved to `SystemPaths::new_lazy()` |
| **Safe casts** | All 15 `as` truncation casts evolved: `as_millis() as u64` → arithmetic duration, `as usize` → `try_from()`, `as char` → `char::from()`, `as f64` → documented precision-loss, `as i32` → `try_from().ok()` |
| **deny.toml** | Evolved for cargo-deny 0.19 (removed deprecated keys: `vulnerability`, `notice`, `unlicensed`, `copyleft`) |
| **File compliance** | `node_handlers.rs` (1015→461 lines) via test extraction to `node_handlers_tests.rs`; 0 files over 1000 lines |
| **Env-var test safety** | 4 race-condition tests marked `#[ignore]` (3 in definition_tests, 1 in primal_start); all env-var tests now serialized |
| **Dead code** | 8 `#[allow(dead_code)]` + TODO sites resolved: fields renamed with `_` prefix, functions wired or `#[cfg(test)]`, TEMPORARY comments evolved |
| **Formatting** | `cargo fmt` clean (16 diffs fixed) |
| **Clippy** | 0 warnings (`-D warnings`, pedantic+nursery) |
| **Test total** | 4,275 → 4,383 (+108), 0 failures, 204 ignored |
| **Coverage** | 75.21% → 76.06% region, 78.14% → 78.93% function, 73.95% → 74.95% line |

### Zero-Copy + Primal Constants + tarpc Wiring + Coverage Push (Mar 14, 2026)

Continued deep debt evolution: zero-copy binary payloads, centralized primal names, tarpc transport, and major test expansion.

| Category | Change |
|----------|--------|
| **Zero-copy (Bytes)** | `SignatureResult.signature` evolved from `Vec<u8>` to `bytes::Bytes` with base64 serde helpers; `bytes` added as workspace dep |
| **Primal name constants** | New `biomeos-types::primal_names` module: `BEARDOG`, `SONGBIRD`, `TOADSTOOL`, `NESTGATE`, `SQUIRREL`, `LOAMSPINE`, `RHIZOCRYPT`, `SWEETGRASS`; 15 production files across 8 crates updated to use constants |
| **tarpc transport** | `unix` feature enabled on workspace tarpc; new `biomeos-primal-sdk::tarpc_transport` module with `prepare_socket()`, `tarpc_socket_name()`, `tarpc_socket_path()` |
| **Coverage expansion** | +183 new tests: capability_taxonomy (35), subfederation manager (20), dark forest beacon (22), service core (27), service security (20), networking types (22), error types (29), tarpc transport (7) |
| **Test extraction** | 6 files over 1000 LOC split into `*_tests.rs` files: nucleus, definition, beacon, core, security, networking_services |
| **Clippy** | 0 warnings (fixed redundant closures in biomeos-nucleus, borrowed expression in beacon tests, duplicated attributes) |
| **File compliance** | 0 production files over 1000 lines (largest: 998) |
| **Test total** | 4,092 → 4,275 (+183), 0 failures, 167 ignored |

### Deep Debt Evolution + ecoBin v3.0 Compliance (Mar 13, 2026)

Comprehensive audit and evolution pass against ecoPrimals wateringHole standards:

| Category | Change |
|----------|--------|
| **nix → rustix** | All 8 crates migrated from `nix` (libc wrapper) to `rustix` (pure Rust syscalls). Zero unsafe code. |
| **sysinfo → /proc** | All 5 crates migrated from `sysinfo` (C deps) to direct `/proc` reads + `rustix::fs::statvfs`. ecoBin v3.0 compliant. |
| **Large file refactoring** | 8 files >1000 lines refactored into domain modules: widgets.rs (1571→3 files), doctor.rs (1075→6 files), ai_first_api.rs (1049→4 files), dark_forest.rs (1041→4 files), subfederation.rs (1019→5 files), rootfs.rs (1005→7 files), model_cache.rs (1002→4 files). Max file now 998 lines. |
| **JSON-RPC consolidation** | 5+ duplicate `JsonRpcRequest`/`JsonRpcResponse` definitions unified into `biomeos-types::jsonrpc` |
| **Hardcoded paths** | `/home/strandgate/Development` removed from 4 `tools/src/*.rs` files → runtime `discover_workspace_root()` |
| **Hardcoded IPs** | `192.168.1.132:8080` in tests → RFC 5737 documentation address; `192.168.1.1` → `192.0.2.1` |
| **Mock production code** | 3 mock implementations in `tools/src/` evolved to real: sovereignty→dep tree inspection, coverage→llvm-cov parsing, health→runtime socket discovery |
| **deny.toml** | New — bans openssl-sys, ring, aws-lc-sys, native-tls, zstd-sys, dirs-sys |
| **rustfmt.toml** | New — enforces edition 2021, max_width 100 |
| **forbid(unsafe)** | Added to `tools/src/lib.rs` (was the only gap) |
| **tools Cargo.toml** | Fixed broken workspace inheritance (self-contained workspace with explicit deps) |
| **Test coverage** | 4,033 → 4,275 tests (+242); 74.91% → 75.21% region coverage; new proc_metrics, nucleus, model_cache, neural-api-client, suggestions, capability_taxonomy, subfederation, beacon, tarpc tests |
| **sysinfo version alignment** | Removed entirely (was 4 different versions: 0.29, 0.30, 0.31, 0.32) |
| **Format regression** | Fixed `capability_domains.rs` formatting diff |

### Spring Absorption — Cross-Spring Evolution (Mar 11, 2026)

Absorbed capabilities from all 7 springs and petalTongue V1.6.1:

| Category | Change |
|----------|--------|
| **wetSpring V110** | +14 translations: kinetics (Gompertz, Monod, Haldane, first-order), beta diversity, rarefaction, NMF, monitoring, brain (observe/attention/urgency), metrics |
| **airSpring v0.7.5** | +5 translations: SPI drought index, autocorrelation, gamma CDF, bootstrap CI, jackknife CI |
| **petalTongue V1.6.1** | +8 translations: sensor stream (subscribe/poll/unsubscribe), interaction (poll/unsubscribe), visualization stream, dashboard |
| **healthSpring V20** | +5 translations: Michaelis-Menten PK, SCFA production, beat classify, stress assessment, TRT pipeline |
| **Deploy Graphs** | +4: hotspring_deploy, groundspring_deploy, healthspring_deploy, cross_spring_ecology |
| **Niche Templates** | +4: ecology-pipeline, hotspring, groundspring, healthspring |
| **Capability Domains** | +5 keywords: kinetics, monitoring, drought, statistics, sensor_stream |

### Provenance Trio Graph Deployments (March 13, 2026)

Integrated loamSpine, rhizoCrypt, and sweetGrass as deployable primals with full Neural API capability routing:

| Category | Change |
|----------|--------|
| **loamSpine** | Deploy graph + 18 translations: spine, entry, certificate, proof, commit, health |
| **rhizoCrypt** | Deploy graph + 13 translations: dag sessions, merkle, dehydration, slice |
| **sweetGrass** | Deploy graph + 11 translations: braid, provenance, attribution, rewards |
| **Combined** | `provenance_trio_deploy.toml` — starts all three in dependency order |
| **Workflow** | `provenance_pipeline.toml` + `rootpulse_commit.toml` ready for execution |
| **Capability Registry** | 35+ new semantic translations in `capability_registry.toml` |

### Continuous Systems + XR/Surgical VR Evolution (Mar 11, 2026)

Live feed, continuous execution, and immersive VR foundations:

| Category | Change |
|----------|--------|
| **Continuous Executor** | `ContinuousExecutor` with `TickClock` (fixed-timestep), `SessionState` lifecycle, feedback edges, per-node budget enforcement |
| **Game Engine Tick** | `game_engine_tick.toml` — 60 Hz continuous graph (input → logic → physics → scene → render) |
| **Push Events** | SSE evolved from 5s poll to push-based `GraphEventBroadcaster` (`tokio::broadcast`); WebSocket wired to event stream |
| **Sensor Routing** | `SensorEventBus` + `SensorCollector` — keyboard/mouse/gamepad/tracking events routed through graph nodes |
| **Stub Resolution** | ~15 production stubs resolved: mDNS (`trust-dns`), network interfaces (`/sys/class/net`), USB statvfs, MAC address (`/sys/class/net/*/address`), mesh file-based discovery |
| **XR Type System** | `biomeos-types::xr` — `VisualOutputCapability`, `StereoConfig`, `Pose6DoF`, `TrackingFrame`, `MotionCaptureConfig`, `HapticCommand`, `HapticDeviceCapabilities`, `TissueMaterial`, `AnatomyModel`, `SurgicalInstrument` |
| **Stereo Rendering** | `StereoRenderAdapter` — negotiate/begin/submit/end stereo sessions with petalTongue via JSON-RPC |
| **Motion Capture** | `MotionCaptureAdapter` — OpenXR/SteamVR backend, 1000Hz tracking, surgical preset (head + hands + tool) |
| **Haptic Pipeline** | `HapticPipeline` — device discovery, safety-clamped force feedback dispatch, emergency stop |
| **Surgical Domain** | `biomeos-types::surgical` — `SurgicalProcedure`, `ToolTissueInteraction`, `DamageType`, `BiosignalType`, `PkModelParams`, `SurgicalSessionMetrics` |
| **Capability Domains** | XR domain (petaltongue: 14 methods) + Medical domain (healthspring: 12 methods) added to registry |
| **Niche Templates** | `surgical-vr` niche (healthSpring + petalTongue + ludoSpring) with `surgical_vr_deploy.toml` graph |
| **Tests** | 3,590 → 3,670+ (80 new tests for XR types, surgical domain, UI adapters, capability domains) |

### Deep Debt Evolution Plan — 8-Phase Execution (Mar 11, 2026)

Data-driven architecture evolution across 8 phases:

| Phase | Scope | Key Changes |
|-------|-------|-------------|
| **1. Capability routing** | Eliminated hardcoded primal names from routing | `primal_spawner.rs` match block → `config/primal_launch_profiles.toml`; `bootstrap.rs`, `ai_advisor.rs` use `CapabilityTaxonomy::resolve_to_primal()` |
| **2. Path elimination** | Removed all hardcoded socket/log/config paths | 7 files migrated to `SystemPaths` XDG; removed personal `/home/eastgate/` path from `genome_dist.rs` |
| **3. Deploy graphs** | Created missing deployment graphs | `nucleus_simple.toml`, `ui_atomic.toml`, `livespore_create.toml`; niche template graph_id naming fixed |
| **4. Large file splits** | 6 files >1000 LOC → domain modules | `system/lib.rs`, `security.rs`, `capability_handlers.rs`, `genome_dist.rs`, `protocol_escalation.rs`, `nucleus.rs` |
| **5. Dead code** | Resolved placeholders and dead code | `usb.rs` metadata.len() bug; `UNVERIFIED_SIGNATURE` constant; `config_builder` domain method |
| **6. Env centralization** | Single source of truth for env vars | New `biomeos-types/src/env_config.rs` with typed accessors |
| **7. Rust modernization** | Table-driven routing, safer fallbacks, doc enforcement | Neural API `ROUTE_TABLE` (78 entries); `unwrap_or_default` → `tracing::warn!`; `#![warn(missing_docs)]` on 4 crates |
| **8. Cargo audit** | Pure Rust dependency tree | `libc` removed from workspace + 3 crates; only `linux-raw-sys` (pure Rust syscall interface) |

### Hardware Learning Capability Wiring (Mar 11, 2026)

5 `compute.hardware.*` capabilities registered for toadStool hw-learn crate (vendor-neutral GPU init learning):
- `observe`, `distill`, `apply`, `share`, `status` → toadstool `hw_learn.*` methods

### Deep Debt Evolution + Sovereignty Audit (Mar 11, 2026)

Previous audit results (preserved as fossil record):

| Category | Change |
|----------|--------|
| **Sovereignty** | STUN defaults evolved from Google/Cloudflare to community-run FOSS servers (nextcloud, sip.us, stunprotocol.org) |
| **Standalone discovery** | Hardcoded primal names/paths in capability handler → runtime socket discovery via XDG `SystemPaths` |
| **SSH legacy** | `ssh_legacy.rs` deleted; Plasmodium uses Songbird mesh RPC only |
| **Deprecated APIs** | All `#[allow(deprecated)]` removed: tempfile `into_path()` → `keep()`, config builder cleaned, dual-mode server removed |
| **Dead code** | ~50 `#[allow(dead_code)]` sites audited: 5 unused functions deleted, remaining given TODO or wire-format justification |
| **Module refactoring** | `agents.rs` (1,471 lines) → `agents/` module (5 files); `lifecycle_manager.rs` (1,333 lines) → `lifecycle_manager/` module (7 files) |
| **NeuralApiServer** | Removed redundant `executions` and `living_graph` fields (handlers own their own Arc clones) |
| **Formatting** | Fixed `genome_dist.rs` regression; all new files formatted |
| **Remaining large files** | 7 files >1000 total lines — all have <1000 production lines (tests inflate, acceptable) |
| **UniBin compliance** | `biomeos api` subcommand now wires real `biomeos-api` library — no separate binary needed |
| **Zero-copy** | `PrimalId`, `FamilyId`, `TowerId` → `Arc<str>` (cheap clone); `HttpResponse.body` → `bytes::Bytes`; `ExecutionContext.family_id` → `Arc<str>` |
| **Test coverage** | 2,716 → 3,590 tests; 59% → 71.47% region coverage; 874 new tests across all crates |
| **Race condition fixes** | Env-var-mutating tests serialized with `#[ignore]` + `Mutex` guards |

### Relay-Assisted Coordinated Punch — biomeOS Implementation (Feb 11)
All biomeOS-owned tasks from the relay-punch protocol handoff:

| Component | File | Status |
|-----------|------|--------|
| Capability translations | `capability_translation.rs` | ✅ `stun.probe_port_pattern`, `punch.coordinate`, `relay.authorize` |
| Neural API routing sugar | `neural_api_server/routing.rs` | ✅ Direct method → `capability.call` transform |
| Connection strategy orchestrator | `biomeos-core/connection_strategy.rs` | ✅ 4-tier: LAN → punch → coordinated → relay |
| Rendezvous beacon payload | `biomeos-api/handlers/rendezvous.rs` | ✅ `connection_info` field (STUN, relay, NAT type) |
| Pre-existing test fix | `neural-api-client/src/lib.rs` | ✅ `test_discover_socket_path` assertion corrected |

Key types: `ConnectionTier`, `NatType`, `PortPattern`, `PeerConnectionInfo`, `StunResults`.
22 new unit tests across `biomeos-core` (11) and `biomeos-api` (11 updated + 1 new).
See: `ecoPrimals/wateringHole/handoffs/` (formerly docs/handoffs/)

### Plasmodium Agent Dispatch + Coverage (Feb 11)
Added `agent.route` RPC method — resolves a capability through an agent's routing table
and returns dispatch instructions (transport type, target socket, formatted method name).
Local routes dispatch via `unix_socket`, remote routes via `mesh_relay`.
35 new comprehensive tests covering all 8 RPC handler methods (`agent.create`, `agent.list`,
`agent.get`, `agent.remove`, `agent.meld`, `agent.split`, `agent.resolve`, `agent.route`),
plus type serialization, priority ordering, meld/split edge cases, and metadata roundtrips.

### Test Coverage Expansion Phase 3 (Feb 11)
61 new tests added across 2 critical modules, bringing total from 2,358 to 2,419.

| Module | Tests Added | Coverage Focus |
|--------|-------------|----------------|
| `capability_handlers.rs` | 28 | All RPC methods: register, list, discover, providers, call, route, metrics, translations |
| `config/mod.rs` | 30 | Builder (all feature flags, aliases, settings), presets (dev/prod/test/local), validation (all paths) |
| `connection_strategy.rs` | 3 (pre-existing) | Type-level coverage already comprehensive |

Key patterns:
- Every builder method and feature flag alias now has a dedicated test
- Validation tests cover all production-readiness criteria (workers, crypto, timeout, registry)
- Capability handler tests cover error paths (missing params, missing fields) alongside happy paths

### Test Coverage Expansion Phase 4 (Feb 11)
114 additional tests added across 6 modules, bringing total from 2,425 to 2,539.

| Module | Tests Added | Coverage Focus |
|---|---|---|
| `protocol_escalation.rs` | 20 | Config serde defaults, partial JSON, cooldown (zero/multi), status details, fallback, escalation without primal state, all ProtocolMode roundtrips |
| `executor/context.rs` | 14 | Checkpoint save/load, status overwrite, output overwrite, all_statuses, clone shared state, family_id env precedence, NodeStatus serde roundtrip |
| `executor/types.rs` | 8 | ExecutionReport serde roundtrip, total_phases/total_nodes from results, phase_result success/multi-failure, summary serde, clone |
| `neural_executor_tests.rs` | 11 | Deep chain sort, wide graph, diamond+tail, self-cycle, 3-node cycle, adjacent env vars, same var repeated, empty string, custom GraphConfig |
| `dark_forest_gate.rs` | 15 | Config defaults, bypass paths, bare OK paths, token enforcement, cache init, sovereign mode |
| `primal_discovery.rs` | 18 | extract_name/family_id edge cases, non-socket filtering, is_primal_name, find_by_family empty |
| `node_handlers.rs` | 28 | substitute_env (all syntaxes), filesystem_check, log handlers, deployment_report (mixed), discover_capability_provider |

### Graph-Based NUCLEUS Deployment Validation (Feb 11)
Overhauled all deployment graphs to use XDG-compliant paths, dynamic `${FAMILY_ID}` resolution,
and complete relay-punch capability translations. Created gate2 deployment graph.

| Graph | Changes | Nodes |
|-------|---------|-------|
| `nucleus_complete.toml` v2.0.0 | XDG paths, full BearDog/Songbird translations, relay-punch caps, Sovereign Onion init, all 5 primals as `start` (not `register_only`) | 11 |
| `ecosystem_full_bootstrap.toml` v2.0.0 | XDG paths (was `/tmp/`), added NestGate (was missing), mesh/punch/relay/stun caps | 6 |
| `gate2_nucleus.toml` v1.0.0 (NEW) | Full gate2 NUCLEUS with mesh init + Tower auto-discover step for covalent bonding | 9 |
| `tower_atomic_bootstrap.toml` | Fixed hardcoded `/tmp/` and `/run/user/1000/` → `${XDG_RUNTIME_DIR}`, port 3492 → 8080 | — |

Key fixes:
- **Neural API is biomeOS** — graphs no longer list it as a separate deployable; it IS the biomeOS capability routing layer
- **No hardcoded paths** — all env vars use `${XDG_RUNTIME_DIR}/biomeos/{primal}-${FAMILY_ID}.sock`
- **Port 3492 eliminated** — Sovereign Onion init now uses port 8080 (consistent with Songbird HTTP)
- **7 new graph validation tests** in `neural_graph.rs` (parse + no-hardcoded-path assertions)
- Total tests: 2,539 (after Phase 4 coverage expansion)

### HTTP JSON-RPC Inter-Gate Transport (Feb 10)
`AtomicClient::http()` pure Rust transport. `TransportEndpoint::HttpJsonRpc` enum.
Plasmodium `query_remote_gate()` uses HTTP POST `/jsonrpc` with runtime port discovery
(env `SONGBIRD_MESH_PORT` → 8080 default). Hardcoded port 3492 eliminated.

### Device Enrollment (Feb 10)
`biomeos enroll` validated on Tower and gate2 with Blake3-Lineage-KDF derivation
from shared `.family.seed`. Unique per-device `.lineage.seed` files.

### Pure Rust System Calls
All production shell-outs replaced with pure Rust (`/proc`, `/sys`, `rustix` crate).

### Internalized `start_nucleus.sh`
`biomeos nucleus start` is the pure Rust replacement. Binary discovery,
dependency-ordered startup, health checks, graceful shutdown -- all in Rust.

### API Route Completion
5 previously dead-code handler modules wired into the API router (capabilities, genome).

### Deep Debt Cleanup
Zero production `unwrap()`, zero hardcoded `/tmp`, zero production mocks,
zero clippy warnings across entire workspace (including biomeos-boot).

### Deep Debt Audit (Feb 10)
Comprehensive codebase audit against ecoPrimals standards:

| Category | Before | After |
|----------|--------|-------|
| Clippy warnings | 83 | 0 |
| Formatting diffs | 6 files | 0 |
| Production `unwrap()` | 46 | 0 (all → `expect()` with context) |
| Hardcoded primal names | 30+ scattered | Centralized via `CapabilityTaxonomy` |
| Production mocks | 1 (`is_mock_mode`) | 0 (removed dead code) |
| Ignored tests | 93 | 92 (fixed `serde(default)` on `ConfigMetadata`) |
| `#[allow]` unnecessary | 2 (`vec_init_then_push`) | 0 (replaced with `vec![]`) |
| Files >1000 lines | 0 | 0 (max: 985 lines) |
| Unsafe code | 0 | 0 |
| External C deps | 1 (`zstd-sys`) | 1 (noted for future format evolution) |

Key evolutions:
- `std::sync::Mutex` → `tokio::sync::Mutex` in async test contexts
- `Config::default()` field reassignment → struct literal update syntax
- Deprecated `Command::cargo_bin()` → `cargo_bin_cmd!()` macro
- `assert!(true)` placeholders → `todo!()` comments or `const {}` blocks
- Bootstrap primal lists → `CapabilityTaxonomy::known_primals()`
- Scattered env var lookups → `resolve_capability_provider()` helper
- `[profile.release]` moved from crate to workspace root
- Deprecated primal-specific socket constants removed → `service_socket()` dynamic resolution
- Deprecated `BearDogConfig`, `SongbirdConfig`, `TowerBuilder` type aliases removed
- Deprecated `legacy_hardcoded_metadata`, `discover_primal_socket`, `AtomicPrimalClient::new` removed
- `RuntimeConfig::service_socket()` fixed to use struct's `socket_dir` (was silently falling to `/tmp`)

### Test Coverage Push Phase 2 (Feb 10)
196 additional tests added across 5 crates/modules, bringing total from 2,101 to 2,297.
Coverage: 51.4% → 56.75% region coverage (+5.3pp).

| Crate/Module | Tests Added | Coverage Focus |
|---|---|---|
| `biomeos-core` concurrent_startup | +21 | DependencyGraph build, topological_waves (empty, chain, diamond, circular) |
| `biomeos-core` primal_orchestrator | +29 | PrimalHealthMonitor, PrimalOrchestrator lifecycle, resolve_dependencies |
| `biomeos-federation` nucleus | +36 | SecureNucleusDiscovery 5-layer protocol, TrustLevel, VerifiedPrimal, selection |
| `biomeos-federation` discovery | +25 | PrimalDiscovery, parse_endpoint, register_songbird_peer, capability filtering |
| `biomeos-spore` beacon_genetics | +70 | types (BeaconId, meetings, clusters, manifest), derivation (LineageDeriver full lifecycle, save/load/enroll, proof gen/verify), capability (DirectBeardogCaller, client creation), manager (initialize, lineage hint, sync edge cases) |
| `biomeos-atomic-deploy` orchestrator | +15 | AtomicType variants, DeploymentConfig serde, DeploymentResult lifecycle, orchestrator creation, deploy error paths |

### Test Coverage Push Phase 1 (Feb 10)
311 new tests added across 8 crates, bringing total from 1,790 to 2,101:

| Crate/Module | Tests Added | Coverage Focus |
|---|---|---|
| `biomeos-types` config | ~20 | BiomeOSConfig validation, builder, merge, serde, env vars, file I/O |
| `biomeos-core` stun_extension | ~10 | StunExtensionConfig defaults, serde, availability, fallback |
| `biomeos-atomic-deploy` lifecycle | ~15 | LifecycleManager creation, state transitions, deployment, apoptosis |
| `biomeos-atomic-deploy` protocol | ~10 | EscalationConfig, connection metrics, auto-escalate |
| `biomeos-graph` graph/node/validation/loader | ~60 | GraphId/NodeId validation, topological sort, env vars, TOML loading |
| `biomeos-spore` (7 modules) | ~120 | error, manifest, verify, refresh, usb, incubation, seed, dark_forest |
| `biomeos-api` handlers (4 modules) | ~76 | trust, rendezvous, events, genome — serde, state, GenomeState I/O |

### Bootstrap Mode Detection
Auto-detects existing ecosystem. Starts supplementary primals only, replaces stale sockets.

### HealthChecker Evolution
Deep JSON-RPC health checks for Active primals, lighter socket checks for Incubating.

### Lifecycle Integration
`LifecycleManager` integrated into nucleus mode. Auto-monitoring at 10s intervals.

### SystemPaths Consolidation
All duplicate path resolution replaced with centralized `SystemPaths::new_lazy()`.
Affected: `nucleus.rs`, `doctor.rs`, `trust.rs`, `topology.rs`, `genome.rs`, `capability_translation.rs`.

### Capability-Based Plasmodium
Dynamic socket scanning replaces hardcoded primal names. Capability taxonomy as single source of truth.

### Capability Translation Socket Consolidation
`resolve_primal_socket()` reduced from 45-line manual fallback to 5-line SystemPaths delegation.

### biomeos-boot Doc Compliance
39 missing-docs warnings fixed. 5 production `unwrap()` replaced with safe alternatives.

### Full Workspace Documentation Coverage (Feb 10)
Resolved all `missing_docs` warnings across the entire workspace (~1,445 total):

| Crate | Warnings Fixed |
|---|---|
| `biomeos-core` | 140 |
| `biomeos-types` | 892 |
| `biomeos-cli` | 249 |
| `biomeos-compute` | 91 |
| `biomeos-api` | 21 |
| `biomeos-deploy` | 20 |
| `biomeos-genome-factory` | 20 |
| `biomeos-genome-deploy` | 12 |

Every public module, struct, enum, field, variant, function, and type alias now has
doc comments (`///` or `//!`). All 2,297 tests pass with 0 failures.

---

## Remaining Bypasses (0 active, 6 evolved)

All bypasses have been resolved:

### 1. ~~HTTP_REQUEST_PROVIDER_SOCKET env var bypass~~ (EVOLVED Mar 19)

**Was**: Squirrel discovered Songbird via explicit env var.
**Now**: `http_bridge` registered as Songbird capability in `capability_sockets.rs` and `CapabilityTaxonomy`. Squirrel uses `BIOMEOS_DISCOVERY_SOCKET` and `discover_capabilities("http_bridge")`. `HTTP_REQUEST_PROVIDER_SOCKET` env var removed from nucleus spawn.

### 2. ~~Socket nucleation symlinks~~ (EVOLVED)

**Was**: `start_nucleus.sh` created symlinks: `songbird.sock -> songbird-{family_id}.sock`.
**Now**: `biomeos nucleus start` creates family-suffixed sockets directly. Multi-family
architecture (Option A) implemented. Socket resolution via `SystemPaths::primal_socket()`.

### 3. ~~NestGate inverted boolean patch~~ (EVOLVED Mar 19)

**Was**: biomeOS passed `--socket-only` to NestGate, getting inverted semantics.
**Now**: biomeOS omits `--socket-only` flag entirely; with the inverted semantics this yields socket-only mode (the desired behavior). Documented with compatibility comment in nucleus.rs.

### 4. ~~Squirrel default model override~~ (EVOLVED Mar 19)

**Was**: Had to pass `model: "claude-3-haiku-20240307"` explicitly.
**Now**: `AI_DEFAULT_MODEL` env var passed through to Squirrel in both nucleus spawn and graph deployment. Squirrel reads it at startup.

### 5. ~~SSH-based Plasmodium queries~~ (EVOLVED)

**Was**: Remote gate queries used SSH, creating new processes per query.
**Now**: Plasmodium uses Songbird mesh RPC as primary transport. SSH retained as
deprecated fallback only. Capability-based primal discovery via socket scanning.

### 6. ~~Hardcoded TCP port 3492 for inter-gate~~ (EVOLVED)

**Was**: `plasmodium.rs` hardcoded port 3492 for `AtomicClient::tcp()` connections.
**Now**: Uses `AtomicClient::http()` with runtime port: `mesh.peers` → `SONGBIRD_MESH_PORT` → 8080.
Beacon discovery payload includes `jsonrpc_port`. Songbird HTTP gateway (port 8080) serves as
covalent bond transport.

---

## Primal Evolution Needs

### What Each Primal Needs

| Primal | Status | Next Evolution |
|--------|--------|----------------|
| **BearDog** | Reference | Stable. No changes needed. |
| **Songbird** | 90% | 1. Fix mesh state split (3 independent MeshHandler instances). 2. Fix UDP discovery protocol (ephemeral port bug). 3. Eliminate hardcoded 3492 (22 occurrences). 4. `discover_capabilities`. |
| **Toadstool** | Operational | 1. GPU job queue. 2. Cross-gate compute delegation. 3. Ollama integration. |
| **NestGate** | Operational (patched) | 1. Fix inverted boolean upstream. 2. Model cache methods. 3. Cross-gate replication. |
| **Squirrel** | Operational | 1. Ollama native adapter. 2. Configurable default model. 3. Provider health monitoring. |
| **biomeOS** | Evolved | 1. ~~Validate graph-based NUCLEUS deployment~~ ✅. 2. ARM64 genomeBin. 3. ~~Plasmodium Agent Model~~ ✅. |

### What biomeOS Needs Next

| Area | Current | Target |
|------|---------|--------|
| ~~**Graph-based deploy**~~ | ~~Manual nohup~~ | ✅ Graphs validated: `nucleus_complete`, `ecosystem_full_bootstrap`, `gate2_nucleus`, `cross_gate_tower`, `cross_gate_pixel` |
| ~~**ARM64 biomeOS**~~ | ~~Not built~~ | ✅ `aarch64-unknown-linux-musl` static, 9.6 MB stripped |
| ~~**Plasmodium agents**~~ | ~~HTTP JSON-RPC collective~~ | ✅ Neural API agent routing (Meld/Split/Mix) |
| ~~**Model orchestration**~~ | ~~List/resolve only~~ | ✅ `inference.schedule` + `inference.gates` — VRAM-aware gate routing |
| **biomeOS on gate2** | Graphs ready | Deploy biomeOS to gate2 and validate `cross_gate_tower.toml` end-to-end |
| **Pixel biomeOS** | Graphs ready | Deploy ARM64 biomeOS on Pixel via ADB, validate `cross_gate_pixel.toml` |

---

## Ecosystem Status

### NUCLEUS Architecture - VALIDATED

```
NUCLEUS = Tower + Node + Nest (orchestrated by biomeOS via Neural API)

Tower Atomic  = BearDog + Songbird       (crypto + network + HTTP)
Node Atomic   = Tower + Toadstool        (+ compute + GPU)
Nest Atomic   = Tower + NestGate         (+ storage + persistence)
Full NUCLEUS  = All 5 primals (biomeOS routes capabilities via Neural API)
```

### Live HPC Configuration

```
Tower (pop-os, x86_64):
  GPU:    RTX 4070 (12 GB VRAM)
  RAM:    31 GB
  CPU:    24 cores (i9-14900)
  AI:     Ollama (phi3, llama3.2, tinyllama)
  Primals: BearDog, Songbird, NestGate, Toadstool, Squirrel
  biomeOS: Neural API capability routing (290+ translations + agent routing)

gate2 (pop-os, x86_64):
  GPU:    RTX 3090 (24 GB VRAM)
  RAM:    251 GB
  CPU:    128 cores (EPYC 9274F)
  Storage: ZFS (native dedup, compression, snapshots)
  Primals: BearDog, Songbird, NestGate, Toadstool, Squirrel (Full NUCLEUS)

Bond:   HTTP JSON-RPC via Songbird (port 8080) / SSH (legacy, retained for management)
Family: Shared .family.seed, both enrolled with Blake3-Lineage-KDF
```

### Primal Binary Status

| Primal | genomeBin | Size | x86_64 | aarch64 |
|--------|-----------|------|--------|---------|
| **songbird** | v3.33.0 | 18.5 MB | Yes | Yes |
| **beardog** | complete | 6.9 MB | Yes | Yes |
| **toadstool** | complete | 8.9 MB | Yes | Yes |
| **nestgate** | complete | 5.6 MB | Yes | Yes |
| **squirrel** | complete | 4.3 MB | Yes | Yes |
| **biomeOS** | complete | 9.6 MB | Yes | Yes |

---

## Remaining Work

### Critical (Songbird Team - Blocks Covalent Bonding)
1. **Fix mesh state split** - 3 independent MeshHandler instances need shared state via Arc
2. **Fix UDP discovery protocol** - Bind to actual discovery port, not ephemeral; avoid mDNS 5353
3. **Eliminate hardcoded 3492** - 22 occurrences across 12 files → runtime `SONGBIRD_HTTP_PORT` / 8080
4. See `ecoPrimals/wateringHole/handoffs/` for root causes + file locations (formerly docs/handoffs/)

### High Priority (Primal Teams)
1. **Songbird `discover_capabilities`** - Enables pure runtime discovery (no env var bypass)
2. **NestGate upstream boolean fix** - Remove downstream patch
3. **Squirrel multi-backend inference** - Local GPU + remote API routing

### Medium Priority (biomeOS Team)
1. ~~**Validate graph-based NUCLEUS deployment**~~ - ✅ Graphs validated: `nucleus_complete`, `ecosystem_full_bootstrap`, `gate2_nucleus`
2. ~~**ARM64 biomeOS genomeBin**~~ - ✅ Built (`aarch64-unknown-linux-musl`, 9.6 MB stripped, static)
3. ~~**Plasmodium Agent Model**~~ - ✅ Neural API agent routing (Meld/Split/Mix) implemented
4. **biomeOS on gate2** - Deploy biomeOS to gate2 for cross-gate capability routing via Neural API
5. **Test coverage** - ✅ All three metrics ~90% (v2.63): 90.26% region / 91.10% function / 89.99% line |

### Low Priority
1. **API key encryption** - NestGate + BearDog secured storage

---

## Test Coverage Analysis (llvm-cov, Mar 20, 2026)

**Overall**: 90.28% region / 91.11% function / 90.02% line coverage (workspace-wide llvm-cov verified, 0 test failures, ~135 ignored cwd-sensitive, 25 doc-tests, 4 proptests)

### Coverage Distribution

| Band | Files | Notes |
|------|-------|-------|
| **90-100%** | 55 | Well-tested core modules |
| **70-89%** | 57 | Good coverage, some edge cases missing |
| **50-69%** | 48 | Partial coverage, needs attention |
| **30-49%** | 37 | Low coverage, significant gaps |
| **1-29%** | 45 | Very low, mostly runtime/integration code |
| **0%** | 72 | Untested (see breakdown below) |

### 0% Coverage Breakdown (72 files)

| Category | Count | Reason |
|----------|-------|--------|
| **Binary entry points** (`main.rs`, `bin/`) | 18 | Thin dispatchers; tested by e2e tests |
| **UniBin mode dispatchers** (`modes/`) | 10 | CLI mode handlers; need integration tests |
| **CLI command handlers** | ~15 | Format output + call core; need e2e tests |
| **Neural API server modules** | 7 | Runtime server; needs mock-server integration tests |
| **Boot/deploy infrastructure** | ~5 | System-level; needs elevated privileges |
| **TUI widgets** | 2 | Terminal UI; hard to unit test |
| **Legacy code** (ssh_legacy) | 1 | Deprecated; shouldn't get new tests |
| **SDK/types** | 3 | Thin type definitions |
| **Other** | ~11 | Misc library code |

### Critical Untested Paths (actionable)

| File | Regions | Impact | Status |
|------|---------|--------|--------|
| `neural_executor.rs` | 916 | Core neural execution engine | ✅ 53 tests (context, types, topo sort, env sub) |
| `dark_forest.rs` | 695 | Security beacon system | ✅ 15 tests (config, bypass, token enforcement) |
| `rootfs.rs` | 677 | Root filesystem management | Needs integration tests |
| `capability_handlers.rs` | 529 | Capability RPC handlers | ✅ 28 tests (all RPC methods, error paths) |
| `subfederation.rs` | 570 | Federation subdivision | Needs mock services |
| `lifecycle_manager.rs` | 605 | Primal lifecycle state machine | Needs mock services |
| `protocol_escalation.rs` | 643 | JSON-RPC → tarpc escalation | ✅ 41 tests (config, cooldown, status, fallback) |
| `device_management/provider.rs` | 940 | Device management UI | Needs integration tests |
| `primal_discovery.rs` | — | Socket-based primal discovery | ✅ 21 tests (extraction, filtering, edge cases) |
| `node_handlers.rs` | — | Graph node execution handlers | ✅ 30 tests (env sub, filesystem, log, report) |

### High-Coverage Successes (90%+)

| File | Coverage | Regions |
|------|----------|---------|
| `ai_first_api.rs` | 100% | 303 |
| `state.rs` (UI) | 100% | 362 |
| `suggestions/mod.rs` | 99.7% | 699 |
| `health.rs` (API) | 99.7% | 297 |
| `nucleation.rs` | 98.2% | 339 |
| `primal/capabilities.rs` | 97.9% | 379 |
| `spore_log_tracker.rs` | 95.0% | 577 |
| `primal_client.rs` | 95.1% | 370 |

### Path to 90% region coverage

1. **Quick wins (add unit tests)**: ~~`config/mod.rs`~~ ✅ 38 tests, `primal_adapter/types.rs` (23.5%), ~~`stun_extension.rs`~~ ✅ flaky tests fixed
2. **Integration test infrastructure**: CLI command handlers, neural API server, boot modules
3. **Mock services**: Federation, lifecycle, protocol escalation tests need mock primals
4. **Accept low coverage**: Binary entry points, TUI widgets, deprecated code

---

## Standards Compliance

| Standard | Status |
|----------|--------|
| ecoBin v3.0 | 100% Pure Rust |
| Universal IPC v3.0 | Multi-transport (Unix/Abstract/TCP/HTTP JSON-RPC) |
| PRIMAL_DEPLOYMENT_STANDARD v1.0 | Deterministic behavior |
| Semantic Method Naming | capability.call routing |
| AGPL-3.0-only License | Compliant |
| Evolved Genetic Model v2.0 | Mitochondrial + Nuclear |
| XDG Base Directory | All paths via SystemPaths |

---

## Quick Commands

```bash
# Build
cargo build --workspace

# Test (7,212 tests — fully concurrent)
cargo test --workspace

# Clippy (0 warnings, entire workspace)
cargo clippy --workspace

# Format
cargo fmt --check

# Start NUCLEUS (Pure Rust)
biomeos nucleus start --mode full --node-id tower1

# Start NUCLEUS (Tower only)
biomeos nucleus start --mode tower --node-id tower1

# Test AI Bridge
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"hello","model":"claude-3-haiku-20240307","max_tokens":10},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/squirrel.sock -w 15 -q 1
```

---

**Status**: Production Ready (v2.81 — BM-04/05 resolved, TCP-only mobile, cross-gate routing, fully concurrent tests, zero blocking debt)
**Tests**: 7,212 passing, 0 failures, fully concurrent (90%+ llvm-cov verified)
**Clippy**: PASS (0 warnings, pedantic+nursery) | **Format**: PASS | **Docs**: Full coverage | **Unsafe**: 0 production | **C deps**: 0
**IPC**: Universal IPC v3.0 (Unix/Abstract/TCP/HTTP JSON-RPC) + tarpc binary escalation + TCP-only mode
**Neural API**: 290+ translations, 26 domains, proxy_http, capability.call, lazy rescan, cross-gate forwarding, graph coordination
**Code Quality**: A++ (Pure Rust, Edition 2024, zero-copy, safe casts, JSON-RPC builders, zero warnings, full doc coverage, `serial_test` eliminated)
**Bypasses**: 0 active (all 6 evolved)