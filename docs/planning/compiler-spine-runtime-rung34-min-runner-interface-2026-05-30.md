# Compiler Spine × Runtime/TestClaim — minimum runner interface (rungs 3–4)

**Authority:** PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §11.4 item 4 — manager pass before worker code.  
**Owners:** Compiler Spine (`smart-stag-871`) + Runtime/TestClaim (`quick-tern-735`).  
**Fixture:** `src/v4/test/claim/algebra_laws/nat_semiring.dag` (ratified Phase 1 candidate in §7).  
**Out of scope here:** target realization rows, SG-class fixes, global corpus (rung 8), lens PR gates (rung 9).

---

## 1. Purpose

Define the **minimum executable runner** needed to gate **rung 3** (parse–emit–parse round-trip) and **rung 4** (emit runs; output matches `.dag` interpreter eval) on the Phase 1 fixture. Workers may implement only against this interface; Ladder/Fixture owns acceptance predicates that *consume* verdicts from it.

---

## 2. Frozen spine carriers (Phase 2 lock)

No rename or field reshuffle during rung 3–4 work. `GroundedProgramGraph` remains prose-only; implementation name stays **`InferredTree`**.

| Carrier | Location | Frozen shape |
| ------- | -------- | ------------ |
| `InferredTree` | `src/v4/compiler/04_infer.dag` | `{ root: Node, facts: Map<Node, InferredFacts> }` |
| `InferredFacts` | same | `{ grounding: CanonicalGrounding, descent: Witness<TerminationProof> }` |
| `ResolvedTree` | `src/v4/compiler/03_resolve.dag` | unchanged — infer input |
| `TestClaimRun<S, A>` | `src/v4/compiler/05_eval.dag` | `{ cache: TestClaimCacheReceipt<S>, verdict: Verdict<A> }` |
| `TestClaimEvalSubject<T>` | `05_eval.dag` | `{ claim, context, tree: InferredTree, input: TestClaimTypedInput<T> }` |

**Spine pipeline contract** (fixture subjects only; no language-specific realization):

```text
CoreNode
  → normalize → resolve → infer → InferredTree
  → emit(TranslateTo { target }) → TargetSource   // T-10; Target Realization supplies rows
  → eval(tree, interpretation, inputs) → Outcome<RuntimeValue>   // T-22
```

**Change control:** any edit to the table above requires Compiler Spine manager sign-off and Ladder/Fixture rerun of rung acceptance predicates.

---

## 3. Rung acceptance predicates (what “minimum” means)

| Rung | Question | Pass condition (fixture-scoped) | Fail-closed |
| ---- | -------- | ------------------------------- | ----------- |
| **3** | Re-parsing emit reproduces source (up to declared normalization)? | `RoundTripClaim` verdict is **`Pass`** (not `Deferred`) for at least one claim bound to the fixture module | `Deferred` / `Fail` blocks rung 3 close |
| **4** | Does emitted Rust run and match interpreter? | `EqualsClaim` (or dedicated emit-vs-eval claim) verdict **`Pass`** where `actual` comes from **host execution** of emit artifact and `expected` from `eval(InferredTree, …)` on the same subject | fabricated `Pass`, shell string match |

Rung 3/4 claims for the fixture live under `src/v4/test/claim/nat_semiring/` (new module tree per §7 Phase 2 output naming).

---

## 4. Minimum runner surface (joint API)

### 4.1 Spine-owned (T-22 / T-8–T-10)

Already landed; workers extend behavior, not signatures:

```dag
// T-9
infer(resolved: ResolvedTree) -> Outcome<InferredTree>

// T-10
emit(tree: InferredTree, target: TargetModel) -> Outcome<TargetSource>

// T-22 — canonical execution entry
eval(tree: InferredTree, interpretation: InterpretationAlgebra, inputs: Inputs) -> Outcome<RuntimeValue>

run_test_claim(subject: TestClaimEvalSubject<Node>) -> TestClaimRun<Node, RuntimeValue>
```

**Spine debt for rung 3 (blocking):** `run_test_claim_runtime_assert` and `run_test_claim_assert_decided` currently return **`Deferred`** for `RoundTripClaim` (`eval_rejected_roundtrip_deferred`). Minimum runner **requires** a modeled round-trip path: `input Node` → emit (dag target) → parse → normalize → resolve → compare to input under `DagTriviaNormalization` (C5). Spine worker owns this; Runtime does not stub `Pass`.

**Spine non-debt for rung 4:** `EqualsClaim` runtime assert already compares `actual` (root eval) to `eval(tree, …, rhs)` — sufficient for **interpreter-side** half of rung 4.

### 4.2 Runtime-owned (T-38 / host harness)

Minimum additions **outside** emit projection tables:

| Symbol | Responsibility |
| ------ | -------------- |
| `EmitHostRunReceipt` | `{ source: TargetSource, exit: HostExit, stdout: RuntimeValue, stderr: String }` — structured, not shell log grep |
| `run_emit_host(target: TargetModel, source: TargetSource, fixture_inputs: Inputs) -> Outcome<EmitHostRunReceipt>` | compile + execute emitted artifact for fixture entrypoint |
| `run_test_claim_emit_vs_eval(subject: TestClaimEvalSubject<Node>, target: TargetModel) -> TestClaimRun<Node, RuntimeValue>` | `expected = eval(…)`; `actual = run_emit_host(…)`; verdict via existing `accepted_runtime_value_outcome_eq` |
| `run_nat_semiring_rung34_eval() -> CorpusEvalReport` | roster: fixture `RoundTripClaim` + one `EqualsClaim`/`CompilesClaim` emit-vs-eval row |

Corpus aggregation reuses `src/v4/test/claim/workflow/testclaim_corpus_runner.dag` (`CorpusEvalReport`, `corpus_report_tally`). CI step consumes tally: **`fail_count == 0` and `deferred_count == 0`** for the rung 3–4 roster (subset gate, not full manual corpus).

### 4.3 Explicit split (no authority bleed)

| Concern | Compiler Spine | Runtime/TestClaim |
| ------- | -------------- | ------------------- |
| `InferredTree` / infer facts stability | ✓ | consumes only |
| Round-trip **semantic** compare in eval | ✓ | — |
| `TargetAtomRealization` / type-expression projection | — | ✗ (Target Realization) |
| rustc / cargo / python driver invocation | — | ✓ |
| `TestClaimRun` / `Verdict` shape | ✓ defines | ✓ wires CI + host |
| Falsification receipts for wrong emit | consult | ✓ owns verdict artifact |

---

## 5. Fixture binding (`nat_semiring`)

**Existing corpus** (`algebra_laws/nat_semiring.dag`): six `EqualsClaim` law rows + one `DiagnosticClaim` falsifier — interpreter-identity checks (`lhs == rhs`), **not** emit/host checks. They remain Tier-1 eval rows; they do **not** satisfy rung 4 alone.

**Phase 2 minimum roster** (authored by Ladder worker, executed via this interface):

1. `claim_nat_semiring_module_roundtrip` — `RoundTripClaim` on module subject (or `dag_round_trip_mvp1`-style structural binding once module-loader lands).
2. `claim_nat_semiring_emit_eval_agrees` — `EqualsClaim` or `CompilesClaim` with `run_test_claim_emit_vs_eval` subject pin.

Until (1) clears `Deferred`, rung 3 is **not closable** regardless of global rustc error histogram.

---

## 6. Worker dispatch shape (post sign-off)

| Order | Owner | Brief |
| ----- | ----- | ----- |
| W1 | Compiler Spine | Implement `RoundTripClaim` eval path (ingest⁻¹ on dag target); no `TargetRealization` edits |
| W2 | Runtime/TestClaim | `EmitHostRunReceipt` + `run_emit_host` for Rust-only on fixture; wire `run_nat_semiring_rung34_eval` |
| W3 | Ladder/Fixture | Land `nat_semiring/rung_3_4_*.dag` claims + CI subset gate on `CorpusEvalReport` tally |

**Forbidden:** SG-1/SG-2 histogram workers, full manual corpus (rung 8), or `InferredTree` rename without spine sign-off.

---

## 7. Sign-off

| Party | Status |
| ----- | ------ |
| Compiler Spine (`smart-stag-871`) | **Proposed** — this document |
| Runtime/TestClaim (`quick-tern-735`) | Pending |
| Ladder/Fixture (`keen-crab-361`) | Consumes §3 predicates when ratifying Phase 2 |

Amendments: spine carrier freeze (§2) requires both manager acks; acceptance predicates (§3) require Ladder/Fixture ack.
