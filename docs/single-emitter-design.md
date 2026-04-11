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
| `src/v2/coercion.dag` dispatch functions | Dissolve into the emitter. There is no separate coercion phase. |

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
| Language mentions in `src/v2/*.dag` | 632 | 0 |
| Language-specific emitter files | 3 (8,081 lines) | 0 — deleted |
| String concat calls producing target syntax | ~680 | 0 |
| String-keyed metadata maps | 14 | 0 |

---

## Current State

### What exists in the compiler (to be deleted)

| File | Lines | Problem |
|---|---|---|
| `src/v2/05_emit.dag` | 2,276 | Target-agnostic dispatch, but also ~77 concat calls that build target syntax |
| `src/v2/05_emit_rust.dag` | 5,709 | Rust-specific rendering — 309 language mentions, string concat, serde heuristics, Rc wrapping |
| `src/v2/05_emit_python.dag` | 1,172 | Python-specific rendering — 96 language mentions |
| `src/v2/05_emit_go.dag` | 1,200 | Go-specific rendering — 84 language mentions |
| `src/v2/coercion.dag` | 298 | Separate coercion dispatch — parallel to emission |

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
| `src/v2/languages.dag` | — | `LanguageSpec` type definitions (presentation facts) |

The data files are the seed of the right architecture. The schema types
(`TypeCheckpoint`, `InhabitantDecl`) are bootstrap scaffolding — they
exist because the compiler can't yet discover homomorphisms structurally.

---

## Target State

### Compiler core (`src/v2/`)

```
src/v2/
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

`match target` in `05_emit.dag`: 20 → 4. Remaining 4:

| Site | What | Design status |
|---|---|---|
| String interpolation (line 251) | 3 structurally different interp models | Designed below |
| Callable wrapping (line 954) | Rust Rc<dyn Fn(...)> wrapping | Designed below |
| Identifier escaping (line 1718) | Case conversion + reserved word escape | Designed below |
| String literal suffix (line 862) | Dag guard (no TypeCheckpoints) | Semantic — not a language decision |

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
| [src/v2/compiler-laws.md](../src/v2/compiler-laws.md) Lane C | Implementation plan — file-level changes, site counts, lane dependencies |
