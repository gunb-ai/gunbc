# R3 Verification Pattern A Coverage Roll-Up

**Status:** SYNTHESIS RECEIPT - docs-only; no producer, fixture, substrate, or
runner edits.
**Dispatch:** Worker B Pattern A coverage roll-up after the per-claim
second-mover audits.
**Purpose:** compress the TC1, TC2, TC3, and RustDagIsomorphism audit results
into the E6-readiness checklist Verification consumes when generic
`DimensionReport<C>` production/evaluation lands.

## 1. Consumer-Envelope Verdict

The consumer-envelope verdict is uniform across the four named Pattern A
claims: `BinaryDimensionReportEquals` is sufficient at the consumer layer.

| Claim | Audit authority | Consumer-envelope verdict |
|---|---|---|
| TC1 eta-equivalence | `docs/briefs/r3-v-tc1-pattern-a-second-mover-conformance-audit.md` (PR #1581) | Composes through `BinaryDimensionReportEquals(tc1_subject_f_report, tc1_subject_eta_expanded_report)` once producer-side reports exist. No TC1-specific consumer predicate is needed. |
| TC2 evaluation-order independence | `docs/briefs/r3-v-tc2-pattern-a-second-mover-audit.md` (PR #1580) | Composes through `BinaryDimensionReportEquals(tc2_leftfirst_strategy_dimension_report, tc2_rightfirst_strategy_dimension_report)` once strategy-conditioned producers exist. No TC2-specific consumer predicate is needed. |
| TC3 strong normalization | PR #1582 (`docs/briefs/r3-v-tc3-pattern-a-second-mover-conformance-audit.md`, about-to-merge authority at synthesis time) | Composes through `BinaryDimensionReportEquals(tc3_evaluation_step_baseline_dimension_report, tc3_evaluation_step_compare_dimension_report)` once evaluation-step / termination producers exist. No TC3-specific consumer predicate is needed. |
| RustDagIsomorphism | `docs/briefs/r3-v-deferred-test-conformance-sweep.md` (PR #1578) | Conforms as a consumer instance: `BinaryDimensionReportEquals(RustEnumExtractionDagShapeReport, DagReflectionDagShapeReport)` over `DimensionReport<Dag>`. Producer is the missing shape-report path, not a new comparison shell. |

The shared standby contract remains the Evaluator #1131 safe contract recorded
by the sweep: refs must produce the same `DimensionReport<C>` carrier; the
runner validates shape and returns `NotYetImplemented` until typed report
production exists; no serialized/string/bytes/prose comparison, new
`TestPredicate` variant, local report shape, or fixture-local producer identity
is authorized.

## 2. Cross-Claim Producer-Side Concept Matrix

The four claims differ in producer-side concepts, not in consumer authority.

| Claim | Producer-side concept unique to the claim | Shared producer substrate |
|---|---|---|
| TC1 | Eta-pair construction or relation: one report is for `f`, the other for `lambda x.apply(f, [x])`. Needs the lens producer/fold path and a coverage decision: universal over all `Lens<C>` instances or a ratified representative lens set. | Generic `fold_lens<C>` -> `DimensionReport<C>`; existing `DimensionReport<C>` carrier; no local report shape. |
| TC2 | Typed strategy-order modifier over closed `EvalStrategy` / `InputEvaluationOrder` carriers. Needs at least two executable strategies/input orders through the same evaluator boundary and memo/state identity keyed by strategy. | Generic report production under an evaluator strategy; existing `EvalMemoKey.strategy` shape; no strategy strings. |
| TC3 | Universal well-typed-fragment quantifier plus bounded forward execution / evaluation-step evidence. Needs termination evidence via B5 + T-FixedPoint and a proof/coverage decision: structural induction, generated exhaustive producer, or bounded representative harness. | Generic `DimensionReport<Dag>` production; evaluation-step producer surface; no TC3-local theorem report. |
| RustDagIsomorphism | Structural shape report producer: one report for Rust enum extraction shape, one report for reflected-Dag shape. The consumer fixture exists; the missing work is the producer path that emits both reports through E6-compatible `DimensionReport<C>` authority. | Generic `DimensionReport<Dag>` production/evaluation; producer-first shape-report surface; no new comparison shell. |

No contradiction appears across these audits. TC2's strategy-order modifier and
TC3's universal-fragment quantifier are orthogonal: both are metadata/producer
selection obligations that feed typed reports into the same equality shell.

## 3. Union Of Strict-Fire Preconditions

The deduplicated strict-fire checklist across all four claims is:

1. **Generic report production/evaluation:** `BinaryDimensionReportEquals` can
   consume real typed `DimensionReport<C>` values, not only shape-validate refs.
2. **E6 generic lens fold:** `fold_lens<C>` or equivalent producer path emits
   the existing `DimensionReport<C>` carrier verbatim.
3. **Live lens/value authoring surface:** at least one representative
   `Lens<C>` instance or approved typed lens-instance handle exists without a
   Rust registry or fixture-local producer identity.
4. **Callable and field projection through the shared evaluator:** E6 can call
   `Lens.read`, `Lens.sequential.op`, `Lens.branch`, `Lens.iterate`, and
   `Lens.validate` through the declared carrier rather than `lens_apply.rs`
   parallel interpretation.
5. **Report lifting:** `Witness<C>`, `OptionalDiagnostic`, `Diagnostic`, and
   `DimensionReport<C>` values are lifted through one evaluator-owned authority.
6. **Program-scope authority:** the producer names its structural program scope
   (for example whole-Dag first slice) without reusing file-path filtering as
   the generic fold authority.
7. **TC1 eta surface:** eta-pair relation/construction is substrate/evaluator
   visible, and TC1's universal-vs-representative coverage decision is ratified.
8. **TC2 strategy surface:** at least two executable `EvalStrategy` /
   `InputEvaluationOrder` inhabitants exist, with structural memo/state identity
   including the selected strategy.
9. **TC3 theorem surface:** B5 remains green, T-FixedPoint termination semantics
   land, and the universal well-typed-fragment coverage/proof shape is ratified.
10. **RustDagIsomorphism shape producer:** both structural shape reports are
    producer-owned `DimensionReport<Dag>` values.
11. **No local bypasses:** if any claim cannot express its modifier without
    local producer identities, string labels, a claim-specific predicate, or a
    local report shape, that is a Substrate/Evaluator STOP+PING.

This union is the readiness checklist. It is not an implementation plan and
does not change E6 pacing ownership.

## 4. E6 Unlock Readiness Per Claim

| Claim | Substrate-landed today | Known lane / in flight | Gap needing routing before strict-fire |
|---|---|---|---|
| TC1 | Consumer fixture and `BinaryDimensionReportEquals` envelope; existing `DimensionReport<C>` carrier; TC1 observation role carrier. | T-Substrate-Lens-Primitive / E6 generic lens fold runway; lens producer retirement / live lens instances. | Eta-pair producer/relation and universal-vs-representative coverage decision. |
| TC2 | `EvalStrategy = ApplicativeOrder { input_order }`, `InputEvaluationOrder = LeftFirst`, and `EvalMemoKey.strategy` in `src/v3/std/runtime.dag`; consumer fixture and report envelope. | Evaluator strategy/eager baseline runway; E6 generic report production. | Second executable strategy or input order, plus typed strategy-order modifier binding reports to strategies without strings. |
| TC3 | Consumer fixture, report envelope, and `LoopBound`/runtime carriers used by the theorem surface. | B5 / T-FixedPoint / PB-to-Verification transition path; E6 evaluation-step producer runway. | Universal well-typed-fragment quantifier and proof/coverage decision; bounded forward execution / termination-evidence producer. |
| RustDagIsomorphism | Consumer fixture and report envelope; reflected-Dag shape-report framing from the conformance sweep / prior producer-first research. | E6 generic `DimensionReport<C>` producer/evaluator path. | Concrete shape-report producers for Rust enum extraction and reflected-Dag structure. |

The table intentionally separates "carrier exists" from "strict-fire producer
exists." A claim is ready for Verification re-dispatch only when its row's
producer-side gap column is resolved without violating the safe contract.

## 5. Pattern A Multiplier Verification

The multiplicity audit on #828 identified the high-leverage gate: one generic
`DimensionReport<C>` producer/evaluator path unlocks the four named Pattern A
report-equality claims:

1. TC1 eta-equivalence;
2. TC2 evaluation-order independence;
3. TC3 strong normalization;
4. RustDagIsomorphism.

The same gate also covers four `BinaryDimensionReportEquals` free-consequences
sub-claims. The other six free-consequences sub-claims use `LensOutputEquals`
and sit on a different runner path, so the full multiplier is:

- **1 unlocks 4** named Pattern A claims; and
- **1 unlocks 8** total deferred `BinaryDimensionReportEquals` evaluations
  when the four free-consequences sub-claims are included.

This roll-up changes the multiplier from a speculative capacity argument into
a structural readiness map. The consumer layer is uniform and already
shape-valid; the remaining work is the union of producer/modifier concepts
listed above.

## Per-PR Receipt

Debt found + routed: none. This synthesis does not close a Debt-Paydown row
directly and does not introduce new debt. It consolidates already-authored
Pattern A audit receipts into an E6-readiness consumable for Verification.
