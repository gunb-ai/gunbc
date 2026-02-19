# H3 Design: Makegen Tool Registry from #[tool_target]

## Problem

Make target registration is distributed across hand-maintained lists. This causes drift between tool metadata and generated Make targets.

## Decision

Make `#[tool_target]` the single source of truth and build a generated registry that makegen consumes.

## Proposed Annotation

```rust
#[tool_target(
  id = "gist-recent",
  command = "gunbc-gist recent",
  category = "tool",
  default = false
)]
```

## Registry Shape

- `ToolTargetSpec { id, command, category, default, help, inputs }`
- Generated registry module exported to makegen.

## Invariants

- Unique `id` across all tool targets.
- Command must reference a registered binary.
- Inputs metadata must match CLI entrypoint definitions when available.

## Migration Plan

1. Finalize macro schema for `#[tool_target]`.
2. Generate registry artifact during build.
3. Replace hardcoded makegen target table with registry-driven rendering.
4. Add conflict/validation tests.

## Follow-up Implementation Tasks

- `H3.1` Extend `#[tool_target]` macro schema and validation.
- `H3.2` Emit generated `ToolTargetSpec` registry.
- `H3.3` Migrate makegen renderer to registry input only.
- `H3.4` Add uniqueness and binary-existence checks.
- `H3.5` Add golden snapshot for rendered Make targets.
