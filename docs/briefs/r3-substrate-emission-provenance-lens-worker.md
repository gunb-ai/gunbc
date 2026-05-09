---
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
