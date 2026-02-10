# NestGate Evolution Handoff - February 9, 2026

**Team**: NestGate (phase1/nestgate)
**Priority**: HIGH - Blocks model cache mesh sharing and Nest Atomic reliability
**Codebase**: `ecoPrimals/phase1/nestgate/`

---

## Context

NestGate is the storage and federation primal. It manages persistent data across the
mesh with ZFS integration on gate2 and filesystem fallback on Tower. Nest Atomic
(Tower + NestGate) is validated but has 4 known bugs that need upstream fixes.

Full bug analysis: `docs/handoffs/NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md`

---

## Known Bugs (Fix First)

### Bug 1: Inverted Boolean in CLI (CRITICAL)

**Symptom**: `nestgate daemon --socket-only` enables HTTP instead of disabling it.

**Root Cause**: CLI passes `use_socket_only` directly as `enable_http` parameter.

**Fix**: One line:
```rust
// BEFORE (broken):
let enable_http = config.socket_only;

// AFTER (fixed):
let enable_http = !config.socket_only;
```

**biomeOS workaround**: `start_nucleus.sh` works around this by starting NestGate
in daemon mode and accepting the HTTP listener.

### Bug 2: `storage.retrieve` Returns Null for Valid Keys

**Symptom**: Storing a value and immediately retrieving it returns `null`.

**Root Cause**: Likely a key prefix or namespace issue in the storage backend.

### Bug 3: ZFS Backend Assumption

**Symptom**: Some code paths assume ZFS is available even on non-ZFS systems.

**Fix**: Check for ZFS availability at startup; fall back to filesystem gracefully.

### Bug 4: Missing `storage.exists` Method

**Symptom**: No way to check if a key exists without attempting full retrieval.

**Fix**: Add `storage.exists` JSON-RPC method. biomeOS already calls this; NestGate
needs to implement it.

---

## Evolution Items

### 1. Fix All 4 Bugs (HIGH PRIORITY)

See details above. The inverted boolean is a 1-line fix. The others may require
investigation into the storage backend.

### 2. Model Cache Methods (MEDIUM PRIORITY)

**What**: Support model-specific storage operations for zero-download model sharing.

Methods needed:
- `model.register(model_id, metadata)` - Register a model in the mesh
- `model.exists(model_id)` - Check if model is cached locally or on any gate
- `model.locate(model_id)` - Return gate(s) that have this model
- `model.metadata(model_id)` - Return model size, format, hash

These are thin wrappers over `storage.*` with model-specific key prefixes.

**Estimated**: 100 lines

### 3. Cross-Gate Replication (FUTURE)

**What**: Replicate key-value data between NestGate instances on different gates.

**How**:
- Songbird mesh provides TCP transport between gates
- NestGate on gate A sends `storage.replicate(key, value)` to NestGate on gate B
- Conflict resolution: last-write-wins with vector clocks
- ZFS snapshots can be used for bulk replication

**Estimated**: 500 lines

### 4. Multi-Family Socket Support (NEW)

**What**: Accept `--family-id` flag, create `nestgate-{family_id}.sock`.

**Why**: Each family has its own storage namespace. Multi-family support requires
separate NestGate instances or family-scoped key prefixes.

**Estimated**: 10 lines (separate instances) or 50 lines (shared instance with namespacing)

### 5. `discover_capabilities` JSON-RPC Method

**Response**:
```json
{
  "capabilities": [
    "storage.store", "storage.retrieve", "storage.exists",
    "storage.delete", "storage.list",
    "model.register", "model.exists", "model.locate"
  ]
}
```

**Estimated**: 30 lines

---

## Test Matrix

| Test | Command | Expected |
|------|---------|----------|
| Health | `echo '{"jsonrpc":"2.0","method":"health","id":1}' \| nc -U .../nestgate.sock` | `{"status":"healthy"}` |
| Store | `echo '{"jsonrpc":"2.0","method":"storage.store","params":{"family_id":"test","key":"k","value":"v"},"id":1}' \| nc -U ...` | `{"stored":true}` |
| Exists | `echo '{"jsonrpc":"2.0","method":"storage.exists","params":{"family_id":"test","key":"k"},"id":1}' \| nc -U ...` | `{"exists":true}` |
| Retrieve | `echo '{"jsonrpc":"2.0","method":"storage.retrieve","params":{"family_id":"test","key":"k"},"id":1}' \| nc -U ...` | `{"value":"v"}` |
