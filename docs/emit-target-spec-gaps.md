# Lane 1e Phase 1 — Emit target-spec gap inventory

**Purpose**: Planning artifact for Lane 1e ("Single-walker emit collapse"). Classifies every target-dispatch point across `src/v3/compiler/src/emit.rs`, `src/v3/compiler/src/emit/rust_target.rs`, and `src/v3/compiler/src/emit/python_target.rs` as either (1) already covered by existing spec rows, (2) missing a spec row and blocking the walker, or (3) genuinely per-target residual that stays as narrow Rust dispatch.

**Status**: Produced by the director to unblock Phase 2+ work. Replaces the Phase 1 audit that the Lane 1e brief had asked the worker to produce.

**Audit base**: `origin/main` as of 2026-04-20.

**LOC / count figures in this document are an as-of-snapshot.** File sizes (e.g. `rust_target.rs` ~5,700 LOC) and the "46 audit points" headline will drift as code moves on `main`. Per INVARIANTS.md §"Documentation Describes Live State," treat these numbers as sizing guidance from the snapshot date, not live invariants. **Phase 2 workers should re-measure at Phase 2 kickoff** — the classification (Cluster A-J) is stable; the absolute counts aren't.

## Overview

| Category | Count | LOC impact |
|---|---:|---:|
| **ALREADY_SPEC_COVERED** — walker consumes existing `spec/*.dag` rows | 21 | ~0 (no new work) |
| **MISSING_SPEC_ROW** — spec extension required before walker can dissolve the branch | 20 | ~3,500-4,200 LOC dissolvable |
| **RESIDUAL_PER_TARGET** — genuine per-target dispatch; stays as narrow Rust | 5 | ~800-1,200 LOC residual |
| **Total branches classified** | 46 | — |

**Net impact at Lane 1e close:**
- Delete `emit/rust_target.rs` (~5,700 LOC) + `emit/python_target.rs` (~2,000 LOC) + Go inline emitter in `emit.rs` (~380 LOC) = **~8,080 LOC removed**
- Add single walker module (`emit/walker.rs` or similar): **~2,000-2,500 LOC**
- Add spec extensions across `src/v3/spec/rust.dag`, `src/v3/spec/go.dag`, `src/v3/spec/python.dag`: **~500-700 LOC of `.dag` declarations**
- **Net hand-authored Rust delta: approximately −5,500 to −6,000 LOC** (8,080 removed − 2,000 to 2,500 added)
- **Plus ~500-700 LOC of new `.dag` data declarations** (counted separately from the hand-Rust delta)

**Feasibility: GREEN.** 71% of the three-file codebase is mechanically dissolvable into data-driven logic. No fundamental design surprises. The five residual-per-target items are narrow and semantically necessary (not laziness).

## How to read this document

Each cluster below names:
- **What it does** — the semantic purpose of the branches this cluster covers
- **Current shape** — where the dispatch lives today (file + function names; worker verifies specific line ranges)
- **Proposed spec-row shape** — the `.dag` extension that makes this data-driven
- **Phase 2 PR assignment** — which follow-on PR lands this cluster's spec extension

File-level line citations are omitted to keep this artifact focused on the *classification*; the worker executing Phase 2 will grep for each branch during spec extension.

---

## Category 1: ALREADY_SPEC_COVERED

The following 21 audit points are already data-driven. Walker consumes the named spec row directly; no new spec work needed.

- **Function-signature rendering syntax** — covered by `rust_functions` / `go_functions` / `python_functions` syntax declarations in `dsl/std/languages.dag` (wrapper for func keywords, arg list delimiters, return-type arrow style).
- **Type-expression rendering syntax** — covered by corresponding `*_types` syntax records.
- **Reserved-words checks** — covered by `*_reserved_words` / `*_scaffold_keywords` sets.
- **Primitive type name mapping** — covered by target `primitive_type_mapping` data in spec files (Int→i64/int64/int, String→String/string/str, etc.).
- **Callable parameter dispositions** — covered by `ParameterDisposition` in `emit_model.dag`; dispatch on `Borrowed` vs `Consumed` is shared-model, not per-target.
- **Variant payload field access rule** — covered by `PatternBindingRule` variants in `clean_emission.dag`.
- **Variant constructor argument ordering** — covered by `rust_variant_ctor_template` / `go_variant_ctor_template` / `python_variant_ctor_template` per target in spec.
- **Arithmetic / comparison operator symbols** — covered by `OperatorKind` + per-target operator symbol tables.
- **Derive attribute emission** (Rust) — covered by `rust_derive_attrs` data (SG-7.1 landed this).
- **Function-visibility prefix** (pub, export-uppercase, no-prefix) — covered by per-target `*_visibility_prefix` data (SG-7.2 landed this).
- **String-literal escape sequences** — covered by `string_escape_spec` rows.
- Plus ~10 additional bridges enumerated in `docs/emit-bridges.md` that have already been spec-covered by prior SG-7.x work.

**Phase 2 action**: none. These are the walker's input.

---

## Category 2: MISSING_SPEC_ROW (proposed extensions, clustered)

20 audit points across 10 logical clusters. Each cluster becomes one Phase 2 PR.

### Cluster A — Go recursive type-argument substitution `(~120 LOC dissolvable)`

**What**: Go renders container types by position-substituting the element type into a template: `[]Element`, `map[K]V`, `*Element`. The current `emit.rs` Go path hand-walks the type tree and splices strings.

**Proposed spec row**:
```
type TypeRecursionStrategy {
  container_template: String           // "[]{element}" / "map[{key}]{value}" / "*{element}"
  recursion_points: List<TypeParamName> // which positions recurse
}

data go_list_recursion: TypeRecursionStrategy = {
  container_template: "[]{element}",
  recursion_points: ["element"]
}
data go_map_recursion: TypeRecursionStrategy = {
  container_template: "map[{key}]{value}",
  recursion_points: ["key", "value"]
}
data go_optional_recursion: TypeRecursionStrategy = {
  container_template: "*{element}",
  recursion_points: ["element"]
}
```

**Phase 2 PR**: PR-2.3 (Type recursion — Go-heavy; Rust/Python largely covered by existing templates).

### Cluster B — Execution model requirement (ownership vs GC) `(~20 LOC dissolvable)`

**What**: Rust emission invokes the ownership/borrow pipeline; Go and Python skip it. Today the branch is a hardcoded `if target == Rust` in emit.rs.

**Proposed spec row**:
```
type ExecutionModelRequirement
  = OwnershipBased
  | GarbageCollected

data rust_execution_model: ExecutionModelRequirement = OwnershipBased
data go_execution_model: ExecutionModelRequirement = GarbageCollected
data python_execution_model: ExecutionModelRequirement = GarbageCollected
```

Walker dispatches on the spec value rather than a target name.

**Phase 2 PR**: PR-2.1 (foundational, small — also covers cluster C).

### Cluster C — Bootstrap path filtering `(~30 LOC dissolvable)`

**What**: Emit filters out stdlib declarations by prefix-matching `dsl/std/` / `src/v3/std/`. Currently hardcoded.

**Proposed spec row**:
```
type SourceFiltering {
  excluded_prefixes: List<String>
}

data stdlib_source_filtering: SourceFiltering = {
  excluded_prefixes: ["dsl/std/", "src/v3/std/", "src/v3/compiler/", "dsl/extdeps/", "dsl/gunbc/"]
}
```

Walker applies before rendering.

**Phase 2 PR**: PR-2.1 (foundational cluster, bundled with cluster B).

### Cluster D — Pattern binding liveness (implementation duplication, not spec gap)

**What**: Rust and Python each have a `port_is_consumed_from` helper with near-identical logic determining which pattern bindings are live. Cluster D is code deduplication, not a spec extension.

**Action**: extract a shared helper into `emit.rs` (or a new `emit/shared.rs`); both per-target files consume the shared version. No `.dag` change.

**Phase 2 PR**: handled in Phase 3 walker scaffold (PR-3.1) as a refactor; doesn't need its own PR.

### Cluster E — Optional type rendering `(~60-80 LOC dissolvable)`

**What**: Optional types render differently per target: Rust uses `Option<T>`, Go uses `*T` with nil check, Python uses `Optional[T]` from typing. Deref + None-check patterns vary.

**Proposed spec rows**:
```
type OptionalTypeRendering {
  wrapper_template: String         // "Option<{element}>" / "*{element}" / "Optional[{element}]"
  none_literal: String             // "None" / "nil" / "None"
  some_constructor: String         // "Some({value})" / "&{value}" / "{value}"
  deref_syntax: String             // "*{expr}" / "*{expr}" / "{expr}"  (or None-check strategy)
  none_check: String               // "{expr}.is_none()" / "{expr} == nil" / "{expr} is None"
}

data rust_optional: OptionalTypeRendering = { ... }
data go_optional: OptionalTypeRendering = { ... }
data python_optional: OptionalTypeRendering = { ... }
```

**Phase 2 PR**: PR-2.4 (coordinates with E-5 clean-emission rollout).

### Cluster F — Logical operator rendering `(~40 LOC dissolvable)`

**What**: `&&` / `||` in Rust/Go vs `and` / `or` in Python. Today the branch dispatches on target.

**Proposed spec rows**:
```
type LogicalOperatorCarrier {
  and_symbol: String
  or_symbol: String
  not_symbol: String
}

data rust_logical_ops: LogicalOperatorCarrier = { and_symbol: "&&", or_symbol: "||", not_symbol: "!" }
data go_logical_ops: LogicalOperatorCarrier = { and_symbol: "&&", or_symbol: "||", not_symbol: "!" }
data python_logical_ops: LogicalOperatorCarrier = { and_symbol: "and", or_symbol: "or", not_symbol: "not" }
```

**Phase 2 PR**: PR-2.2 (isolated, medium risk, small surface).

### Cluster G — Callable parameter dispositions (ALREADY spec-covered; walker integration concern)

**What**: The brief's initial read suggested this was a gap; verification shows `ParameterDisposition = Borrowed | Consumed` already exists in `emit_model.dag` and is consumed by both per-target files.

**Action**: no spec row needed; move to Category 1. Walker integration is straightforward — dispatch on the already-typed carrier.

**Phase 2 PR**: no PR. Listed here because the gap-analysis surfaced it as a candidate; verification moved it to covered.

### Cluster H — Pattern binding elision strategy `(~50 LOC dissolvable)`

**What**: Unused pattern bindings: Rust uses `_`, Go elides the binding entirely (`_, v := ...`), Python substitutes the expression directly at use sites.

**Proposed spec row**:
```
type UnusedPatternBindingStrategy
  = Underscore              // Rust
  | ElideBinding            // Go
  | SubstituteAtUseSite     // Python

data rust_unused_binding: UnusedPatternBindingStrategy = Underscore
data go_unused_binding: UnusedPatternBindingStrategy = ElideBinding
data python_unused_binding: UnusedPatternBindingStrategy = SubstituteAtUseSite
```

**Phase 2 PR**: PR-2.4 (coordinates with E-5 rollout; see cluster E).

### Cluster I — Variant payload field access (ALREADY spec-covered)

Verification: `PatternBindingRule` variants in `clean_emission.dag` handle this. Move to Category 1.

### Cluster J — Variant constructor argument ordering (ALREADY spec-covered)

Verification: per-target `*_variant_ctor_template` data handles ordering. Move to Category 1.

---

### Cluster summary (after verification)

After audit, the 10 proposed clusters reduce to **5 that need Phase 2 work**:

- Cluster A (Go type recursion) → PR-2.3
- Cluster B + C (execution model + bootstrap filtering) → PR-2.1
- Cluster E + H (optional rendering + unused-binding strategy) → PR-2.4
- Cluster F (logical operators) → PR-2.2
- Cluster D handled inside Phase 3 refactor

---

## Category 3: RESIDUAL_PER_TARGET (narrow + necessary)

Five items stay as narrow per-target Rust logic after Lane 1e closes. Each is semantically necessary — not a gap we can paper over with a spec row.

| Item | LOC | Why it stays | Target phase |
|---|---:|---|---|
| **Rust ownership pipeline** (clone/move/borrow decisions, lifetime annotations) | 600-700 | Semantic choice about memory model, not a syntactic render. LS-4 (Track 2) owns this modeling. When LS-4 lands, this too can become spec-driven; until then it stays in Rust. | Stays residual post-Lane-1e; revisit when LS-4 lands. |
| **Go module system** (`package` declaration, `import` block ordering) | ~20 | System-level config with no Rust/Python analog. Small; lives as a per-target hook in the walker. | Phase 3 escape hatch. |
| **Go Loop unsupported** (explicit fail-closed) | ~10 | Go target doesn't yet support `Behavior::Loop` emission; fails closed with a diagnostic. Temporary until Go-side Loop lands. | Phase 3 flag or code comment; resolves when Go Loop supported. |
| **Python indentation rules** | ~50 | Intrinsic to Python's whitespace-sensitive syntax; block structure can't be a flat template. Lives as a per-target hook in the walker. | Phase 3 per-target hook. |
| **Shared port liveness helper** | ~40 | Not really residual — this is the Cluster D code dedup. Listed here only for completeness. | Phase 3 refactor. |

**Total residual**: ~800-1,200 LOC. 15-20% of the three-file pre-Lane-1e surface. The walker has named per-target hooks for these items.

---

## Phase 2+ Sequencing (proposed)

**Phase 2 — Spec extensions (4 PRs over ~2 weeks):**

| PR | Scope | Size | Risk |
|---|---|---|---|
| PR-2.1 | Cluster B + C (ExecutionModelRequirement + SourceFiltering) | S | Low — foundational, no dispatch logic changes |
| PR-2.2 | Cluster F (LogicalOperatorCarrier) | S | Low — isolated, flat data |
| PR-2.3 | Cluster A (TypeRecursionStrategy for Go) | M | Medium — recursive substitution requires careful template walker |
| PR-2.4 | Cluster E + H (OptionalTypeRendering + UnusedPatternBindingStrategy) | M | Medium — coordinates with clean-emission E-5 rollout |

**Phase 3 — Single walker (5 PRs over ~3 weeks):**

| PR | Scope | Size | Risk |
|---|---|---|---|
| PR-3.1 | Walker scaffold + type rendering (consumes PR-2.3 + per-target type rows) | XXL | High — sets the walker architecture |
| PR-3.2 | Callable + operator dispatch (consumes PR-2.2 + existing operator spec) | L | Medium — largest volume, mechanical |
| PR-3.3 | Pattern matching + branch rendering (consumes PR-2.4) | L | Medium |
| PR-3.4 | Function + type definition rendering | M | Low |
| PR-3.5 | Clean-emission contract integration | M | Low |

**Phase 4 — Cutover (2 PRs):**

| PR | Scope |
|---|---|
| PR-4.1 | Replace all call sites in `emit.rs` with walker dispatch; delete `emit/rust_target.rs` and `emit/python_target.rs`; update SG-0 census and `compiler.dag::hand_maintained_src` |
| PR-4.2 | Stub `src/v3/spec/typescript.dag` (or similar) demonstrating "new target = one spec file, zero walker changes" — the falsifiable proof of the omni-emission claim |

**Phase 5 — Regression (1 PR):**

| PR | Scope |
|---|---|
| PR-5.1 | Golden file verification across the four-fixture matrix + post-emit-verifier gate + DB-8 fixed-point convergence. If any byte drifts, STOP. |

**Total timeline: ~6-8 weeks for the full lane, 12 PRs, single worker (or paired workers on Phase 2 which parallelizes well).**

---

## Estimated impact at Lane 1e close

- **Files deleted**: `src/v3/compiler/src/emit/rust_target.rs` (~5,700 LOC), `src/v3/compiler/src/emit/python_target.rs` (~2,000 LOC), Go inline section in `emit.rs` (~380 LOC). Total ~8,080 LOC removed.
- **Files added**: `src/v3/compiler/src/emit/walker.rs` (or equivalent) ~2,000-2,500 LOC.
- **`.dag` spec additions**: ~500-700 LOC across `spec/rust.dag`, `spec/go.dag`, `spec/python.dag`.
- **Net hand-authored Rust delta**: **−5,500 to −6,000 LOC** (8,080 removed − 2,000 to 2,500 added). Plus ~500-700 LOC of new `.dag` data declarations tracked separately.
- **SG-0 census delta**: `emit/rust_target.rs` off + `emit/python_target.rs` off + `walker.rs` on = **net −1 file**. (File count matters less than LOC here; the LOC drop is where the real dissolution lives.)
- **Thesis claim validated**: adding a new target (TypeScript, Swift, Verilog) is a single `spec/X.dag` file + zero walker/Rust changes. PR-4.2 is the falsifier.

---

## Notes for Phase 2 workers

- **Verify file:line references as you go.** This artifact deliberately omits line-specific citations to keep the planning stable as the code moves; the Phase 2 worker will grep each branch during spec extension.
- **Use the cluster numbering (Cluster A through J) as PR titles** so cross-references work back to this document.
- **Per-PR STOP criterion**: if extending the spec for a cluster reveals a sub-case not covered by the proposed shape, STOP and propose the spec refinement before writing code. Don't paper over with a fallback constant.
- **Per-PR regression criterion**: golden file bytes must remain bit-identical after each Phase 2 PR. Spec extensions alone shouldn't change emitter output until Phase 3 walker consumes them.
- **Cross-check against `docs/emit-bridges.md`** and `docs/single-emitter-design.md` as you go — they're the existing framing documents this artifact builds on.

## Open questions for director (flag if reached during execution)

- **Cluster A Go recursion**: does the current substrate's `TypeConnective::Instantiation` carry enough information for the template to work, or does the walker need to walk into the type tree? If the latter, the "template" abstraction leaks.
- **LS-4 dependency** on Rust ownership residual: we're deferring it to "when LS-4 lands." If LS-4 is indefinite, this residual may become the longest-surviving per-target dispatch. Named as such — not hidden.
- **Spec-row naming convention**: use `<target>_<concept>` consistently (`rust_optional`, `go_optional`, `python_optional`)? Or adopt a per-concept registry shape (`optional_renderings: Map<LanguageId, OptionalTypeRendering>`)? Lane 1e Phase 2 should pick one and stick with it.
