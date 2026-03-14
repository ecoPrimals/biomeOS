> **HISTORICAL** — This handoff predates v2.37. See CURRENT_STATUS.md for latest.

# Songbird Evolution Handoff - February 9, 2026

**Team**: Songbird (phase1/songbird)
**Priority**: HIGH - Blocks pure runtime discovery and Plasmodium mesh
**Codebase**: `ecoPrimals/phase1/songbird/`

---

## Context

Songbird is the network primal. It provides HTTP/HTTPS, discovery, mesh relay,
STUN/hole-punching, and onion services. It currently works at 93% -- all major
features functional but three gaps prevent fully autonomous operation.

The AI Bridge was validated end-to-end: `Squirrel -> Songbird -> BearDog TLS -> Anthropic`.
This works via an explicit `HTTP_REQUEST_PROVIDER_SOCKET` env var bypass. Eliminating this
bypass requires Songbird to implement one new JSON-RPC method.

---

## Evolution Items

### 1. `discover_capabilities` JSON-RPC Method (HIGH PRIORITY)

**What**: Add a `discover_capabilities` JSON-RPC handler to Songbird's IPC server.

**Why**: When Squirrel (or any primal) scans sockets looking for capability providers,
it sends `{"jsonrpc":"2.0","method":"discover_capabilities","params":{},"id":1}` to each
socket. Songbird currently doesn't handle this method, causing the scanner to time out
after 10 seconds and fall back to explicit env vars.

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "primal": "songbird",
    "capabilities": [
      "http.request",
      "secure_http",
      "discovery.peers",
      "relay.status",
      "relay.connect",
      "stun.detect",
      "stun.bind",
      "mesh.status",
      "mesh.find_path",
      "punch.request"
    ]
  },
  "id": 1
}
```

**Where to implement**: In the IPC server's method dispatch (alongside `health`, `http.request`, etc.)

**Estimated**: 50 lines of Rust

**Validation**: After implementing, Squirrel should auto-discover Songbird without
`HTTP_REQUEST_PROVIDER_SOCKET` being set. Test:
```bash
# Should work WITHOUT HTTP_REQUEST_PROVIDER_SOCKET set
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"hello","model":"claude-3-haiku-20240307","max_tokens":10},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/squirrel.sock -w 15 -q 1
```

### 2. TLS BearDog Socket Discovery Alignment (MEDIUM PRIORITY)

**What**: `songbird-tls/src/crypto.rs` has its own `discover_beardog_socket()` function
that falls back to `/tmp/beardog-nat0.sock` or `/tmp/neural-api-nat0.sock`.

**Why**: When Songbird makes HTTPS requests, its TLS module needs BearDog for crypto.
The TLS module doesn't honor the `BEARDOG_SOCKET` env var that the rest of the ecosystem
uses, causing HTTPS failures when the fallback paths don't exist.

**Fix**: In `songbird-tls/src/crypto.rs`, change `discover_beardog_socket()` to:
1. Check `BEARDOG_SOCKET` env var first
2. Check `SONGBIRD_SECURITY_PROVIDER` env var
3. Try XDG runtime dir: `$XDG_RUNTIME_DIR/biomeos/beardog.sock`
4. Try `/run/user/$UID/biomeos/beardog.sock`
5. Only then fall back to `/tmp/` paths

**Estimated**: 30 lines changed in `crypto.rs`

**Validation**: Songbird HTTPS requests should work when `BEARDOG_SOCKET` is set:
```bash
BEARDOG_SOCKET=/run/user/1000/biomeos/beardog.sock \
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://httpbin.org/get"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird.sock -w 10 -q 1
```

### 3. Auto Mesh Peer Discovery (MEDIUM PRIORITY)

**What**: Currently `PLASMODIUM_PEERS` env var must be manually set for multi-gate discovery.
Songbird's `mesh.peers` should auto-discover bonded gates.

**Why**: Plasmodium queries remote gates via SSH because Songbird doesn't auto-discover peers.
Once Songbird knows about peers, Plasmodium can use JSON-RPC over Songbird mesh instead of SSH.

**How**: Beacon broadcast on local network (UDP multicast or mDNS) using the shared family seed
for authentication. Gates that share the same `.family.seed` respond to discovery probes.

**Estimated**: 200 lines

### 4. Multi-Family Socket Support (NEW)

**What**: Accept `--family-id` flag and create sockets as `songbird-{family_id}.sock`.

**Why**: Currently Songbird creates `songbird.sock`. For multi-family support (multiple
ecosystems on one machine), sockets must be family-scoped. A single Songbird instance
can serve one family; multiple instances serve multiple families.

**Also consider**: A single Songbird could serve multiple families if it tracks which
family a connection belongs to (via the socket path used to connect). This enables
the "shared primal" architecture where one Songbird coordinates both local Tower
and HPC Tower roles.

**Estimated**: 10-50 lines depending on approach

---

## Current Bypasses That Songbird Evolution Removes

| Bypass | Songbird Evolution | Status |
|--------|-------------------|--------|
| `HTTP_REQUEST_PROVIDER_SOCKET` env var | Item 1 (`discover_capabilities`) | Pending |
| HTTPS fails without symlinks | Item 2 (TLS socket alignment) | Pending |
| SSH-based Plasmodium queries | Item 3 (auto mesh peers) | Pending |
| Socket nucleation symlinks | Item 4 (multi-family sockets) | Pending |

---

## Test Matrix

| Test | Command | Expected |
|------|---------|----------|
| Health | `echo '{"jsonrpc":"2.0","method":"health","id":1}' \| nc -U .../songbird.sock` | `{"status":"healthy"}` |
| Capabilities | `echo '{"jsonrpc":"2.0","method":"discover_capabilities","id":1}' \| nc -U .../songbird.sock` | List of capabilities |
| HTTP | `echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"http://localhost:11434/api/tags"},"id":1}' \| nc -U .../songbird.sock` | Ollama tags |
| HTTPS | Same with `https://httpbin.org/get` | JSON response |
| Mesh | `echo '{"jsonrpc":"2.0","method":"mesh.status","id":1}' \| nc -U .../songbird.sock` | Peer list |
