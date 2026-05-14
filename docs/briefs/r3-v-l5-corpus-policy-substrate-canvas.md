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

`design-cross-target-equivalence.md` §Corpus Policy requires every valid L5 row to carry **six** facts. Mapped against HEAD substrate (`src/v3/std/verification.dag` `TestClaim` + `TestPredicate::ForAllTargets` + `ProgramOutputBind`):

| Corpus Policy fact | Already cashed by HEAD substrate? | Where |
|---|---|---|
| Single source `.dag` program + stable entry-point identity | ✅ | `TestClaim.source` + `TestClaim.file_name` (single authority — L562) |
| Required target set | ✅ | `TestClaim.requires: List<ResourceReference>` (L567); L5 rows attach `L5RustcToolchain` / `L5Python3Toolchain` / `L5GoToolchain` |
| Declared input sample or finite input family | ⚠️ partial | `BehavioralObservation.input_sample: DeclarationRef` exists (L307); `ForAllTargets` carries no input slot (L339). L5 programs today are nullary at the entry point. **Question Q5 below.** |
| Expected semantic observation / oracle authority | ❌ | No carrier. Today `ForAllTargets` carries only `(command, args, expect_exit_code, ProgramOutputBind)` — the bind names *where* to look, not the *expected normalized value* or oracle. |
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
**B3.** Free-form `String` policy slot — disqualified up front (P5 boundary-discipline violation; substring authority is exactly what design doc §"Oracle Policy" forbids).

**Disqualifiers:**
- **B3** ruled out by design doc §"Oracle Policy" *"Diagnostic string matching, substring checks, … are invalid"*.
- **B2** is incomplete — refinement vocabulary describes the type, not the per-row *policy decision* (e.g. "overflow-free at this row's literal range" vs "named wrap semantics"). Extension-spec §1.1 explicitly requires the policy-decision surface, not just the type.

**Canvas recommendation:** **B1** with **B2 as a payload** — `NamedOverflowSemantics` references a refinement authored in `dsl/std/`, so the new carrier is the *policy decision*, not parallel numeric arithmetic.

### Q3 — Coverage reason carrier

**C1.** Free-form `String coverage_reason` — fast, but design extension §1 already classifies coverage by **program-class taxonomy** (Phase A primitives / Phase B collections / Lane 1 import). String is unstructured.
**C2.** Closed enum `CoverageReason = LanguageConstruct | RuntimeValueShape | TargetRealizationEdge | L4CorpusLift(L4ClaimRef)` matching design doc §"Corpus Policy" L45 verbatim.
**C3.** `(CoverageReason, NonEmptyStr description)` pair — closed taxonomy + author-supplied prose for triage.

**Disqualifiers:** **C1** loses the L4-lift edge — design extension §"Slice 4" makes the L4 corpus identity load-bearing; need a typed `L4ClaimRef` not a string description.

**Canvas recommendation:** **C3**. Closed enum gives structural triage + `feedback_reason_not_label` discipline; `NonEmptyStr` description is human-readable evidence at audit time (mirror existing `Notes` discipline in §1.8 ledger).

### Q4 — Expected semantic observation / oracle authority carrier

**D1.** Extend `ProgramOutputBind` from `{output_ref: DeclarationRef}` to `{output_ref: DeclarationRef, expected_value: DeclarationRef}` — bind names *where* + *what* in one record.
**D2.** Keep `ProgramOutputBind` unchanged; introduce sibling `ExpectedObservation { bind: ProgramOutputBind, oracle: OracleAuthority }` and an `OracleAuthority` closed sum mirroring design doc §"Oracle Policy" 4 valid forms.
**D3.** Promote `ForAllTargets` to a `DifferentialEquals`-style row that carries `oracle_ref: DeclarationRef` directly (collapse the two scaffold variants once dissolution-trigger fires).

**Disqualifiers:**
- **D1** silently parallel-authors the oracle taxonomy — `oracle: DeclarationRef` says nothing about *which* of the 4 valid oracle forms it is (hand-authored value, `.dag`-evaluator result, algebraic-law witness, `DifferentialEquals` pair); design doc §"Oracle Policy" requires the form to be named, not implied.
- **D3** is a `ForAllTargets`-dissolution proposal — out of scope for this canvas (`feedback_load_bearing_ratchet_preservation`); existing `ForAllTargets` is the slice-1 scaffold and dissolves on its own trigger.

**Canvas recommendation:** **D2**. New `OracleAuthority` sum with 4 closed arms (one per design-doc valid form). `ExpectedObservation` is the policy-row payload. `ForAllTargets` itself is **not** modified in this PR.

### Q5 — Per-row policy attachment shape (where does it live?)

**E1.** Add `corpus_policy: Maybe<L5CorpusRowPolicy>` field directly on `TestClaim` (universal — every claim can carry it, only L5 rows populate it).
**E2.** New sibling carrier `L5CorpusRow { claim: TestClaim, policy: L5CorpusRowPolicy }` indexed by `TestClaim.name`; lives in a separate `std.r3_l5_corpus` module.
**E3.** New `TestPredicate` variant `ForAllTargetsWithPolicy { …existing ForAllTargets fields…, policy: L5CorpusRowPolicy }` — disqualified by **upstream worker brief** §"Explicitly out of scope": *"New `TestPredicate` variants — `ForAllTargets` already on substrate; INVARIANTS §P1 only for genuinely new facts."*

**Disqualifiers:**
- **E3** ruled out by [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md) §"Explicitly out of scope".
- **E1** introduces a universal field for a single-consumer fact — most `TestClaim` rows have no L5 semantics; `Maybe<>` slot is parallel-representation drift unless ratified explicitly.

**Canvas recommendation:** **E2**. Keeps `TestClaim` shape stable; co-locates L5 policy facts in their own module under the existing `r3_verification_l5_corpus` fixture authority. Boundary consumer (`l5_cross_target_consistency.rs`) gains a 1:1 fail-closed mapping check (every L5 corpus `TestClaim.name` must appear exactly once in the policy table; every policy row must point at a `TestClaim.name` present in the fixture). Mirrors the **`SuiteClaim`** pattern (`src/v3/std/verification.dag:619`) — closed two-variant ordered carrier scoped to its own consumer.

## 5. Substrate carrier shape (preliminary, conditional on Q1–Q5 ratification)

Assuming canvas recommendations land (**A1 + B1 + C3 + D2 + E2**), the substrate delta at `src/v3/std/verification.dag` is:

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
  = LanguageConstruct
  | RuntimeValueShape
  | TargetRealizationEdge
  | L4CorpusLift(DeclarationRef)

type L5CorpusRowPolicy {
  claim_name: String
  observation: ExpectedObservation
  effect: EffectShape
  numeric: NumericPolicy
  coverage: CoverageReason
  coverage_description: NonEmptyStr
}
```

**Boundary consumer ratchet (implementation PR — NOT this PR):** `l5_cross_target_consistency.rs` extends the existing N>0 corpus check to require that every L5 `TestClaim` row has exactly one `L5CorpusRowPolicy` companion, and that `EffectShape` for every row is in the design-doc-allowed set (`Pure` or `ControlledStdout`; `TypedFailure` only when claim text says so; `DeferredEffectful` is fail-closed per design doc §"Side-effect Policy").

## 6. Dispatch sequence (if Director ratifies)

1. **PR-1 (this PR):** canvas land as research-only `.md`. **No substrate edit.**
2. **PR-2 (follow-up worker):** substrate land in `src/v3/std/verification.dag` per ratified shape. Sample one of the 4 existing rows; rest blocked.
3. **PR-3 (follow-up worker):** all 4 existing rows back-fill `L5CorpusRowPolicy` companions; boundary consumer enforces 1:1 fail-closed.
4. **PR-4 (Verification Mgr):** flip §1.8 row #15 Status `CONSUMER_LANDED` → **PASSING** with §1.8 Notes citing the four landed `L5CorpusRowPolicy` rows.

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
