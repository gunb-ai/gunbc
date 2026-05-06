---
status: queued (worker brief; revised by Substrate Mgr per Director Q1 (a) RATIFICATION at gunbc#1739 #issuecomment-4392562911 (2026-05-06); supersedes earlier PM-authored PROPOSAL — PR #1902 merged then revised here. Dispatch fires post-T-Rule-Enumeration landing on main.)
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
worker pin: TBD (queued post-T-Rule-Enumeration landing; smart-ram-167 likely on T-Rule-Enumeration so valiant-ibex-312 likely takes this — Mgr discretion at dispatch)
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

## Precondition gate

**T-Rule-Enumeration MUST land first** per Director Q1 reasoning:

> Without T-Rule-Enumeration: emission is opaque to the static lens;
> (a) reads as "per-Behavior list of unknown provenance." That's not
> honest.
>
> With T-Rule-Enumeration: emission becomes a `.dag` algebra over
> Behavior + LangSpec rule data; static lens can fold it; (a) returns
> honest per-line provenance.

Brief dispatches when:
1. T-Rule-Enumeration substrate-fact-introduction lands on main (gate
   `langspec_emission_rules_enumerable_data` advances DECLARED →
   CONSUMER_LANDED) — see `docs/briefs/r3-substrate-t-rule-enumeration-worker.md`
2. Worker re-greps `src/v3/` to confirm rule-name carrier exists +
   emission code dispatches via named-rule lookup

If precondition is missing at dispatch, STOP and surface — this brief
consumes T-Rule-Enumeration substrate; not a substrate-producer on
the rule-enumeration axis.

## Scope

### Deliverable 1 — `EmissionProvenance` + `EmissionOrigin` carriers

Author carriers in `src/v3/std/`. Per codex Finding 1 reshape (typed-sum
origin, NOT optional-pair-with-runtime-invariant — applied in PR #1902
at 0110f739d before supersession):

```dag
type EmissionOrigin = SubstrateDeclMirror(SourceSpan) | FoldRuleAutoEmit(EmissionRule)

type EmissionProvenance {
  emitted_line: Int      // line number in emitted target output
  origin: EmissionOrigin // structural fail-closed: every entry HAS an origin
}
```

`EmissionRule` is the carrier landed by T-Rule-Enumeration (α sum type
or β named-string lookup, whichever shape T-Rule-Enumeration ratified).
Worker imports the rule-name carrier verbatim; brief does NOT re-author it.

**Practice 4 classification**: 🟢 GREEN (terminal) — `EmissionOrigin`
is a closed sum type with structural enumeration; both arms have
non-trivial carriers. No richer source exists; the variants trace
to the actual two emission origin classes (substrate-decl mirror vs
LangSpec rule emission). No 🟡 YELLOW or 🔴 RED classification applies.

**Terminal receipt** (per `docs/modeling-discipline.md#4-coproduct-dissolution` "GREEN (terminal) — no richer source exists"; documenting attempted dissolutions and why they fail):

- *Attempted dissolution 1: collapse into single carrier with parameterized
  origin*. Form: `EmissionOrigin = Origin(SourceSpan?, EmissionRule?)`.
  FAILS — this is the optional-pair-with-runtime-invariant shape that
  codex BLOCKING rejected (PR #1902 finding 1) as non-structural
  fail-closed. Both-absent invariant relies on runtime assertion, not
  type-system enforcement. Non-terminal: there's no richer-source
  derivation; it's a *weaker* shape that loses fail-closed structure.
- *Attempted dissolution 2: factor into common richer source*. The two
  arms carry orthogonal payload types — `SourceSpan` (file/byte-range
  triple from existing diagnostics substrate) vs `EmissionRule` (named
  rule from emission code). No common richer source they both project
  from; they describe two structurally distinct origin classes.
- *Attempted dissolution 3: third arm for unattributable lines*. STOP
  trigger #3 explicitly forbids adding an `Unknown` variant — that's
  the placeholder anti-pattern Director rejected on Slice 2.5 (Option 4
  fail-closed-with-named-dep). If a third origin class surfaces
  during cementing test, that's a substrate-fact-introduction cascade
  (separate brief), not silent variant-extension.

Conclusion: 🟢 GREEN terminal at this scope. The two-arm sum is the
faithful representation of "every emitted line came from substrate-decl
mirror OR LangSpec rule emission"; there is no richer source.

**§1.8 ledger entry for GREEN classification** (per
`docs/modeling-discipline.md#4-coproduct-dissolution` — *"checkpoint
comment naming its classification (🟢/🟡/🔴), with a ledger entry if
GREEN"*): `emission_origin_classification_green` (or canonical
project-naming convention) — sibling row to the parent
`emission_provenance_lens_landed` gate. Worker authors both ledger
rows + in-source `// 🟢 GREEN (terminal)` checkpoint comment on the
live `EmissionOrigin` declaration.

### Deliverable 2 — `Lens<List<EmissionProvenance>>` instance

Author lens instance per Director-locked 6-field shape:

- `name: "EmissionProvenance"` (or canonical project-naming convention)
- `read: fn(Dag, Behavior) -> Witness<List<EmissionProvenance>>` —
  per-Behavior fold computing the provenance list. For each emitted
  line attributable to this Behavior:
  - Line directly mirrors a Behavior/Declaration → `SubstrateDeclMirror(span)`
  - Line emitted by LangSpec rule → `FoldRuleAutoEmit(rule)` where
    `rule: EmissionRule` is looked up from emission code's named-rule
    dispatch (T-Rule-Enumeration carrier)
  - Missing-origin case CANNOT occur structurally (typed sum; no None)
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
  validation surface is reduced (typed-sum EmissionOrigin makes
  "both-absent" structurally impossible). Aggregate validation:
  surface diagnostic if rule-name in `FoldRuleAutoEmit(rule)` refers
  to a name not in `EmissionRule` enumeration (mechanically caught by
  type system if α; runtime check if β with named dissolution trigger)

Per `feedback_compositional_not_templating`: per-Behavior fold composes
to per-line view at the visualization layer (flatten the per-Behavior
List<EmissionProvenance>); no parallel substrate needed for slide.

### Deliverable 3 — Cementing test

Author cementing test that:
1. Walks structural fold over a representative `.dag` source (likely
   reuses T-CostLens cementing-test source corpus)
2. Emits target-language output (Rust acceptance target)
3. Runs `Lens<List<EmissionProvenance>>` over the same source
4. **Verifies**: every emitted line has a corresponding
   `EmissionProvenance` entry in the per-Behavior aggregate; flatten
   to per-line view matches emitted-line numbering
5. **Verifies origin classes**: ≥1 `SubstrateDeclMirror` arm fires; ≥1
   `FoldRuleAutoEmit` arm fires; both span/rule references resolve to
   real substrate facts
6. **Fail-closed paths**: missing rule-name carrier (precondition broke
   post-merge) → test errors out

### Deliverable 4 — §1.8 ledger receipt

Add `emission_provenance_lens_landed` to §1.8 ledger under
T-CostLens-Composition cluster (Director Q3). Advance DECLARED →
PRODUCER_LANDED on merge (cementing test verifies producer shape; no
Grounding-side visualization-consumer wiring in this PR).

## Slice — single PR

Phase ordering (PR-internal):
1. Verify precondition: T-Rule-Enumeration on main; rule-name carrier
   exists; emission code dispatches via named-rule lookup
2. Author `EmissionOrigin` + `EmissionProvenance` carriers (Deliverable 1)
3. Author `Lens<List<EmissionProvenance>>` instance (Deliverable 2)
4. Author cementing test (Deliverable 3)
5. Verify all standard ratchets green
6. §1.8 ledger row receipt (Deliverable 4)

## Acceptance

- `EmissionOrigin` typed-sum + `EmissionProvenance` record landed in
  `src/v3/std/` with Practice 4 in-source `// 🟢 GREEN (terminal)`
  checkpoint comment + §1.8 ledger entry for GREEN classification
  (`emission_origin_classification_green` sibling row)
- `Lens<List<EmissionProvenance>>` instance landed per 6-field
  Director-locked shape; T-CostLens-Composition precedent verified for
  shape parity
- Cementing test landed: per-Behavior fold output flattens to per-line
  view matching emitted-line numbering; ≥1 of each origin class fires;
  rule-name references resolve via T-Rule-Enumeration carrier
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

- **T-Rule-Enumeration not landed at dispatch**: STOP — reopen on
  rule-name carrier landing on main
- **`iterate` identity assumption breaks** (body emitted multiple times
  per LoopBound, NOT once): STOP — surface to Substrate Mgr; lens
  iterate field shape may need rework
- **Cementing test reveals provenance gap** — emitted line that the
  per-Behavior fold cannot attribute (i.e., a third origin class beyond
  SubstrateDeclMirror / FoldRuleAutoEmit): STOP — typed sum
  EmissionOrigin needs a third variant (substrate-fact-introduction
  cascade); surface to Mgr. Do NOT add an `Unknown` variant — that's
  the placeholder anti-pattern Director rejected on Slice 2.5
- **Bundled-scope drift**: do NOT bundle T-Rule-Enumeration edits or
  Grounding-side visualization-consumer wiring into this PR

## Authority audit receipt

1. **Substrate exists?** At brief-author time:
   - `Lens<C>` carrier landed (`src/v3/std/lens.dag`, 🟢 TERMINAL) ✓
   - `EmissionRule` carrier — gates on T-Rule-Enumeration landing
   - `EmissionOrigin` / `EmissionProvenance` carriers — this brief is producer
   - Lens instance — this brief is producer
2. **Existing brief?** PM-authored proposal at PR #1902 (merged at
   54419badf) is the prior artifact; this brief revises per Substrate
   Mgr canvas + Director Q1 (a) RATIFICATION
3. **Design-doc match?** Director Q1 (a) RATIFIED + canvas Q1 disposition
   + Lens<C> Director-locked shape. T-CostLens-Composition is shape
   precedent
4. **Citations live?** Worker re-verifies at dispatch
5. **Carrier dissolves the bridge?** Yes — typed-sum `EmissionOrigin`
   dissolves the "is this line span-attributable or rule-attributable?"
   bridge structurally (closed sum; no third silent class). Per-Behavior
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
