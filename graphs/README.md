# biomeOS Deployment Graphs

**Updated**: April 13, 2026
**Status**: 42 deploy graphs (4 core + 36 domain + 2 Pipeline), all XDG-compliant, all tests passing, 290+ capability translations across 26 domains

---

## Active Deployment Graphs

| Graph | Purpose | Nodes | Validated |
|-------|---------|-------|-----------|
| `nucleus_complete.toml` | Full NUCLEUS (BearDog + Songbird + Toadstool + NestGate + Squirrel) | 11 | Yes |
| `ecosystem_full_bootstrap.toml` | Full ecosystem bootstrap with optional NestGate | 11 | Yes |
| `tower_atomic_bootstrap.toml` | Tower Atomic (BearDog + Songbird) | 5 | Yes |
| `gate2_nucleus.toml` | Full NUCLEUS on a second gate | 12 | Yes |

All graphs use `${XDG_RUNTIME_DIR}/biomeos/` and `${FAMILY_ID}` placeholders — zero hardcoded paths.
`DISCOVERY_ADDRESS`/`DISCOVERY_ENDPOINT` default to the local Neural API (`127.0.0.1:8080`)
and are overridden at runtime by `$BIOMEOS_DISCOVERY_ENDPOINT` or capability-based socket scanning.

### Execute via Neural API

```bash
echo '{"jsonrpc":"2.0","method":"graph.execute","params":{"graph_id":"nucleus_complete"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api-$(biomeos family-id).sock
```

---

## Atomic Patterns

All deployment follows three atomic building blocks:

| Atomic | Primals | Capabilities |
|--------|---------|--------------|
| **Tower** | BearDog + Songbird | Crypto, TLS, HTTP, Discovery, Mesh, Relay |
| **Node** | Tower + Toadstool | + Compute, GPU |
| **Nest** | Tower + NestGate | + Storage, Persistence |
| **Full NUCLEUS** | All + Squirrel | + AI Orchestration |

**BearDog MUST start first** — all other primals depend on it for crypto.
Neural API is part of biomeOS and orchestrates capability routing across all primals.

---

## Capability Translations in Graphs

Each primal declares `capabilities_provided` — semantic-to-actual method mappings:

```toml
[nodes.capabilities_provided]
"crypto.encrypt" = "chacha20_poly1305_encrypt"
"mesh.status" = "mesh.status"
"stun.probe_port_pattern" = "stun.probe_port_pattern"
```

These align with the 260+ translations across 19 capability domains.

---

## Validation

7 unit tests in `neural_graph.rs` verify:
- All 4 core graphs parse correctly
- Zero hardcoded paths (`/tmp/`, `/run/user/1000/`, hardcoded family IDs)
- Relay-punch translations present in NUCLEUS graphs

```bash
cargo test -p biomeos-atomic-deploy -- test_parse_nucleus test_parse_ecosystem test_parse_tower test_parse_gate2 test_no_hardcoded test_all_deployment test_relay_punch
```

---

## Coordination Patterns

| Pattern | Method | Description | Example Graph |
|---------|--------|-------------|---------------|
| Sequential | `graph.execute` | Nodes execute in dependency order | `nucleus_complete.toml` |
| Parallel | `graph.execute` | Independent nodes execute concurrently | `ecosystem_full_bootstrap.toml` |
| ConditionalDag | `graph.execute` | DAG with `condition`/`skip_if` branching | `rootpulse_commit.toml` |
| Pipeline | `graph.execute_pipeline` | Streaming — items flow through mpsc channels | `streaming_telemetry_pipeline.toml` |
| Continuous | `graph.start_continuous` | Fixed-timestep tick loop (e.g., 60Hz) | `game_engine_tick.toml` |

### Pipeline Graphs (v2.43)

| Graph | Nodes | Description |
|-------|-------|-------------|
| `streaming_telemetry_pipeline.toml` | 3 | groundSpring sensor -> filter -> store |
| `pharmacology_etl_pipeline.toml` | 4 | compound fetch -> descriptors -> Lipinski filter -> docking score |

Pipeline graphs use `coordination = "pipeline"`. The PipelineExecutor wires bounded mpsc
channels between nodes — items flow through as soon as each node produces them.

## Archives

Previous archive directories (`stale_pre_v2_feb11_2026/`, `old_test_graphs_jan_2026/`,
`outdated_atomic_patterns/`) were cleaned during the v2.38-v2.40 evolution. Fossil records
are kept at `ecoPrimals/archive/`.

---

## Provenance Trio Graphs (March 2026)

### Deployment Graphs

| Graph | Purpose | Primals | Status |
|-------|---------|---------|--------|
| `loamspine_deploy.toml` | LoamSpine permanence primal deployment | LoamSpine (tarpc 9001 + JSON-RPC 8301) | Ready |
| `rhizocrypt_deploy.toml` | rhizoCrypt ephemeral DAG deployment | rhizoCrypt (tarpc 9400) | Ready |
| `sweetgrass_deploy.toml` | sweetGrass attribution primal deployment | sweetGrass (HTTP 8302 + tarpc 8091) | Ready |
| `provenance_trio_deploy.toml` | Full provenance stack (all three) | LoamSpine → rhizoCrypt → sweetGrass | Ready |

### Workflow Graphs

| Graph | Purpose | Primals | Status |
|-------|---------|---------|--------|
| `rootpulse_commit.toml` | RootPulse commit workflow (dehydrate → sign → store → commit → attribute) | rhizoCrypt + BearDog + NestGate + LoamSpine + sweetGrass | Defined |
| `provenance_pipeline.toml` | Universal provenance for any Spring experiment | rhizoCrypt + BearDog + NestGate + LoamSpine + sweetGrass | Defined |

### Provenance Trio Ports

| Primal | tarpc | JSON-RPC/HTTP | Capabilities |
|--------|-------|---------------|-------------|
| LoamSpine | 9001 | 8301 | permanence, spine, certificate, proof, commit |
| rhizoCrypt | 9400 | — | dag, session, merkle, dehydration, slice |
| sweetGrass | 8091 | 8302 | attribution, braid, provenance, contribution |

Deploy the trio: `graph.execute provenance_trio_deploy` (requires Tower running).

---

**Tests**: 7,784 passing | **Core graphs**: 4 | **Domain graphs**: 36 | **Pipeline graphs**: 2
