# Archived Tests - January 2026

**Date Archived**: January 26, 2026
**Reason**: Tests disabled during Tower Atomic evolution

## Disabled Tests

These tests were disabled during the Tower Atomic integration phase
because they required refactoring for the new capability.call pattern.

| Test | Crate | Status |
|------|-------|--------|
| integration_tests.rs | biomeos-ui | Needs capability.call update |
| protocol_integration_tests.rs | biomeos-core | Needs refactoring |
| squirrel_integration_test.rs | biomeos-core | Squirrel evolution pending |
| e2e_tests.rs | biomeos-spore | Needs Tower Atomic |
| graph_execution_tests.rs | biomeos-atomic-deploy | Partially integrated |
| fault_injection_tests.rs | biomeos-atomic-deploy | Future work |
| collaborative_intelligence_e2e.rs | biomeos-graph | Future work |
| websocket_integration.rs | biomeos-api | WebSocket pending |
| atomic_lineage_deployment_test.rs | tests | Lineage evolution |
| e2e_tests.rs | tests | Needs update |
| health_monitoring_integration_tests.rs | tests | Health monitoring |
| real_primal_integration.rs | tests | Real primal testing |
| chaos_tests.rs | tests | Chaos engineering |

## TLS Debug Scripts

Scripts used during TLS 1.3 debugging, now superseded by
`deploy_tower_atomic.sh` and production validation.

| Script | Purpose |
|--------|---------|
| compare_tls_trace.sh | Compare TLS traces |
| https_test_suite.sh | HTTPS endpoint testing |

## Reactivation

To reactivate a test:
1. Copy from archive to original location
2. Remove `.disabled` extension
3. Update for capability.call pattern
4. Run `cargo test` to validate
