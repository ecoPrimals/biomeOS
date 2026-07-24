# biomeOS Session Handoff — Wave 150x Connection Pooling + Crash-Loop Guard

**Date**: July 24, 2026
**Wave**: 150x
**Version**: 4.38
**Agent**: biomeOS (eastGate)

---

## Summary

Received Wave 150x cascade from eastGate overwatch. Primary finding: systemd
crash-loop divergence (29,081 restarts) that throttled AT&T gateway. Tower deep
analysis revealed UDS per-request connect/disconnect as the dominant LAN latency
factor (0.6ms per hop, 3.5× network cost).

## Work Completed

### 1. UDS Connection Pooling (`biomeos-core::ipc::ConnectionPool`)

Implemented a new connection pool that eliminates per-request connect/disconnect
overhead. The Neural API server already reads in a NDJSON loop (persistent
connections), but the client side was creating a fresh stream per call.

**Design:**
- `DashMap<String, VecDeque<PooledStream>>` keyed by endpoint Display string
- Max 4 idle connections per endpoint
- 30-second idle timeout (stale connections discarded)
- Automatic reconnect on stale connection (transparent retry)
- Byte-level line reading to avoid split/unsplit complexity

**Impact (projected):**
- LAN dispatch latency: 0.6ms → <0.1ms per hop (connection already established)
- Throughput: eliminates connect syscall + 3-way handshake overhead per request
- This is the key enabler for the chimera target: 97% IPC savings per hop

### 2. Service Crash-Loop Guard

Fixed `biomeos-beacon.service` on eastGate:
- Pointed `ExecStart` to depot binary (`infra/plasmidBin/primals/x86_64-unknown-linux-musl/biomeos`)
- Added `StartLimitBurst=5` / `StartLimitIntervalSec=300` systemd rate limiting
- Service remains disabled (intentional — start when deployment is ready)

### 3. LifecycleManager Assessment

Verified that biomeOS's internal LifecycleManager already has a complete
crash-loop breaker:
- `ResurrectionConfig::max_attempts` (configurable per-primal)
- Exponential backoff (`base_delay * 2^attempts`, capped at `max_delay`)
- Terminal state: `Apoptosis { reason: ResurrectionExhausted }`

The Wave 150x gap is specifically about **systemd-managed services** that bypass
the LifecycleManager — this is cellMembrane's scope (P0 assigned).

## Metrics

| Metric | Value |
|--------|-------|
| Tests passing | 8,494 |
| Test failures | 0 |
| Clippy warnings | 0 |
| New IPC pool tests | 2 |
| New source file | `crates/biomeos-core/src/ipc/pool.rs` |

## Upstream Gaps (for other primal teams)

| Gap | Owner | Priority | Detail |
|-----|-------|----------|--------|
| systemd crash-loop breaker | cellMembrane | P0 | Detect `NRestarts > threshold`, disable + alert |
| nestgate service unit fix | cellMembrane | P0 | Remove stale `--socket` flag |
| UDS connection pooling (client) | songBird | P1 | Mirror pool pattern for dispatch |
| `federation.broadcast` handler | songBird | P1 | skunkBat has client, songBird missing server |
| `duration_ms` truncation fix | songBird | P1 | `as_millis()` → `as_micros()` |
| bearDog rate limiting | bearDog | P1 | No UDS connection cap or backpressure |
| Enrollment replay tracking | bearDog | P1 | No timestamp window, no replay detection |
| Process spawn anomaly | skunkBat | P1 | No rapid-restart detection |

## biomeOS Chimera (P3 — Phase 0 Ready)

The connection pool enables Phase 0 of the chimera design. When Tower primals
(bearDog + songBird + skunkBat) are collapsed into a single process, the UDS
hops become function calls. The pool is the intermediate step that reduces IPC
cost without requiring architectural changes.

---

*End of session handoff — v4.38*
