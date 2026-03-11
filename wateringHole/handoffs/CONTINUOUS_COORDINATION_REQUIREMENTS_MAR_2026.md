# Handoff: Continuous Coordination Mode for biomeOS

**From**: ludoSpring scaffold + neuralAPI niche analysis
**To**: biomeOS graph executor maintainers
**Date**: March 10, 2026
**Priority**: High — blocks first continuous niche (game engine)

---

## Context

The game engine niche is the first coordination pattern requiring biomeOS to
run a primal graph **continuously** rather than transactionally. This handoff
captures the exact requirements, references the whitepaper design, and lists
the code locations that need modification.

Reference: `whitePaper/neuralAPI/08_NICHE_API_PATTERNS.md`
Reference: `ludoSpring/specs/GAME_ENGINE_NICHE_SPECIFICATION.md`

---

## What exists today

biomeOS supports four coordination modes:

| Mode | File | Status |
|------|------|--------|
| Sequential | `crates/biomeos-graph/src/parser.rs` | Implemented |
| Parallel | `crates/biomeos-graph/src/parser.rs` | Implemented |
| ConditionalDAG | `crates/biomeos-graph/src/parser.rs` | Implemented |
| Pipeline | `crates/biomeos-graph/src/parser.rs` | Implemented |

All existing deploy graphs (`graphs/*.toml`) use `Sequential` coordination.
All modes are run-to-completion: the graph executes once and terminates.

---

## What the game engine niche needs

### 1. Continuous coordination pattern

A fifth enum variant in `CoordinationPattern`:

```rust
pub enum CoordinationPattern {
    Sequential,
    Parallel,
    ConditionalDAG,
    Pipeline,
    Continuous,  // NEW
}
```

Parser extension to recognize `coordination = "Continuous"` in TOML graphs.

### 2. Tick configuration in graph TOML

```toml
[graph.tick]
target_hz = 60
max_accumulator_ms = 250
budget_warning_ms = 14
```

New fields on the parsed `GraphDefinition`:

```rust
pub struct TickConfig {
    pub target_hz: u32,
    pub max_accumulator_ms: u64,
    pub budget_warning_ms: u64,
}
```

### 3. ContinuousExecutor

A new executor alongside the existing `NeuralExecutor` (or as a mode within
it). Key characteristics:

- **Fixed timestep loop**: accumulates real time, drains in fixed-size ticks
- **Budget-aware node execution**: each node has `budget_ms`, exceeded nodes
  use their previous tick's output
- **Node output cache**: `HashMap<NodeId, NodeOutput>` persisted across ticks
- **Feedback edges**: marked as `feedback_to = ["target_node"]` in TOML,
  excluded from topological sort, read from cache at tick N+1

### 4. Session lifecycle

Continuous niches have sessions (not one-shot executions):

```rust
pub enum SessionState {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
}
```

biomeOS needs:
- `start_session(graph_id)` → spawns primals, allocates resources, begins ticking
- `pause_session(session_id)` → freezes clock, primals idle
- `resume_session(session_id)` → resumes clock
- `stop_session(session_id)` → graceful shutdown, save state

### 5. Metrics export for Neural API learning

The continuous executor must emit per-tick telemetry for the Pathway Learner:

| Metric | Frequency | Purpose |
|--------|-----------|---------|
| `node_latency_ms` | Every tick, per node | Budget optimization |
| `budget_violation` | On occurrence | Overload detection |
| `frame_time_ms` | Every tick | Overall health |
| `staleness_count` | Every tick | Quality degradation tracking |
| `accumulator_overshoot_ms` | When > 0 | Systemic overload |

---

## Files to modify

### Must modify

| File | Change |
|------|--------|
| `crates/biomeos-graph/src/parser.rs` | Add `Continuous` variant, parse `[graph.tick]` |
| `crates/biomeos-graph/src/executor.rs` (or new file) | `ContinuousExecutor` |
| `crates/biomeos-graph/src/lib.rs` | Export new types |
| `config/capability_registry.toml` | Already has ludoSpring entries (done) |

### Must create

| File | Content |
|------|---------|
| `graphs/game_engine_tick.toml` | Continuous tick graph (see 08_NICHE_API_PATTERNS.md) |
| `niches/game-engine.toml` | Niche manifest |
| `crates/biomeos-graph/src/session.rs` | Session lifecycle |
| `crates/biomeos-graph/src/tick_clock.rs` | Fixed timestep clock |

### Already done (from ludoSpring scaffold)

| File | What |
|------|------|
| `config/capability_registry.toml` | ludoSpring "game" domain registered |
| `graphs/ludospring_deploy.toml` | Deploy graph (Sequential, for spawning) |
| `crates/biomeos-atomic-deploy/src/capability_domains.rs` | ludoSpring domain entry |

---

## Design constraints

### Budget reuse, not retries

Transactional graphs can retry failed nodes. Continuous graphs MUST NOT
retry within a tick — the tick deadline is absolute. Instead:

- Use previous output (staleness)
- Skip the node entirely (degradation)
- Log for the Pathway Learner to optimize across ticks

### Feedback edges are not cycles

Standard DAG validation rejects cycles. Feedback edges (marked
`feedback_to`) must be excluded from cycle detection because they connect
tick N output to tick N+1 input, not within a single tick.

### Clock must not drift

The tick clock should use a fixed-timestep accumulator pattern. If real time
exceeds `max_accumulator_ms`, excess time is dropped (the game slows down
rather than trying to catch up). This prevents the "spiral of death" where
catching up causes more overload.

### Pause must be zero-cost

When a session is paused, no tick processing occurs. The clock accumulator
freezes. Primals receive an idle signal but are not despawned (preserving
warm caches for resume).

---

## Testing strategy

### Unit tests

- `TickClock`: verify accumulator math, fixed timestep, spiral-of-death
  prevention
- `ContinuousExecutor`: verify node execution order, feedback edge handling,
  budget violation fallback
- `SessionState`: verify state machine transitions

### Integration tests

- Run a minimal 2-node continuous graph at 10 Hz for 100 ticks, verify all
  ticks execute, verify metrics emitted
- Run a continuous graph where one node exceeds budget, verify fallback to
  cached output
- Run a feedback-edge graph, verify tick N+1 sees tick N's output

### Validation experiment

`ludoSpring/experiments/exp005_continuous_niche_validation` (to be created):
- Runs a toy game loop through biomeOS's continuous executor
- Measures tick jitter, budget adherence, feedback latency
- Outputs a `ValidationResult` per the standard ludoSpring pattern

---

## Dependency chain

```
1. biomeOS: Add Continuous to CoordinationPattern enum
2. biomeOS: Implement TickClock
3. biomeOS: Implement ContinuousExecutor (uses TickClock)
4. biomeOS: Implement SessionState lifecycle
5. biomeOS: Create game_engine_tick.toml graph
6. biomeOS: Create niches/game-engine.toml manifest
7. ludoSpring: Create exp005 validation experiment
8. Integration: Run game_engine_tick.toml through ContinuousExecutor
```

Steps 1-4 are biomeOS-internal. Steps 5-6 are configuration. Step 7 is
ludoSpring. Step 8 validates the cross-primal composition.

---

## Open questions

1. **Should ContinuousExecutor be a mode of NeuralExecutor or a separate
   type?** NeuralExecutor already has learning hooks — duplicating them would
   be wasteful. Recommendation: add a `run_continuous` method to
   NeuralExecutor.

2. **Should feedback edges be declared in the graph TOML or inferred from
   the Neural API's Pathway Learner?** For correctness, they should be
   declared explicitly. The Learner can later *suggest* new feedback edges
   that humans approve.

3. **How does session state interact with biomeOS's existing primal
   lifecycle?** The deploy graph (Sequential) spawns primals. The tick graph
   (Continuous) runs after deploy completes. This is a two-phase model:
   deploy → tick. biomeOS needs a way to chain a Sequential graph into a
   Continuous one.

---

**This handoff is the critical path for the game engine niche. The game
engine cannot exist without Continuous coordination in biomeOS.**
