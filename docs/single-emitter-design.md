> Part of: [THESIS.md](../THESIS.md) > [ROADMAP.md](../ROADMAP.md) > **Track 13: Single emitter**

# Single Emitter Design

**Status:** Design (no compiler changes)
**Track:** 13 (ROADMAP.md)
**Thesis claim:** Emission is mechanical translation.
**Supersedes:** Parts of [coercion-design.md](coercion-design.md) — the coercion
model described there is bootstrap scaffolding. This document describes the
end state where coercion dissolves into emission.

---

## The Unification

Three concepts that appear distinct are the same mechanism viewed from
different angles (THESIS.md, "Concept unification"):

| Apparently distinct | Actually |
|---|---|
| Coercion cost | Complexity — CX proves bounds on .dag functions |
| Coercion | Emission — target spec generates code |
| Target language spec | Transport spec — same role, different domain |

Maintaining coercion as a parallel mechanism violates "No duplicate
representations" and "No parallel implementations" (INVARIANTS.md).

**Coercion is not a step before emission — it IS emission.** The compiler
reads a target spec and generates code. Whether that code is "a Rust struct"
or "a SPICE subcircuit" or "an HTTP client" is determined by the spec, not
by a separate coercion engine.

---

## The Insight: Algebra Homomorphism, Not Lookup Tables

The current coercion infrastructure uses `TypeCheckpoint` — a manual lookup
table: `{ dag_name: "Int", target_type: "i64", is_copy: true }`.

But `.dag Int` already IS `Word64 + OrderedRing witness` (std/algebra.dag,
concept DAG in MODELING.md M9). And Rust's `i64` IS the same thing — a
64-bit word inhabiting an ordered ring. The mapping should fall out from
the algebra, not from a hand-maintained table.

The same applies to `InhabitantDecl`: it says "FreeMonoid maps to Vec<T>
in Rust." This is closer to right — it references the algebra — but it's
still a manual declaration keyed by string. If both `.dag List<T>` and
Rust `Vec<T>` inhabit `FreeMonoid<T>`, the compiler should discover that.
The translation is the unique algebra-preserving map (homomorphism).

### What dissolves

| Bootstrap scaffolding | End state |
|---|---|
| `TypeCheckpoint` (manual name→type table) | Structural matching: source and target both inhabit the same algebra over the same bit-width. The mapping is discovered, not declared. |
| `InhabitantDecl` (manual algebra→template table) | Target language declares its types with algebraic identity (same as .dag types do). The compiler finds the homomorphism. |
| `coercion-design.md` resolution order (checkpoint → algebra → structural → fail) | One mechanism: walk the source type's algebraic identity, find the target type that inhabits the same structure. Structural types (Product, Coproduct) are the base case — they compose recursively. |
| `src/v1/coercion.dag` dispatch functions | Dissolve into the emitter. There is no separate coercion phase. |

### What remains

Target languages genuinely differ in **presentation**, not semantics:

- Syntax templates (block delimiters, statement terminators, lambda syntax)
- Naming conventions (snake_case vs camelCase, reserved word escaping)
- Import conventions (use/import paths, prelude membership)
- Sharing strategy (Rust clone/borrow vs GC languages)
- Annotation requirements (type annotations on let bindings, lambda params)

These are `LanguageSpec` data — facts about presentation, not about types.
They belong in `dsl/extdeps/languages/<lang>/` as data declarations.

---

## Done Criterion

From INVARIANTS.md ("Emission reads data, never decides"):

> If adding a new target language requires writing emission *logic* (not
> just data declarations), the shared emitter is making decisions that
> should be data-driven.

**Track 13 is done when:** adding a new target language means adding a
`dsl/extdeps/languages/<lang>/` directory with data declarations, and
touching zero compiler files.

### Ratchets at zero

| Metric | Current | Target |
|---|---|---|
| Language mentions in `src/v1/*.dag` | 632 | 0 |
| Language-specific emitter files | 3 (8,081 lines) | 0 — deleted |
| String concat calls producing target syntax | ~680 | 0 |
| String-keyed metadata maps | 14 | 0 |

---

## Current State

### What exists in the compiler (to be deleted)

| File | Lines | Problem |
|---|---|---|
| `src/v1/05_emit.dag` | 2,276 | Target-agnostic dispatch, but also ~77 concat calls that build target syntax |
| `src/v1/05_emit_rust.dag` | 5,709 | Rust-specific rendering — 309 language mentions, string concat, serde heuristics, Rc wrapping |
| `src/v1/05_emit_python.dag` | 1,172 | Python-specific rendering — 96 language mentions |
| `src/v1/05_emit_go.dag` | 1,200 | Go-specific rendering — 84 language mentions |
| `src/v1/coercion.dag` | 298 | Separate coercion dispatch — parallel to emission |

The emitter decides: it branches on type names, checks hardcoded lists,
and builds target syntax via string concatenation. Each of these is a fact
that was lost at an upstream boundary.

### What exists as data (to be preserved and extended)

| File | Lines | Role |
|---|---|---|
| `dsl/std/coercion.dag` | 132 | Schema types (bootstrap form — dissolves) |
| `dsl/std/algebra.dag` | ~200 | Algebraic hierarchy — the real authority |
| `dsl/extdeps/languages/rust/types.dag` | 250 | Rust type checkpoints + inhabitants (bootstrap data) |
| `dsl/extdeps/languages/rust/emit.dag` | 283 | Container templates, method specs, reserved words |
| `dsl/extdeps/languages/python/types.dag` | 178 | Python type data |
| `dsl/extdeps/languages/python/emit.dag` | 108 | Python rendering data |
| `dsl/extdeps/languages/go/types.dag` | 173 | Go type data |
| `dsl/extdeps/languages/go/emit.dag` | 101 | Go rendering data |
| `src/v1/languages.dag` | — | `LanguageSpec` type definitions (presentation facts) |

The data files are the seed of the right architecture. The schema types
(`TypeCheckpoint`, `InhabitantDecl`) are bootstrap scaffolding — they
exist because the compiler can't yet discover homomorphisms structurally.

---

## Target State

### Compiler core (`src/v1/`)

```
src/v1/
  05_emit.dag          One emitter: (annotated graph + LanguageSpec) → text
                       Zero language mentions. Zero string concat for syntax.
                       Reads algebraic identity from the graph, finds the
                       target inhabitant, applies syntax templates.
```

No `05_emit_rust.dag`. No `05_emit_python.dag`. No `05_emit_go.dag`.
No `coercion.dag` (coercion IS emission — no separate dispatch).

### Language data (`dsl/extdeps/languages/`)

```
dsl/extdeps/languages/
  rust/
    types.dag          Algebraic identity declarations (i64 inhabits OrderedRing, etc.)
    emit.dag           Presentation: syntax templates, method specs, imports
    naming.dag         Case conventions, reserved word strategy
  python/
    types.dag, emit.dag, naming.dag
  go/
    types.dag, emit.dag, naming.dag

  # Challenge targets (design validation):
  verilog/
    types.dag          Products → module ports, Coproducts → mux
    emit.dag           Verilog syntax templates
  spice/
    types.dag          Products → subcircuit params
    emit.dag           SPICE netlist syntax
  english/
    types.dag          Products → bullet lists, Coproducts → "either/or"
    emit.dag           Markdown rendering templates
```

Adding a new language = add a directory. Zero compiler changes.

Challenge targets validate the architecture: if the emitter works for
Verilog, SPICE, and English, it works for anything. These are the hardest
targets — they force the compiler to find every implicit decision.

---

## Architectural Constraints

### .dag source is a valid emission target

The emitter architecture must not assume the target is a foreign language.
`.dag` source is just another target spec. Error diagnostics ("show the
corrected code") are emission targeted at the developer — same mechanism,
different spec. Code migration, round-trip editing, and diagnostic
rendering are projections of the same graph onto the `.dag` target spec.

This is the concept unification principle again: error diagnostics =
emission to .dag target. A separate diagnostic rendering system would be
a parallel mechanism.

**Non-goal for Track 13:** implementing bidirectional emission. But the
graph-to-graph model must not preclude it. The actual bidirectional
emission work would be a separate milestone, tied to the "show the
correct code" error handling vision in THESIS.md.

**The test:** if the emitter's internal representation cannot express
"emit this graph as .dag source," the representation has baked in an
assumption that the target is foreign. That assumption must not exist.

---

## What This Doc Does NOT Solve

- **Bidirectional emission:** .dag as an emission target, error diagnostics
  as emission. Architecturally enabled (see constraint above), not implemented.
- **Omni-emission (Track 14):** Multi-target artifact planning. Blocked on
  Track 13 — need target-agnostic emission before multi-target is meaningful.
- **Runtime safety (Track 11):** Refinement types, total operations. Separate
  design project.
- **Verification (Track 12):** Test generation from algebraic laws. Depends
  on Track 5 (working emission to execute against).
- **LS-4 borrow model:** The sharing strategy (clone vs borrow) design for
  Track 2. This is a dependency, not a deliverable of Track 13.

---

## Dependencies

| Dependency | Track | Why needed | Status |
|---|---|---|---|
| LanguageSpec modeling | Track 2 | Presentation facts must be modeled as data before the emitter can read them | LS-4 (borrow model) in design |
| Core table dissolution | Track 7 | String-keyed maps must become node-keyed before the emitter can use structural identity | Partially done |
| Node.name deletion | Track 3 | String identity must dissolve before homomorphism discovery works | ~15 n.name reads remaining |
| CX gate | KF-1 | CX must be able to prove bounds on emission functions | 421 violations remaining (M1) |

### What can move without dependencies

Not everything requires the full dependency chain. Concrete work that
unblocks Track 13 without waiting:

1. **Extract hardcoded language facts to LanguageSpec data.** Every
   `if target == Rust` branch in `05_emit.dag` can become a LanguageSpec
   field read today.
2. **Unify method rendering.** Rust has `SimpleMethodSpec`; Python and Go
   have hardcoded method dispatch. Unify on the data-driven pattern.
3. **Move serde/attribute logic to data.** The Rust emitter's serde
   attribute resolution is raw node traversal. Model `VariantEncoding`
   as a structured type, emit reads it.
4. **Delete parallel expression dispatch.** Three copies of 20-arm
   ExprData walks (one per language). Factor to shared dispatch +
   per-target syntax templates.

---

## Open Items

### C-1 progress

`match target` in `05_emit.dag`: 20 → 0. **All 4 remaining sites resolved (PR #400):**

| Site | What | Resolution |
|---|---|---|
| String interpolation | 3 structurally different interp models | `StringInterpSyntax` type (FormatArgs \| InlineExpr) + `apply_escape_pairs` |
| Callable wrapping | Rust Rc<dyn Fn(...)> wrapping | `callable_type_template: String?` on LanguageSpec |
| Identifier escaping | Case conversion + reserved word escape | `NamingCase` enum (SnakeCase \| CamelCase \| PascalCase \| AsAuthored) |
| String literal suffix | Dag guard (no TypeCheckpoints) | Added `dsl/extdeps/languages/dag/types.dag`; fail-closed restored |

### Design: String interpolation

Three structurally different interpolation models:
1. **Format function + positional args** (Rust, Go): `format!("text {} text", expr)` / `fmt.Sprintf("text %v text", expr)`
2. **Inline interpolation** (Python): `f"text {expr} text"`

Proposed types:
```
type StringInterpSyntax {
  style: InterpStyle
  format_template: String         // "format!(\"{0}\", {1})" / "fmt.Sprintf(\"{0}\", {1})" / "f\"{0}\""
  placeholder: String             // "{}" / "%v" / "" (inline uses expr directly)
  plain_template: String          // "\"{0}\".to_string()" / "\"{0}\""
  escape_pairs: List<EscapePair>  // [{from: "{", to: "{{"}, ...] — replaces per-language escape fns
}

type InterpStyle = FormatArgs | InlineExpr
type EscapePair { from: String  to: String }
```

A generalized `apply_escape_pairs(s, pairs)` helper replaces the three
per-language escape functions (`escape_rust_interp_text`, etc.).
`emit_simple_string_interp` dispatches on `style` for arg-collection
vs inline-embedding.

### Design: Callable wrapping

The Rust arm capitalizes `fn(` → `Fn(` and wraps with `Rc<dyn ...>`.
Go suppresses the return type when void.

Proposed: add `callable_type_template: String?` to LanguageSpec or
SharingStrategy. When non-null, replaces entire callable rendering:
- Rust: `"Rc<dyn Fn({0}) -> {1}>"` (params, return)
- Others: null (use `CallableRepr.template` as-is)

Go's void-return suppression: if `void_type == ""` and ret_str is
empty, omit the return part of the template.

Interacts with LS-4 (borrow model) — the callable wrapper is an
ownership decision.

### Design: Identifier escaping

Two parts: case conversion (computation) + reserved word escape (data).
Reserved word escaping is already data-driven via `ReservedWordStrategy`.

Proposed: add `NamingCase` to LanguageSpec:
```
type NamingCase = SnakeCase | CamelCase | AsAuthored
```

Extract Go's inline camelCase logic to `to_camel(name) -> String`.
Then `emit_ident` reads `spec.naming_case` and dispatches on the enum
instead of matching on target.

---

## Phased Dissolution Plan

Seven phases, each with a clear "what it accomplishes" and "what it
deletes." Phases 2-4 can run in parallel after Phase 1.

### Phase 1: Shared emitter is language-clean (05_emit.dag) — **DONE** (PR #400)

`match target` branches: 20 → 0. All language knowledge reads from
LanguageSpec data. New types: `StringInterpSyntax`, `RecordLitSyntax`,
`NamingCase` (extended), `callable_type_template`, `async_call_prefix`,
`bridge_method_*`, `emit_export_ident`, `emit_field_access_unified`.

### Phase 2: Unify expression dispatch (biggest bang) — **DONE** (PR #400)

`emit_unified_typed_expr` in shared emitter replaces per-language
expression dispatch. Python and Go entry points are 3-line aliases.
~50 per-language functions deleted (25 Python + 25 Go). CX ratchet
421 → 416 (-5). Remaining per-language pattern rendering stays (Phase 5).

**Actual deletions:** ~50 functions, ~580 lines deleted (net -73 lines
including new shared infrastructure).

### Phase 3: Unify TCO and block rendering

TCO is 85% duplicated (14 Rust / 10 Python / 10 Go functions).
Python and Go are already thin wrappers around shared functions —
8 of 10 functions in each just delegate. Block statement emission
is 100% reused via `_shared` functions already.

**Audit results (2026-04-12):**

100% identical (delete per-language, use shared directly):
- `*_tco_non_self_call`, `*_tco_if`, `*_tco_let`,
  `*_tco_default_return`, `*_typed_tco_expr`, `*_typed_tco_reassign`
  — 6 functions × 3 languages = 18 functions → 0

Structurally parallel (parameterize via LanguageSpec):
- `*_typed_tco_body` — Rust hard-codes `loop`, Py/Go use spec
- `*_tco_block` — Rust has custom param-shadowing init

Language-specific match rendering (needs callback pattern):
- `*_tco_match` — Rust: Rc deref analysis. Python: if/elif. Go: switch.
  45% overlap. Unify via `match_renderer` callback in shared dispatch.

Rust-only (defer to Phase 6 / ownership extraction):
- `emit_tco_params`, `emit_tco_param` — `mut` annotation
- `emit_tco_init_block_stmts`, `emit_tco_init_stmt` — param shadowing
- `emit_rust_tco_match` Rc analysis — isolate as optional callback

New LanguageSpec fields needed:

| Field | Rust | Python | Go |
|-------|------|--------|-----|
| `loop_keyword` | `loop` | `while True` | `for` |
| `break_return` | `break` | `return` | `return` |
| `continue_str` | `continue;` | `continue` | `continue` |
| `temp_var_prefix` | `__tco_` | `__tco_` | `tco` |
| `temp_decl_prefix` | `let ` | (empty) | (empty) |
| `temp_assign_op` | ` = ` | ` = ` | ` := ` |

**Accomplishes:** Control flow is language-agnostic. Only syntax tokens vary.
**Deletes:** ~20 Python/Go functions, Rust functions stay until Phase 6.
**Size:** Medium. Python/Go deletion is mechanical; Rust isolation needs care.

### Phase 4: Unify service/transport rendering

75% overlap — same dispatch (REST/shell/file/local), different HTTP
client APIs. Transport client syntax (`reqwest` vs `aiohttp` vs
`http.Client`) becomes LanguageSpec data.

**Accomplishes:** Adding a new transport doesn't require per-language work.
Cost of change = 1 file.
**Deletes:** ~24 functions (~600 lines).
**Size:** Medium.

### Phase 5: Unify type rendering

Rust has 12 type-rendering functions (ownership, derives, serde),
Python has 6, Go has 7. Product/Coproduct rendering is structural and
shared; ownership decoration is per-language.

**Accomplishes:** Type definitions emit from structural facts +
LanguageSpec presentation data.
**Deletes:** ~15 functions (~400 lines).
**Size:** Medium. Partially blocked on LS-4 (borrow model) for full
Rust ownership extraction.

### Phase 6: Extract Rust ownership to shared model

~60 Rust-specific functions for Rc-wrapping, clone/move, borrow
decisions. These aren't "Rust syntax" — they're ownership semantics
that should be modeled in LanguageSpec:
- Rust: clone/borrow
- Go: pass-by-value (small) / pointer (large)
- Python: no-op (GC)

**Accomplishes:** Ownership becomes a LanguageSpec dimension, not
Rust-specific logic. This is the LS-4 design work.
**Deletes:** The last ~60 Rust-specific functions.
**Size:** Large. This is the design project, not just extraction.
**Blocked on:** LS-4 (Track 2).

### Phase 7: Delete per-language emitters + coercion.dag

After Phases 1-6, the three per-language files are empty or near-empty.
`coercion.dag` dispatch merges into `05_emit.dag` (coercion IS emission).

**Accomplishes:** M4 done criterion met. Zero language-specific emitter
files. Adding a new language = add a `dsl/extdeps/languages/<lang>/`.
**Deletes:** `05_emit_rust.dag` (5,709), `05_emit_python.dag` (1,172),
`05_emit_go.dag` (1,200), `coercion.dag` (298) — ~8,400 lines total.

### Dependency map

```
Phase 1 (emit.dag clean) ← nothing, start now
Phase 2 (expressions)    ← Phase 1
Phase 3 (TCO/blocks)     ← Phase 1        ← can run in parallel
Phase 4 (services)       ← Phase 1
Phase 5 (types)          ← Phase 1, partially LS-4
Phase 6 (ownership)      ← LS-4 design (Track 2)
Phase 7 (delete)         ← Phases 2-6
```

### Summary

| Phase | Functions deleted | Lines deleted | Blocked on |
|-------|-------------------|---------------|-----------|
| 1 | 0 (refactor) | 0 | Nothing |
| 2 | ~70 | ~2,000 | Phase 1 |
| 3 | ~30 | ~800 | Phase 1 |
| 4 | ~24 | ~600 | Phase 1 |
| 5 | ~15 | ~400 | Phase 1 + LS-4 partial |
| 6 | ~60 | ~1,800 | LS-4 design |
| 7 | remaining | ~2,800 (file deletion) | Phases 2-6 |
| **Total** | **~200** | **~8,400** | |

---

## The Cost Question

There is no cost model for emission.

A type coercion is a .dag function. Its cost is whatever CX proves —
the same complexity machinery that bounds every other .dag function.
Inventing a separate cost lattice (Native/Isomorphic/Lowered/Synthesized)
for coercions would be a parallel mechanism. CX already does this.

When coercion rules become .dag functions, CX proves their bounds
automatically. No new infrastructure needed.

---

## Relationship to Existing Docs

| Document | Relationship |
|---|---|
| [THESIS.md](../THESIS.md) | Parent — concept unification, emission completeness |
| [ROADMAP.md](../ROADMAP.md) Track 13 | Status tracking — current state, blockers, progress |
| [coercion-design.md](coercion-design.md) | Bootstrap design — describes the manual scaffolding (TypeCheckpoint, InhabitantDecl, resolution order). Accurate for the transitional state. Dissolves when Track 13 completes. |
| [INVARIANTS.md](../INVARIANTS.md) | Governing rules — "Emission reads data, never decides," "No duplicate representations," "No parallel implementations" |
| [src/v1/compiler-laws.md](../src/v1/compiler-laws.md) Lane C | Implementation plan — file-level changes, site counts, lane dependencies |
