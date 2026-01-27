# Archived Legacy Binaries

**Archived**: January 27, 2026  
**Reason**: Violate TRUE PRIMAL port-free architecture or obsolete

## Files

| File | Issue |
|------|-------|
| `mock_primal_server.rs` | Uses TCP ports (violates TRUE PRIMAL) |
| `live_demo.rs` | Demo code, not production |
| `live_integration_demo.rs` | Demo code, not production |
| `byob-test.rs` | Testing utility, outdated |

## Context

These binaries were using HTTP/TCP ports instead of Unix sockets.
TRUE PRIMAL architecture requires port-free communication via Unix sockets.

Modern deployment uses:
- `./deploy_tower_atomic.sh` for Tower Atomic deployment
- `biomeos` UniBin for all operations
- Unix sockets for all inter-primal communication

