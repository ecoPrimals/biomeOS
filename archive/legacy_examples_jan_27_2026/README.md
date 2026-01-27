# Archived Legacy Examples

**Archived**: January 27, 2026  
**Reason**: Use HTTP/TCP ports instead of Unix sockets (violates TRUE PRIMAL architecture)

## Files

| File | Issue |
|------|-------|
| `mock_primal_server.rs` | Uses TCP ports for mock server |
| `atomic_orchestration_true_primal.rs` | Contains HTTP URLs |
| `full_integration_test.rs` | Uses HTTP for integration testing |
| `enhanced_functionality_demo.rs` | HTTP-based demo |
| `working_unified_demo.rs` | HTTP-based demo |
| `config_builder_demo.rs` | HTTP endpoint references |
| `biomeos_enhanced_demo.rs` | HTTP-based demo |

## Migration Path

These examples should be evolved to use:
- Unix sockets (`/run/user/$UID/biomeos/` or `/tmp/biomeos-$USER/`)
- JSON-RPC 2.0 protocol
- `biomeos_core::atomic_client::AtomicClient` for communication
- `biomeos_types::SystemPaths` for XDG-compliant path discovery

## Current Architecture

TRUE PRIMAL architecture requires:
- Port-free communication (Unix sockets only)
- Capability-based discovery via Songbird
- JSON-RPC 2.0 for all inter-primal communication
- XDG-compliant socket paths

See `crates/biomeos-core/src/atomic_client.rs` for the modern approach.

