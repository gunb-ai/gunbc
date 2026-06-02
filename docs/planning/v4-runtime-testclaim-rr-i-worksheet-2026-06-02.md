# v4 Runtime / TestClaim Round-Robin Worksheet (RR-I)

> **Status:** RATIFIED FOR W2 DISPATCH — Branch I lens-closure posture + corpus-runner contract (ctrl#1425 §3 Branch I, 2026-06-02).
> **Work item:** `node://adhoc-9c3209a0-860` — RR-I worksheet (`vivid-eagle-128`) under Runtime/TestClaim Mgr (`royal-gull-451`).
> **Gate:** Class 1 design closure. I.1 states the closure contract; the I.1.1 worker (`adhoc-8b1709c7-7c0`) executes UnknownCoupling ratification/collapse if pulled forward. I.3 is design-only — no `eval_parallel` runtime lands from this PR.

## §10.0-adapted worksheet

```text
Migration class:        I1-DEP-LENS-CLOSURE + I3-CORPUS-RUNNER-CONTRACT
Representative failure:  Four dependency-lens families (parallelism/effect/ownership/
                         idempotency) reached the T-13 family shape, but "closure" was
                         never written down: a reader cannot tell whether the single
                         ClassifiedDependencyView authority is real or whether parallel
                         roster fields / ad-hoc Bool predicates still co-author. Separately
                         testclaim_corpus_runner.dag carries two unsupported subject
                         families (ci_pipeline, non_runtime_value) with a generic reason
                         string and no contract for what would promote or retire them.
Immediate local patch:   Add per-lens "looks closed" comments; add a fifth/sixth lens to
                         the family by copy; widen run_test_claim with an ad-hoc second
                         entry point per unsupported family.
Why forbidden:           P2 — a second classification authority parallel to
                         ClassifiedDependencyView<C> re-introduces the exact parallel-payload
                         defect the T-13 ratchet dissolved (MODELING M6 / Practice 11);
                         a per-family run_test_claim fork co-authors the eval interpreter
                         (SELF_HOSTING §1, RR-A §6). Registry migration of Complexity/Cost/
                         TableDecisionTree is OUT (these are not dependency-edge lenses).
DFS path:
  shared substrate (CONSUME — do not fork):
    - v4.std.dependency — DependencyView, ClassifiedDependencyView<C>,
      DependencyKindClassifier<C>, classify_dependency_view<C>
  closed family members (DOCUMENT closure posture):
    - v4.lens.parallelism — ParallelismRelation; parallelism_dependency_kind_classifier
    - v4.lens.effect — EffectClassification (signature-deferred slot; #3468 follow-up)
    - v4.lens.ownership — OwnershipMode; RequiresAccessWitness access routing
    - v4.lens.idempotency — IdempotencyVerdict; algebra-law witness as payload
  adjacent family members (NOTE, not in-scope for I.1):
    - v4.lens.unused_parameters, v4.lens.structural_resolution — same family shape,
      different (non-dependency-runtime) verdict surfaces
  corpus runner contract (DESIGN only):
    - v4.compiler.eval — run_test_claim : TestClaimEvalSubject<Node> -> TestClaimRun<Node, RuntimeValue>
    - v4.test.claim.workflow.testclaim_corpus_runner — testclaim_subject_roster_unsupported_rows
    - v4.test.claim.manual.manual_corpus_roster — manual 5-row Node/RuntimeValue wedge
Deepest unsound boundary:
  run_test_claim is monomorphic over (Node, RuntimeValue). The two unsupported families
  are not "not yet wired" — they are domains whose evaluator input and actual-value type
  are NOT (Node, RuntimeValue). Promoting them requires a real projection or a parametric
  evaluator, NOT another roster row.
Systemic fix:
  I.1: ratify single-authority closure for the four families; collapse honest-but-wide
       UnknownCoupling default into a typed unresolved coproduct (I.1.1 lane) — design only.
  I.3: state the projection contract that would promote each unsupported family, OR the
       dissolution trigger that keeps it explicitly unsupported. Gate any eval_parallel
       runtime behind the §5.0.2 CI-needs predicate.
Non-goals:
  - Registry migration of Complexity / Cost / TableDecisionTree (ctrl#1425 §2.5.2 defer)
  - Scaffold (`T-13-*-unresolved`) dissolution of ANY family — tracked per §2 per-family trigger table, not this PR (only effect / ownership-access cross-ref #3468)
  - eval_parallel runtime (R5 / RR-A §6 "I.3 eval_parallel runtime — design only")
  - Forking run_test_claim per subject family
Falsification probe:
  §4 table (R1–R7) — mandatory before any I.1.1 collapse or I.3 promotion PR lands.
Metric allowed only as secondary:
  Count of family members under single ClassifiedDependencyView authority; corpus
  unsupported-row count after a promotion lands.
```

---

## §1 Branch I row map (ctrl#1425 §3)

| Row | Deliverable | Readiness | Owner lane |
| --- | ----------- | --------- | ---------- |
| **I.1** | Dependency-lens closure posture — single `ClassifiedDependencyView<C>` authority for parallelism/effect/ownership/idempotency; honest classifier tables; no parallel roster fields | **GREEN (design ratified)** — four families on main post-#4264/#4292 | This worksheet |
| **I.1.1** | UnknownCoupling (and peers) ratification/collapse into a typed unresolved coproduct | **NOT STARTED** — execute if pulled forward | Class 2 child `adhoc-8b1709c7-7c0` |
| **I.3** | Corpus-runner contract for the two unsupported subject families (promote-vs-keep-unsupported) + §5.0.2 CI-needs predicate | **GREEN (design-only)** — types/authority doc; no runtime | This worksheet |

### 1.1 Layering vs adjacent worksheets

| Layer / doc | State | RR-I posture |
| ----------- | ----- | ------------ |
| RR-A (Branch A runtime engine) | Ratified #4296 | **Consume** A.2 survey + manual 5-row wedge; do not re-litigate `run_test_claim` ownership |
| T-13 family ratchet (#4264, #4292) | MERGED | Prerequisite — the four families are the closure subject, not re-authored here |
| #3468 signature-derived effect-kind set | Open (BLOCKING follow-up) | Dissolves **only** the `effect` kind-set and `ownership` access-carrier scaffolds; the `parallelism` / `idempotency` / `ownership-mode` `*-unresolved` predicates dissolve on their own per-coproduct triggers (§2 table). This PR documents, does not dissolve |
| I.1.1 collapse lane | Class 2 child | RR-I states the contract; collapse PR cites §4 R1–R4 |

---

## §2 I.1 — dependency-lens closure posture (landed-tree survey)

**Scope:** `origin/main` landed tree. Verification receipt (re-run before dispatch):

```bash
grep -l 'ClassifiedDependencyView' src/v4/lens/*.dag
# → effect, idempotency, ownership, parallelism, structural_resolution, unused_parameters
grep -n 'type DependencyKindClassifier\|type ClassifiedDependencyView' src/v4/std/dependency.dag
# → single shared substrate authority (lines 52, 57)
```

**The four in-scope dependency-runtime families** (RR-I I.1):

| Lens | Classification coproduct | Classifier honesty | Single-authority evidence |
| ---- | ------------------------ | ------------------ | ------------------------- |
| `parallelism` | `ParallelismRelation = DataDependent \| EffectCoupled \| BarrierCoupled \| UnknownCoupling` | 9 of 15 `DependencyKind`s map to `UnknownCoupling` — **honest** (does not fabricate coupling it cannot derive) | `ParallelismFact.dependencies: List<ClassifiedDependencyView<ParallelismRelation>>` — no copied endpoints |
| `effect` | `EffectClassification {}` — single signature-deferred slot | Effect kind routed via `tree.facts.lookup(dependency.source)`; `DependencyKind` is NOT the effect authority (B3 / P2 / Practice 5) | Parametric `ClassifiedDependencyView<EffectClassification>`; closing the kind set is #3468 |
| `ownership` | `OwnershipMode = OwnedContainment \| SharedRead \| RequiresAccessWitness \| BorrowedDependency \| UnknownOwnership` | Access/alias-mode evidence routed via `RequiresAccessWitness`; closed access carrier is a #3468 follow-up | `ClassifiedDependencyView<OwnershipMode>` — no parallel substrate |
| `idempotency` | `IdempotencyVerdict = AlgebraicIdempotenceProven { law: Witness<InferredFacts> } \| RequiresAlgebraWitness` (the law `Witness` itself carries `Holds`/`Violates { diagnostic }`) | Algebra-law witness IS the verdict payload; edge identity authority is `ClassifiedDependencyView.dependency` | No nested per-row `EffectDependencyFact` |

**Closure contract (what "closed" means, ratified):**

1. **Single classification authority.** Each family projects exactly one `List<ClassifiedDependencyView<C>>`; the dependency-edge identity lives on the inner `DependencyView`, never copied into a parallel roster field (Practice 11 sub-rule: typed-reference, not parallel-payload).
2. **Honest unresolved arm, per-family dissolution trigger.** Each `C` carries an explicit unresolved/unknown variant; the `*_unresolved` predicate lives **inside** the `*_witness`, gated `feature:T-13-*-unresolved`, with no ad-hoc Bool predicate outside the witness. The dissolution triggers are **distinct per family** (they are NOT all #3468 — that compression would make the scaffold receipts uncheckable):

   | Lens | Gate mark (live) | Concrete dissolution trigger | #3468? |
   | ---- | ---------------- | ---------------------------- | ------ |
   | `parallelism` | `feature:T-13-parallelism-relation-unresolved` | substrate projects `ParallelismRelation` resolved/unresolved from coproduct (`parallelism.dag:75`) | No |
   | `idempotency` | `feature:T-13-idempotency-verdict-unresolved` | substrate projects `IdempotencyVerdict` resolved/unresolved from coproduct (`idempotency.dag:82`) | No |
   | `ownership` (mode) | `feature:T-13-ownership-mode-unresolved` | substrate projects `OwnershipMode` resolved/unresolved from coproduct (`ownership.dag:77`) | No |
   | `ownership` (access carrier) | (status note `ownership.dag:3`) | deriving the closed access carrier behind `RequiresAccessWitness` | Yes (#3468 follow-up) |
   | `effect` | `feature:T-13-effect-signature-deferred-unresolved` | `InferredFacts` decodes to a closed effect-kind carrier (`effect.dag:88`) | Yes (#3468) |

   So only **effect**'s kind-set closure and **ownership**'s access-carrier follow-up cross-reference #3468; the three per-coproduct `*-unresolved` predicates (parallelism / idempotency / ownership-mode) each dissolve on their own lens's substrate-projects-resolved/unresolved trigger. An I.1.1 collapse PR closes one trigger at a time and cites the matching gate mark.
3. **No registry migration.** Complexity / Cost / TableDecisionTree are NOT dependency-edge lenses and stay out of the family (ctrl#1425 §2.5.2). `unused_parameters` / `structural_resolution` share the shape but project non-dependency-runtime surfaces — noted, not re-authored here.

**Forbidden for I.1 close:** adding a fifth dependency-runtime family by copy instead of by `ClassifiedDependencyView<NewC>`; re-authoring effect kind from `DependencyKind` (second authority); promoting `UnknownCoupling` to a concrete relation without a derived fact (I.1.1 collapse must consume a substrate fact, not a heuristic).

---

## §3 I.3 — corpus-runner contract (design-only)

**Current authority (consume, do not fork):**

- `run_test_claim : TestClaimEvalSubject<Node> -> TestClaimRun<Node, RuntimeValue>` — monomorphic over `(Node, RuntimeValue)` (`05_eval.dag:1931`).
- `testclaim_corpus_runner.dag` — folds `manual_corpus_node_subject_rows` (5-row wedge) through `run_test_claim` into `CorpusEvalReport`; `TestClaimRun<S, A>` is already generic, only the runner driver is monomorphic.
- `testclaim_subject_roster_unsupported_rows` — two rows, both reason `..._unsupported_until_runner_projection_lands`.

**Why the two families are unsupported (root cause, not "not yet wired"):**

| Unsupported family | Domain mismatch | What promotion requires |
| ------------------ | --------------- | ----------------------- |
| `ci_pipeline` | Subject's evaluator input is a CI-pipeline node whose actual value is a CI report / `Upsert` step, **not** `RuntimeValue` | A projection `ci_pipeline_subject → (Node, RuntimeValue)` (lower the pipeline to a Node root + RuntimeValue tally), **or** a parametric `run_test_claim<A>` with a per-domain evaluator algebra |
| `non_runtime_value` | Subject's actual answer is a structural/`Hash`/verdict surface, not `RuntimeValue` | Same: a projection into `RuntimeValue`, or parametric over actual type `A` with an `A`-evaluator pin |

**Contract decision (ratified):** Keep both rows **explicitly unsupported** with typed dissolution triggers; do NOT widen `run_test_claim` with per-family entry points (P2 fork). Promotion lands only when one of:

- **T (projection trigger):** a single-authority projection from the family's subject into `(Node, RuntimeValue)` exists in substrate (preferred — keeps one evaluator). The unsupported row's `reason` is then replaced by a `feature:` mark naming the projection.
- **T' (parametric trigger):** `run_test_claim` is generalized to `run_test_claim<A>` driven by a registered per-domain evaluator algebra (heavier; only if a domain genuinely cannot project to `RuntimeValue`). This is a substrate change requiring its own L2.5 model PR — escalate, do not improvise.

Until a trigger fires, the rows MUST stay in `testclaim_subject_roster_unsupported_rows` with a reason that names the missing projection — silent omission is forbidden (a dropped family reads as "covered").

### §5.0.2 CI-needs predicate (eval_parallel gating)

`eval_parallel` runtime (parallel corpus evaluation) is **R5 / design-only** here. Implement the runtime **only if** the landed corpus runner trips at least one:

```text
eval_parallel_needed(metrics) =
     metrics.p95_corpus_eval_minutes   > 15
  OR metrics.shadow_eval_minutes       >  5
  OR metrics.dispatch_queue_days       >  3
```

Default deliverable is this types/authority doc, NOT an `eval_parallel` runtime (RR-A §6: "I.3 eval_parallel runtime — design only"). If a future operator metric trips the predicate, that runtime is a separate implementation lane with its own falsification table.

---

## §4 Falsification table (closure/promotion PROVEN)

| ID | Probe | Receipt |
| -- | ----- | ------- |
| R1 | Each of the four families projects exactly one `List<ClassifiedDependencyView<C>>`; no parallel roster field on the `*Fact` | `grep` `*Fact` type bodies — single `dependencies:` field |
| R2 | Every `C` has an unresolved arm and its `*_unresolved` predicate sits inside `*_witness`, tagged `feature:T-13-*-unresolved` | `grep -n 'feature:T-13-.*-unresolved' src/v4/lens/*.dag` |
| R3 | `UnknownCoupling`/peers are not promoted to a concrete relation without a derived substrate fact (I.1.1) | I.1.1 PR review — classifier change cites a `tree.facts.lookup` source, not a heuristic |
| R4 | No second classification authority parallel to `ClassifiedDependencyView<C>` introduced | `grep` for new `*DependencyFact` / endpoint-copy types — none |
| R5 | No `eval_parallel` runtime lands unless `eval_parallel_needed` trips | Absent from impl PRs; predicate cited |
| R6 | An unsupported corpus family is promoted ONLY via a projection-into-`(Node,RuntimeValue)` or an escalated parametric-evaluator L2.5 PR — never a per-family `run_test_claim` fork | Promotion PR diff: row reason → `feature:` mark; single `run_test_claim` |
| R7 | A removed unsupported row is removed because promoted, never silently dropped | `testclaim_subject_roster_unsupported_rows` diff matches a new supported subject row |

---

## §5 Landing order (post-worksheet)

```text
1. RR-I merged (this doc) — manager may pull I.1.1 forward.
2. I.1.1 (if dispatched): collapse UnknownCoupling/peers into typed unresolved coproduct
   sourced from a derived fact — cites §4 R1–R4. Design contract is §2 here.
3. I.3 promotion (only when trigger T or T' fires): land the projection OR escalate the
   parametric-evaluator substrate PR; move the row out of unsupported_rows (§4 R6/R7).
4. eval_parallel runtime: only if §5.0.2 predicate trips — separate lane.
```

**Lane split:** RR-I (this doc) owns the design contract; I.1.1 child owns the collapse; corpus promotion is a Runtime/TestClaim implementation lane gated on a real projection.

---

## §6 Forbidden patterns (grep discipline)

| Pattern | Why forbidden |
| ------- | ------------- |
| New `*DependencyFact` / endpoint-copy parallel to `ClassifiedDependencyView<C>` | P2 second classification authority (the T-13 defect) |
| `*_unresolved` / `Unknown*` Bool predicate outside the `*_witness` | Predicate-dissolution interim leak (Practice 10) |
| Re-authoring effect kind from `DependencyKind` | B3 / P2 — `DependencyKind` is not the effect authority |
| Promoting `UnknownCoupling` to a concrete relation via heuristic | Heuristics recoverable to substrate facts — derive the fact |
| Per-family `run_test_claim` entry point | P2 eval-interpreter fork (SELF_HOSTING §1) |
| Silent removal of an `unsupported_rows` entry | Dropped family reads as covered — must be a documented promotion |
| `eval_parallel` runtime without `eval_parallel_needed` | R5 — premature parallelism |

---

## §7 Downstream handoffs (§6.7)

- **I.1.1 child (`adhoc-8b1709c7-7c0`)**: consume §2 closure contract + §4 R1–R4 before any UnknownCoupling collapse.
- **Runtime/TestClaim Mgr**: corpus promotion (§3 trigger T/T') is an implementation lane; the parametric-evaluator path (T') requires an L2.5 substrate model PR — escalate, do not improvise (`run_test_claim` is load-bearing per SELF_HOSTING).
- **#3468 lane**: closes the `effect` signature-derived kind set and the `ownership` access carrier only; the `parallelism` / `idempotency` / `ownership-mode` `*-unresolved` predicates dissolve on their own per-coproduct triggers (§2 table), independent of #3468.

---

## §8 Modeling DFS Arbiter checklist

- [x] Single-authority: one `ClassifiedDependencyView<C>` per family; one `run_test_claim` interpreter
- [x] Distinct from RR-A (runtime engine) and #3468 (kind-set closure) — no shared fork
- [x] Spot-fix forbidden: copy-a-fifth-family, per-family runner fork, heuristic UnknownCoupling promotion, silent unsupported-row drop
- [x] Non-goals accepted: Complexity/Cost/TableDecisionTree registry migration; `T-13-*-unresolved` scaffold dissolution of any family (per-family triggers in §2, not all #3468); eval_parallel runtime
- [x] Corpus contract: promote-via-projection-or-escalated-parametric, else keep explicitly unsupported with typed trigger
- [x] §5.0.2 CI-needs predicate accepted (eval_parallel gated p95>15m OR shadow>5m OR queue>3d)
- [x] Falsification R1–R7 accepted
- [x] **READY-FOR-WORKER-DISPATCH** (RR-I Class 1 closure — I.1.1 collapse + corpus promotion implementation workers)

---

## Related artifacts

- gunb-ai/gunbc#4296 — RR-A worksheet (runtime engine; consumed: A.2 survey + manual wedge)
- gunb-ai/gunbc#4264 — T-38B `lens_ownership` (T-13 family closure; MERGED)
- gunb-ai/gunbc#4292 — T-38B `lens_parallelism` (T-13 family closure; MERGED)
- gunb-ai/gunbc#3468 — signature-derived effect/ownership kind-set closure (BLOCKING follow-up)
- `src/v4/std/dependency.dag` — `ClassifiedDependencyView<C>`, `DependencyKindClassifier<C>`
- `src/v4/lens/{parallelism,effect,ownership,idempotency}.dag` — the four closed families
- `src/v4/compiler/05_eval.dag:1931` — `run_test_claim`
- `src/v4/test/claim/workflow/testclaim_corpus_runner.dag` — `testclaim_subject_roster_unsupported_rows`
- `src/v4/test/claim/manual/manual_corpus_roster.dag` — manual 5-row Node/RuntimeValue wedge
