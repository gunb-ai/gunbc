# R3 Lane 2 — L5 corpus-policy substrate canvas (gate #15 CONSUMER_LANDED → PASSING precondition)

**Status:** PROPOSAL — research-only canvas. **No implementation in this PR.** This document enumerates the substrate shape needed to flip `r3-program-plan.md` §1.8 row **#15 `l5_cross_target_consistency`** from **CONSUMER_LANDED** (PR #3060 runner + 4 Int-stdout scaffold rows) to **PASSING**, and routes the carrier shape to Director per **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)** before any `.dag` edit to `src/v3/std/verification.dag` lands.

## 1. Authority

- **Gate authority:** [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-Verification-L5-Corpus (L93) — *"for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus."*
- **Status authority:** [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 **row #15** — current Status = **CONSUMER_LANDED** (PR #3060). PASSING blocker quoted verbatim: *"current #3060 rows do not model … expected semantic observation/oracle authority, effect class, numeric policy, and coverage reason."*
- **Close-plan authority:** [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md) §Gap 2 — operator §4 Item 2 IN-R3 (full 3-target Python+Go) ratified 2026-05-13; R4-defer / Rust-only narrow paths FORECLOSED.
- **Corpus Policy semantic lock:** [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) §"Corpus Policy" (L35–L49).
- **Substrate-introduction discipline:** [`INVARIANTS.md`](../../INVARIANTS.md) §P1 — modeling faithfulness; new substrate facts require Director ratification before `.dag` substrate edits land.
- **Upstream worker briefs:** [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md), [`r3-v-l5-corpus-extension-spec.md`](r3-v-l5-corpus-extension-spec.md), [`r3-v-l5-corpus-readiness-audit.md`](r3-v-l5-corpus-readiness-audit.md), [`r3-v-l5-corpus-scaffold-notes.md`](r3-v-l5-corpus-scaffold-notes.md).

## 2. What is missing (Corpus Policy → substrate gap)

`design-cross-target-equivalence.md` §Corpus Policy requires every valid L5 row to carry **seven** facts (L39–L45 bullets). Mapped against HEAD substrate (`src/v3/std/verification.dag` `TestClaim` + `TestPredicate::ForAllTargets` + `ProgramOutputBind`):

| Corpus Policy fact | Already cashed by HEAD substrate? | Where |
|---|---|---|
| Single source `.dag` program + stable entry-point identity | ✅ | `TestClaim.source` + `TestClaim.file_name` (single authority — L562) |
| Required target set | ✅ | `TestClaim.requires: List<ResourceReference>` (L567); L5 rows attach `L5RustcToolchain` / `L5Python3Toolchain` / `L5GoToolchain` |
| Declared input sample or finite input family | ⚠️ partial | `BehavioralObservation.input_sample: DeclarationRef` exists (L307); `ForAllTargets` declares `input_ref: DeclarationRef` (L343), but the HEAD L5 fixture uses that slot to point at a `ProgramOutputBind` declaration rather than a typed input family — the input-vs-output role is currently overloaded, not absent. L5 programs are nullary at the entry point today. **Question Q5 below.** |
| Expected semantic observation / oracle authority | ❌ | No carrier as a declared field on `ForAllTargets` (fields at L339–L344 are `command: String, args: List<String>, expect_exit_code: Int, input_ref: DeclarationRef`). The `ProgramOutputBind` cited in the L331–L334 doc-comment is observed by the runner conceptually but is not a typed field on this variant; the bind names *where* to look, not the *expected normalized value* or oracle. |
| Effect class | ❌ for L5 rows | `std.effects` already declares `EffectShape` (`src/v3/std/effects.dag:338`) but no row on `TestClaim` / `ForAllTargets` declares it. Reuse-vs-introduce is **Q1 below**. |
| Numeric policy | ❌ | No carrier; design doc §"Float Policy" + extension-spec §1.1 (`Int` overflow gate) both require it. **Q2 below.** |
| Coverage reason | ❌ | No carrier. Free-form prose vs structured enum is **Q3 below**. |

**Three facts have no HEAD carrier (expected observation/oracle, numeric policy, coverage reason). One has a carrier in a sibling module (effect class — `EffectShape`) but no edge to a corpus row. One is partial (input sample).** This canvas does not author any of them; it asks Director to ratify the carrier shape so a follow-up implementation PR can land without re-litigating shape mid-review.

## 3. Existing HEAD seed (what would need to be back-filled)

`src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag` declares four `TestClaim` rows today (PR #3060 + #3039):

| Row name | Program shape | Plausible policy facts (illustration — **not** a ratification) |
|---|---|---|
| `l5_cross_target_consistency_add_then` | `Int` add + `match` on `Bool` equality | obs=`Int(3)`, effect=`Pure`, numeric=`Int64-overflow-free`, coverage=`L4 add-then-branch seed → cross-target L5 lift` |
| `r3_l5_cert_branch_literal_true` | `Bool` literal + `match` → `Int` | obs=`Int(3)`, effect=`Pure`, numeric=`Int64-overflow-free`, coverage=`branch literal True arm` |
| `r3_l5_cert_branch_literal_false` | `Bool` literal + `match` → `Int` (negative) | obs=`Int(-4)`, effect=`Pure`, numeric=`Int64-overflow-free`, coverage=`branch literal False arm + signed Int` |
| `r3_l5_cert_nested_branch` | nested `Bool` `match` → `Int` | obs=`Int(7)`, effect=`Pure`, numeric=`Int64-overflow-free`, coverage=`nested match lowering across targets` |

All four are Phase A (extension-spec §2): pure, `Int` observable, no IO/floats/host libs, overflow-free literals. **Q4 below** asks whether the carrier admits these four under the simplest shape that still makes the policy facts grep-checkable from the boundary consumer at `src/v3/compiler/tests/boundary/l5_cross_target_consistency.rs`.

## 4. Open questions for Director ratification

Each question lists structurally distinct options, the disqualifying axis for each, and a **canvas recommendation** (preliminary per `feedback_canvas_recommendations_are_preliminary.md` — to be revised under audit).

### Q1 — Effect class carrier

**A1.** Reuse `std.effects::EffectShape` (existing TERMINAL authority — `src/v3/std/effects.dag:338`; partition `IsIdempotent | IsBreaking`). No new sum type.
**A2.** Introduce L5-narrow `CorpusEffectClass = Pure | ControlledStdout | TypedFailure | DeferredEffectful` mirroring design doc §"Side-effect Policy" prose exactly.
**A3.** Hybrid — reuse `EffectShape` and project a derived `CorpusEffectClass` view at the runner boundary.

**Disqualifiers:**
- **A2** stands up a parallel effect-classification authority for one consumer (L5 corpus rows) when `EffectShape` already exists — direct INVARIANTS §P1 single-authority risk; canvas-finding taxonomy "parallel representation".
- **A3** keeps `EffectShape` as authority but adds an extra projection layer for a single consumer; pays Practice-4 cost without dissolution trigger.

**Canvas recommendation:** **A1**. Single authority. Design doc §"Side-effect Policy" wording maps cleanly onto `EffectShape` once Director confirms `Pure / ControlledStdout / TypedFailure / DeferredEffectful` are all expressible in the existing partition (this canvas does not assume; it asks).

### Q2 — Numeric policy carrier

**B1.** New nominal sum `NumericPolicy = Int64OverflowFree | NamedOverflowSemantics(RefinementRef) | FloatExcluded | FloatPolicyDeferred`. Captures extension-spec §1.1 directly.
**B2.** Reuse existing refinement vocabulary (`dsl/std/integer.dag` Int<N> + width refinements per gate #18) — no new carrier; row carries a `RefinementRef`.
**B3.** Free-form `String` policy slot — disqualified up front (INVARIANTS §P2 boundary-discipline / §P3 fail-closed violation; substring authority is exactly what design doc §"Oracle Policy" forbids).

**Disqualifiers:**
- **B3** ruled out by design doc §"Oracle Policy" *"Diagnostic string matching, substring checks, … are invalid"*.
- **B2** is incomplete — refinement vocabulary describes the type, not the per-row *policy decision* (e.g. "overflow-free at this row's literal range" vs "named wrap semantics"). Extension-spec §1.1 explicitly requires the policy-decision surface, not just the type.

**Canvas recommendation:** **B1** with **B2 as a payload** — `NamedOverflowSemantics` references a refinement authored in `dsl/std/`, so the new carrier is the *policy decision*, not parallel numeric arithmetic.

### Q3 — Coverage reason carrier

**C1.** Free-form `String coverage_reason` — fast, but design extension §1 already classifies coverage by **program-class taxonomy** (Phase A primitives / Phase B collections / Lane 1 import). String is unstructured.
**C2.** Closed enum `CoverageReason = LanguageConstruct | RuntimeValueShape | TargetRealizationEdge | L4CorpusLift(L4ClaimRef)` — closed taxonomy but no payload edges, so the *which construct / which value-shape / which target edge / which L4 row* identity falls back to author prose. Not structural under INVARIANTS §P2.
**C3.** `(CoverageReason, NonEmptyStr description)` pair — same taxonomy + author prose for triage; inherits **C2**'s missing-payload-edge gap (description carries the identity fact rather than the carrier).
**C4.** Closed sum with **typed payload per arm**: `LanguageConstruct(DeclarationRef)` / `RuntimeValueShape(DeclarationRef)` / `TargetRealizationEdge(TargetEdgeRef)` / `L4CorpusLift(DeclarationRef)`. Identity of the covered construct / value-shape / target edge / L4 row is a typed edge, so invalid coverage rows are unrepresentable.

**Disqualifiers:**
- **C1** loses the L4-lift edge — design extension §"Slice 4" makes the L4 corpus identity load-bearing; need a typed `DeclarationRef` not a string description.
- **C2** + **C3** record only the *category*; the actual construct / value-shape / target-edge / L4 row identity falls back to `coverage_description` prose, so the Corpus Policy coverage fact is not structural under INVARIANTS §P2 (string sidecar carries the identity, not the carrier).

**Canvas recommendation:** **C4**. Closed sum + per-arm typed payload edges. Coverage identity is cashed at the type level; no `coverage_description` prose slot. (`TargetRealizationEdge` requires a `TargetEdgeRef` substrate authority — either a new nominal or an existing carrier from the per-target spec layer; this canvas asks Director to name the existing authority if one exists, otherwise routes its introduction in the same ratification.)

### Q4 — Expected semantic observation / oracle authority carrier

**D1.** Extend `ProgramOutputBind` from `{output_ref: DeclarationRef}` to `{output_ref: DeclarationRef, expected_value: DeclarationRef}` — bind names *where* + *what* in one record.
**D2.** Keep `ProgramOutputBind` unchanged; introduce sibling `ExpectedObservation { bind: ProgramOutputBind, oracle: OracleAuthority }` and an `OracleAuthority` closed sum mirroring design doc §"Oracle Policy" 4 valid forms.
**D3.** Promote `ForAllTargets` to a `DifferentialEquals`-style row that carries `oracle_ref: DeclarationRef` directly (collapse the two scaffold variants once dissolution-trigger fires).

**Disqualifiers:**
- **D1** silently parallel-authors the oracle taxonomy — `oracle: DeclarationRef` says nothing about *which* of the 4 valid oracle forms it is (hand-authored value, `.dag`-evaluator result, algebraic-law witness, `DifferentialEquals` pair); design doc §"Oracle Policy" requires the form to be named, not implied.
- **D3** is a `ForAllTargets`-dissolution proposal — out of scope for this canvas (`feedback_load_bearing_ratchet_preservation`); existing `ForAllTargets` is the slice-1 scaffold and dissolves on its own trigger.

**Canvas recommendation:** **D2**. New `OracleAuthority` sum with 4 closed arms (one per design-doc valid form). `ExpectedObservation` is the policy-row payload. `ForAllTargets` itself is **not** modified in this PR.

### Q5 — Per-row policy attachment shape (where does it live?)

**E1.** Add `corpus_policy: Maybe<L5CorpusRow>` field directly on `TestClaim` (universal — every claim can carry it, only L5 rows populate it).
**E2.** New sibling carrier `L5CorpusRow` in a dedicated `std.r3_l5_corpus` module, carrying a **typed `claim: TestClaim` edge** + the policy fields inline (not a `{claim, policy}` two-record wrapper — flat carrier keeps single authority and matches the §5 substrate delta below). String name keys are disqualified: a string-keyed join would defer P2 boundary discipline to a runner check rather than making invalid policy rows unrepresentable.
**E3.** New `TestPredicate` variant `ForAllTargetsWithPolicy { …existing ForAllTargets fields…, policy: L5CorpusRow }` — disqualified by **upstream worker brief** §"Explicitly out of scope": *"New `TestPredicate` variants — `ForAllTargets` already on substrate; INVARIANTS §P1 only for genuinely new facts."*

**Disqualifiers:**
- **E3** ruled out by [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md) §"Explicitly out of scope".
- **E1** introduces a universal field for a single-consumer fact — most `TestClaim` rows have no L5 semantics; `Maybe<>` slot is parallel-representation drift unless ratified explicitly.

**Canvas recommendation:** **E2** — single flat `L5CorpusRow` carrier in `std.r3_l5_corpus`. Keeps `TestClaim` shape stable; co-locates L5 policy facts in their own module under the existing `r3_verification_l5_corpus` fixture authority. The typed `claim: TestClaim` edge means a policy row pointing at a non-existent claim is **unrepresentable at the type level** (INVARIANTS §P2 boundary discipline cashed at the carrier, not at a runner string-equality check). Boundary consumer (`l5_cross_target_consistency.rs`) only enforces the remaining cardinality fact: every L5 corpus claim appears in exactly one `L5CorpusRow`. Mirrors the **`SuiteClaim`** pattern (`src/v3/std/verification.dag:619`) — closed carrier over `TestClaim`, typed edges throughout.

## 5. Substrate carrier shape (preliminary, conditional on Q1–Q5 ratification)

Assuming canvas recommendations land (**A1 + B1 + C4 + D2 + E2**), the substrate delta is a new module `src/v3/std/r3_l5_corpus.dag` (Q5-E2 module placement; no edit to `src/v3/std/verification.dag` other than the `import` line in the L5 fixture):

```
type OracleAuthority
  = HandAuthoredValue(DeclarationRef)
  | DagEvaluatorResult(DeclarationRef)
  | AlgebraicLawWitness(DeclarationRef)
  | DifferentialOraclePair { subject: DeclarationRef, oracle: DeclarationRef }

type ExpectedObservation {
  bind: ProgramOutputBind
  oracle: OracleAuthority
}

type NumericPolicy
  = Int64OverflowFree
  | NamedOverflowSemantics(RefinementRef)
  | FloatExcluded
  | FloatPolicyDeferred

type CoverageReason
  = LanguageConstruct(DeclarationRef)       // typed edge to the std-library construct
  | RuntimeValueShape(DeclarationRef)       // typed edge to the value-shape type
  | TargetRealizationEdge(TargetEdgeRef)    // typed edge to the per-target realization fact
  | L4CorpusLift(DeclarationRef)            // typed edge to the originating L4 TestClaim

type L5CorpusRow {
  claim: TestClaim
  observation: ExpectedObservation
  effect: EffectShape           // reuses `EffectShape` from `src/v3/std/effects.dag` (Q1-A1)
  numeric: NumericPolicy
  coverage: CoverageReason
}
```

**Boundary consumer ratchet (implementation PR — NOT this PR):** `l5_cross_target_consistency.rs` extends the existing N>0 corpus check to require that every L5 `TestClaim` row has exactly one `L5CorpusRow` companion, and that `EffectShape` for every row is in the design-doc-allowed set (`Pure` or `ControlledStdout`; `TypedFailure` only when claim text says so; `DeferredEffectful` is fail-closed per design doc §"Side-effect Policy").

## 6. Dispatch sequence (if Director ratifies)

1. **PR-1 (this PR):** canvas land as research-only `.md`. **No substrate edit.**
2. **PR-2 (follow-up worker):** substrate land in `src/v3/std/verification.dag` per ratified shape. Sample one of the 4 existing rows; rest blocked.
3. **PR-3 (follow-up worker):** all 4 existing rows back-fill `L5CorpusRow` companions; boundary consumer enforces 1:1 fail-closed.
4. **PR-4 (Verification Mgr):** flip §1.8 row #15 Status `CONSUMER_LANDED` → **PASSING** with §1.8 Notes citing the four landed `L5CorpusRow` rows.

`feedback_post_merge_ledger_receipt_sync.md`: PR-4 lands the §1.8 row flip in the same PR that wires the last back-fill; no deferred ledger sync.

## 7. Live-path receipt

Re-run if `main` moves materially. Every path hyperlinked from this canvas:

```bash
git fetch origin
for p in \
  INVARIANTS.md \
  docs/r3-structure.md \
  docs/r3-program-plan.md \
  docs/r3-actual-close-plan.md \
  docs/design-cross-target-equivalence.md \
  docs/briefs/r3-v-l5-corpus-worker.md \
  docs/briefs/r3-v-l5-corpus-extension-spec.md \
  docs/briefs/r3-v-l5-corpus-readiness-audit.md \
  docs/briefs/r3-v-l5-corpus-scaffold-notes.md \
  src/v3/std/verification.dag \
  src/v3/std/effects.dag \
  src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag \
  src/v3/compiler/tests/boundary/l5_cross_target_consistency.rs
do git cat-file -e "origin/main:$p" || exit 1; done
```

## 8. Explicit non-claims

- **No new `TestPredicate` variant** — Q5-E3 ruled out up front by worker brief.
- **No modification to `ForAllTargets`** — Q4-D3 deferred to its own dissolution trigger.
- **No L4/L6 scope absorption** — L4 stays in Lane 1; L6 stays in R2-T-Ground-CrossTarget-Meta per `r3-structure.md` L92-93.
- **No oracle-string-matching surface** — Q2-B3 + design doc §"Oracle Policy" mutually rule out.
- **No `.dag` substrate edit in this PR.** Implementation PR follows on ratification.

## 9. Cross-refs

- Parent manager: [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md)
- Sibling Lane 1: [`docs/briefs/r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md)
- Upstream PR-D semantic lock: [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md)
- Upstream extension spec: [`docs/briefs/r3-v-l5-corpus-extension-spec.md`](r3-v-l5-corpus-extension-spec.md)
- Close-plan §Gap 2: [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md#gap-2--l5-cross-target-consistency-gate-15)
- §1.8 row #15: [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 row 15
- Audit re-derivation receipt: [`docs/audit/r3-close-predicate-execution-2026-05-13.md`](../audit/r3-close-predicate-execution-2026-05-13.md) row #15 N/A_NOT_PASSING attribution
