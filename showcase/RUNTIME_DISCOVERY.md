# Runtime Discovery Guide

**Principle**: Zero hardcoding. Primals discover capabilities at runtime.

---

## 🎯 Core Concept

### What Primals Know
**Primals have ONLY self-knowledge**:
```bash
# A primal knows:
- "I provide 'storage' capability"
- "I serve on port 9020"
- "My health endpoint is /health"
- "I am healthy/unhealthy"

# A primal does NOT know:
- Other primals exist
- Other primals' endpoints
- How to coordinate with others
```

### What BiomeOS Knows
**BiomeOS discovers at runtime**:
```bash
# BiomeOS discovers:
- Which capabilities are available
- Who provides each capability
- How to communicate with each provider
- When to failover or load balance

# BiomeOS does NOT hardcode:
- Primal names
- Primal endpoints
- Primal APIs
```

---

## 🔍 Discovery Methods

### 1. Capability-Based Discovery

**Pattern**: Ask for capability, not implementation
```bash
# ❌ BAD - Hardcoded primal name
nestgate_url="http://localhost:9020"

# ✅ GOOD - Discover by capability
storage_provider=$(discover_capability "storage")
# Returns: URL of ANY primal providing "storage"
# Could be: NestGate, MinIO, S3, custom storage
```

**Example**: Multi-provider scenario
```bash
# Discover all storage providers
providers=$(discover_all_capabilities "storage")
# Returns: ["http://nestgate:9020", "http://minio:9000", "http://s3-gateway:9001"]

# BiomeOS can:
- Load balance across all three
- Failover if one goes down
- Route based on capability metadata (fast vs durable)
```

### 2. Service Registry Discovery

**Pattern**: Query central registry (if available)
```bash
# Check if Songbird (discovery service) is available
if discovery_service=$(discover_capability "discovery"); then
  # Use Songbird's registry
  providers=$(curl "$discovery_service/api/v1/services?capability=storage")
else
  # Fallback to other methods
  providers=$(mdns_discover "_primal._tcp.local")
fi
```

### 3. mDNS Discovery

**Pattern**: Automatic local network discovery
```bash
# Discover all primals on local network
primals=$(mdns_discover "_primal._tcp.local")

# Filter by capability
storage_primals=$(echo "$primals" | jq '.[] | select(.capabilities[] == "storage")')
```

### 4. Environment Variables (Dev Override)

**Pattern**: Override for development/testing only
```bash
# Development override
if [ -n "$STORAGE_ENDPOINT" ]; then
  provider="$STORAGE_ENDPOINT"  # Use override
else
  provider=$(discover_capability "storage")  # Use discovery
fi

# Production: Never set these variables
# Prod relies 100% on runtime discovery
```

---

## 🛠️ Discovery Functions

### Source: `common/discovery.sh`

```bash
#!/usr/bin/env bash
# Runtime discovery utilities

# Discover single provider for capability
discover_capability() {
  local capability="$1"
  local timeout="${2:-5}"  # 5 second default timeout
  
  # Try methods in order of preference
  local provider
  
  # 1. Environment override (dev only)
  provider=$(check_env_override "$capability")
  if [ -n "$provider" ]; then
    echo "$provider"
    return 0
  fi
  
  # 2. Service registry (Songbird)
  provider=$(query_service_registry "$capability" "$timeout")
  if [ -n "$provider" ]; then
    echo "$provider"
    return 0
  fi
  
  # 3. mDNS discovery
  provider=$(mdns_discover_capability "$capability" "$timeout")
  if [ -n "$provider" ]; then
    echo "$provider"
    return 0
  fi
  
  # 4. Failed to discover
  return 1
}

# Discover all providers for capability
discover_all_capabilities() {
  local capability="$1"
  
  # Aggregate from all sources
  local providers=()
  
  # Check service registry
  if registry=$(discover_capability "discovery"); then
    readarray -t registry_providers < <(
      curl -s "$registry/api/v1/services?capability=$capability" | jq -r '.[].endpoint'
    )
    providers+=("${registry_providers[@]}")
  fi
  
  # Check mDNS
  readarray -t mdns_providers < <(
    mdns_discover_all "_primal._tcp.local" | \
      jq -r ".[] | select(.capabilities[] == \"$capability\") | .endpoint"
  )
  providers+=("${mdns_providers[@]}")
  
  # Return unique providers
  printf '%s\n' "${providers[@]}" | sort -u
}

# Query service registry
query_service_registry() {
  local capability="$1"
  local timeout="$2"
  
  # Discover registry first
  local registry
  registry=$(check_env_override "discovery")
  if [ -z "$registry" ]; then
    registry=$(mdns_discover_capability "discovery" "$timeout")
  fi
  
  if [ -z "$registry" ]; then
    return 1
  fi
  
  # Query for capability
  curl -s --max-time "$timeout" \
    "$registry/api/v1/services?capability=$capability" | \
    jq -r '.[0].endpoint' 2>/dev/null
}

# mDNS discovery
mdns_discover_capability() {
  local capability="$1"
  local timeout="$2"
  
  # Use avahi-browse or dns-sd depending on platform
  if command -v avahi-browse &>/dev/null; then
    timeout "$timeout" avahi-browse -t -p "_primal._tcp" 2>/dev/null | \
      parse_avahi_output "$capability"
  elif command -v dns-sd &>/dev/null; then
    timeout "$timeout" dns-sd -B "_primal._tcp" 2>/dev/null | \
      parse_dns_sd_output "$capability"
  else
    # No mDNS available
    return 1
  fi
}

# Check environment override
check_env_override() {
  local capability="$1"
  
  # Map capability to env var
  case "$capability" in
    storage)   echo "${STORAGE_ENDPOINT:-}" ;;
    compute)   echo "${COMPUTE_ENDPOINT:-}" ;;
    security)  echo "${SECURITY_ENDPOINT:-}" ;;
    discovery) echo "${DISCOVERY_ENDPOINT:-}" ;;
    ai)        echo "${AI_ENDPOINT:-}" ;;
    *)         echo "" ;;
  esac
}
```

---

## 📝 Usage Examples

### Example 1: Store Data
```bash
#!/usr/bin/env bash
source common/discovery.sh

echo "Discovering storage provider..."
STORAGE=$(discover_capability "storage") || {
  echo "No storage provider found, using local filesystem"
  STORAGE="file://$(pwd)/data"
}

echo "Storing data to $STORAGE"
store_data "$STORAGE" "myfile.txt" "content"
```

### Example 2: Secure Compute Pipeline
```bash
#!/usr/bin/env bash
source common/discovery.sh

# Discover all needed capabilities
STORAGE=$(discover_capability "storage" || fallback_to_local)
SECURITY=$(discover_capability "security" || fallback_to_os_crypto)
COMPUTE=$(discover_capability "compute" || fallback_to_local_cpu)

# Build pipeline
echo "Pipeline:"
echo "  Storage:  $STORAGE"
echo "  Security: $SECURITY"
echo "  Compute:  $COMPUTE"

# Execute pipeline
data=$(fetch_data "$STORAGE" "input.csv")
encrypted=$(encrypt_data "$SECURITY" "$data")
result=$(process_data "$COMPUTE" "$encrypted")
decrypted=$(decrypt_data "$SECURITY" "$result")
store_result "$STORAGE" "output.csv" "$decrypted"

echo "Pipeline complete!"
```

### Example 3: Load Balanced Compute
```bash
#!/usr/bin/env bash
source common/discovery.sh

# Discover ALL compute providers
COMPUTE_PROVIDERS=($(discover_all_capabilities "compute"))

echo "Found ${#COMPUTE_PROVIDERS[@]} compute providers:"
printf '  - %s\n' "${COMPUTE_PROVIDERS[@]}"

# Distribute work across all providers
for i in "${!tasks[@]}"; do
  provider_idx=$((i % ${#COMPUTE_PROVIDERS[@]}))
  provider="${COMPUTE_PROVIDERS[$provider_idx]}"
  
  echo "Task $i -> $provider"
  submit_task "$provider" "${tasks[$i]}" &
done

wait  # Wait for all tasks
echo "All tasks complete!"
```

---

## 🚫 Anti-Patterns

### ❌ DON'T: Hardcode Primal Names
```bash
# BAD
if [ -f "/usr/bin/nestgate" ]; then
  use_nestgate
fi

# GOOD
if provider=$(discover_capability "storage"); then
  use_storage "$provider"
fi
```

### ❌ DON'T: Hardcode Endpoints
```bash
# BAD
NESTGATE_URL="http://localhost:9020"
curl "$NESTGATE_URL/api/v1/data"

# GOOD
STORAGE=$(discover_capability "storage")
curl "$STORAGE/api/v1/data"
```

### ❌ DON'T: Hardcode API Paths
```bash
# BAD
curl "$provider/api/v1/datasets"  # Assumes specific API

# GOOD
# Probe for API pattern
if curl -f "$provider/api/v1/datasets" 2>/dev/null; then
  api_path="/api/v1/datasets"
elif curl -f "$provider/datasets" 2>/dev/null; then
  api_path="/datasets"
else
  echo "Unable to discover API"
fi
```

### ❌ DON'T: Assume Single Provider
```bash
# BAD
STORAGE=$(discover_capability "storage")
# What if there are 3 storage providers?

# GOOD
STORAGE_PROVIDERS=($(discover_all_capabilities "storage"))
# Now can load balance, failover, etc.
```

---

## ✅ Best Practices

### 1. Always Provide Fallbacks
```bash
STORAGE=$(discover_capability "storage") || {
  echo "Warning: No storage provider, using local filesystem"
  STORAGE="file://$(pwd)/data"
}
```

### 2. Timeout All Discovery
```bash
# Don't hang forever
COMPUTE=$(discover_capability "compute" 5) || fallback
#                                        ^ 5 second timeout
```

### 3. Cache Discovery Results
```bash
# Discovery can be expensive, cache for session
if [ -z "$CACHED_STORAGE" ]; then
  CACHED_STORAGE=$(discover_capability "storage")
fi
STORAGE="$CACHED_STORAGE"
```

### 4. Log Discovery for Debugging
```bash
provider=$(discover_capability "storage")
echo "[$(date)] Discovered storage provider: $provider" >> discovery.log
```

### 5. Validate Discovered Providers
```bash
provider=$(discover_capability "storage")
if ! curl -f "$provider/health" >/dev/null 2>&1; then
  echo "Warning: Provider unhealthy, trying next..."
  provider=$(discover_capability "storage")  # Try again
fi
```

---

## 🎯 Why This Matters

### Scenario 1: Primal Evolution
```
Day 1: NestGate v1.0 has API at /api/v1/datasets
       biomeOS discovers and uses it ✅

Day 30: NestGate v2.0 changes API to /data/sets
        biomeOS probes, discovers new API, adapts ✅
        ZERO code changes needed ✅
```

### Scenario 2: Custom Primals
```
User creates "MyStorage" primal
  - Advertises "storage" capability
  - Uses different API entirely
  
biomeOS:
  - Discovers via mDNS
  - Probes API patterns
  - Uses MyStorage transparently ✅
  ZERO configuration needed ✅
```

### Scenario 3: Multiple Providers
```
Environment has:
  - NestGate (fast, local)
  - MinIO (durable, remote)
  - S3 (archive, cloud)

biomeOS discovers all three ✅
Routes based on requirements:
  - Hot data → NestGate
  - Warm data → MinIO
  - Cold data → S3
  
Automatic load balancing ✅
Automatic failover ✅
```

### Scenario 4: Zero-Config Expansion
```
New ToadStool instance joins LAN
  - Advertises "compute" via mDNS
  - biomeOS discovers automatically
  - Workload distributed to new instance
  
User experience:
  - Joined network
  - Computation got 2x faster
  - ZERO configuration ✅
```

---

## 🏗️ Integration with benchScale

Every showcase demo using runtime discovery can be validated:

```yaml
# topology.yaml
name: runtime-discovery-demo
tests:
  - name: discover-storage
    command: |
      provider=$(discover_capability "storage")
      [ -n "$provider" ] || exit 1
      
  - name: validate-discovered
    command: |
      provider=$(discover_capability "storage")
      curl -f "$provider/health"
      
  - name: use-discovered
    command: |
      provider=$(discover_capability "storage")
      echo "test" | store_data "$provider" "demo.txt"
```

---

## 📚 Reference

### Discovery Flow
```
1. Check environment override (dev only)
   ↓ (not found)
2. Query service registry (Songbird if available)
   ↓ (not found)
3. mDNS local network discovery
   ↓ (not found)
4. Return error or fallback
```

### Capability Names (Standard)
- `storage` - Data persistence
- `compute` - Computation/processing
- `security` - Encryption/authentication
- `discovery` - Service registry
- `ai` - AI/ML inference
- `routing` - Network routing
- `ui` - User interface

Custom capabilities welcome! Primals can advertise any capability string.

---

**Status**: Foundation Complete  
**Next**: Create `common/discovery.sh` implementation  
**Usage**: Source in all showcase demos

🔍 **Discover at runtime, adapt to evolution!** 🌱

