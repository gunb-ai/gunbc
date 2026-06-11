# v4 Runtime / TestClaim Round-Robin Worksheet (RR-A)

> **Status:** RATIFIED FOR W2 DISPATCH — Branch A runtime/TestClaim readiness (ctrl#1425 §6.8, 2026-06-02).
> **Work item:** `node://adhoc-b0d2b916-0f1` — Runtime/TestClaim Mgr (`royal-gull-451`).
> **Gate:** Class 1 design closure before A.1.5a / A.2 family / A.3b implementation workers land.

## §10.0-adapted worksheet

```text
Migration class:        A-T38-RUNTIME-ENGINE + A2-CORPUS-FAMILIES + A3B-CI-RECEIPT
Representative failure:  T-38 structural bridge (#4115) closed, but CI still treats
                         authoring-time `run_test_claim` const folds as pass authority;
                         `.dag` marks still name `scripts/v4-testclaim-corpus-eval.sh` but
                         the path was deleted on main (#4252); host transport must be
                         re-landed as harness invoke; 30/31 A.2 families lack full T-38B
                         (subject_roster + family_receipt); Wave-3 shadow roster conflated
                         with runtime verdict lane.
Immediate local patch:   Extend grep/substring checks; add more `data run_*: TestClaimRun`
                         const rows; cargo-test M1-emitted Rust for manual corpus only.
Why forbidden:           P2 — authoring-time const runs co-authority with runtime harness;
                         path (i) M1 emit-Rust parallel to bootstrap evaluator (SELF_HOSTING §1);
                         grep witnesses without Bool evaluation (M5 fabrication-adjacent).
DFS path:
  eval authority (CONSUME):
    - v4.compiler.eval — run_test_claim, eval_test_claim_subject, TestClaimRun
  runtime pin (CONSUME):
    - v4.workflow.bootstrap — bootstrap_projection_inputs.runtime_model
    - v4.extdeps.runtimes.v4_evaluator — v4_evaluator_runtime_id, v4_evaluator_runtime_wave1
  corpus runner (AMEND in impl — not this PR):
    - v4.test.claim.workflow.testclaim_corpus_runner — run_manual_testclaim_corpus_eval
    - v4.test.claim.workflow.manual_corpus_eval — witness_manual_corpus_gate_closed
    - v4.test.claim.manual.manual_corpus_roster — interim explicit roster (T-19 dissolve)
  CI binding (AMEND host transport in impl):
    - v4.workflow.ci — TestClaimCorpusEvalCommand, ci_upsert_testclaim_corpus_eval_*
  shadow (SEPARATE lane — do not merge):
    - v4.test.claim.workflow.wave3_shadow_roster — roster authority only
  P5 Layer 3 authority (prerequisite doc — gunb-ai/gunbc#4143, path removed #4192 public cleanup):
    - BootstrapEvaluatorCorpusRuntimeEval — runtime run_test_claim via bootstrap pin
Deepest unsound boundary:
  run_manual_testclaim_corpus_eval() repackages compile-time `run_test_claim` results;
  no bootstrap fixed-point harness invokes eval at CI time with v4_evaluator pin.
Systemic fix:
  A.1 finish: bootstrap/evaluator harness + A.1.5a in-process equivalence receipt.
  A.2: per-family subject_roster + family_receipt under T-38B pattern (lens_idempotency model).
  A.3b: JSON receipt with execution_status=runtime_verdicts + runtime Bool gate.
Non-goals:
  - F.2 testgen automation / synthesis (R5; RR-F scope)
  - I.3 eval_parallel runtime (R5; RR-I design only)
  - lens/termination.dag substrate fork (§2.9.5 — std/cardinality.dag only)
  - Re-litigating P5 Layer 2 structural bridge (#4115 CLOSED)
Falsification probe:
  §4 table (R1–R8) — mandatory before Branch A implementation PROVEN.
Metric allowed only as secondary:
  Corpus family count with family_receipt.dag; CI wall-clock after R1–R3 land.
```

---

## §1 Branch A row map (ctrl#1425 §3)

| Row | Deliverable | Readiness | Owner lane |
| --- | ----------- | --------- | ---------- |
| **A.1** | T-38 runtime engine — `run_test_claim` at execution time via bootstrap `v4_evaluator_runtime_wave1` pin | **YELLOW** — eval primitives landed; harness entry + CI invoke missing | A.1.5a (Class 2 child) + Compiler Spine bootstrap entry |
| **A.1.5a** | In-process `TestClaimRun` equivalence claim (compile-time vs harness path) | **NOT STARTED** — dispatch after this worksheet | Class 2 child `adhoc-*` |
| **A.2** | 31 corpus families under `src/v4/test/claim/*/` (excl. `workflow/` orchestration) — continuous T-38B activation | **YELLOW** — 1 full (`lens_idempotency`); `manual` wedge (4 rows); 5 partial `run_test_claim`; 22 scaffold (+2 in flight) | Per-family PRs; lens_* in flight (#4264, #4266, #4289) |
| **A.3b** | CI host JSON receipt schema (`execution_status`, `CorpusEvalReport`, gate Bool) | **RED** — #4143 F1; script path **deleted on main** (#4252); re-land thin harness transport + runtime receipt (not “script already exists”) | With A.1 harness PR |

### 1.1 Layering vs adjacent worksheets

| Layer / doc | State | RR-A posture |
| ----------- | ----- | ------------ |
| P5 Layer 2 structural (#4115) | CLOSED | Prerequisite — do not regress `ci_upsert_testclaim_corpus_eval_*` |
| P5 Layer 3 bootstrap runtime (#4143) | Ratified; file removed public tree #4192 | **Consume F1–F6** as A.3b + A.1 acceptance; re-cite PR body not deleted path |
| Wave 3 shadow roster | Landed (`wave3_shadow_roster.dag`) | **Forbidden** as runtime pass — selection only |
| RR-F / RR-I | Parallel Class 1 children | No shared eval fork; F.5 lenses gate after A.2 families activate |

---

## §2 A.2 corpus family readiness (landed-tree survey)

**Scope:** `origin/main` landed tree only (merge-base `0cae0103c5` at worksheet authoring). In-flight PR lanes (#4264, #4266, #4289, #4259, #4260) are **not** counted as landed tiers.

**Verification receipt** (re-run before dispatch):

```bash
git ls-tree origin/main scripts/v4-testclaim-corpus-eval.sh   # → empty (deleted #4252)
find src/v4/test/claim -mindepth 1 -maxdepth 1 -type d ! -name workflow | wc -l   # → 31
find src/v4/test/claim -name family_receipt.dag    # → lens_idempotency/family_receipt.dag only
find src/v4/test/claim -name subject_roster.dag     # → lens_idempotency/subject_roster.dag only
```

`workflow/` (`testclaim_corpus_runner.dag`, `manual_corpus_eval.dag`, `wave3_shadow_roster.dag`) is orchestration, not an A.2 family. Tier sum: 1 + 1 + 5 + 22 + 2 (in flight) = 31.

| Tier | Count | Families | Landed evidence |
| ---- | ----- | -------- | --------------- |
| **ACTIVE (T-38B complete)** | 1 | `lens_idempotency` | Only tree paths with `family_receipt.dag` + `subject_roster.dag` |
| **ACTIVE (manual wedge)** | 1 | `manual` | `manual_corpus_roster.dag` (4 subjects); no per-family `family_receipt.dag` |
| **PARTIAL (`run_test_claim` only)** | 5 | `branch_dispatch`, `generated`, `lens_effect`, `loop_linear_bound`, `nat_semiring` | `.dag` files call `run_test_claim`; no family receipt module |
| **SCAFFOLD (landed)** | 22 | remaining excl. in-flight | Claims compile; no roster/receipt pattern |
| **IN FLIGHT (not landed)** | 2 | `lens_ownership`, `lens_parallelism` | #4264 / #4266 / #4289 — external until merged |

**External lane (not a landed T-38B tier):** `lens_testgen/` — PM charter #4260; profile-gated; no `family_receipt.dag` on `origin/main`.

**Forbidden for A.2 close:** treating Wave-3 `wave3_shadow_generated_runtime_value_rows` pre-built runs as CI runtime authority.

---

## §3 A.1 runtime engine — landed vs open

### Landed (consume, do not fork)

- `v4.compiler.eval::run_test_claim` / `eval_test_claim_subject` — single eval interpreter.
- `CorpusEvalReport` fold in `testclaim_corpus_runner.dag` — maps subjects → `run_test_claim` → tally.
- `manual_corpus_eval.dag` — `manual_corpus_gate` requires zero Fail/Deferred on report.
- `ci.dag` — `TestClaimCorpusEvalCommand` + declaration-qualified `TestClaimCorpusVerdictSurfaceAuthority`.
- `tools/ci_affected_components` — `testclaim_corpus` path class for affected-set.

### Open (blocks A.1 PROVEN)

| Gap | Receipt to close |
| --- | ---------------- |
| Host transport missing on main (script deleted #4252; `.dag` marks stale-name path) | R2 — **re-land** `scripts/v4-testclaim-corpus-eval.sh` as thin bootstrap harness invoke; no host-owned `gunbc compile` loop |
| No bootstrap harness entry calling `run_manual_testclaim_corpus_eval` at runtime | R1 — `self_host_fixed_point` or staged evaluator binary |
| `witness_manual_corpus_gate_closed` is compile-time const | R3 — Bool from runtime-produced report |
| Authoring-time `run_test_claim` in runner body | R4 — subjects from roster; runs from harness eval |

**Option (i) M1 emitted-Rust cargo-test** — patient-wait; **out of scope** for Branch A per #4143 §1.1.

---

## §4 Falsification table (implementation PROVEN)

| ID | Probe | Receipt |
| -- | ----- | ------- |
| R1 | CI JSON `execution_status` = `runtime_verdicts` (not `authoring_time_verdict_surface`) | Host receipt + CI step log |
| R2 | Re-landed `scripts/v4-testclaim-corpus-eval.sh` invokes bootstrap harness; no sole-pass grep of `witness_manual_corpus_gate_closed` | Script in impl PR + `git ls-tree` shows blob on main |
| R3 | `manual_corpus_gate(report)` evaluated on **runtime** report | Receipt Bool + fail-closed on any Fail |
| R4 | Subjects use `evaluator_pin == v4_evaluator_runtime_id` | `.dag` grep or TestClaim row |
| R5 | A.2 family: new `family_receipt.dag` fails if `structural_witness` not derived from claim authority (#4264 lesson) | PR review + CI |
| R6 | Wave-3 shadow receipts not used as T-38 runtime pass | ci.dag / host script separation |
| R7 | A.1.5a equivalence: harness path matches in-process `run_test_claim` on fixed slice | Hermetic test in implementation PR |
| R8 | Dissolution marks amended: `rg -n 'feature:t38-testclaim-corpus-eval' src/v4/test/claim/workflow/{testclaim_corpus_runner,manual_corpus_eval}.dag` and `src/v4/workflow/ci.dag` — on `CiCommand`, the mark must sit on the comment line immediately above `\| TestClaimCorpusEvalCommand` (not `Phase1NatSemiringRungGateCommand` / `feature:phase1-nat-semiring-rung-gate`) | Symbol-anchored grep receipt (line numbers drift) |

---

## §5 Landing order (post-worksheet implementation)

```text
1. RR-A merged (this doc) — manager dispatches A.1.5a + harness child.
2. Compiler Spine: bootstrap binary entry → invokes run_manual_testclaim_corpus_eval.
3. Runtime/TestClaim: re-land scripts/v4-testclaim-corpus-eval.sh (harness invoke-only; #4252 deleted prior blob); A.3b JSON receipt.
4. A.1.5a equivalence claim lands (gates harness correctness).
5. A.2: land lens_* in-flight PRs; continuous family PRs for remaining 22 scaffolds.
6. F.5 mandatory lens ratchet → silent-crane CI after A.2 families activate (§6.7 handoff).
```

**Lane split:** Runtime/TestClaim Mgr owns 3–5–6; Compiler Spine owns 2.

---

## §6 Forbidden patterns (grep discipline)

| Pattern | Why forbidden |
| ------- | ------------- |
| `execution_status.*authoring_time` as final CI pass | A.3b not closed |
| Host `gunbc compile` inside corpus eval script | P5 Layer 2 dissolved compile authority |
| Shadow roster runs as runtime verdict source | P2 duplicate authority |
| New `Corpus*Report` parallel to `CorpusEvalReport` | P2 |
| `data run_*: TestClaimRun = run_test_claim(...)` to close gate without harness | Authoring-time co-authority |
| `lens/termination.dag` for descent checks | §2.9.5 — use `std/cardinality.dag` |

---

## §7 Downstream handoffs (§6.7)

- **F.13 spine harness** (Grounding/Spine Mgr): after G.3.5 — consume `claim_pipeline/*` when A.2 activates folders.
- **F.5 lens CI enforcement** (silent-crane): after A.2 family receipts land — not before.
- **Source-authority Mgr**: F.2-P1 testgen CLI is RR-F scope; coordinate only on shared `GeneratorId` facts.

---

## §8 Modeling DFS Arbiter checklist

- [x] Single-authority: runtime `run_test_claim` via bootstrap evaluator pin (not M1 emit-Rust path (i))
- [x] Distinct from P5 Layer 2 (#4115) and Wave-3 shadow roster
- [x] Spot-fix forbidden: const-run pass, grep-only witness; host transport re-land (#4252), not “convert existing script”
- [x] A.2 family tier survey accepted (31 families; `workflow/` orchestration excluded)
- [x] Falsification R1–R8 accepted
- [x] Landing order §5 + lane split accepted
- [x] **READY-FOR-WORKER-DISPATCH** (RR-A Class 1 closure — implementation workers A.1.5a / harness / A.2 families)

---

## Related artifacts

- gunb-ai/gunbc#4143 — P5 bootstrap-evaluator corpus runtime (Layer 3; doc path removed #4192)
- gunb-ai/gunbc#4115 — P5 structural bridge (Layer 2 CLOSED)
- gunb-ai/gunbc#4120 — T-38-PR2 verdict surface
- `src/v4/test/claim/workflow/testclaim_corpus_runner.dag`
- `src/v4/workflow/bootstrap.dag` — `bootstrap_projection_inputs.runtime_model`
- `src/v4/workflow/ci.dag` — `ci_upsert_testclaim_corpus_eval_*`
- Open PRs: #4264, #4266, #4289 (lens T-38B), #4259 (manual)
