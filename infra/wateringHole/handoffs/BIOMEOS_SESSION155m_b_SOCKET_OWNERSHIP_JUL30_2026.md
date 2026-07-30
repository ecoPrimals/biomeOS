# biomeOS Session 155m-b — Socket Ownership + Upstream Triage

**Date**: Jul 30, 2026
**Version**: v4.51
**Wave**: 155m
**Gate**: eastGate

---

## P2 Resolved: Socket Ownership on Creation

**Root cause**: biomeOS creates Unix sockets with `0o660` permissions but never
`chown`s them to the `membrane` group. On multi-user gates (sporeGate Sovereign CI),
processes running as different UIDs within the `membrane` group cannot connect because
the socket's group is the creating user's primary group — not `membrane`.

**Fix**: Centralized `apply_socket_ownership()` in `biomeos-core::ipc::listener`:
1. Sets `0o660` (owner+group read/write)
2. Resolves `MEMBRANE_SOCKET_GROUP` env var (default: `membrane`)
3. Parses `/etc/group` for GID resolution
4. `chown :<gid>` on the socket file
5. Non-fatal on failure (single-user deployments work unchanged)

**Directory fix**: `apply_dir_ownership()` sets `0o770` (adds execute for traversal)
on `$XDG_RUNTIME_DIR/membrane/` at nucleation time.

**Call sites unified** (3 duplicate `set_permissions` blocks removed):
- `TransportListener::bind_unix` (covers Neural API + biomeOS API)
- `CapabilityRegistry::serve_inner` (direct `UnixListener::bind`)
- Device management server (`biomeos-ui`)
- `SocketNucleation` directory creation

**Environment variable**: `MEMBRANE_SOCKET_GROUP=<group>` overrides default.

**Files changed**:
- `crates/biomeos-core/src/ipc/listener.rs` — new `apply_socket_ownership`, `apply_dir_ownership`, `resolve_group_id`
- `crates/biomeos-core/src/ipc/mod.rs` — export new functions
- `crates/biomeos-atomic-deploy/src/neural_api_server/listeners.rs` — remove duplicate chmod
- `crates/biomeos-api/src/unix_server.rs` — remove duplicate chmod
- `crates/biomeos-core/src/capability_registry/server.rs` — replace with centralized call
- `crates/biomeos-ui/src/device_management_server/mod.rs` — add ownership call
- `crates/biomeos-atomic-deploy/src/nucleation.rs` — directory ownership at nucleation

---

## Upstream Triage (NOT biomeOS Code)

### rootpulse.ledger (P2) — Owner: cellMembrane/sporeGate

`rootpulse.ledger` is a **gate health probe** in cellMembrane (`membrane-shadow/src/gate/health.rs`).
It checks for `.rootpulse_state.toml` which is written by a successful `rootpulse.commit`.
biomeOS has no `rootpulse.ledger` RPC method — this is a local state file probe.

**Fix**: Run `membrane rootpulse.commit` on sporeGate (or ensure cascade rootpulse succeeds).

### Sandbox False Positive (P2) — Owner: cellMembrane

cellMembrane's `spawn_primal_server` in `plasmid/mod.rs` spawns primals with:
```
Command::new(binary).arg("server").arg("--socket").arg(socket)
```

But biomeOS has **no `server` subcommand**. Valid modes: `neural-api` or `api`.
cellMembrane's own `ServerContract::BiomeosApi` knows the correct command is
`{binary} neural-api --socket {path}` — but `spawn_primal_server` doesn't use it.

**Fix**: Update `spawn_primal_server` to honor `MembraneService.server_contract`.
For biomeOS, use `neural-api --socket <path> --btsp-optional`.

### checksums.toml, sources.toml, tmpfiles.d, golgi hook

These are operational/infra items owned by sporeGate CI, cellMembrane, and golgiBody respectively.
No biomeOS code changes required.

---

## Validation

- Workspace builds clean (`cargo check --workspace`)
- 0 clippy warnings (pedantic, -D warnings)
- All tests pass for affected crates
- `cargo fmt` clean
- Zero `unsafe` blocks (rustix 1.x `Gid::from_raw` is safe)

---

## Status

**biomeOS**: v4.51 STANDBY. All biomeOS-owned P2s RESOLVED. Ready for redeploy.
