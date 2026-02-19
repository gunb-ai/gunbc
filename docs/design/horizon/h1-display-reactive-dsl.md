# H1 Design: Display Orchestration Reactive DSL

## Problem

Display orchestration currently assumes straight-line DAG execution. UI loops that react to runtime events and timer ticks require ad-hoc host logic outside the DSL.

## Decision

Adopt a minimal reactive DSL layer with two trigger types:

- `on(event_channel)` for event-driven updates.
- `tick(interval)` for periodic updates.

This is the minimum needed to model display loops without embedding arbitrary imperative runtimes.

## Proposed DSL Surface

```text
reactive display_pipeline {
  state model: DisplayState

  on channel("exec.progress") as evt {
    model = reduce_progress(model, evt)
    emit render(model)
  }

  tick every "250ms" {
    emit render(model)
  }
}
```

## IR and Runtime Shape

- Add `ReactiveSubDag` with typed channels and trigger blocks.
- Add channel endpoint types: `ChannelIn<T>`, `ChannelOut<T>`.
- Add scheduler-owned `Tick` event source.
- Preserve deterministic replay by recording event order + tick timestamps.

## Invariants

- Event handlers are pure DAG fragments.
- No blocking transport operations inside trigger blocks.
- Tick interval lower bound enforced (for example >= 50ms).
- Channel payload types must be registry-backed and serializable.

## Migration Plan

1. Introduce parser + IR nodes for `reactive`, `on`, and `tick`.
2. Lower to runtime event loop executor.
3. Port one existing display orchestration path.
4. Add replay test harness for deterministic snapshots.

## Follow-up Implementation Tasks

- `H1.1` Parser and AST support for `reactive/on/tick`.
- `H1.2` IR additions: `ReactiveSubDag`, typed channels.
- `H1.3` Lowerer support and runtime scheduler.
- `H1.4` Deterministic replay test framework.
- `H1.5` Migrate display workflow from host code into DSL.
