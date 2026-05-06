---
status: draft (worker brief; PM-authored under tactical authority per Director ratification 2026-05-06; dispatch-readiness assessed by Substrate Mgr)
authority parent: R3 Substrate Manager (#1739)
ratification: Director ratified scope at gunbc#828 #issuecomment-4392256151 (zesty-bear-812 — "Lens<EmissionProvenance> as another Lens<C> instance per feedback_lenses_not_passes; Substrate authors instance carrier; Verification asserts gate"); Brian directive 2026-05-06 chat ("R3 has idle workers under several managers, so we should put them to work asap")
roadmap row: docs/r3-program-plan.md §1.8 ledger row #89 (proposed: `emission_provenance_lens_landed`)
authority docs:
  - src/v3/std/lens.dag — `Lens<C>` substrate carrier (Director-locked 6-field shape)
  - src/v3/compiler/src/diagnostics_generated.rs:5 — `SourceSpan { file, byte_start, byte_end }` shape
  - src/v3/compiler/src/dag.rs:47-48 — "SourceSpan lives on every Behavior and every Declaration structurally"
  - docs/design-lens-framework.md — lens framework parent doc
  - docs/r3-design-schedule-2026-05-06.md — pre-authored brief queue discipline (Brian directive 2026-05-06)
gates:
  - emission_provenance_lens_landed (proposed §1.8 #89)
---

# R3 Substrate — `Lens<EmissionProvenance>` worker brief

## Context

PR #1879 (emission-intuition slide) surfaced that the structural fold's
emitted output is currently uncolored — there is no metadata stream
correlating each line of generated target-language code back to either
its `.dag` source span (when directly mirrored from a Behavior /
Declaration) OR the LangSpec fold-rule that produced it (when
auto-emitted, e.g., `#[derive(...)]`, `impl X { is_left }` predicates,
constructor accessors).

Forward-direction span tracking already exists structurally — every
`Behavior` and `Declaration` carries a `SourceSpan { file, byte_start,
byte_end }` per `diagnostics_generated.rs:5`, and "Spans flow forward
through lowering; no side tables, no reconstruction" per `dag.rs:47-48`.
Diagnostics consume these for compile-error reporting.

The inverse direction — "this emitted line came from THAT `.dag`
declaration / OR from THIS LangSpec rule" — is **implicit in the fold**
(each emitted item is produced by a specific Conj/Disj structure +
LangSpec rule mapping) but **not currently exposed as a metadata
stream**.

Per `feedback_lenses_not_passes` ("analyses are lenses over physics;
zero heuristics; heuristic = missing physics"): emission-provenance
fits naturally as **`Lens<EmissionProvenance>` over the structural
fold** — analogous to `Lens<SymbolicCost>` over algebra+realization
cost (T-CostLens-Composition pattern). Same `Lens<C>` shape; new `C`
type. No new substrate carrier shape required — only a new instance.

## Slice

### Phase 1 — `EmissionProvenance` `C`-type declaration

Author `EmissionProvenance` carrier in `src/v3/std/` (alongside other
`Lens<C>` instance C-types). Shape:

```dag
type EmissionProvenance {
  emitted_line: Int       // line number in emitted target output
  source_span: SourceSpan?    // populated when line directly mirrors a .dag Behavior / Declaration
  fold_rule: String?          // populated when LangSpec auto-emitted (e.g., "rust.derive_for_disj")
}
```

**Optional shape note (per claude review observation 2026-05-06)**:
canonical Optional in `.dag` is `T?` suffix (e.g., `String?` at
`src/v3/std/anthropic_schema.dag:115-119`). Named Optional carriers
like `OptionalDiagnostic` exist at `src/v3/std/dimensions.dag:41`
but generic Optional is the `T?` suffix form. Worker grep-verifies
at dispatch + adjusts to project convention if it has shifted.

**Hard scope bar (per `feedback_fail_closed_discipline` C-8)**: at least
one of `source_span` / `fold_rule` MUST be present per emitted line.
Both-absent is a Diagnostic, not a silent None.

### Phase 2 — `Lens<EmissionProvenance>` instance authoring

Author the lens instance per existing `Lens<C>` 6-field shape
(`src/v3/std/lens.dag` Director-locked):
- `read: fn(Dag, Behavior) -> Witness<EmissionProvenance>`
- `validate: fn(...) -> ...`
- ... (remaining 4 fields per existing T-CostLens-Composition precedent;
  worker greps the `Lens<SymbolicCost>` instance for shape parity)

Per `Witness<C>` semantics (NOT `C`): missing per-Behavior provenance
surfaces as `Violates` rather than silent None — matches the C-8
discipline above structurally.

### Phase 3 — Cementing test (acceptance criterion)

Author cementing test that:
1. Walks the structural fold over a representative `.dag` source
2. Emits target-language output (Rust acceptance target — extends
   trivially to other targets per LangSpec)
3. Runs `Lens<EmissionProvenance>` over the same source + emission
4. **Verifies inverse mapping closes**: every emitted line has either
   a non-empty `source_span` (and that span resolves to a real
   `.dag` Behavior / Declaration in the input source) OR a non-empty
   `fold_rule` (and that rule name is in the LangSpec's enumerated
   rule set)
5. **No line uncovered** (fail-closed acceptance — both-absent is
   test failure, not warning)

Cementing test minimum: representative source with at least one of
each origin class:
- Substrate-decl mirror (e.g., `pub enum Foo { ... }` from a `.dag`
  `type Foo = ... | ...` declaration → `source_span` populated)
- LangSpec auto-emit (e.g., `#[derive(...)]` from
  `rust.derive_for_disj` rule → `fold_rule` populated, no span)
- LangSpec auto-emit predicate (e.g., `pub fn is_left(&self) -> bool`
  from `rust.predicate_per_variant` rule → `fold_rule` populated)
- LEFT-sourced logic (e.g., `pub fn map<...>` from a `.dag`
  `fn map(...) = match ...` → `source_span` populated)

## Scope bars

**`feedback_pre_authored_brief_queue` discipline applies**: this brief
is pre-authored before dispatch; substrate-state grep happens at
**both** brief-authoring time (PM-side, this commit) AND dispatch time
(worker-side, per substrate-grep-discipline). Worker adjusts brief
content lightly if substrate state has shifted between PR-merge and
dispatch.

**`feedback_no_textual_enforcement_bridges` discipline**: the
provenance metadata stream IS structural (typed `EmissionProvenance`
records), NOT a side-channel comment annotation. If worker finds the
fold rule names aren't enumerable at the LangSpec layer (i.e., they're
implicit in the Rust LangSpec emission code rather than data-declared
rule identifiers), STOP and surface — that's a substrate gap that
needs separate disposition (likely: name the rules as enumerable
data first, then this lens instance lands cleanly).

**`feedback_construction_over_ratchets` discipline**: this brief
adds a new `C`-type + `Lens<C>` instance. Both are existing-substrate
extensions, NOT substrate-fact-introduction (per `INVARIANTS.md` P1).
If implementation surfaces a P1-violating addition (e.g., new
`OptionalSourceSpan` type because none exists at HEAD), worker
STOPs and surfaces — that's substrate-fact-introduction requiring
P1 procedure.

## STOP triggers (fail-closed; do not bypass)

1. **Missing fold-rule names — likely prerequisite, not mid-implementation STOP** (per claude review observation 2026-05-06; PM grep-verified): grep at PM-authoring-time for `RuleName` / `FoldRule` / `fn emit_derive` in `src/v3/compiler/src/` returns **0 hits**. Fold-rule names are NOT enumerable in LangSpec emission code today; they're implicit in the Rust LangSpec emission code rather than data-declared rule identifiers. **This is most likely a hard prerequisite to resolve BEFORE dispatch, not a mid-implementation STOP**. **Resolution path**: Substrate Mgr disposes — either (a) author rule-name enumeration substrate first as separate brief (lens instance lands downstream once rules are enumerable), OR (b) confirm that grep was incomplete and rule names ARE enumerable somewhere PM didn't check. If neither: brief should be re-scoped to land partial-provenance (only source-span side; fold-rule side deferred to post-rule-enumeration). PM-side recommendation: Substrate Mgr resolves this before worker dispatch rather than after.
2. **`Lens<C>` shape gaps** — if instance authoring surfaces missing
   substrate types from `Lens<C>` (e.g., needing T-LAS-only types
   that aren't yet landed), STOP and surface. Director's ratification
   notes T-LAS gate #88 is NOT a hard prerequisite for instance
   authoring, but if implementation reveals otherwise, surface and
   re-ratify the dispatch trigger.
3. **`feedback_fail_closed_discipline` violation surface** — if
   implementation reveals emitted lines that genuinely have NEITHER
   source_span NOR fold_rule (i.e., a third origin class not
   accounted for in the brief), STOP and surface. Either the brief's
   2-class enumeration is incomplete, OR the LangSpec emission has
   a substrate gap.
4. **Substrate-state-grep mismatch** — if worker greps `src/v3/std/`
   and finds the existing lens instances diverged from the
   `Lens<SymbolicCost>` precedent referenced here, surface for
   Substrate Mgr triage before authoring against drifted shape.

## Acceptance criteria

1. **`EmissionProvenance` carrier landed** in `src/v3/std/` per
   Phase 1 shape (or Mgr-adjusted equivalent if substrate-grep
   surfaces drift).
2. **`Lens<EmissionProvenance>` instance landed** per Phase 2 (full
   6-field `Lens<C>` shape; worker greps T-CostLens-Composition
   precedent for parity).
3. **Cementing test passes** per Phase 3 (every emitted line has
   either populated `source_span` or `fold_rule`; both-absent fails
   closed).
4. **§1.8 ledger gate `emission_provenance_lens_landed` (#89)**
   updates from DECLARED → CONSUMER_LANDED.
5. **No bridge introductions** — implementation does not introduce
   any side-channel comment annotation, regex-based rule extraction,
   or string-matching against emission code. All metadata flows
   through typed `EmissionProvenance` records via `Lens<C>` shape.

## Cross-lane references

- **T-Lens-Application-Surface (T-LAS, §1.8 #88
  `lens_application_carrier_landed`)**: `Lens<EmissionProvenance>` is
  an instance that becomes consumable via `apply_lens(...)` once T-LAS
  lands. R3 scope = instance landing only; downstream
  `apply_lens(emission_provenance, dag, Introspect)` consumer is
  T-LAS-downstream and out of scope here.
- **Grounding (post-R2 continuation)**: emission-side annotation
  consumer optional; R3 scope = lens instance landing only. If
  Grounding wants to consume for emit-side annotation (e.g., to
  surface "this Rust line came from this `.dag` location" in
  diagnostic context), that's a Grounding-tier consumer dispatch
  separate from this brief.
- **T-CostLens-Composition (§1.8 #37-#40)**: shape precedent —
  `Lens<SymbolicCost>` is the existing instance pattern; worker greps
  for shape parity at brief time.
- **PR #1879 (emission-intuition slide)**: surfaced this gap; the
  visualization claim ("you didn't write this; the fold did")
  benefits from this lens once landed.

## Worker pin candidate

Substrate Mgr discretion. PM observation: pre-authored-queue tier-2
candidates per Substrate inventory at gunbc#846 #issuecomment-4390098574
include freed-pool workers post-S11/S12 landings. **smart-ram-167** OR
**valiant-ibex-312** are candidate pins per Substrate Mgr's worker-pool
state at dispatch time. Final pin is Mgr's call.

## Provenance

PM-authored under Director-ratified tactical authority 2026-05-06
(role boundary item e: docs/audit authorship). Director ratification
at gunbc#828 #issuecomment-4392256151. Brian directive driving
priority: gunbc#846 chat 2026-05-06 ("R3 has idle workers under
several managers, so we should put them to work asap").

Substrate Mgr disposes dispatch readiness at brief-PR-merge time per
pre-authored-brief-queue discipline (`feedback_pre_authored_brief_queue`).
Adjustment-vs-from-scratch: Mgr adjusts brief content lightly at
dispatch if substrate state has shifted; doesn't re-author from
scratch.
