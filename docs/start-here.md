# Start Here

Orientation guide for working in the gunbc repo. Read this before starting a branch.

**Audience**: LLM agents and contributors working on features, fixes, or refactoring.
**Goal**: Ship features that land cleanly by following the patterns that work.

---

## What is gunbc

A DSL-first workflow compiler where everything is a DAG. You write `.dag` files that
declare types, data, and workflows. The compiler lowers them to executable graphs.
Rust is the substrate — it compiles, executes, and emits artifacts. It does not own
domain logic, policy data, or rendering decisions.

The long-term vision is an SDLC pipeline where GitHub issues drive a claim-based
worker fleet through design → implementation → review → testing stages, all modeled
as DAG workflows. See `docs/design/sdlc/mega-modeling-design.md` for the full design.

---

## Architecture in 60 seconds

```
.dag files (DSL)                    Rust substrate
─────────────────                   ──────────────
dsl/std/         types + data       core/daglang/     compiler (55k LOC)
dsl/extdeps/     external facts     core/ir/          IR types (51k LOC)
dsl/config/      repo policy        core/exec/        DAG executor (13k LOC)
dsl/tools/       workflows          lib/transport/    I/O boundary (6k LOC)
                                    gunbc-dag/        tool glue + extern impls
```

**Data flows one way**: `.dag` declarations → compiler → lowered DAG → executor → artifacts.
Rust never decides what lints to deny or how workflows are ordered. The model decides.

**Key numbers** (Feb 2026):
- 13 tools discovered from DSL structural inference
- 8 extern bridge functions remaining (509 lines, documented elimination plan)
- 2,984 passing tests in gunbc-dag alone
- Zero clippy warnings workspace-wide

---

## How to develop a feature

### Step 1: Model first, build never (until the model holds)

Write types and data declarations in `.dag` files. No `func` items. No Rust code.

```
// extdeps/foo.dag — "What is Foo?"
type FooSpec { id: String, version: String }
type FooCapability = Read | Write | Admin
data spec: FooSpec = { id: "foo", version: "2.1" }

// config/foo_policy.dag — "What do we require?"
import extdeps.foo { FooSpec, FooCapability }
type FooRule { capability: FooCapability, required: Bool }
data rules: List<FooRule> = [ { capability: Read, required: true } ]
```

**Litmus test**: If the types don't compose cleanly without `func` items or Rust,
you don't understand the domain yet. A `.dag` file with only types and data costs
nothing to create and nothing to delete.

This step typically takes 30 minutes and saves days.

### Step 2: Check if the pattern already exists

Before writing anything new, check if testgen, makegen, or pragma already solved the
same structural problem. Common patterns:

| You need | Look at |
|----------|---------|
| Generate a config file | `tools/pragma.dag` → `content_upsert` pattern |
| Declare build targets | `config/build_targets.dag` → `CoreWorkflow` data |
| Model an external tool | `extdeps/clippy.dag` → tautological facts |
| Enforce a repo invariant | `config/arch_rules.dag` → tiered invariants |
| Auto-discover from DSL | `dsl_registry.rs` → structural entrypoint inference |
| Auto-register a component | `#[testgen_target]` → inventory-based discovery |

Propagating an existing pattern is always faster than inventing a new one.

### Step 3: Write the design note (if non-trivial)

For anything touching the compiler, IR, executor, or transport boundary:

1. Create a design doc in `docs/design/v4/` or `docs/design/modeling/`
2. State: what changes, why, what breaks if it's wrong
3. Identify affected match sites (new enum variants radiate to 6-9 files)
4. Get review before writing code

For tool-layer changes (new `.dag` files, new extern impls): the Step 1 model IS
the design note. If types compose, proceed.

### Step 4: Implement with fail-closed defaults

When the compiler encounters something it cannot handle:

| Do this | Not this |
|---------|----------|
| Return a typed error | Emit `/* unsupported */` comment |
| Add explicit match arm | Use `_ =>` catch-all |
| Hard error on missing symbol | Return empty string |
| Fail the build | Silently degrade |

If you add a new enum variant, update ALL match sites in the same commit.
Use `cargo clippy --all-targets -- -D warnings` to catch stragglers.

### Step 5: Verify alignment

Before pushing:

```bash
cargo test --workspace                           # all tests pass
cargo clippy --all-targets -- -D warnings        # zero warnings
```

If you changed DSL tools or outputs, the drift tests in
`gunbc-dag/tests/tool_registration.rs` will catch misalignment between DSL
declarations and Rust mirrors.

---

## Repo invariants

These are structural properties of the codebase. Every successful feature respects
all of them. Every bug that takes days to find traces back to violating one.

### I1. One source of truth

Every fact lives in exactly one place. If data exists as both a Rust const array and
a DSL data declaration, one will drift. Either delete the duplicate or add a drift test
that fails when they diverge.

**How this works in practice**: `config/build_targets.dag` declares 20 workflows.
`makegen/registry.rs` consumes them. `tool_registration.rs` has a drift test ensuring
the two agree. Change the `.dag` file, the test catches any mismatch.

### I2. Generated files never committed

All tool outputs are `.gitignore`d. Exception: bootstrap seed files (`.gitignore`,
`clippy.toml`, `deps.toml`) declared in `config/arch_rules.dag`. CI tests verify:
- `no_generated_files_committed()` — `git ls-files` finds no tracked outputs
- `all_tool_outputs_gitignored()` — `git check-ignore` passes for every output path

### I3. All I/O through transport boundary

Runtime I/O goes through `TransportOps::Execute`. No `std::fs` or `std::process::Command`
outside `lib/transport/`. Exemptions are declared in `config/arch_rules.dag` with
invariant ID (I6) and rationale. `clippy.toml` enforces this via `disallowed_methods`.

### I4. Errors are explicit, never silent

No silent fallbacks. No panics for unhandled cases. No catch-all match arms that
swallow variants. Every failure path produces a typed error. When the compiler
encounters an unsupported construct, it emits a `LoweringError` or `ResolveError`,
not a comment or empty string.

### I5. Structure until the boundary

Functions return structured types (`MarkdownDoc`, `ClippyTomlConfig`, `MakeTarget`),
not `String`. Rendering to `String` happens once, at the final output step. This
enables validation, composition, multi-format rendering, and testing without string
parsing.

### I6. Closed sets use enums, not strings

If the set of valid values is known at compile time, use a sum type:
```
type PackageManager = Apt | Brew | Cargo    // not: type PackageManagerId = String
```
The compiler catches missing match arms. String IDs push validation to runtime.

### I7. New syntax participates in all pipeline stages

Adding a new declaration form (like `extern func`) to the parser means also adding it
to: typechecking, lowering, endpoint registration, edge wiring, emission, and
resolution. Partial support creates invisible data flow gaps that surface as silent
missing-data bugs at runtime.

### I8. Proven patterns propagate to structural peers

When a pattern works in one subsystem (e.g., `inventory` auto-discovery for testgen,
`TestRenderer` trait for structured rendering), apply it to structurally identical
peers before building new features. The codebase has 13 rendering systems — the ones
using structured IR work well; the ones using raw `format!()` are consolidation targets.

### I9. Semantic information preserved at boundaries

When a producer knows something (e.g., hermeticity, key/value types, capability
identity), that information is carried through the pipeline as structure, not erased
into untyped strings. Reconstructing erased information via heuristics downstream is
always more expensive than preserving it.

### I10. Invariants enforced by types, not convention

If an invariant matters, make it structurally impossible to violate. `ReachableDag<T>`
wraps a DAG so emitters cannot access unreachable nodes. This is better than a
`compute_reachable_node_ids()` cleanup pass that every caller must remember to invoke.

### I11. Model-first, build second

Write types and data declarations before writing executable code. If the types don't
compose without `func` items, the domain isn't understood well enough to implement.
Modeling in `.dag` files (types + data only) costs minutes. Building Rust modules,
binaries, and test suites costs days — and may be thrown away if the model doesn't hold.

### I12. Root causes over workarounds

When the same subsystem needs a second workaround, stop and fix the root cause.
Workarounds accumulate faster than they're cleaned up. Two workarounds cost more
total effort than one root-cause fix.

### I13. Metadata erasure is semantics-preserving

Deleting non-semantic metadata from a compiled graph must not change observable
behavior. If a node carries metadata (pipeline structure, output path annotations,
profiling tags), the system behaves identically with or without it. This is the
litmus test for whether something is truly "metadata" vs "hidden behavior" -- if
removing it changes results, it's not metadata, it's a bug.

### I13. Metadata erasure is semantics-preserving

Deleting non-semantic metadata from a compiled graph must not change observable
behavior. If a node carries metadata (pipeline structure, output path annotations,
profiling tags), the system behaves identically with or without it. This is the
litmus test for whether something is truly "metadata" vs "hidden behavior" — if
removing it changes results, it's not metadata, it's a bug.

---

## DSL modeling patterns

Detailed pattern catalog with examples: **[`docs/modeling.md`](modeling.md)**

Quick summary of the layer taxonomy:

```
dsl/
├── std/           Layer 0 — Universal types and pure functions
├── extdeps/       Layer 1 — Facts about external systems (no opinions)
├── config/        Layer 2 — Our repo's rules (composing Layer 0 + 1)
└── tools/         Layer 3 — Executable workflows (consuming all layers)
```

**Import direction**: `tools/ → config/ → extdeps/ → std/`. Never backwards.

Key patterns from the catalog:

| Pattern | Example | When to use |
|---------|---------|-------------|
| Tautological data | `std/languages.dag` — "What is Rust?" | Universal facts about external systems |
| Tiered invariants | `config/arch_rules.dag` — I6/I7/I8/M7 | Repo rules with scoped exemptions |
| Policy composition | `config/clippy_policy.dag` | Deriving artifacts from extdeps + invariants |
| Workspace-as-extdep | `extdeps/gunbc.dag` | Modeling your own repo structure |
| Full enforcement chain | `build_targets.dag` → makegen → Makefile → CI | End-to-end from declaration to enforcement |
| `@outputs` traceability | `@outputs("Makefile")` on func | Tracking what tools produce |

Key anti-patterns:

| Anti-pattern | Symptom | Fix |
|--------------|---------|-----|
| Anemic modeling | `extern func` returns `String` | Return structural type instead |
| Scattered ownership | Same data in Rust const + DSL | Delete Rust copy or add drift test |
| Policy in a test | Hardcoded approved-module list | Derive from model |
| fn items in std modules | "missing input" at runtime | Separate types from functions |
| Same-module extern call | Data flow broken | Use shadow fn body (NF-7 workaround) |

---

## Pre-flight checklist

Answer these before pushing any branch:

- [ ] **Model first**: Can you write the `.dag` types + data without func items or Rust?
- [ ] **One source**: No data duplicated between Rust and DSL without a drift test.
- [ ] **Fail closed**: No `_ =>` catch-alls, `/* unsupported */` comments, or empty returns.
- [ ] **Structure preserved**: Functions return typed IR, not `String`.
- [ ] **Pattern exists**: Checked testgen/makegen/pragma for existing solutions.
- [ ] **Rename-safe**: Renaming a variant would break at compile time, not runtime.
- [ ] **All match sites updated**: New enum variant → every match site in same commit.
- [ ] **Tests pass**: `cargo test --workspace && cargo clippy --all-targets -- -D warnings`

---

## Acceptance criteria for a branch

A branch is ready to merge when:

1. **Zero warnings**: `cargo clippy --all-targets -- -D warnings` clean
2. **All tests pass**: `cargo test --workspace` green
3. **Drift tests pass**: `tool_registration.rs` tests verify DSL ↔ Rust alignment
4. **No scattered ownership**: Any new data lives in exactly one place
5. **No new placeholders**: No `/* unsupported */`, `todo!()`, `_ =>` catch-alls
6. **Design note exists** (if touching compiler/IR/executor/transport): doc in `docs/design/`
7. **Model exists** (if adding a tool/feature): Layer 1/2 `.dag` file with types + data
8. **Outputs declared**: Any generated files have `@outputs` and appear in `.gitignore`

---

## Repo map

```
dsl/                          DSL source files (truth lives here)
├── std/                      Universal types + pure functions
├── extdeps/                  External system models (facts)
├── config/                   Repo policy (invariants + exemptions)
├── tools/                    Executable workflows
├── pipelines/                Multi-stage pipelines (SDLC)
├── interfaces/               Domain contracts (ClaimStore, etc.)
├── services/                 Provider implementations
└── profiles/                 Deployment binding profiles

core/                         Compiler + runtime infrastructure
├── daglang/                  DSL compiler (8 crates, 55k LOC)
│   ├── daglang-syntax/       Lexer + parser
│   ├── daglang-typecheck/    Type system
│   ├── daglang-lower/        Lowering to DAG
│   ├── daglang-emit/         Code generation (Rust/Go/C)
│   └── daglang-driver/       Compilation orchestration
├── ir/                       IR types (51k LOC)
├── exec/                     DAG executor (13k LOC)
├── infra/                    Hash, manifest, freshness
├── codegen/                  CLI + test generation
└── test/                     Test infrastructure

lib/
├── transport/                I/O boundary (the ONLY place with std::fs)
├── cloud-ops/                Cloud provider abstractions
└── primitives/               Stable hashing

gunbc-dag/                    Workspace DAG assembly
├── src/extern_impls.rs       8 bridge functions (shrinking)
├── src/resolve.rs            Generic LoweredOp → DynOp (any .dag file)
├── src/dsl_registry.rs       Structural tool discovery
├── src/makegen/              Makefile generation from registry
├── src/policy/               Pragma policy rendering
└── tests/tool_registration.rs  Drift detection test suite

docs/
├── start-here.md             THIS FILE — read first
├── modeling.md               DAG modeling pattern catalog
├── handbook.md               Architecture handbook
└── design/                   Design documents by area
    ├── v4/                   Current architecture decisions
    ├── modeling/             Modeling-specific decisions
    ├── horizon/              Future direction
    ├── sdlc/                 SDLC pipeline design
    └── workflow/             Workflow design packs

TODO/
├── tasks.md                  Index — points to three lane docs
├── type-system.md            Lane 1: Compositional type coverage (WS-1 through WS-7)
├── gunbc-dag-simplification.md  Lane 2: Reduce gunbc-dag to minimum Rust
├── sdlc.md                   Lane 3: SDLC pipeline end-to-end (the objective)
└── TODONE/                   Completed work archive
```

---

## Design doc index

Before making a decision in any of these areas, read the relevant doc.

### Architecture and modeling

| Doc | Key decision |
|-----|-------------|
| [`docs/modeling.md`](modeling.md) | DAG modeling patterns, layer taxonomy, anti-patterns |
| [`docs/handbook.md`](handbook.md) | Compositional modeling philosophy, transport boundary |
| [`docs/design/modeling/protocol-stack-layering.md`](design/modeling/protocol-stack-layering.md) | Protocol stack composition (TCP→TLS→HTTP→REST) |
| [`docs/design/modeling/repo-self-understanding.md`](design/modeling/repo-self-understanding.md) | Workspace-as-external-system |
| [`docs/design/v4/extern-bridge-gap-analysis.md`](design/v4/extern-bridge-gap-analysis.md) | Why externs exist, elimination phases 5-8 |

### Compiler

| Doc | Key decision |
|-----|-------------|
| [`docs/design/v4/compiler-densification-roadmap.md`](design/v4/compiler-densification-roadmap.md) | Prioritized bridge elimination roadmap: kill interpreter → hermeticity → service codegen |
| [`docs/design/v4/compiler-densification-roadmap.md`](design/v4/compiler-densification-roadmap.md) | Prioritized bridge elimination roadmap: kill interpreter -> hermeticity -> service codegen |
| [`docs/design/v4/compositional-type-coverage.md`](design/v4/compositional-type-coverage.md) | Compositional type coverage: vision, audit, gaps, workstreams, extern linking |
| [`docs/design/v4/by-construction-reachability.md`](design/v4/by-construction-reachability.md) | ReachableDag<T> — invariants via types, not passes |
| [`docs/design/v4/externcall-same-module-port-wiring.md`](design/v4/externcall-same-module-port-wiring.md) | NF-7: lowerer limitation for same-module extern func |
| [`docs/design/v4/dsl-design.md`](design/v4/dsl-design.md) | DSL language reference |

### Code generation

| Doc | Key decision |
|-----|-------------|
| [`docs/design/unified-emission.md`](design/unified-emission.md) | Unify 13 rendering systems under layered IR |
| [`docs/design/unified-registration.md`](design/unified-registration.md) | inventory-based auto-discovery |
| [`docs/design/consolidation-plan.md`](design/consolidation-plan.md) | 6-stream consolidation |

### Transport and execution

| Doc | Key decision |
|-----|-------------|
| [`docs/design/shell-hermeticity-annotation.md`](design/shell-hermeticity-annotation.md) | Tag hermeticity at producer boundary |
| [`docs/design/interface-stub-transport.md`](design/interface-stub-transport.md) | InterfaceStub for unbound interfaces |
| [`docs/design/modeling/m7-secret-redaction-by-default.md`](design/modeling/m7-secret-redaction-by-default.md) | Secret = redacted by default |
| [`docs/design/modeling/m11-strict-dry-run.md`](design/modeling/m11-strict-dry-run.md) | DryRun intercepts all transport |

### Testing

| Doc | Key decision |
|-----|-------------|
| [`docs/design/testgen.md`](design/testgen.md) | 4-bucket obligation model, anti-tautology rule |
| [`docs/design/integration-testgen.md`](design/integration-testgen.md) | Repo-wide integration contracts, Fermi cost tiers |
| [`docs/design/black-box-node-testing.md`](design/black-box-node-testing.md) | Cross-workflow mock corpus, transport fidelity ladders |

### SDLC pipeline

| Doc | Key decision |
|-----|-------------|
| [`docs/design/sdlc/mega-modeling-design.md`](design/sdlc/mega-modeling-design.md) | Canonical SDLC workflow + contracts |
| [`docs/design/sdlc/e2e-gap-analysis.md`](design/sdlc/e2e-gap-analysis.md) | Implementation deltas (all gaps resolved) |
| [`docs/design/sdlc/domain-modeling-comprehensive.md`](design/sdlc/domain-modeling-comprehensive.md) | Entity catalog, state machines, invariants |

### Domain modeling (Phase 2)

| Doc | Key decision |
|-----|-------------|
| [`docs/design/domain-model-porting.md`](design/domain-model-porting.md) | Ported behavioral data from sibling repos for Lane 4 (domain model foundation) |

---

## Common commands

```bash
# Verify everything
cargo test --workspace
cargo clippy --all-targets -- -D warnings

# Run a specific tool
cargo run -p gunbc-dag --bin pragma
cargo run -p gunbc-dag --bin makegen

# Compile a DSL file
cargo run -p daglang-cli -- compile dsl/tools/pragma.dag

# Check what DSL discovers
cargo run -p daglang-cli -- list-tools

# Bootstrap (regenerate Makefile, .gitignore, clippy.toml)
make install
```
