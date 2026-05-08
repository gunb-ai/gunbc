# Canvas — Q-PerfWithinBaseline (TestPredicate variant substrate-shape question)

**Authority**: Director Q6 RATIFIED canvas-tier required at gunb-ai/gunbc#828 #issuecomment-4403163639 — substrate-fact-introduction with three substantive unresolved axes; canvas-tier P1 procedure applies per `feedback_substrate_principle_audit`.

**Status**: **canvas — DRAFT 2026-05-08**; PROPOSAL maturation pending Director ratification of options.

**Sub-issue**: gunb-ai/gunbc#2204 (Substrate T-Tier3 PerfWithinBaseline TestPredicate variant); cross-routed from PB Mgr #2074 c#4403061394 for T-Tier3-Dissolution lane #2085 Phase-2 R-4 prereq.

## What this canvas resolves

**Three substantive substrate-shape questions** (per Director's Q6 ratification framing):

1. **What "baseline" IS**: literal-Int / DeclarationRef-to-stored-data / runtime-fixture-injection / new substrate type?
2. **ComparisonOp composition**: existing `ComparisonOp` shape vs new relative-comparison op-set?
3. **Baseline-storage substrate scope**: where does the baseline value live?

## Substrate-grep grounding (HEAD: `src/v3/std/verification.dag`)

Existing `TestPredicate` variants relevant to factoring claim:

```
type TestPredicate
  = ...
  | CostBounded {                     // line 125+: absolute-bound perf check
      bind_name: String
      comparator: ComparisonOp
      bound: Int                      // ← literal-Int absolute
    }
  | LensOutputEquals {                // line 173+: stored-reference comparison
      lens_ref: DeclarationRef
      input_ref: DeclarationRef
      expected_ref: DeclarationRef    // ← reference to `data X: SymbolicCost = ...`
    }
  | OracleEquals {                    // line 181+: oracle-keyed comparison
      subject_ref: DeclarationRef
      oracle_ref: DeclarationRef
      input_ref: DeclarationRef
    }
  | ...
```

**Precedent finding**: `DeclarationRef`-to-stored-`data X: SymbolicCost` is **already-substrate** at HEAD (per `LensOutputEquals.expected_ref` + the comment at line 190 citing the pattern). Stored-cost-reference via DeclarationRef is NOT novel substrate; it composes existing patterns.

`ComparisonOp` is imported from `v3.std.substrate` at line 21; the type definition is upstream. `CostBounded.comparator` consumes it for literal-Int comparison; the composition surface for stored-reference comparison is the open question.

## Q1 — What "baseline" IS

### Option (a) — Literal-Int baseline (strict-dual of CostBounded)

```dag
| PerfWithinBaseline {
    bind_name: String
    comparator: ComparisonOp
    baseline: Int                     // literal absolute value
  }
```

**Pro**: strict-mirror of `CostBounded`; `feedback_strict_mirror_vs_novel_substrate_fact` would apply at slice-tier-direct.
**Con**: defeats the purpose — "baseline" semantically implies stored/computed reference, not absolute literal. Rename of CostBounded with no new fact. Director Q6 ratified canvas-tier-required because this option doesn't carry the load-bearing semantics; rejecting (a).

### Option (b) — DeclarationRef-to-stored-data baseline (compositional, existing precedent)

```dag
| PerfWithinBaseline {
    subject: DeclarationRef           // bind to be measured
    comparator: ComparisonOp
    baseline_ref: DeclarationRef      // reference to `data baseline_X: SymbolicCost = ...`
  }
```

**Pro**: composes existing precedent at HEAD (`LensOutputEquals.expected_ref` + `data X: SymbolicCost = ...` form already substrate); single-authority discipline preserved. Baseline value lives in a substrate `data` declaration; predicate references it. Cross-target stable per `feedback_construction_over_ratchets`.
**Con**: requires a `data` declaration per baseline (one-baseline-per-test program); may proliferate boilerplate vs literal form. Mitigated by reusable baseline declarations across multiple PerfWithinBaseline call-sites.

### Option (c) — Runtime-fixture-injection baseline (genuinely-novel substrate)

```dag
| PerfWithinBaseline {
    subject: DeclarationRef
    comparator: ComparisonOp
    baseline_fixture: FixtureKey       // new substrate primitive
  }
data FixtureKey = ...                  // new substrate type
```

**Pro**: allows runtime-injected baselines (e.g., per-test-environment values not declarable at substrate-time).
**Con**: introduces new substrate primitive (`FixtureKey`); breaks substrate-declared-form discipline; couples test-execution to runtime-config. P1 substrate-fact-introduction procedure required (DAG-ancestor / coproduct-vs-coordinate / primitive-vs-lens-extensible). Significantly higher cost than (b).

**Mgr lean Q1**: **(b)** — composes existing precedent at HEAD; minimal substrate-fact delta; baseline-as-DeclarationRef-to-stored-data matches `LensOutputEquals` pattern. (a) defeats semantic load; (c) is over-authoring for the present need.

## Q2 — ComparisonOp composition

Existing `ComparisonOp` (per `v3.std.substrate` import) operates on literal-typed comparand pairs. Question: does it compose under (b)'s stored-reference shape?

### Option (i) — Reuse existing ComparisonOp (compositional)

`PerfWithinBaseline.comparator: ComparisonOp` operates on `cost_of(subject) ⟨op⟩ value_of(baseline_ref)`. Both sides resolve to `SymbolicCost` (or projected `Int` per asymptotic-class extraction); existing `ComparisonOp` semantics apply. Test-runner injects the resolution.

**Pro**: zero new substrate; composes existing op-set. Single-authority discipline.
**Con**: requires test-runner to resolve `baseline_ref` to a comparable value before applying ComparisonOp; the resolution path needs to exist already (likely does per `LensOutputEquals` + `data X: SymbolicCost` consumption in test_runner per `test_runner.rs:2451` cite).

### Option (ii) — New relative-comparison op-set (novel)

Introduce `RelativeComparisonOp` parallel to `ComparisonOp`:
```dag
type RelativeComparisonOp = WithinTolerance(Float) | StrictlyBetter | StrictlyWorse | ...
```

**Pro**: explicit semantics for tolerance-bounded relative comparison.
**Con**: parallel-representation debt vs `ComparisonOp`; introduces `Float` substrate (new); P1 procedure required. Higher cost; not justified by present need.

**Mgr lean Q2**: **(i)** — reuse existing ComparisonOp; test-runner-side resolution of baseline_ref is the natural composition (matches `LensOutputEquals` resolution path).

## Q3 — Baseline-storage substrate scope

Under (b) + (i), baseline value lives in a substrate `data X: SymbolicCost = ...` declaration referenced via `DeclarationRef`. Question: is the baseline-storage in-scope for #2204 slice or a separate sub-issue?

### Option (α) — In-scope: slice authors example baseline data + uses it

#2204 slice authors a representative `data baseline_example: SymbolicCost = ...` plus the `PerfWithinBaseline` variant; downstream consumers (PB #2138) reuse the pattern + author their own baseline data declarations as needed.

**Pro**: same-slice acceptance; slice ships with working example.
**Con**: scope-creep risk; example baseline may not match production baseline storage shape needed by PB consumer.

### Option (β) — Out-of-scope: variant-only slice; baseline-storage in PB consumer slice

#2204 slice authors only the `PerfWithinBaseline` variant; baseline-storage authoring is downstream PB consumer work. PB #2138 adds the `data baseline_X: SymbolicCost = ...` declarations they need + references them via the new variant.

**Pro**: clean scope boundary; substrate-introduces-the-shape; consumer-introduces-the-data. Matches single-authority discipline (substrate doesn't author test-specific data).
**Con**: slice can't fully demonstrate without consumer data; cementing test deferred to PB consumer.

**Mgr lean Q3**: **(β)** — variant-only substrate slice; baseline-storage data declarations are PB consumer-tier authoring. Matches the layering at HEAD (`LensOutputEquals` is substrate-tier; `data X: SymbolicCost` declarations are consumer-tier-authored).

## Combined Mgr lean

**Q1 (b) + Q2 (i) + Q3 (β)**: PerfWithinBaseline variant authored as compositional extension over existing `DeclarationRef` + `ComparisonOp` substrate; baseline-storage is PB consumer-tier work; #2204 slice scope is variant-only.

```dag
// proposed addition to TestPredicate at src/v3/std/verification.dag
| PerfWithinBaseline {
    subject: DeclarationRef
    comparator: ComparisonOp
    baseline_ref: DeclarationRef
  }
```

This is **canvas-tier-then-slice-tier**: canvas ratifies the (b)+(i)+(β) shape; slice authors the variant verbatim per ratified shape; PB consumer authors baseline data declarations downstream.

## Director ratification ask

1. **Q1**: ratify (b) DeclarationRef-to-stored-data baseline (composes existing `LensOutputEquals` precedent)?
2. **Q2**: ratify (i) reuse existing ComparisonOp (test-runner-side resolution per `LensOutputEquals` path)?
3. **Q3**: ratify (β) variant-only slice scope (baseline-storage to PB consumer)?
4. **Q3-followup**: if (β), confirm slice cementing-test acceptance gates either (a) include trivial baseline data declaration in slice OR (b) defer cementing to PB consumer slice landing?

**Mgr lean**: Q1 (b), Q2 (i), Q3 (β), Q3-followup (a) — slice includes trivial baseline data for cementing receipt; PB consumer authors production baselines. Maintains slice self-testing per `TESTING.md` + matches Q4 PB cross-Mgr ownership boundary.

## Cross-Mgr coordination

- **PB Mgr (#2074)**: T-Tier3-Dissolution #2085 Phase-2 R-4 prereq is #2204; PB has authored `Depends on: #2204` against #2085. Ratification disposition shapes PB's #2138 worker brief authoring (which baseline data declarations they author + reference).
- **Verification Mgr (#2075)**: PerfWithinBaseline ratchet authoring at PR-open per Pattern-A executable-gate; cross-Mgr PING when slice opens.

## Framework discipline anchors

- **`feedback_strict_mirror_vs_novel_substrate_fact`**: (a) literal-Int baseline would have been strict-mirror; rejected because semantic load doesn't transfer. Director ratified canvas-tier-required; Mgr's preferred (b) composes existing precedent without introducing new fact.
- **`feedback_construction_over_ratchets`**: (b) composes `LensOutputEquals` + `data X: SymbolicCost` precedent at HEAD; structural truth from substrate-grep drives the choice.
- **`feedback_substrate_grep_before_authoring`**: substrate-grep findings (DeclarationRef + data declaration precedent) load-bearing for option-disambiguation.
- **`feedback_substrate_principle_audit`**: P1 procedure applied; DAG-ancestor (TestPredicate sum), coproduct-vs-coordinate (sum variant; not coordinate), primitive-vs-lens-extensible (substrate primitive; not lens-extensible) all resolved.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-08 per Director Q6 RATIFIED canvas-tier-required at gunb-ai/gunbc#828 #issuecomment-4403163639. Sibling sub-issue at gunb-ai/gunbc#2204; cross-routed from PB Mgr ask at #2074 c#4403061394.
