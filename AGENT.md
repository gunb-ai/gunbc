# Agent Onboarding Guide — gunbc

This document is the short onboarding guide for contributors and agents. It points you to the canonical docs and summarizes the non-negotiable invariants in this repo.

## Installation

```bash
# Clone and bootstrap
git clone <repo>
cd gunbc
make install
```

The `make install` target bootstraps the repository:
1. Generates CLI entrypoints (`gunbc-codegen --mode=ensure`)
2. Generates `Makefile` and `.gitignore` (`gunbc-bootstrap --mode=ensure`)

After installation, run `make help` to see all available targets.

**Note:** The generated `Makefile` is gitignored. The handwritten `GNUmakefile` provides the `install` target and delegates everything else to the generated `Makefile`.

## Start Here

- `docs/handbook.md` for the conceptual map, pattern catalog, and e2e examples (single file — copy-friendly)
- `docs/design/v4/dsl-design.md` for the full DSL language specification
- `docs/design/service-codegen.md` for DSL-driven service codegen architecture
- `docs/design/overview.md` for design rationale and invariants
- `SPEC.md` for the formal IR specification
- `docs/design/testgen.md` for test generation and proof obligations

## Quick Context

gunbc is a **DSL-first workflow compiler** where **everything is a DAG**. The primary authoring surface is the `.dag` language — declarative definitions that compile to a typed Graph IR. The compiler pipeline is: `.dag` → parse → typecheck → lower → emit (Rust/Go/C/MIPS). The system aims for **structural soundness**: if a DAG validates, its wiring is correct.

### Compositional modeling

Every external system is modeled as a **composition of layered concerns** (TCP → TLS → HTTP → REST → provider → operation), where each layer imposes invariants on the generated code. Workflows name only the top layer; the compiler composes all layers into transport code, mocks, and test obligations. DSL annotations (`@rest`, `@auth`, `@endpoint`, `@permissions`, `@idempotent`) are the mechanism — each annotation adds constraints that compose additively. Where the Rust substrate currently hand-wires what the DSL can derive (credential chains, transport triplets), those are active consolidation targets. See `docs/handbook.md` § "Compositional Modeling Philosophy" for the full treatment with examples.

## Repo Map

| Path | Purpose |
| --- | --- |
| `dsl/` | **Primary authoring surface** — all `.dag` source files |
| `dsl/services/` | Service definitions (REST, Shell): gcp, github, cargo, git, llm |
| `dsl/tools/` | Tool workflows: clippy, gist, codegen, makegen, etc. |
| `dsl/pipelines/` | Pipeline compositions: ci |
| `core/daglang/` | DSL compiler: parse → typecheck → lower → emit |
| `core/ir/` | Core IR types, patterns, transport model, resource system |
| `core/exec/` | Execution engine, DryRun interception, simulation |
| `core/codegen/` | CLI and test generation |
| `core/test/` | MockSpec and test utilities |
| `lib/transport/` | Canonical I/O boundary; a few bootstrap/generator crates do direct I/O by exception (see `TODO/TODONE/clippy-pragma-audit.md`) |
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

- All runtime DAG world I/O happens through `TransportOps::Execute` nodes; build-time generators and a small set of bootstrap/config loaders are explicit exceptions (see `TODO/TODONE/clippy-pragma-audit.md`).
- Boundaries and entrypoints are inferred from unconnected ports.
- Tool handles are capability-based. When used, they flow through `tool:<id>` ports.
- Tool ports are excluded from user-facing workflow signatures.
- Errors are explicit; there are no silent fallbacks or warning-only failures.
- External systems are layered compositions — each DSL annotation (`@rest`, `@auth`, `@endpoint`, etc.) adds invariants that the compiler enforces in generated code, mocks, and tests.
- Generated files are never committed — the compiler extracts all output paths from `content_upsert` and `@outputs` annotations, propagates them to the tool registry and `.gitignore`, and CI enforces that no generated file is tracked in git.

## Common Tasks

### DSL-first (primary path)

- **Add a new REST/Shell service:** Create `dsl/services/<provider>/<name>.dag` with `service` block and `operation` definitions. Identify the full layer stack (protocol, auth, provider, operation) and express each layer's invariants via annotations: `@endpoint` (provider), `@auth` (auth scheme), `@rest`/`@shell` (transport), `@permissions` (scopes), `@idempotent`/`@readonly` (behavioral properties), `@mock_response` (test data). Each annotation composes — the compiler generates transport code reflecting all layers.
- **Add a new tool workflow:** Create `dsl/tools/<name>.dag` — import services, define `fn` (pure) and `func` (effectful) blocks. Use `uses` declarations for resource/capability requirements — the compiler resolves them transitively.
- **Add a new pipeline:** Create `dsl/pipelines/<name>.dag` — import tools, define `pipeline` block with `stage` dependencies.

### Framework internals (rare)

- Add a new pattern: `core/ir/src/patterns/` and `core/ir/src/patterns/mod.rs`.
- Add a new transport: `core/ir/src/transport/` plus executor support in `lib/transport/`.
- Extend the emit pipeline: `core/daglang/daglang-emit/src/` (add `service_emit` functions per backend).

## Testing

```bash
cargo test
cargo test -p gunbc-ir
cargo test -p gunbc-ir -- --nocapture
```

## Related Projects

The `the-gunbai` repo contains the original design rationale, long-form design docs referenced by `SPEC.md`, and the **Understanding pattern** — the compositional modeling system that gunbc's DSL annotations are inspired by. Key inspirational patterns from gunbai:

- **Understanding = structured data about external systems** — behaviors, constraints, assumptions, unknowns, and explicit dependencies. Each Understanding composes with others via `depends_on` with behavior-scoped resolution.
- **Automatic derivation** — behavioral properties (`ReadOnly`, `Idempotent`, `FailsWhen`) automatically generate block I/O, mock specs, and contract tests. No manual `TestgenTargetDef` per target.
- **Layered semantic composition** — REST depends on HTTP; curl depends on network/DNS/TCP. Each layer overrides or extends the layer below. Transitive dependency resolution is automatic.
- **LanguageUnderstanding** — language-agnostic specs mapped to Rust/Python/TypeScript via structured type/syntax/naming tables. Same generator, multiple backends.
- **External dependency modeling** — tools declare runtime requirements (`uses net: Network`) and the system resolves prerequisites transitively.

gunbc's DSL achieves ~80% of this via interface contracts, annotation composition, and `uses` declarations. The remaining gap is in the Rust substrate, where graph builders hand-wire what the DSL could derive. Active consolidation lanes target this gap (see `TODO/tasks.md`).
