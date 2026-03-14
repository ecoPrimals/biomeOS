> **HISTORICAL** — This handoff predates v2.37. See CURRENT_STATUS.md for latest.

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

### Bug 1: Inverted Boolean in Socket-Only Mode (CRITICAL) -- PATCHED DOWNSTREAM

**Status**: PATCHED in biomeOS fork, awaiting upstream fix
**Detailed Report**: `nestgate/UPSTREAM_BUG1_INVERTED_BOOLEAN_FEB09_2026.md`

The Feb 9 NestGate evolution added `socket_only: bool` (default `true`) alongside the existing `enable_http: bool` field. The computed `use_socket_only` (which is `true`) is then passed directly to `run_daemon()` whose 4th parameter is `enable_http: bool`. Result: `--socket-only` starts HTTP mode.

```rust
// BUG: use_socket_only (true) passed as enable_http parameter
crate::commands::service::run_daemon(port, &bind, dev, use_socket_only)  // <-- inverted

// DOWNSTREAM PATCH (one-line fix):
crate::commands::service::run_daemon(port, &bind, dev, !use_socket_only)  // <-- negated
```

NestGate team's investigation called this a "false positive" because they analyzed the pre-evolution code (which only had `enable_http` and was correct). The bug was introduced by their own evolution adding the dual-boolean pattern.

**Recommended upstream fix**: Remove the `socket_only` field entirely; revert to the pre-evolution pattern where only `enable_http` exists (defaults false = socket-only). See detailed report for three fix options.

### Bug 2: `storage.retrieve` Returns Null -- RESOLVED

**Status**: FIXED (was caused by Bug 1)

When NestGate started in HTTP mode (due to Bug 1), the `storage.*` JSON-RPC methods were only available via the Unix socket server codepath, which was never started. With the Bug 1 patch applied, `storage.retrieve` correctly returns stored data:

```bash
# storage.retrieve now WORKS:
{"jsonrpc":"2.0","result":{"data":{"model":"TinyLlama","size":2100}},"id":4}
```

NestGate also added enhanced logging throughout the store/retrieve pipeline in this evolution, which will help diagnose any future issues.

### Bug 3: ZFS Backend Assumed -- RESOLVED BY NESTGATE

**Status**: FIXED upstream in this evolution

NestGate added `capabilities.rs` with runtime ZFS detection (`zpool version`). When ZFS is not available, it gracefully falls back to standard filesystem mode:

```
INFO 🗄️  Storage backend: Filesystem (universal compatibility)
INFO    Works on ANY filesystem: ext4, NTFS, APFS, btrfs, etc.
```

---

## What biomeOS Needs From NestGate (Next Evolution)

### REMAINING: Fix Inverted Boolean Upstream

The downstream one-line patch (`!use_socket_only`) works but should be resolved properly in NestGate. The cleanest fix is to revert to the pre-evolution single-boolean pattern:

```rust
// Remove socket_only field, keep only enable_http (defaults false = socket-only)
Daemon {
    #[arg(long)]
    enable_http: bool,
}
```

See `UPSTREAM_BUG1_INVERTED_BOOLEAN_FEB09_2026.md` in NestGate root for full analysis and three fix options.

### RESOLVED: `storage.retrieve` -- Works with Bug 1 patch
### RESOLVED: ZFS Backend Fallback -- `capabilities.rs` added upstream
### RESOLVED: `storage.exists` Method -- Implemented and routed upstream

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

## Current Status (Post-Patch)

With the downstream Bug 1 patch applied, Nest Atomic is fully operational:

- `storage.exists` -- Efficient existence check (new from this evolution)
- `storage.store` -- Stores with `size_bytes` in response (enhanced)
- `storage.retrieve` -- Returns actual data (no longer null)
- `model_cache::find_on_mesh()` -- Uses two-phase `exists` -> `retrieve` lookup
- NestGate socket-only mode runs correctly at `/run/user/1000/biomeos/nestgate.sock`
- Validated live on Tower with family_id `8ff3b864a4bc589a`

## Evolution Path

1. **Upstream Bug 1 fix**: NestGate resolves the inverted boolean, biomeOS removes downstream patch
2. **Mesh discovery**: `resolve` on Tower finds Mistral-7B on gate2 via NestGate
3. **Model transfer**: Songbird-based model file transfer (rsync-like, BearDog-encrypted)
4. **Blob storage**: Use `storage.store_blob` for smaller model files directly in NestGate
5. **Deduplication**: NestGate can deduplicate identical models across gates (same SHA256)
6. **Auto-sync**: Background task syncs model manifests across gates periodically
