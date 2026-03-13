# biomeOS Deployment Graphs

**Updated**: March 12, 2026
**Status**: 26 deploy graphs (4 core + 22 domain), all XDG-compliant, all tests passing

---

## Active Deployment Graphs

| Graph | Purpose | Nodes | Validated |
|-------|---------|-------|-----------|
| `nucleus_complete.toml` | Full NUCLEUS (BearDog + Songbird + Toadstool + NestGate + Squirrel) | 11 | Yes |
| `ecosystem_full_bootstrap.toml` | Full ecosystem bootstrap with optional NestGate | 11 | Yes |
| `tower_atomic_bootstrap.toml` | Tower Atomic (BearDog + Songbird) | 5 | Yes |
| `gate2_nucleus.toml` | Full NUCLEUS on a second gate | 12 | Yes |

All graphs use `${XDG_RUNTIME_DIR}/biomeos/` and `${FAMILY_ID}` placeholders — zero hardcoded paths.

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

These align with the 170+ translations in `capability_translation.rs`.

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

## Archives

| Archive | Contents | Count |
|---------|----------|-------|
| `archive/stale_pre_v2_feb11_2026/` | Pre-v2.0 graphs (broken parsing, hardcoded paths, stale capabilities) | 30 |
| `archive/old_test_graphs_jan_2026/` | Bonding test graphs from January 2026 | 8 |
| `archive/outdated_atomic_patterns/` | Pre-atomic-pattern graphs from before Jan 19, 2026 | 15 |

These are kept as fossil record — do not delete.

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

**Tests**: 4,033 passing | **Core graphs**: 4 | **Domain graphs**: 26 | **Archived**: 53
