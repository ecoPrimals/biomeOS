# NestGate Persistence Configuration Handoff

**Date**: January 29, 2026  
**Status**: 🟡 IN-MEMORY ONLY (Persistence needs evolution)  
**biomeOS Integration**: ✅ JSON-RPC working  
**Priority**: Medium - Production deployments need persistence

---

## Executive Summary

NestGate JSON-RPC is successfully integrated with NUCLEUS and responding to storage commands. However, data is only stored in-memory and is lost on restart. The `storage.retrieve` method returns `null` even for keys that were just stored successfully.

**Working**:
- ✅ `storage.store` - Accepts data and returns success confirmation
- ✅ `storage.list` - Returns list of stored keys
- ✅ JSON-RPC socket communication

**Not Working**:
- ❌ `storage.retrieve` - Returns `null` for all keys (data not persisted)
- ❌ `storage.exists` - Method not implemented
- ❌ Data persistence to filesystem

---

## Current Status

### Environment Configuration Tested

```bash
NESTGATE_STORAGE_PATH=/home/eastgate/Development/ecoPrimals/phase2/biomeOS/data/nestgate \
NESTGATE_SOCKET=/run/user/1000/biomeos/nestgate-nat0.sock \
NESTGATE_JWT_SECRET=$(openssl rand -base64 48) \
FAMILY_ID=nat0 \
./nestgate server
```

**Result**: 
- Socket created successfully at XDG-compliant path
- Directory `/data/nestgate/` created but remains empty
- All operations appear to use in-memory HashMap only

### Test Commands and Results

```bash
# 1. STORE - Works ✅
$ echo '{"jsonrpc":"2.0","method":"storage.store","params":{"family_id":"nat0","key":"test:key","value":{"data":"hello world"}},"id":1}' | nc -U /run/user/1000/biomeos/nestgate-nat0.sock

# Response:
{"jsonrpc":"2.0","result":{"key":"test:key","success":true},"id":1}

# 2. LIST - Works ✅
$ echo '{"jsonrpc":"2.0","method":"storage.list","params":{"family_id":"nat0"},"id":2}' | nc -U /run/user/1000/biomeos/nestgate-nat0.sock

# Response:
{"jsonrpc":"2.0","result":{"keys":["test:key"]},"id":2}

# 3. RETRIEVE - FAILS ❌
$ echo '{"jsonrpc":"2.0","method":"storage.retrieve","params":{"family_id":"nat0","key":"test:key"},"id":3}' | nc -U /run/user/1000/biomeos/nestgate-nat0.sock

# Response:
{"jsonrpc":"2.0","result":{"data":null},"id":3}
```

---

## Analysis

### Hypothesis 1: Dual Storage Paths
The `storage.store` method may be writing to one internal structure (e.g., a transaction log or index) while `storage.list` queries that index, but `storage.retrieve` attempts to read from a filesystem path that was never written to.

### Hypothesis 2: Missing Persistence Hook
The store operation may be designed to batch writes asynchronously, and the actual file write is never triggered or the filesystem backend is disabled.

### Hypothesis 3: Configuration Not Fully Applied
The `NESTGATE_STORAGE_PATH` environment variable may be read but the internal storage engine may not be properly initialized with it.

---

## Evolution Requirements

### Priority 1: Fix storage.retrieve
```rust
// Current behavior (hypothetical)
fn retrieve(&self, family_id: &str, key: &str) -> Option<Value> {
    // Attempts to read from filesystem path that doesn't exist
    let path = self.storage_path.join(family_id).join(key);
    fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok())
}

// Expected behavior
fn retrieve(&self, family_id: &str, key: &str) -> Option<Value> {
    // Should read from same location as store writes
    self.data_store.get(&format!("{}:{}", family_id, key)).cloned()
}
```

### Priority 2: Implement Filesystem Persistence
```rust
impl NestGateStorage {
    fn store(&mut self, family_id: &str, key: &str, value: Value) -> Result<()> {
        // 1. Store in memory
        self.cache.insert(format!("{}:{}", family_id, key), value.clone());
        
        // 2. Persist to filesystem
        let path = self.storage_path
            .join(family_id)
            .join(sanitize_key(key));
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
        
        Ok(())
    }
    
    fn retrieve(&self, family_id: &str, key: &str) -> Result<Option<Value>> {
        // 1. Check cache first
        if let Some(value) = self.cache.get(&format!("{}:{}", family_id, key)) {
            return Ok(Some(value.clone()));
        }
        
        // 2. Fall back to filesystem
        let path = self.storage_path
            .join(family_id)
            .join(sanitize_key(key));
        
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            let value: Value = serde_json::from_str(&data)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
```

### Priority 3: Startup Recovery
```rust
impl NestGateStorage {
    fn load_from_disk(&mut self) -> Result<()> {
        for entry in fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let family_id = entry.file_name().to_string_lossy().to_string();
            
            for key_entry in fs::read_dir(entry.path())? {
                let key_entry = key_entry?;
                let key = key_entry.file_name().to_string_lossy().to_string();
                let data = fs::read_to_string(key_entry.path())?;
                let value: Value = serde_json::from_str(&data)?;
                
                self.cache.insert(format!("{}:{}", family_id, key), value);
            }
        }
        Ok(())
    }
}
```

### Priority 4: Add storage.exists Method
```rust
fn exists(&self, family_id: &str, key: &str) -> Result<bool> {
    let cache_key = format!("{}:{}", family_id, key);
    if self.cache.contains_key(&cache_key) {
        return Ok(true);
    }
    
    let path = self.storage_path.join(family_id).join(sanitize_key(key));
    Ok(path.exists())
}
```

---

## JSON-RPC Method Inventory

### Working Methods
| Method | Parameters | Returns |
|--------|------------|---------|
| `storage.store` | `{family_id, key, value}` | `{key, success: true}` |
| `storage.list` | `{family_id, prefix?}` | `{keys: [...]}` |

### Broken Methods
| Method | Parameters | Expected | Actual |
|--------|------------|----------|--------|
| `storage.retrieve` | `{family_id, key}` | `{data: {...}}` | `{data: null}` |
| `storage.exists` | `{family_id, key}` | `{exists: bool}` | Method not found |

### Recommended New Methods
| Method | Parameters | Returns | Purpose |
|--------|------------|---------|---------|
| `storage.delete` | `{family_id, key}` | `{success: bool}` | Remove data |
| `storage.count` | `{family_id, prefix?}` | `{count: u64}` | Statistics |
| `storage.config` | `{}` | `{path, engine, family}` | Debug config |

---

## Integration Context

### NUCLEUS Architecture
```
NUCLEUS = Tower + Node + Nest

Tower Atomic: BearDog + Songbird (Security + Network)
Node Atomic:  Tower + Toadstool  (Security + Compute)
Nest Atomic:  Tower + NestGate   (Security + Storage)  ← YOU ARE HERE
```

### biomeOS Usage Pattern
```
Consumer → Neural API → capability.call("storage", "store", data)
                ↓
        Semantic Translation
                ↓
        NestGate JSON-RPC → storage.store
```

### Deployment Graph
NestGate is registered in `graphs/nest_deploy.toml` with capabilities:
- `storage.store`
- `storage.retrieve`
- `storage.list`
- `storage.delete`

---

## Test Validation Script

Save and run this to verify persistence is working:

```bash
#!/bin/bash
# validate_nestgate_persistence.sh

SOCKET="/run/user/1000/biomeos/nestgate-nat0.sock"
TEST_KEY="persist:$(date +%s)"
TEST_VALUE='{"test":"persistence","timestamp":"'$(date -Iseconds)'"}'

echo "=== NestGate Persistence Validation ==="

# Store
echo "1. Storing data..."
STORE_RESULT=$(echo '{"jsonrpc":"2.0","method":"storage.store","params":{"family_id":"nat0","key":"'$TEST_KEY'","value":'$TEST_VALUE'},"id":1}' | nc -U "$SOCKET")
echo "   Store result: $STORE_RESULT"

# Retrieve immediately
echo "2. Retrieving immediately..."
RETRIEVE_RESULT=$(echo '{"jsonrpc":"2.0","method":"storage.retrieve","params":{"family_id":"nat0","key":"'$TEST_KEY'"},"id":2}' | nc -U "$SOCKET")
echo "   Retrieve result: $RETRIEVE_RESULT"

# Check if retrieve worked
if echo "$RETRIEVE_RESULT" | grep -q '"data":null'; then
    echo "   ❌ FAILED: Data not retrieved (in-memory only)"
else
    echo "   ✅ SUCCESS: Data retrieved correctly"
fi

# List
echo "3. Listing keys..."
LIST_RESULT=$(echo '{"jsonrpc":"2.0","method":"storage.list","params":{"family_id":"nat0"},"id":3}' | nc -U "$SOCKET")
echo "   List result: $LIST_RESULT"

if echo "$LIST_RESULT" | grep -q "$TEST_KEY"; then
    echo "   ✅ Key found in list"
else
    echo "   ❌ Key not found in list"
fi

echo "=== Validation Complete ==="
```

---

## Contact

**Handoff from**: biomeOS NUCLEUS Team  
**Handoff to**: NestGate Development Team  
**Date**: January 29, 2026

**For questions**: Review `docs/handoffs/` for related primal integration patterns.

---

## References

- `graphs/nest_deploy.toml` - NestGate deployment graph
- `graphs/nucleus_complete.toml` - Full NUCLEUS deployment
- `specs/NUCLEUS_DEPLOYMENT_SPEC.md` - Architecture overview
- `docs/SOCKET_DISCOVERY.md` - XDG-compliant socket patterns
