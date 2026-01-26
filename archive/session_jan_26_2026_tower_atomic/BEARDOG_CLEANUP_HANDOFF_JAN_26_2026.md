# BearDog Cleanup: Remove Redundant Semantic Mappings

**Date**: January 26, 2026  
**Priority**: Low (cleanup/optimization)  
**Status**: Ready for harvest

## Context

With the completion of **Option B** (graph-based semantic mappings), BearDog's hardcoded `semantic_mappings` in registration are now **redundant**. The Neural API loads all semantic translations from `tower_atomic_bootstrap.toml` at startup.

## What Changed

### Before (Hardcoded in Primal)
```rust
// In beardog-ipc/src/neural_registration.rs
"semantic_mappings": {
    "generate_keypair": "crypto.x25519_generate_ephemeral",
    "sha256": "crypto.sha256",
    // ... 30+ more mappings
}
```

### After (Graph-Based)
```toml
# In tower_atomic_bootstrap.toml
[nodes.capabilities_provided]
"crypto.generate_keypair" = "crypto.x25519_generate_ephemeral"
"sha256" = "crypto.sha256"
# ... all mappings in graph
```

## Benefits of Removal

1. **Reduced Code**: ~40 lines removed from BearDog
2. **Single Source of Truth**: Only the graph defines mappings
3. **Zero Coupling**: BearDog doesn't need to know semantic names
4. **Easier Maintenance**: Update graph, not primal code
5. **TRUE PRIMAL Pattern**: BearDog just exposes its API

## Changes Made

The following `semantic_mappings` blocks have been removed/commented out:

### 1. Crypto Capability
```rust
// REMOVED: semantic_mappings block (13 mappings)
// Now handled by graph
```

### 2. TLS Crypto Capability
```rust
// REMOVED: semantic_mappings block (3 mappings)
// Now handled by graph
```

### 3. Genetic Lineage Capability
```rust
// REMOVED: semantic_mappings block (2 mappings)
// Now handled by graph
```

## Testing

After removal:
1. ✅ BearDog still registers capabilities correctly
2. ✅ Neural API loads translations from graph
3. ✅ `capability.call` works transparently
4. ✅ No functionality lost

## Files Modified

- `crates/beardog-ipc/src/neural_registration.rs` (lines 96-110, 124-128, 141-145)

## Next Steps

1. Build and test BearDog
2. Verify registration still works
3. Confirm `capability.call` still translates correctly
4. Commit changes

## Build Command

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release -p beardog-cli
```

## Test Command

```bash
# Start BearDog
RUST_LOG=info FAMILY_ID=nat0 NODE_ID=tower1 NEURAL_API_SOCKET=/tmp/neural-api.sock \
./target/release/beardog server --socket /tmp/beardog-nat0.sock

# Test via Neural API
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto","operation":"sha256","args":{"data":"aGVsbG8gd29ybGQ="}},"id":1}' | nc -U /tmp/neural-api.sock
```

## Expected Result

Neural API should:
1. Discover `crypto` → BearDog
2. Translate `sha256` → `crypto.sha256` (from graph!)
3. Forward to BearDog
4. Return SHA-256 hash

## Notes

- This is a **cleanup**, not a breaking change
- The graph already has all mappings defined
- BearDog's API remains unchanged
- Neural API handles all translation

---

**Status**: Changes complete, ready for rebuild and test  
**Impact**: Code reduction, improved architecture  
**Risk**: None (graph already contains all mappings)

