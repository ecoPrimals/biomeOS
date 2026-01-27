# Legacy HTTP Patterns Archive

**Archived**: January 27, 2026

These files used HTTP-based patterns that violate the TRUE PRIMAL architecture:
- HTTP ports for primal-to-primal communication
- `curl` commands to HTTP endpoints
- `reqwest` patterns

## Archived Files

1. **test_genetic_lineage_verification.sh**
   - Used: `BEARDOG_API="http://localhost:19000/api/v1"`
   - Should use: JSON-RPC over Unix sockets via BearDog

2. **QUICK_START_TOWER_DEPLOYMENT.md**
   - Used: `curl http://localhost:3000/api/v1/primals`
   - Should use: `scripts/test_federation.sh` or USB deploy scripts

## TRUE PRIMAL Replacements

- **Lineage verification**: `scripts/verify_sibling_lineage.sh` (offline) or `graphs/federation_verify_lineage.toml` (runtime)
- **Tower deployment**: USB `deploy.sh` scripts or `graphs/tower_atomic_bootstrap.toml`
- **Federation testing**: `scripts/test_federation.sh`

## Why Archived

The TRUE PRIMAL architecture mandates:
- Zero HTTP ports for primal-to-primal communication
- Unix sockets only for IPC
- JSON-RPC 2.0 protocol
- Capability-based discovery (no hardcoded endpoints)

See `specs/TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md` for details.

