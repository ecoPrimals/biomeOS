# capability.call routing contract

> Stable semantics for biomeOS Neural API `capability.call` and semantic fallback (`domain.operation` methods).  
> Complements primalSpring `docs/DISCOVERY_WIRE_CONTRACT.md` (Neural API `capability.*` surface).

## Pipeline (three phases)

Every successful local route executes:

1. **Route** — Resolve the semantic name `domain.operation` to a concrete JSON-RPC method (and optional provider hint) using the capability translation registry and/or dotted-capability rules.
2. **Resolve** — Resolve a transport endpoint for the capability domain via `discover_capability(domain)` (registry + discovery cache + optional lazy rescan).
3. **Forward** — Send the JSON-RPC request to the resolved endpoint and return the primal’s `result` payload.

Cross-gate and Tower Atomic paths preserve the same phase *meaning* but may combine transports (see below).

## Inputs and outputs

| Phase | Input | Output |
|--------|--------|--------|
| **Route** | `capability` / dotted `capability`, `operation`, optional `args` | Translation hit: provider id + actual method name; or direct route: forward method = `operation` |
| **Resolve** | Capability domain string (e.g. `crypto`) | `primary_endpoint`, primal list, health hints |
| **Forward** | Endpoint, method name, `args` | JSON value from the provider (Neural API returns it as JSON-RPC `result`) |

## Error semantics

| Failure | When | Client-visible behavior |
|---------|------|-------------------------|
| **Route** | Missing `capability` / `operation` (and no dotted operation), malformed params | JSON-RPC error (invalid params / internal) before any forward |
| **Resolve** | No provider for domain, discovery failure | Error from `discover_capability` (typically internal `-32603` with message) |
| **Forward** | Transport error, timeout, primal JSON-RPC error | Preserved primal JSON-RPC `error` when applicable; otherwise internal error |

Unregistered `gate` (when `gate` ≠ `local`) fails in the gate preflight with an explicit error (no silent local fallback).

## Fallback: semantic `domain.operation`

JSON-RPC methods of the form `domain.operation` that are **not** in the Neural API route table are rewritten to:

`capability.call` with `{ "capability": "domain", "operation": "operation", "args": <original params> }`.

Optional `_routing_trace` in the **top-level** `params` object is copied onto the synthetic `capability.call` params when present.

## Gate-scoped routing

- `gate: "local"` — Skips remote gate lookup; runs the same Route → Resolve → Forward pipeline locally.
- `gate: "<label>"` — Resolves a registered remote Neural API endpoint; forwards a nested `capability.call` there. Trace (when enabled) records gate endpoint resolution and forward timing.

## Translations: registration and matching

- **Registration**: `capability.register` may include `semantic_mappings` (`{ "op": "actual.method" }`). Entries are stored as `"{domain}.{op}"` → provider + actual method + socket.
- **Runtime / TOML**: Additional translations load from deployment/registry initialization (see `CapabilityTranslationRegistry`).
- **Matching**: Lookup key is `"{capability}.{operation}"` after normalizing dotted vs explicit `operation`.

## Observability: `_routing_trace`

Clients may set `"_routing_trace": true` on `capability.call` params. On success, the JSON-RPC response may include a top-level `_routing_trace` object (alongside `result`) with phase names and timing. When `false` or omitted, no trace field is added.

See Rust type `RoutingPhase` in `crates/biomeos-atomic-deploy/src/handlers/capability_routing.rs`.
