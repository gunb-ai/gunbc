# Agent Onboarding Guide — gunbc

This document is the short onboarding guide for contributors and agents. It points you to the canonical docs and summarizes the non-negotiable invariants in this repo.

## Start Here

- `docs/handbook.md` for the conceptual map and pattern catalog
- `docs/design/overview.md` for design rationale and invariants
- `SPEC.md` for the formal IR specification
- `docs/design/testgen.md` for test generation and proof obligations

## Quick Context

gunbc is a Rust-based workflow IR. The core idea is that **everything is a DAG**: workflows, types, validations, and resource flows. The system aims for **structural soundness**: if a DAG validates, its wiring is correct.

## Repo Map

| Path | Purpose |
| --- | --- |
| `core/ir/` | Core IR types, patterns, transport model, resource system |
| `core/exec/` | Execution engine, DryRun interception, simulation |
| `core/codegen/` | CLI and test generation |
| `core/test/` | MockSpec and test utilities |
| `lib/transport/` | The only crate that performs direct I/O |
| `lib/tools/` | General-purpose tool wrappers (clippy, deps, gist) |
| `gunbc-dag/` | Repo-specific DAGs and CLI entrypoints (ci, makegen, codegen, testgen, bootstrap) |
| `docs/design/` | Design documentation |

## Refactor-Pressure Checklist (PR Gate)

- Single source of truth: new concepts must have exactly one authoritative definition.
- No stringly references: names of nodes/targets/resources must be typed or derived.
- No hidden env/IO: env vars, clock, platform, and FS handles only via env/resource nodes.
- No ambient globals: exec mode and policy flags are explicit inputs.
- Fast path declared: any freshness/check logic documents fast and slow paths.
- Generated code linting: fix IR or clippy config, never add `#[allow]` in generated output.

## Invariants That Matter

- All world I/O happens through `TransportOps::Execute` nodes.
- Boundaries and entrypoints are inferred from unconnected ports.
- Tool handles are capability-based. When used, they flow through `tool:<id>` ports.
- Tool ports are excluded from user-facing workflow signatures.
- Errors are explicit; there are no silent fallbacks or warning-only failures.

## Common Tasks

- Add a new pattern: `core/ir/src/patterns/` and `core/ir/src/patterns/mod.rs`.
- Add a new CLI tool: `core/ir/src/transport/cli.rs`, plus a wrapper crate under `lib/tools/` if needed.
- Add a new ToolDef for planning: `core/ir/src/transport/tool.rs` and `lib/tools/deps/` for deps.toml generation.
- Add a new repo-specific tool: `gunbc-dag/src/` plus a bin in `gunbc-dag/src/bin/`.
- Add a new transport: `core/ir/src/transport/` plus executor support in `lib/transport/`.

## Testing

```bash
cargo test
cargo test -p gunbc-ir
cargo test -p gunbc-ir -- --nocapture
```

## Related Projects

The `the-gunbai` repo contains the original design rationale and long-form design docs referenced by `SPEC.md`.
