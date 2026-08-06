# Primal CLI Bind Flag Standard

**Status**: PROPOSAL (Wave 155i)
**Owner**: biomeOS team
**Affects**: All primal binaries

## Problem

Primals use inconsistent flags for network binding:
- `--bind` (some primals)
- `--host` (some primals)
- `--bind-address` (songBird)
- `--http-port` (sweetGrass, loamSpine)
- `--port` (biomeOS, toadStool)

This prevents biomeOS from uniformly starting primals via `composition.start` because
socket path construction and health check targeting differ per-primal.

## Standard

All primals MUST accept these environment variables (already in `biomeos-types`):

| Variable | Purpose | Default |
|----------|---------|---------|
| `FAMILY_ID` | Family-scoped socket naming | from `.family.seed` |
| `XDG_RUNTIME_DIR` | Socket directory parent | `/run/user/$UID` |
| `BIND_ADDRESS` | TCP bind address | `127.0.0.1` |
| `PRIMAL_BIND_MODE` | Transport: `uds_only`, `dual` (`tcp_only` DEPRECATED) | `uds_only` |

All primals SHOULD accept these CLI flags (biomeOS already does):

| Flag | Short | Purpose | Env equivalent |
|------|-------|---------|----------------|
| `--bind-mode <MODE>` | `-m` | Transport selection | `PRIMAL_BIND_MODE` |
| `--port <PORT>` | `-p` | TCP port (when mode includes TCP) | `PRIMAL_TCP_PORT` |
| `--family-id <ID>` | `-f` | Override family ID | `FAMILY_ID` |

### Socket Path Convention

UDS sockets bind to:
```
$XDG_RUNTIME_DIR/membrane/{primal}-{family_id}.sock
```

### Health Check Contract

Every primal MUST respond to `health` JSON-RPC method on its socket with:
```json
{"jsonrpc": "2.0", "id": 1, "result": {"status": "ok"}}
```

### Adoption Path

1. biomeOS already implements this standard
2. Primals add `--bind-mode` and `--port` flags
3. Deprecated flags (`--bind`, `--host`, `--http-port`) emit a warning and map to standard
4. biomeOS `composition.start` uses standard flags for uniform primal startup

## Rationale

- Env vars are the primary mechanism (set by graph executor, systemd, containers)
- CLI flags are secondary (human operator, debugging)
- Socket path is deterministic from `(primal_name, family_id, XDG_RUNTIME_DIR)`
- biomeOS can start ANY primal without per-primal flag knowledge
