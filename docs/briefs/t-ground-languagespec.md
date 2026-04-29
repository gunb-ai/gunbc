# T-Ground-LanguageSpec — `LanguageSpec` substrate authoring + per-target population

**Status:** PROPOSAL — dispatchable when **PR-I** (Q3 `RealizationCost` + Q4 universal four-property gate) merges. Authored 2026-04-29 ahead of PR-I to keep the lane queue warm; Tier 1 design locks (Q1 / reflection-completeness / Q6.5) are LIVE on main via PRs [#1129](https://github.com/gunb-ai/gunbc/pull/1129), [#1156](https://github.com/gunb-ai/gunbc/pull/1156), [#1162](https://github.com/gunb-ai/gunbc/pull/1162).

**Lane:** T-Ground-LanguageSpec (M) — item 6 of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (line 32 + lane row line 65).

**Manager:** R2 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md)).

**Lineage / authorities consumed (no re-litigation):**
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Modeling problem 6 (line 192), §"Affected lanes" option (c) (lines 900-910), MethodContract consolidation (line 942), `BoundDeclaration` consumer note (line 1008), apparent-multi-inhabitance audit (line 895), Q3 `RealizationCost` lock (lines 1143-1208), lane row line 384.
- Reflection completeness: [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) — LanguageSpec as a reflection consumer (no per-consumer projection; line 103).
- THESIS: `THESIS.md:171` — "Coercion = emission. No separate coercion engine."
- Substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 (line 86 onward).
- Brief shape template: [`t-ground-engine-phase-1-typestructure.md`](t-ground-engine-phase-1-typestructure.md).

---

## Framing question this lane answers

What is the *substrate shape* of a language spec, and does populating it for Rust + Python + Go retire the parallel-representation debt that the table-driven `coercion.dag` + `extdeps/languages/*/types.dag` + per-runtime `MethodTranslation` / per-emit `SimpleMethodSpec` schemas accumulated under the prior engine framing?

A "yes" lands the substrate Coercion-Fold consumes for refinement composition; cleans up the slice-1 mirror-consistency probe footprint (PR #989) by re-homing it under structural LanguageSpec authority; and dissolves Reflective Pattern E (`RUST_PILOT_PRIMITIVES` Rust mirror).

A "no" — or any discovery that scope is mis-modeled / authority docs have drifted — escalates to manager (#1133) per the discipline reminders below; do NOT paper over.

---

## Scope

### A. `LanguageSpec` schema authoring (canonical location)

Author the `LanguageSpec` schema as `.dag` substrate. Per Modeling problem 6 (`design-emission-model.md:192-209`) the structural shape declares:

- **Primitive set** (per target) — each carrying refinement-bound shape.
- **Algebra inhabitance per primitive** — with refinement parameters; consumes Q1 `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent` (locked PR-F; per `r2-grounding-manager.md:48`).
- **Structural axes that distinguish candidates** — ownership / growability / encoding / lifetime, modeled as algebra refinements per Modeling problem 2 corrected (NOT canonical-choice; cosmetic equivalents collapse).
- **Structural ordering for diagnostic enumeration** — diagnostic-only per Modeling problem 4 corrected; not load-bearing for emission.
- **Construction patterns** — how a target value is constructed from other target values (compound emission).
- **Operator dispatch** — projection of algebra ops onto target-language operators (`OrderedRing.add` → `i64.add`).
- **External-realization shape** — `Arrow.body` per E-9.
- **Per-primitive `RealizationCost`** — see B below.

**Canonical home (P1-Step-1 DAG-ancestor check):** the brief proposes `src/v3/std/language_spec.dag` (or, if a closer ancestor surfaces during authoring, attach via inhabitance rather than declare a sibling). Worker MUST run the P1 procedure (`INVARIANTS.md:86-123`) and cite which steps resolved the location, *before* declaring the type. If Step 1 surfaces an existing parent (e.g., a target-spec carrier already in `dsl/extdeps/languages/`), re-home there and cite — escalate if no parent fits and you are tempted to declare a fifth bound-style sibling (see Q1 receipt at `design-emission-model.md:1019-1029` for the worked dissolution pattern).

### B. Per-primitive `RealizationCost` (Q3 lock — `design-emission-model.md:1143-1208`)

Each per-target primitive carries:

```dag
type RealizationCost {
  storage: Cost<Bits>
  access:  Map<AlgebraOp, Cost<CPUCycles>>
}
type Cost<Unit> = Dimension<Unit, SymbolicExpr>
```

Substrate primitives `Bits` and `CPUCycles` are substrate-declared sibling to `Meters` / `Seconds` / `Kilograms` in `src/v3/std/dimensions.dag` (per `design-emission-model.md:1155-1158`); PR-I lands these. The `Map<AlgebraOp, Cost<CPUCycles>>` is **sparse fail-closed** (`design-emission-model.md:1206`) — missing op = `Witness.Violates`; no silent zero-cost.

Worker populates per-primitive `RealizationCost` for Rust, Python, Go inhabitants during this lane. Mirror probe footprint (slice 1, see C) supplies the existing per-target primitive set; this lane attaches the cost coordinates.

### C. Re-home target for PR #989 slice-1 mirror-consistency probe — option (a) cleanup

Per `design-emission-model.md:900-910` (option (c) sequencing): slice-1's ~370-line mirror-consistency probe (`validate_loaded_rust_primitive_type_structure`, `validate_rust_primitive_type_structure`, `validate_mirror_consistency`, `validate_first_rust_pilot_row_matches_mirror` at `src/v3/grounding_engine/src/lib.rs`) re-homes under T-Ground-LanguageSpec scope as a **rename + crate-relocation** follow-up. Failure type stays `StructureMismatch { location, expected, actual }`. **Nothing to retract** (no selection logic, no inhabitance-search, no tie-breaking — option (a) sizing is S, not M).

The typed `EmissionDiagnostic` carrier (`UnderRefined` / `NoInhabitant`) lands when fold consumers actually start using it (T-Ground-Coercion-Fold scope), NOT here. Slice 2+ remains held until LanguageSpec lands (option (c) immediate hold).

### D. Reflective Pattern E retirement — `RUST_PILOT_PRIMITIVES` mirror dissolution

Per `r2-grounding-manager.md:32` and lane row line 65: `Dag::rust_pilot_primitives()` becomes the structural authority; the Rust-side `RUST_PILOT_PRIMITIVES` mirror retires. The grounding_pilot crate's deprecation note (deferred from `t-ground-engine-phase-1-typestructure.md` Phase E line 96-98) lands here once consumers walk LanguageSpec structurally. Scope boundary: pilot-crate **deletion** stays in T-Ground-Dissolve; this lane retires the *mirror*, not the crate.

### E. `MethodContract` consolidation (parallel-rep × 3 dissolution — `design-emission-model.md:942`)

Today `dsl/extdeps/languages/{rust,python,go}/runtime.dag` declares `MethodTranslation { dag_method, rust_template }` AND `dsl/extdeps/languages/{rust,python,go}/emit.dag` declares `SimpleMethodSpec { method_name, template, wraps_result }` — different schemas, drifted templates (Rust `count`: runtime `"{recv}.len()"` vs emit `"({recv}.len() as i64)"`; placeholder names `{arg0}` vs `{arg}`), parallel-rep × 3 targets.

Consolidate to one row per `(target, dag_method)`:

```dag
type MethodContract {
  dag_method:             MethodRef
  runtime_template:       Template
  emit_template:          Template
  wraps_result:           Bool
  placeholder_convention: PlaceholderConvention
}
```

Single authority; both runtime and emit consume the same row. Method-translation IS substrate per Modeling problem 2 + the engine-retraction discipline; two parallel authorities for one fact violates THESIS:171 directly. Migration is part of this lane because the consolidated shape is a `LanguageSpec` field, not a sidecar.

### F. Apparent-multi-inhabitance audit (`design-emission-model.md:895`)

For every case that previously looked like "multiple inhabitants needing canonical," re-audit per Modeling problem 2 corrected: is the difference cosmetic (collapse) or meaningful (model the structural axis)?

Sub-tasks (per target):
1. Enumerate the apparent-multi-inhabitance cases against the loaded `LanguageSpec` candidates (Rust `String`/`Box<str>`/`&str`/`Cow<str>` etc.; Python `str`/`bytes`; Go `string`/`[]byte`).
2. For each: classify as **cosmetic** (collapse — retract the redundant candidate) or **meaningful** (extend substrate refinement axis — ownership / growability / encoding / lifetime).
3. Land the disposition as `LanguageSpec` data; cite the worked example in `design-emission-model.md` (Examples 4-7 cover String/&str/Cow shape, lines 525-786) for each meaningful axis.

This is the load-bearing consequence of Q4's universal four-property gate (Faithful / Correct / Minimal / Performant; PR-I lock). Each candidate retained must pass all four; each candidate retracted must produce a structural reason citation.

### G. Substrate-fact-introduction P1 procedure (per `INVARIANTS.md` §P1)

Worker MUST run the 3-step procedure (`INVARIANTS.md:86-123`) for every new substrate type / variant / field introduced under this lane and cite the receipts in the PR body:
- **Step 1 (DAG-ancestor):** which existing parent does the new fact attach to? (Worked example: `LanguageSpec` itself — does an ancestor target-spec carrier exist already?)
- **Step 2 (Coproduct-vs-coordinate):** for each new sum, do all variants ever co-inhabit (→ record) or alternate (→ sum)? (`RealizationCost` is correctly a record — Q3 lock receipt; `BoundDeclaration` is correctly a sum — Q1 lock receipt.)
- **Step 3 (Primitive-vs-lens-extensible):** for new leaves (e.g., `PlaceholderConvention`), are they substrate primitives or lens-extensible labels?

Per [`feedback_substrate_principle_audit.md`] and `r2-grounding-manager.md:106`, this is non-optional for LanguageSpec / Lifetime-Analyzer briefs.

---

## Out of scope (do NOT do)

- **Coercion-Fold body.** This lane lands the substrate facts the fold reads; the fold itself is T-Ground-Coercion-Fold (S; held per option (c) until this lane lands).
- **Lifetime / ownership derivation.** T-Ground-Lifetime-Analyzer (M; sibling lane). LanguageSpec declares the structural axis (ownership: Owned / Borrowed{lifetime} / etc.); the *derivation from program use* belongs in Lifetime-Analyzer.
- **`EmissionDiagnostic` carrier authoring.** T-Ground-Diagnostic (S; sibling lane). Q6.5 two-layer authority is LIVE per #1129 — this lane is a Layer-1 consumer of `CompilerDiagnosticKind`; do NOT extend.
- **Cross-target portability meta-spec.** T-Ground-CrossTarget-Meta (S).
- **Track-13 dissolution** — `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` deletion stays in T-Ground-Dissolve.
- **Pilot-crate deletion** — T-Ground-Dissolve.
- **Touching `src/v3/compiler/`** — SG-0 ratchet.
- **Q5 cardinality enumeration work** — gated on PR-J (likely no-op); T-Ground-CrossTarget-Meta scope.
- **Re-litigating Q1 / Q2 / Q3 / Q4 / Q6.5 locks.**

---

## Dependencies / gates

| Gate | Status | Lane impact |
|---|---|---|
| **PR-PreF** (Substrate; `Interval<D>` consolidation) | per Substrate Manager | Required for Q1 instance consumption |
| **PR-F** (Q1 BoundDeclaration + Q2 Rust axes) | per `r2-grounding-manager.md:48` | Required for primitive-bound declaration shape |
| **PR-G** (Q2 Python axes) | per `r2-grounding-manager.md:49` | Required for Python population (B, F) |
| **PR-H** (Q2 Go axes) | per `r2-grounding-manager.md:50` | Required for Go population (B, F) |
| **PR-I** (Q3 `RealizationCost` + Q4 universal gate) | **dispatch gate** | Lands `Bits`/`CPUCycles` substrate primitives + `Cost<Unit>` alias + `RealizationCost` record schema; this lane populates per-inhabitance |
| #1129 / #1156 / #1162 (Tier 1 locks) | LIVE on main | Consumed (Q1 / reflection-completeness / Q6.5) |

**Cross-program signals:**
- **Substrate Manager — `Diagnostic.kind` / Q6.5:** consumer-only (Layer 1); no handoff.
- **Substrate Manager — ValueBody-list/sum + std.unicode:** NOT a hard gate for this lane (Coercion-Fold consumes it); LanguageSpec can land its substrate shape + per-target population without it. Coordinate via cross-manager queue if population requires walking pilot-list data.

---

## Sizing

**M** per `r2-grounding-manager.md:65` and `design-emission-model.md:384`. Distribution (informal):
- Schema authoring (A): S — one substrate type + sub-types; P1 receipts.
- `RealizationCost` per-target population (B): S × 3 targets ≈ M.
- Slice-1 re-home (C): S — rename + crate relocation; no selection-logic retraction.
- Reflective Pattern E retirement (D): S — consumer-migration, mirror delete.
- `MethodContract` consolidation (E): S — schema unification + drift-resolution per target.
- Apparent-multi-inhabitance audit (F): S × 3 targets ≈ M.

Bundle into one PR per `feedback_bundle_workstreams_per_pr.md` unless scope balloons; if (E) or (F) surfaces an unanticipated substrate gap, escalate to manager (#1133) before splitting.

---

## Test plan

Per `TESTING.md` — hermetic, behavior-driven, unit-first; sub-second per `feedback_test_timeout_2s.md`.

Acceptance lifted to a `.dag` `TestClaim` (gate: `language_spec_realization_cost_landed` per `r2-grounding-manager.md:125`):

1. **Schema-load test** — `LanguageSpec` substrate loads through reflection without per-consumer projection (per `design-reflection-completeness.md:103`).
2. **Per-target population parity** — Rust / Python / Go specs each declare every primitive that previously appeared in `extdeps/languages/*/types.dag` plus the cost coordinates from B; no primitive lost.
3. **`RealizationCost` sparseness fail-closed** — missing `AlgebraOp` in the `access` map produces `Witness.Violates`, not silent zero-cost (per `design-emission-model.md:1206`).
4. **Mirror-consistency probe (re-homed)** — slice-1's `StructureMismatch` shape continues to fire on intentional drift between `Dag::rust_pilot_primitives()` and the (about-to-retire) Rust mirror, until D's mirror retirement lands; final state: probe transitions to "structural LanguageSpec walk only" with no Rust mirror to compare.
5. **MethodContract consolidation** — `runtime_template` and `emit_template` co-resolved per `(target, dag_method)`; the prior `count` drift (Rust `"{recv}.len()"` vs `"({recv}.len() as i64)"`) is resolved into one declared row that carries both templates with `wraps_result` made explicit (no silent reconciliation).
6. **Apparent-multi-inhabitance audit receipts** — every retained candidate cites the meaningful axis it inhabits; every retracted candidate cites the cosmetic-equivalent it collapsed into. (Receipt-shaped test: `LanguageSpec` row carries the citation; missing citation = test failure.)
7. **Q4 four-property gate** — every per-target inhabitance verifies against Faithful / Correct / Minimal / Performant (PR-I lock; consume the verifier landed there).

`cargo test --workspace --exclude v2-compiler-tests`, `cargo test -p v2-compiler-tests`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check` all clean.

---

## Dissolution claim

When this lane lands:

- `dsl/std/coercion.dag` table-driven schema retires (consumed via `T-Ground-Dissolve`); the structural fold reads `LanguageSpec` directly. Specifically: `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` lose their last consumer (final Track-13 deletion happens in T-Ground-Dissolve, not here).
- `RUST_PILOT_PRIMITIVES` Rust mirror retires (Reflective Pattern E closure).
- Parallel `MethodTranslation` (runtime.dag) ⊕ `SimpleMethodSpec` (emit.dag) × 3 targets collapses to single `MethodContract` row per `(target, dag_method)`.
- Slice-1 mirror-consistency probe re-homes from `grounding_engine` to LanguageSpec's structural authority.
- `extdeps/languages/{rust,python,go}/types.dag` table shape converges with `LanguageSpec` reflection (specifics depend on what schema authoring surfaces; if the table shape IS the LanguageSpec realization, the dissolution is in-place renaming + extension, not a separate file delete).

The `coercion.dag` shape that retires is named per the receipt landed on PR — worker MUST cite the specific declarations dissolved (so the dissolution claim is verifiable, not aspirational, per `feedback_holistic_over_patches.md` + `feedback_root_causes_over_quick_fixes.md`).

---

## Hand-off discipline

Escalate to manager (post on #1133, do NOT absorb in lane) if:

- The P1 DAG-ancestor check (Step 1) reveals a parent type whose extension would itself be a non-trivial substrate change (i.e., authoring `LanguageSpec` would require *first* extending an upstream parent).
- An apparent-multi-inhabitance case in (F) cannot be classified cleanly as cosmetic or meaningful — the four-property gate from Q4 doesn't disambiguate it. (This is a Modeling problem 2 design call; manager → Director if needed.)
- `MethodContract` consolidation surfaces a third schema or per-target divergence not captured at `design-emission-model.md:942` (e.g., a target whose template language is incompatible with the unified `placeholder_convention`).
- The slice-1 re-home requires modifying the `StructureMismatch` failure shape (it shouldn't — option (a) is rename + relocation only).
- Any work would require touching `src/v3/compiler/` (SG-0 ratchet violation).
- A referenced authority doc has drifted from the cited line numbers — STOP and post to manager. Don't paper over.

Per `feedback_root_causes_over_quick_fixes.md`: no quick fixes. Per `feedback_no_textual_enforcement_bridges.md`: no grep/regex bridges to "be structural."

---

## Acceptance — `.dag` gate

Lane closes under the `r2-grounding-manager.md:125` acceptance gate:

> `language_spec_realization_cost_landed` — per-primitive `RealizationCost { storage, access }` populated; `RUST_PILOT_PRIMITIVES` mirror retired.

Authored as a `.dag` `TestClaim`. Per the **structural-acceptance-per-lane-close discipline** (`r2-grounding-manager.md:11`), the gate IS the demo — no separate artifact.

PR body covers: scope (A-G); per-target population status (Rust full / Python full / Go full); P1 receipts cited per new substrate type; Q4 four-property gate per inhabitance; dissolution receipts (specific `coercion.dag` declarations retired, `MethodTranslation` ⊕ `SimpleMethodSpec` rows consolidated, `RUST_PILOT_PRIMITIVES` mirror deletion).

---

## What unblocks on merge

- **Manager** updates lane row at `r2-grounding-manager.md:65` to LANDED; signals R2 Release Manager (closure ledger).
- **T-Ground-Coercion-Fold** Phase 2 slice 2+ unblocks (option (c) hold lifts; further fold slices dispatch).
- **T-CostLens-Composition** (R3) substrate prereq for per-target realization-cost declarations is satisfied (per `design-emission-model.md:340`).
- **T-Ground-Dissolve** can begin retiring `coercion.dag` table-driven shape (`TypeCheckpoint` / `InhabitantDecl` / `carrier: String` deletions).
- **Apparent-multi-inhabitance receipts** populate the substrate; future Coercion-Fold examples cite them rather than re-derive.

---

## Cross-refs

- Parent: [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane 6 of 11)
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Modeling problem 6 + §"Affected lanes" + Q3 lock + line 942 MethodContract
- Reflection completeness: [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md)
- THESIS: `THESIS.md:171` (engine-retraction)
- Substrate-fact-introduction: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
- Brief shape template: [`t-ground-engine-phase-1-typestructure.md`](t-ground-engine-phase-1-typestructure.md)
- Sibling lanes: [`t-ground-engine-substrate-audit.md`](t-ground-engine-substrate-audit.md), [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md) (both pre-cascade; post-cascade naming aligns with the 11-lane structure)
