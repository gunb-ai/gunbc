---
<<<<<<< HEAD
status: PROPOSAL (worker brief; PM-authored under tactical authority per Director ratification 2026-05-06; **NOT dispatch-ready** pending Substrate Mgr canvas on Lens<C> read-shape question raised by codex BLOCKING 2026-05-06; dispatch-readiness assessed by Substrate Mgr)
authority parent: R3 Substrate Manager (#1739)
ratification: Director ratified scope at gunbc#828 #issuecomment-4392256151 (zesty-bear-812 — "Lens<EmissionProvenance> as another Lens<C> instance per feedback_lenses_not_passes; Substrate authors instance carrier; Verification asserts gate"); Brian directive 2026-05-06 chat ("R3 has idle workers under several managers, so we should put them to work asap")
roadmap row: **TBD — gate authority pending** per codex BLOCKING 2026-05-06; no #89 (already taken by `section_ref_substrate_landed` under T-Lens-Application-Surface). Candidate retargets per Substrate Mgr disposition: (a) new gate added to T-CostLens-Composition lane scope (analogous to existing #37-#40 cluster); (b) new gate added to T-Lens-Application-Surface lane scope; (c) deferred until per-Behavior framing ratified
authority docs:
  - src/v3/std/lens.dag — `Lens<C>` substrate carrier (Director-locked 6-field shape; **`read: fn(Dag, Behavior) -> Witness<C>`** — per-Behavior read; codex BLOCKING 2026-05-06 surfaced category mismatch with per-emitted-line goal)
  - src/v3/compiler/src/diagnostics_generated.rs:5 — `SourceSpan { file, byte_start, byte_end }` shape
  - src/v3/compiler/src/dag.rs:47-48 — "SourceSpan lives on every Behavior and every Declaration structurally"
  - docs/design-lens-framework.md — lens framework parent doc
  - docs/r3-design-schedule-2026-05-06.md — pre-authored brief queue discipline (Brian directive 2026-05-06)
gates:
  - TBD per Substrate Mgr canvas — not landed in §1.8 ledger
---

## STATUS — PROPOSAL pending Substrate Mgr canvas (codex BLOCKING 2026-05-06)

Codex BLOCKING review at sha `3c96212d` (PR #1902) surfaced 3 valid findings:

1. **Optional+invariant origin → typed-sum origin** (applied below; structural fix per `feedback_state_space_vs_behavioral_invariants`)
2. **Lens<C> reads (Dag, Behavior) → Witness<C>; emission provenance is per-line, not per-Behavior** (category mismatch — substantive Substrate Mgr canvas territory; brief stays PROPOSAL until ratified)
3. **§1.8 #89 already taken by `section_ref_substrate_landed`** (gate authority retargeting required; PM grep-error)

**The load-bearing finding is #2 — DEEPER than initially framed** (per codex BLOCKING inline at line 73, sha `c375eba45`):

Initial frame: "Lens<C> is per-Behavior; goal is per-emitted-line — category mismatch on granularity."

**Deeper frame**: Lens<C>.read **domain** is `Behavior = Value | Transform | Branch` (per `src/v3/std/substrate.dag:465`). The brief's acceptance surface includes lines from **Declaration-origin** (e.g., `pub enum Sum<A,B>` from a `.dag` `type Sum = ...` declaration; `pub struct HttpError` from `type HttpError {...}` declaration) AND **LangSpec auto-emits** (e.g., `#[derive(...)]` / `impl X { is_left, is_right }` / `impl X { new }` from no-`.dag`-source LangSpec rules). **Both are outside Lens<C>.read's domain entirely** — Declarations are at `src/v3/std/substrate.dag:235`, structurally separate from Behaviors. Auto-emits have no `.dag` source at all.

So the mismatch is structural at TWO axes simultaneously:
- **Granularity**: per-Behavior (lens) vs per-emitted-line (goal)
- **Domain coverage**: Lens<C> can read only Behaviors; brief's acceptance includes Declaration-origin + LangSpec auto-emits, which Lens<C> structurally cannot read

To satisfy P2 boundary discipline without parallel lookups, the brief would need either:
1. A new structural event surface that includes Declarations + LangSpec auto-emits (so Lens<C> CAN read them) — substantive new substrate
2. Narrow acceptance to only Behavior-source lines (so Lens<C> read domain matches) — extremely narrow scope; doesn't cover Brian's visualization
3. A different shape entirely (NOT Lens<C>) — emission-fold instrumentation as a separate substrate

Reframing paths (revised per deeper codex finding):

- **(a) Per-Behavior, narrow domain** (Lens<C>-compatible, Behavior-only): "for Behavior B, the emitted lines attributable to B are [line-range / typed-rule] tuples." Lens<C> reads (Dag, Behavior) → Witness<List<EmissionAttribution>>. **Excludes Declaration-origin + auto-emit lines entirely**. Very narrow scope; doesn't support Brian's full visualization.
- **(b) Per-emitted-line instrumentation, NEW substrate** (NOT a lens): emission-fold instrumentation that runs DURING emission and records per-line origin (with typed `LangSpecRule`). Different substrate shape entirely; matches Brian's visualization need; not a `Lens<C>` instance. Requires substrate-fact-introduction (P1 procedure).
- **(c) Withdraw brief**: pre-canvas the question via Substrate Mgr canvas first; re-author brief once shape ratified. **Stronger PM recommendation given the deeper mismatch.**

PM read (revised): **strongly recommend (c)** — withdraw + canvas first. The Lens<C> framing isn't just narrow; it's structurally insufficient for the brief's acceptance surface. Substantive reshape needs Substrate Mgr canvas + Director ratification on the right shape (per-Behavior narrow lens vs new emission-instrumentation substrate vs different framing entirely).

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
// Typed origin sum — codex BLOCKING 2026-05-06 (parent + inline at line 57):
// (a) provenance origin must be a non-empty typed authority, NOT optional
//     coordinates with a runtime-asserted "at-least-one" invariant
// (b) fold-rule reference must be a typed LangSpec rule ref, NOT an
//     arbitrary string (otherwise admits arbitrary fold-rule strings;
//     fails P2 illegal-states-unrepresentable)
// Per feedback_state_space_vs_behavioral_invariants: type enforcement >
// API enforcement.
type EmissionOrigin =
    SubstrateDeclMirror { span: SourceSpan }     // line directly mirrors a .dag Behavior / Declaration
  | FoldRuleAutoEmit { rule: LangSpecRule }      // LangSpec auto-emitted; rule is typed enum, NOT String

type EmissionProvenance {
  emitted_line: Int     // line number in emitted target output
  origin: EmissionOrigin   // REQUIRED, typed sum — fail-closed by construction
}
```

**Hard prerequisite (per codex BLOCKING + cross-relay to Substrate Mgr at gunbc#1739 #issuecomment-4392435376)**: `LangSpecRule` typed enumeration MUST exist as a substrate carrier before this brief can dispatch. PM grep at brief-authoring time: `RuleName` / `FoldRule` / `fn emit_derive` returns 0 hits in `src/v3/compiler/src/`; fold-rule names are NOT enumerable today. **Substrate Mgr disposes**: (a) author `LangSpecRule` enumeration substrate first as separate brief; (b) confirm grep was incomplete + rules ARE enumerable elsewhere; (c) brief dispatched only after typed-rule prerequisite lands.

**Structural fail-closed (per codex BLOCKING finding 1 + `feedback_state_space_vs_behavioral_invariants`)**: the `Disj` carrier `EmissionOrigin` over typed `SourceSpan` + typed `LangSpecRule` makes both:
- "at least one origin class is present" structurally true by construction
- "rule names are well-formed LangSpec identifiers" structurally true by construction (no arbitrary strings admitted)

Eliminates two structural-recovery patterns simultaneously.

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
=======
status: dispatchable per Director Reading C RATIFICATION at gunbc#1739 #issuecomment-4392797954 (2026-05-07). T-Rule-Enumeration retired (substrate gap closed structurally via existing field-path enumeration on `*SyntaxBinding`/`*OpsBinding` structs). Worker pin: smart-ram-167 (Mgr discretion per Director ratification; smart-ram has fresh context from substrate-state-grep that surfaced Reading C; valiant-ibex-312 already dispatched on Rust-primitive-full-coverage / T-Interval-Representation).
authority parent: R3 Substrate Manager (#1739)
ratification: Q1 (a) per-Behavior Lens<C>-compatible RATIFIED at gunbc#1739 #issuecomment-4392562911 (zesty-bear-812, 2026-05-06). Q1 (b) per-line instrumentation REJECTED. Q1 (d) parallel substrates REJECTED. Q3: gate `emission_provenance_lens_landed` under T-CostLens-Composition cluster.
roadmap row: §1.8 ledger row TBD slot — gate `emission_provenance_lens_landed` under T-CostLens-Composition cluster per Director Q3 ratification
authority docs:
  - gunbc#1739 #issuecomment-4392562911 (Director Q1+Q2+Q3 RATIFICATION)
  - docs/briefs/r3-substrate-emission-provenance-shape-canvas.md (Substrate Mgr canvas; parent)
  - docs/briefs/r3-substrate-t-rule-enumeration-worker.md (PREREQUISITE — must land first)
  - src/v3/std/lens.dag (Lens<C> Director-locked 6-field carrier)
  - PR #1879 (emission-intuition slide — visualization consumer)
gates:
  - `emission_provenance_lens_landed` (proposed §1.8 row; slot pending T-CostLens-Composition cluster)
worker pin: smart-ram-167 (Mgr discretion per Director Reading C ratification; fresh context on field-path enumeration from substrate-state-grep)
---

# R3 Substrate — `Lens<List<EmissionProvenance>>` worker brief (per (a) per-Behavior framing)

## Context

Director Q1 (a) RATIFIED at gunbc#1739 #issuecomment-4392562911:
per-Behavior Lens<C>-compatible framing. Reasoning per Director:

> `feedback_lenses_not_passes`: analyses are lenses over physics; zero
> heuristics; heuristic = missing physics. Emission-provenance IS
> analysis (which lines came from where); the right substrate shape
> is a lens over physics, not runtime instrumentation.

> `feedback_compositional_not_templating`: don't materialize parallel
> structures when one composes correctly. (a) composes; (b) is
> instrumentation that drops compositional structure.

(b) per-line instrumentation REJECTED; (d) parallel-substrates
REJECTED — Brian's slide visualization is served by (a) projection
(per-Behavior list flattens to per-line view at the visualization
layer), not parallel substrate.

This brief revises the earlier PM-authored proposal (PR #1902 merged
at 54419badf, but the brief shape needed Substrate Mgr canvas + Director
re-ratification per the codex BLOCKING category-mismatch finding).

## No precondition gate (Reading C ratification)

Earlier draft gated dispatch on T-Rule-Enumeration landing. Per
Director Reading C RATIFICATION at gunbc#1739 #issuecomment-4392797954:
T-Rule-Enumeration is RETIRED — substrate gap was a phantom; rule
enumeration is structurally already provided by the `*SyntaxBinding`
/ `*OpsBinding` field-path set in `src/v3/compiler/src/emit/rust_target.rs`
(~37 distinct field paths catalogued by smart-ram-167's substrate-state-grep
at gunbc#1759 #issuecomment-4392696623).

`EmissionRule` carrier IS the field-path string itself (e.g.,
`"indexes.syntax.statements.let_binding"`). No new substrate
authoring needed; `EmissionRule = String` of field-path per Director
Q3 ratification:

> `feedback_reason_not_label`: field-path IS the stable reason;
> nominal alias is the volatile label. Substrate carries the structural
> truth; aliases are derived display surface.

Brief dispatches immediately.

## Scope

### Deliverable 1 — `EmissionProvenance` carrier (record form)

Author carriers in `src/v3/std/`. Per codex Finding 1 reshape (typed-sum
origin, NOT optional-pair-with-runtime-invariant — applied in PR #1902
at 0110f739d before supersession):

```dag
type EmissionProvenance {
  emitted_line: Int                  // line number in emitted target output
  rule: EmissionRule                 // MANDATORY — every emitted line is produced by some rule
                                     //   (even trivial mirror-rules); type system enforces
  source_span: Option<SourceSpan>    // OPTIONAL — populated when the rule's input traces to a
                                     //   `.dag` source declaration; absent for purely structural
                                     //   emissions (e.g., headers, fixed scaffolding)
}
```

**Reshape rationale** (per codex BLOCKING at PR #1910 sha 2ed1046e Finding #1): earlier typed-sum form `EmissionOrigin = SubstrateDeclMirror(SourceSpan) | FoldRuleAutoEmit(EmissionRule)` conflated two orthogonal axes. A single emitted line genuinely can have BOTH attributions — e.g., `#[derive(Debug)]` emitted on a Foo enum has rule=`derive_for_disj` AND source-span attribution to the Foo declaration in the `.dag` source. The two facts are not mutually exclusive; modeling them as sum variants forced false either-or framing. Record (product) form with mandatory rule + optional span is faithful to actual emission semantics and structurally fail-closed (rule presence enforced by type system; never both-absent).

`EmissionRule = String` (field-path; Director Q3 ratification at
gunbc#1739 #issuecomment-4392797954). Worker references the existing
`*SyntaxBinding` / `*OpsBinding` field-path set as the de-facto
enumeration; no new rule-name carrier authored. Field-path-string
preserves structural navigability (per `feedback_reason_not_label`);
nominal aliases live at display layer if desired (visualization-side
projection, not substrate-side).

**Practice 4 classification**: N/A — `EmissionProvenance` is a record
(product), not a sum (coproduct). Practice 4 coproduct-dissolution
rubric (🟢 / 🟡 / 🔴) applies to N≥2-variant sum types; record types
are structurally classified by their field set, not coproduct status.

The earlier draft used a typed-sum `EmissionOrigin` shape with GREEN
classification + terminal receipt; codex BLOCKING at PR #1910 sha
2ed1046e Finding #1 surfaced that the sum framing was mis-modeled
(the two facts can co-inhabit; sum forces false either-or). Record
form with mandatory `rule: EmissionRule` + optional `source_span:
Option<SourceSpan>` is the faithful shape. No Practice 4 ledger
entry needed; structural fail-closed is enforced by `rule` field's
required-presence in the record type.

No §1.8 sibling-row needed (record-type, not coproduct; Practice 4
ledger-entry-if-GREEN rubric doesn't apply). Parent gate
`emission_provenance_lens_landed` is the sole row.

### Deliverable 2 — `Lens<List<EmissionProvenance>>` instance

Author lens instance per Director-locked 6-field shape:

- `name: "EmissionProvenance"` (or canonical project-naming convention)
- `read: fn(Dag, Behavior) -> Witness<List<EmissionProvenance>>` —
  per-Behavior fold computing the provenance list. For each emitted
  line attributable to this Behavior, populate:
  - `rule: EmissionRule` (= field-path String) — the field-path of the
    rule that produced this line (always present; trivial-mirror rules
    count). Captured from the `render_named_template(...)` call site's
    field-path argument at emission time.
  - `source_span: Option<SourceSpan>` — populated when the rule's
    input traces to a `.dag` source declaration; absent for purely
    structural emissions (e.g., headers, fixed scaffolding rules
    that don't have a per-line source-decl input)
- `sequential: Monoid<List<EmissionProvenance>>` — list-concat monoid
  (`empty: []`, `concat: [...] ++ [...]`)
- `branch: (List, List) -> List` — concat over both arms (static
  emission tracking captures both branches; runtime exclusivity is
  orthogonal to static-emission analysis)
- `iterate: (List, LoopBound) -> List` — identity (body emitted once;
  bound is data, not source). Worker confirms at dispatch via
  substrate-state-grep on actual emission semantics; STOP if assumption
  breaks
- `validate: fn(Dag, List<EmissionProvenance>) -> OptionalDiagnostic` —
  record form makes "rule absent" structurally impossible (type system
  enforces). Aggregate validation: surface diagnostic if `rule`
  field-path doesn't correspond to a real field on
  `*SyntaxBinding`/`*OpsBinding` structs (worker DFS-catalogs the
  field set at dispatch; round-trip verifies the field-path resolves);
  optional-`source_span` absence is structurally legal, NOT a diagnostic

Per `feedback_compositional_not_templating`: per-Behavior fold composes
to per-line view at the visualization layer (flatten the per-Behavior
List<EmissionProvenance>); no parallel substrate needed for slide.

### Deliverable 3 — Cementing test

Author cementing test that:
1. Walks structural fold over a representative `.dag` source (likely
   reuses T-CostLens cementing-test source corpus)
2. Emits target-language output (Rust acceptance target)
3. Runs `Lens<List<EmissionProvenance>>` over the same source
4. **Verifies (scoped to Behavior-attributable lines only)**: every
   emitted line that traces to a Behavior in the source `.dag` has a
   corresponding `EmissionProvenance` entry in the per-Behavior
   aggregate; flatten to per-Behavior-attributable-line view matches
   the emitted-line subset. Declarations + program scaffold
   (prelude / function / main-wrapper sections) are OUT-OF-SCOPE per
   codex Finding #2 at sha 4229cd09 — separate substrate cascade if
   needed
5. **Verifies field population**: every entry's `rule` field-path
   resolves to a real `*SyntaxBinding`/`*OpsBinding` field
   (round-trip: emitter writes field-path → lens recovers same
   field-path → no information loss); ≥1 entry has `source_span:
   Some(...)` (substrate-decl-traceable); ≥1 entry has `source_span:
   None` (structural-only emission); span-Some entries' SourceSpan
   resolves to a real `.dag` Behavior/Declaration in the input source

### Deliverable 4 — §1.8 ledger receipt

Add `emission_provenance_lens_landed` to §1.8 ledger under
T-CostLens-Composition cluster (Director Q3). Advance DECLARED →
PRODUCER_LANDED on merge (cementing test verifies producer shape; no
Grounding-side visualization-consumer wiring in this PR).

## Slice — single PR

Phase ordering (PR-internal):
1. DFS-catalog `*SyntaxBinding`/`*OpsBinding` field set in `src/v3/compiler/src/emit/rust_target.rs` (Reading C confirmed ~37 field paths; worker re-greps at HEAD for actual count)
2. Author `EmissionProvenance` record carrier (Deliverable 1)
3. Author `Lens<List<EmissionProvenance>>` instance (Deliverable 2)
4. Author cementing test (Deliverable 3)
5. Verify all standard ratchets green
6. §1.8 ledger row receipt (Deliverable 4)

## Acceptance

- `EmissionProvenance` record carrier landed in `src/v3/std/` with
  mandatory `rule: EmissionRule` field + optional
  `source_span: Option<SourceSpan>` field. Record form per codex
  BLOCKING reshape (PR #1910 sha 2ed1046e Finding #1) — replaces
  earlier typed-sum framing; Practice 4 N/A (record, not coproduct)
- `Lens<List<EmissionProvenance>>` instance landed per 6-field
  Director-locked shape; T-CostLens-Composition precedent verified for
  shape parity
- Cementing test landed (scope = **Behavior-attributable lines only**;
  declarations + program scaffold OUT-OF-SCOPE per codex Finding #2
  at sha 4229cd09 — separate substrate cascade if needed): per-Behavior
  fold output flattens to per-Behavior-attributable-line view; ≥1
  Behavior-attributable entry with span-Some + ≥1 Behavior-attributable
  entry with span-None; field-path round-trip verified (every `rule`
  field-path resolves to a real `*SyntaxBinding`/`*OpsBinding` field at
  HEAD; STOP if field-path threading requires substrate cascade per
  codex Finding #1)
- §1.8 row `emission_provenance_lens_landed` advances DECLARED →
  PRODUCER_LANDED (Grounding-consumer for visualization wiring is a
  separate downstream brief if needed; not bundled per Director
  bundled-scope discipline at gunbc#1739 #issuecomment-4392225548)
- `cargo test --workspace --exclude v2-compiler-tests` green
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`
- 5-question authority audit in PR body

## STOP-AND-ESCALATE

- **`*SyntaxBinding` / `*OpsBinding` field set materially differs from Reading C catalog at HEAD** (e.g., heavy refactor renamed the structs): STOP — substrate-state-divergence; surface to Substrate Mgr; brief absorbs corrected field-path enumeration before authoring
- **Field-path identity not threadable through emission runtime** (per codex BLOCKING at PR #1910 sha 4229cd09 Finding #1, post-merge): the `render_named_template(...)` call site receives a template *value* extracted from the binding struct field — the field-path that names the rule may not be carried alongside the value at emission time. If lens `read` cannot recover field-path from runtime emission state without structural threading (typed template-with-rule-path or DeclarationRef carrier wrapping the template), STOP and surface — that's a substrate-fact-introduction cascade (separate brief; thread rule-path through binding carrier before lens consumes). Worker DFS at dispatch confirms whether field-path threading is structurally possible OR requires substrate cascade
- **Non-Behavior emission coverage gap** (per codex BLOCKING at PR #1910 sha 4229cd09 Finding #2): `Lens<C>.read: fn(Dag, Behavior) -> Witness<List<EmissionProvenance>>` is per-Behavior; emitted output also includes declarations + program scaffold (e.g., `pub mod`, file headers, `#[derive(...)]` on declared types). Brief Acceptance scope is **Behavior-attributable lines only** — declarations and program scaffold are out-of-scope for this slice. If cementing test surfaces emitted lines unattributable to any Behavior (i.e., the per-Behavior fold doesn't see them), STOP and surface — declared-provenance / scaffold-provenance is separate substrate-fact-introduction (different lens or per-Declaration extension). Acceptance bullet narrowed below to "≥1 Behavior-attributable span-Some + ≥1 Behavior-attributable span-None" rather than full-emission coverage
- **`iterate` identity assumption breaks** (body emitted multiple times
  per LoopBound, NOT once): STOP — surface to Substrate Mgr; lens
  iterate field shape may need rework
- **Cementing test reveals provenance gap** — emitted line that the
  per-Behavior fold cannot attribute to any rule: STOP. Record form
  requires `rule` populated; rule absence indicates emission code path
  not covered by T-Rule-Enumeration carrier (substrate-fact-introduction
  cascade). Do NOT add a sentinel rule like "Unknown" — surface to
  Substrate Mgr
- **Bundled-scope drift**: do NOT bundle T-Rule-Enumeration edits or
  Grounding-side visualization-consumer wiring into this PR

## Authority audit receipt

1. **Substrate exists?** At brief-author time:
   - `Lens<C>` carrier landed (`src/v3/std/lens.dag`, 🟢 TERMINAL) ✓
   - `EmissionRule` carrier — gates on T-Rule-Enumeration landing
   - `EmissionProvenance` record carrier — this brief is producer
   - Lens instance — this brief is producer
2. **Existing brief?** PM-authored proposal at PR #1902 (merged at
   54419badf) is the prior artifact; this brief revises per Substrate
   Mgr canvas + Director Q1 (a) RATIFICATION
3. **Design-doc match?** Director Q1 (a) RATIFIED + canvas Q1 disposition
   + Lens<C> Director-locked shape. T-CostLens-Composition is shape
   precedent
4. **Citations live?** Worker re-verifies at dispatch
5. **Carrier dissolves the bridge?** Yes — record `EmissionProvenance`
   with mandatory `rule` + optional `source_span` dissolves the
   "what produced this emitted line?" bridge structurally (rule field
   always present; no None possible; type system enforces). Per-Behavior
   `Lens<List<EmissionProvenance>>` composes to per-line view at
   visualization layer per `feedback_compositional_not_templating`

## Provenance

Revised 2026-05-06 by Substrate Mgr per Director Q1 (a) RATIFICATION
at gunbc#1739 #issuecomment-4392562911. Brief revises PM-authored
proposal at PR #1902 (merged at 54419badf; brief shape needed canvas
+ re-ratification per codex BLOCKING category-mismatch finding).

Cross-references:
- Canvas `r3-substrate-emission-provenance-shape-canvas.md` (parent)
- T-Rule-Enumeration brief (PREREQUISITE)
- PR #1902 (PM proposal — brief shape superseded by this revision)
- Lens<C> Director-locked shape at `src/v3/std/lens.dag`
- T-CostLens-Composition shape precedent (#37-#40 cluster)
>>>>>>> origin/main
