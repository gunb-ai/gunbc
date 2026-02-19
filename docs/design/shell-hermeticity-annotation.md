# Shell Hermeticity Annotation (Producer-Level)

## Problem

`TransportRequest::Shell(ShellRequest)` erases whether the producer is hermetic
(local, deterministic, no external network/auth) or external.

Example:

- `GitRequest::ls_files()` and `CargoCommand::build()` are typically hermetic.
- `gh gist create` is external.

Once lowered to `ShellRequest`, these become structurally identical and
downstream consumers cannot classify test scope or risk correctly.

## Design Goal

Attach hermeticity at the producer boundary, and preserve it through lowering
to transport execution and test categorization.

## Proposed IR Shape

Extend `ShellRequest` metadata in `core/ir/src/transport/mod.rs` with producer
semantics (not inferred from command strings):

```rust
pub enum Hermeticity {
    Hermetic,
    External,
}

pub struct ShellProducerSemantics {
    pub producer: String,        // e.g. "git.ls_files", "github.gist.create"
    pub hermeticity: Hermeticity,
    pub idempotent: Option<bool>,
    pub rationale: Option<String>,
}
```

Add optional field on `ShellRequest`:

```rust
pub semantics: Option<ShellProducerSemantics>
```

## Rules

1. Producer APIs creating shell requests should set `semantics`.
2. Generic/raw shell constructors may leave `semantics=None`.
3. Validation layer can enforce strict mode:
   - reject `semantics=None` for workflows requiring hermetic classification.
4. Testgen categorization should use `semantics.hermeticity` when present.

## Why Producer-Level

- Avoid brittle command-string heuristics (`git`/`gh` prefix checks).
- Keep classification stable across argument changes.
- Preserve intent authored at domain API level.

## Migration Plan

1. Add `ShellProducerSemantics` + `Hermeticity` types.
2. Thread `semantics` through `ShellRequest` builders and cloning/serde.
3. Annotate known producers first:
   - `core/ir/src/transport/git.rs` -> `Hermetic`.
   - GitHub/Gist shell producers -> `External`.
   - Cargo/local tool wrappers -> `Hermetic` (where applicable).
4. Add validation/test hooks:
   - unit tests asserting semantics propagation.
   - testgen categorization tests (hermetic vs external).
5. Enable strict-mode enforcement after producer coverage is complete.

## Non-Goals

- No command parsing to infer hermeticity.
- No transport-variant-level global default that hides producer intent.

