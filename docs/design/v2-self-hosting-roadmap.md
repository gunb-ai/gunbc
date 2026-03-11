# v2 Self-Hosting Roadmap

## Current Status (2026-03-11)

### What works

The v2 compiler is a 6,045-line DSL program across 8 .dag files. All 5
pipeline stages (tokenize → parse → resolve → typecheck → emit) execute
correctly on trivial input through the v1 evaluator:

- **34 integration tests pass** (15 parse audit + 4 compilation gate +
  4 tokenizer E2E + 11 stage-by-stage integration)
- Trivial input `"module test\ntype Foo { x: Int }"` → correct Rust:
  `pub struct Foo { pub x: i64 }`
- All 7 .dag files parse with zero v1 diagnostics
- Clippy clean across workspace

### What doesn't work

1. **Stack overflow on real files.** The v1 evaluator uses direct Rust
   recursion — every DSL function call, match arm, and expression is a
   Rust stack frame. parse.dag's recursive descent exhausts 64MB of
   stack on anything beyond ~50 tokens. Self-hosting requires parsing
   parse.dag itself (2,500 lines, ~10,000 tokens).

2. **`emit_rust` list concat fails.** `[lib_file] + module_files +
   test_files` triggers "arithmetic on non-integers" in the evaluator.
   The `[x]` list literal produces a Map (not a List) in some evaluator
   paths. Workaround: call `emit_module` per module individually.

3. **No native binary exists.** The v2 compiler only runs interpreted
   inside v1. Self-hosting requires: v2 emits Rust → rustc compiles it
   → native binary can compile v2's own source. The interpreted path
   can never self-host due to stack depth.

4. **Evaluator limitations surface at scale.** Record field access
   patterns, list operations on large data, and deep match nesting all
   work on toy input but are untested at the scale of real .dag files.

### Architectural debt (from review)

The v2 compiler's internal structure still mirrors v1's Rust
implementation rather than defining its own compositional semantics:

- **Proliferation of pass-local wrapper types.** typecheck.dag defines
  7 one-off `*Result` wrappers (ResolveResult, ItemResult, FieldResult,
  VariantResult, ParamResult, OperationResult, CapabilityResult,
  ResourceUseResult). Each threads `{value, diagnostics}` manually.

- **Redundant type composition.** TypeExpr has both `TypeApp` (general)
  and `Container`/`MapType` (special-cased). Parser, typechecker, and
  emitter all have parallel arms for each — duplicated logic.

- **No backend-neutral middle layer.** emit.dag goes directly from
  TypedGraph to `reqwest::Client` / `std::process::Command`. A second
  backend would re-derive transport semantics.

- **Cross-module type duplication.** TypedGraph, TypedModule, TypeEnv,
  TypeBinding are defined independently in typecheck.dag and emit.dag.

- **Pipeline is hand-stitched.** compile_sources manually chains stages
  instead of composing a uniform pass algebra. StageResult, TokenizeResult,
  ParseStageResult coexist without a shared contract.

---

## Design: The Self-Hosting Path

Self-hosting means: the v2 compiler, compiled to native Rust, can
compile its own .dag source and produce identical output (fixed point).

The fundamental constraint is **stack depth**. The interpreted evaluator
will never handle parse.dag's 2,500 lines. Therefore:

```
Phase 0: Model cleanup (compositional contracts)
Phase 1: Native bootstrap (v2 → Rust source → rustc → binary)
Phase 2: Progressive self-compilation (binary compiles itself)
Phase 3: Fixed point (output matches, tests pass)
```

### The bootstrap ladder

```
                    ┌─────────────────────────┐
                    │  v1 evaluator (host)     │
                    │  interprets v2's .dag    │
                    │  source on TRIVIAL input │
                    └───────────┬─────────────┘
                                │ emits Rust source
                                ▼
                    ┌─────────────────────────┐
                    │  rustc                   │
                    │  compiles emitted Rust   │
                    └───────────┬─────────────┘
                                │ produces
                                ▼
                    ┌─────────────────────────┐
                    │  v2-native binary        │
                    │  no stack depth limit    │
                    │  compiles any .dag file  │
                    └───────────┬─────────────┘
                                │ compiles own source
                                ▼
                    ┌─────────────────────────┐
                    │  v2-native' binary       │
                    │  output == v2-native     │
                    │  FIXED POINT             │
                    └─────────────────────────┘
```

The key insight: the **first** native binary doesn't need to be
produced by compiling all of v2's source at once. It can be assembled
from the v1 evaluator running on trivial input per-module, producing
per-module Rust files, then combining them with a hand-written driver.

---

## Phase 0: Model Cleanup

**Goal:** Make the .dag source express compiler semantics, not host
implementation bookkeeping. This makes the Rust output cleaner and
the self-hosting delta smaller.

### 0a: Unified pass contract

Replace the 7+ bespoke `*Result` types with one generic pattern.
Since the DSL doesn't have generics at the value level, each function
returns its primary value plus a `diagnostics` list as separate return
fields. The `*Result` wrapper types become unnecessary.

Concrete changes:
- typecheck.dag: eliminate ResolveResult, ItemResult, FieldResult,
  VariantResult, ParamResult, OperationResult, CapabilityResult,
  ResourceUseResult. Functions return bare `{ field, diagnostics }`.
- pipeline.dag: eliminate StageResult, TokenizeResult, ParseStageResult.

### 0b: TypeApp subsumes Container and MapType

`Container { kind: List, element: T }` is `TypeApp { name: "List", args: [T] }`.
`MapType { key: K, value: V }` is `TypeApp { name: "Map", args: [K, V] }`.

Consolidate to eliminate 3 match arms per pass. The parser produces
TypeApp directly. The emitter pattern-matches on TypeApp.name for
built-ins (`List` → `Vec`, `Map` → `BTreeMap`, `Set` → `BTreeSet`).

### 0c: Canonical type homes

core.dag is the single source of truth. TypedGraph, TypedModule,
TypeEnv, TypeBinding, ModuleGraph, ResolvedModule move to core.dag.
Pass-local algorithm scratch (ParserState, PR, KahnState) stays local.

### 0d: Backend-neutral operation model (deferred)

Insert OperationPlan/BackendPlan between TypedGraph and emit_rust.
Deferred until after self-hosting when adding a second backend.

### Phase 0 estimated effort: 1-2 sessions

---

## Phase 1: Native Bootstrap

**Goal:** Produce a native v2 binary without v1 evaluator limitations.

### Strategy: Per-module emission via v1, assemble with driver

1. For each v2 module, emit Rust source via v1 evaluator (per-module
   emission works on trivial input).
2. Assemble per-module output into a crate with a hand-written driver.
3. `cargo build` → first native v2 binary.

### 1a: Emitter produces compilable Rust

Test that `emit_module` output compiles with `cargo check`.

### 1b: Bootstrap driver

Hand-written `main.rs` (~50 lines) that wires the stages together.

### 1c: First native build

v1 emits each module → combine with driver → `cargo build` → binary.

### Phase 1 estimated effort: 3-5 sessions

---

## Phase 2: Progressive Self-Compilation

**Goal:** Native binary compiles progressively larger .dag files.

| # | Input | Lines | Exercises |
|---|-------|-------|-----------|
| M1 | trivial type module | 2 | Minimal pipeline |
| M2 | `dsl/std/types.dag` | 523 | Type-only, no fn bodies |
| M3 | `src/v2/std/core.dag` | 331 | v2's own type definitions |
| M4 | `src/v2/compiler/tokenize.dag` | 470 | fn bodies, data decls |
| M5 | `src/v2/compiler/resolve.dag` | 465 | Algorithm-heavy fn bodies |
| M6 | `src/v2/compiler/typecheck.dag` | 1,011 | Large fn bodies, match nesting |
| M7 | `src/v2/compiler/emit.dag` | 1,118 | String-heavy emission |
| M8 | `src/v2/compiler/parse.dag` | 2,492 | Deepest recursion |
| M9 | All 8 modules | 6,045 | Full self-compilation |

### Phase 2 estimated effort: 5-10 sessions

---

## Phase 3: Fixed Point and Verification

### 3a: Fixed point test
Compile v2 with v2-native → output A. Compile with A → output B. A == B.

### 3b: Corpus test
Run v2-native on full .dag corpus. Compare against v1 output.

### 3c: v1 retirement gate
v2-native becomes default. v1 remains as bootstrap fallback only.

### Phase 3 estimated effort: 2-3 sessions

---

## Known Evaluator Bugs

### Bug 1: `[x] + y` list concatenation
`emit_rust`'s `[lib_file] + module_files` fails. Root cause likely in
lowerer emitting `[x]` as record instead of list literal.

### Bug 2: DataDef arity (FIXED)
typecheck.dag `collect_unresolved_in_item` called `collect_unresolved_in_type_expr`
with 2 args instead of 3. Fixed 2026-03-11.

---

## Tokenizer gaps (from review)

### Pipe token for sum types
The tokenizer only handles `||` (logical or), not standalone `|` (sum
type variant separator). Declarations like `type X = A | B` produce
Unknown tokens for `|`. Needs a `Pipe` token.

### Float literals
`scan_number` only consumes digits via `parse_int`. A literal like
`1.0` becomes `LitInt(1), Dot, LitInt(0)` instead of `LitFloat`.

---

## Dependency Graph

```
Phase 0 (model cleanup) ──→ Phase 1a (compilable Rust)
Bug fixes               ──→      │
                                  ▼
                           Phase 1b (driver) → Phase 1c (first build)
                                  │
                                  ▼
                           Phase 2 (M1-M9) → Phase 3 (fixed point)
```

**Total estimated effort: 12-20 sessions.**

## Non-goals

- Python backend (deferred until after Rust self-hosting)
- LSP / error recovery (additive, comes later)
- Performance optimization (correctness first)
- Backend-neutral operation model (defer until second backend)
