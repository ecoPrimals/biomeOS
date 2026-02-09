# NestGate Model Cache Integration Handoff

**Date**: February 9, 2026  
**Status**: biomeOS model cache LIVE, NestGate needs evolution  
**biomeOS Integration**: biomeos-core::model_cache module complete  
**Priority**: HIGH - NestGate bugs block mesh-wide model discovery

---

## Summary

Built a model cache layer in biomeOS (`biomeos-core::model_cache`) that manages AI model artifacts across the NUCLEUS mesh. The module is designed to use NestGate for distributed model registry when available, with graceful fallback to local filesystem.

**The biomeOS side is done. NestGate needs two bugs fixed to enable cross-gate model discovery.**

---

## What Works (biomeOS Side)

### Model Cache Module (`crates/biomeos-core/src/model_cache.rs`)
- **HuggingFace import**: Auto-discovers and registers models from `~/.cache/huggingface/hub/`
- **Symlink-aware scanning**: Correctly follows HF blob symlinks for real file sizes
- **Manifest persistence**: JSON manifest at `~/.biomeos/model-cache/manifest.json`
- **Format detection**: Automatically detects safetensors, GGUF, PyTorch formats
- **Resolution pipeline**: Local cache -> NestGate mesh -> NotFound
- **NestGate graceful degradation**: Works in filesystem-only mode when NestGate is offline
- **4 unit tests passing**

### CLI (`biomeos model-cache`)
- `import-hf` - Import all models from HuggingFace cache
- `list` - List all cached models with size and format
- `resolve <model-id>` - Check local and mesh for a model
- `register <model-id> <path>` - Register an arbitrary model directory
- `status` - Show cache status including NestGate connection and unregistered HF models

### Deployed and Tested
- **Tower (pop-os)**: TinyLlama/1.1B (2.1 GB) + stable-diffusion-v1-5 (4.1 GB) = 6.0 GB
- **gate2 (pop-os)**: TinyLlama/1.1B (2.1 GB) + Mistral-7B-Instruct (13.8 GB) = 15.5 GB

---

## NestGate Bugs Found

### Bug 1: Inverted Boolean in Socket-Only Mode (CRITICAL)

**File**: `nestgate/code/crates/nestgate-bin/src/cli.rs` line ~337

```rust
// CURRENT (BUG):
crate::commands::service::run_daemon(port, &bind, dev, use_socket_only)

// The 4th parameter is named `enable_http` in run_daemon():
pub async fn run_daemon(port: u16, bind: &str, dev: bool, enable_http: bool)

// When use_socket_only = true, enable_http receives true -> starts HTTP mode!
// FIX:
crate::commands::service::run_daemon(port, &bind, dev, !use_socket_only)
```

**Impact**: `nestgate daemon --socket-only` starts in HTTP mode instead of socket mode. The storage JSON-RPC methods (`storage.store`, `storage.retrieve`, etc.) are only available via the Unix socket server, not the HTTP server. This means NestGate's storage API is effectively unreachable.

### Bug 2: `storage.retrieve` Returns Null (KNOWN)

**Reference**: `docs/handoffs/NESTGATE_PERSISTENCE_HANDOFF.md`

```bash
# storage.store works:
{"jsonrpc":"2.0","result":{"key":"test:key","success":true},"id":1}

# storage.list works:
{"jsonrpc":"2.0","result":{"keys":["test:key"]},"id":2}

# storage.retrieve FAILS:
{"jsonrpc":"2.0","result":{"data":null},"id":3}
```

**Hypothesis**: `store` writes to one internal structure (index/transaction log) while `retrieve` reads from a different path (filesystem) that was never written to.

### Bug 3: ZFS Backend Assumed

NestGate's HTTP mode tries to execute `zpool list` on startup, which fails on systems without ZFS:

```
ERROR Command failed: zpool list -H -o name,size,alloc,free,health (exit code: 1)
ERROR Error output: The ZFS modules cannot be auto-loaded.
```

This is non-fatal but noisy. NestGate should gracefully skip ZFS when it's not available.

---

## What biomeOS Needs From NestGate

### Priority 1: Fix Socket-Only Mode
Fix the inverted boolean so `nestgate daemon --socket-only` actually starts the Unix socket JSON-RPC server with `storage.*` methods.

### Priority 2: Fix `storage.retrieve`
Make `storage.retrieve` return the data that `storage.store` wrote. The model cache module sends:

```json
{
  "method": "storage.store",
  "params": {
    "family_id": "nat0",
    "key": "model-cache:TinyLlama/TinyLlama-1.1B-Chat-v1.0",
    "value": { "model_id": "...", "gate_id": "pop-os", "size_bytes": 2202500000, ... }
  }
}
```

And later queries:

```json
{
  "method": "storage.retrieve",
  "params": {
    "family_id": "nat0",
    "key": "model-cache:TinyLlama/TinyLlama-1.1B-Chat-v1.0"
  }
}
```

This must return the stored value, not null.

### Priority 3: Filesystem Backend Fallback
When ZFS is not available, NestGate should use a simple filesystem backend (the code for this exists in `universal_storage/backends/filesystem/` but isn't wired up as default fallback).

### Priority 4: `storage.exists` Method
Currently returns "Method not found". The model cache can work without it (uses `retrieve` + null check) but a proper `exists` method would be more efficient.

---

## biomeOS Integration Architecture

```
biomeos model-cache resolve "TinyLlama/..."
    |
    v
ModelCache::resolve()
    |
    +-- Check local manifest (filesystem)
    |   +-- Found? Return ModelResolution::Local(path)
    |
    +-- Check NestGate via AtomicClient (when available)
    |   +-- AtomicClient::discover("nestgate")
    |   +-- client.call("storage.retrieve", {key: "model-cache:..."})
    |   +-- Found? Return ModelResolution::Remote(gate_id)
    |
    +-- Return ModelResolution::NotFound
```

When a model is registered locally, it's also pushed to NestGate:

```
ModelCache::register_model()
    |
    +-- Scan directory (follow symlinks, calculate sizes)
    +-- Write to local manifest.json
    +-- AtomicClient::call("storage.store", {key: "model-cache:...", value: entry})
```

---

## Files Modified (biomeOS)

| File | Change |
|------|--------|
| `crates/biomeos-core/src/model_cache.rs` | **NEW** - Full model cache module |
| `crates/biomeos-core/src/lib.rs` | Added `pub mod model_cache` |
| `crates/biomeos/src/main.rs` | Added `model-cache` CLI subcommand |
| `crates/biomeos/src/modes/model_cache.rs` | **NEW** - CLI handler |
| `crates/biomeos/src/modes/mod.rs` | Added `pub mod model_cache` |
| `crates/biomeos-spore/src/spore/types.rs` | Added `family_id` to `SporeConfig` (fixed pre-existing build error) |
| `crates/biomeos-spore/src/spore/core.rs` | Pass `family_id` in `SporeConfig` constructors |
| `crates/biomeos-spore/src/spore/config.rs` | Use `family_id` named param in format string |

---

## Test Validation

```bash
# On any gate:
biomeos model-cache import-hf     # Import HuggingFace models
biomeos model-cache list           # See all cached models
biomeos model-cache resolve "TinyLlama/TinyLlama-1.1B-Chat-v1.0"  # Check availability
biomeos model-cache status         # NestGate connection + cache stats
```

---

## Evolution Path

Once NestGate bugs are fixed:

1. **Mesh discovery works**: `resolve` on Tower would find Mistral-7B on gate2 via NestGate
2. **Model transfer**: Add Songbird-based model file transfer (rsync-like, BearDog-encrypted)
3. **Blob storage**: Use `storage.store_blob` for smaller model files directly in NestGate
4. **Deduplication**: NestGate can deduplicate identical models across gates (same SHA256)
5. **Auto-sync**: Background task that syncs model manifests across gates periodically
