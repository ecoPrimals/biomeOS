# Common Showcase Utilities

**Purpose**: Shared utilities for all showcase demos  
**Philosophy**: Runtime discovery, no hardcoding  

## Files

### discovery.sh
Runtime primal discovery library
- `discover_primal_bin()` - Find primal binary
- `primal_exists()` - Check if primal available
- `list_available_primals()` - Show all available
- `check_primal_health()` - Health check
- `wait_for_primal()` - Wait for startup

### Usage

```bash
source ../common/discovery.sh

# Check if beardog exists
if primal_exists "beardog"; then
    BEARDOG_BIN=$(discover_primal_bin "beardog")
    echo "Found beardog: $BEARDOG_BIN"
fi

# List all available
list_available_primals
```

### Principles

1. ✅ NO HARDCODED PATHS
2. ✅ NO HARDCODED PORTS
3. ✅ RUNTIME DISCOVERY ONLY
4. ✅ GRACEFUL FALLBACKS
5. ✅ CLEAR ERROR MESSAGES
