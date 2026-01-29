# NestGate Persistence Configuration Handoff

**Date**: January 29, 2026  
**Status**: 🟡 IN-MEMORY ONLY (Persistence needs evolution)  
**biomeOS Integration**: ✅ JSON-RPC working

---

## Current Status

NestGate JSON-RPC is integrated with NUCLEUS:
- ✅ `storage.store` - Accepts and confirms storage
- ✅ `storage.list` - Returns stored keys
- ❌ `storage.retrieve` - Returns `null` (data not persisted)
- ❌ `storage.exists` - Not implemented

The current implementation appears to use in-memory storage only.

## Environment Variables Detected

From NestGate source code analysis:

| Variable | Default | Purpose |
|----------|---------|---------|
| `NESTGATE_STORAGE_PATH` | `/var/lib/nestgate` or `./data` | Base storage directory |
| `NESTGATE_SOCKET` | XDG-compliant | Unix socket path |
| `NESTGATE_JWT_SECRET` | (required) | Security token |
| `NESTGATE_FAMILY_ID` | `default` | Family identification |

## Tested Configuration

```bash
NESTGATE_STORAGE_PATH=/home/eastgate/Development/ecoPrimals/phase2/biomeOS/data/nestgate \
NESTGATE_SOCKET=/run/user/1000/biomeos/nestgate-nat0.sock \
NESTGATE_JWT_SECRET=$(openssl rand -base64 48) \
FAMILY_ID=nat0 \
./nestgate server
```

**Result**: Directory created but remains empty. Data stored in memory only.

## Evolution Needed

1. **File-based persistence**: Implement actual file writes to `NESTGATE_STORAGE_PATH`
2. **SQLite backend**: Alternative persistent storage option
3. **Retrieve implementation**: Ensure `storage.retrieve` reads from persistent store
4. **Startup recovery**: Load existing data from disk on restart

## Current Available Methods

```
Storage:
  • storage.store(family_id, key, value) ✅
  • storage.retrieve(family_id, key) ❌ returns null
  • storage.delete(family_id, key) ?
  • storage.list(family_id, prefix?) ✅
  • storage.store_blob(family_id, key, data_base64) ?
  • storage.retrieve_blob(family_id, key) ?
  • storage.exists(family_id, key) ❌ not implemented
```

## Test Commands

```bash
# Store
echo '{"jsonrpc":"2.0","method":"storage.store","params":{"family_id":"nat0","key":"test:key","value":{"data":"value"}},"id":1}' | nc -U /run/user/1000/biomeos/nestgate-nat0.sock

# List
echo '{"jsonrpc":"2.0","method":"storage.list","params":{"family_id":"nat0"},"id":2}' | nc -U /run/user/1000/biomeos/nestgate-nat0.sock

# Retrieve (currently returns null)
echo '{"jsonrpc":"2.0","method":"storage.retrieve","params":{"family_id":"nat0","key":"test:key"},"id":3}' | nc -U /run/user/1000/biomeos/nestgate-nat0.sock
```

## Priority

**Medium** - In-memory storage works for validation, but production deployments need persistence.

---

**Handoff from**: biomeOS NUCLEUS Team  
**Handoff to**: NestGate Development Team

