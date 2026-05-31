# P5 Worksheet — Structural-bridge replacement (`v4-testclaim-corpus-gate.sh` → positive-Y `CiUpsertStep`)

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Manager §8 sign-off 2026-05-31 (`proud-pike-680`; PR [#4114](https://github.com/gunb-ai/gunbc/pull/4114)).
> **Date:** 2026-05-31 (dispatch node://adhoc-520e78dc-d69; `witty-lark-788`)
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` §3.5 Layer 2; PR #4094 decision 2; operator GO msg_30a4b598 (`proud-pike-680`).
> **Predicate:** P5 TestClaim suite passes — **Layer 2 authority gate** (Layer 1 fixture/law bundle 3/3 CLOSED).
> **Ratified design authority:** [#4091](https://github.com/gunb-ai/gunbc/pull/4091) — `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` on `main` (squash `c05a5a84`).

### Status (single authority — no contradiction)

| Layer | State |
| ----- | ----- |
| **Worksheet** | **READY-FOR-WORKER-DISPATCH** — §8 closed 2026-05-31 (`proud-pike-680` msg_a4040844) |
| **Worksheet PR** | [#4114](https://github.com/gunb-ai/gunbc/pull/4114) — docs-only; merge unblocks implementation dispatch |
| **Prerequisites on `origin/main`** | `v4-ci-schema-worksheet` §8 CLOSED (#3989); W2.3 `ci_upsert_steps` bijection landed; #4091 exploration ratified; #4095 Worksheets A+B §8 CLOSED (composition vocabulary only — **not** bridge implementation) |
| **Implementation dispatch** | **Authorized** after #4114 on `main` — Compiler Spine + Runtime/TestClaim per §6 |

---

## Mechanical dispatch rule

> **No P5 structural-bridge implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved.**

Acceptance is **§4 falsification rows (shell gone + modeled step sole structural authority)**, not wall-clock CI reduction or full T-38 runtime verdict execution on M1 emit-Rust (separate milestone; see §7).

---

## §10.0-adapted worksheet

```text
Bridge-replacement class:  P5-LAYER-2-STRUCTURAL-BRIDGE (T-22 manual corpus CI authority)
Representative failure:    Parallel CI authority — `scripts/v4-testclaim-corpus-gate.sh` + hand-authored
                           `ci_github_actions_workflow.dag` / `ci.yml` steps compile `src/v4` twice
                           (rust+dag) while `ci.dag` already declares `testclaim_corpus_eval_*` +
                           `CiUpsertStep` rows that do not drive the live workflow (negative authority:
                           shell owns compile + jq/python artifact inspection).
Immediate local patch:   Keep shell; add `hashFiles(...)` cache on bridge receipt; wire
                           `V4_TESTCLAIM_REUSE_*` env from prior M1/bootstrap steps only in YAML;
                           extend gate.sh python receipt checks.
Why forbidden:           Preserves hand-YAML + shell as co-authority (Phase 2 forbidden); duplicates
                           M1 rust + bootstrap dag compiles per #4091 §1.2 (C+D = A+B); hashFiles
                           cache key is parallel cache authority (Worksheet B §3 — must dissolve to
                           content_hash(CiUpsertStep projection)); bypasses IRT-1 selection_fn discipline
                           already on `TestClaimCorpusEvalCommand`.
DFS path:
  CONSUME (edges only — do not re-catalog):
    - docs/planning/v4-ci-schema-worksheet-2026-05-30.md §8 — CiUpsertStep<T>, UpsertInputRef
    - docs/planning/v4-elastic-compute-fabric-worksheet-2026-05-30.md — WorkUnit ingress at composition
    - docs/planning/v4-elastic-cache-interface-worksheet-2026-05-30.md — CachedArtifactReceipt /
      content_hash discipline at composition (no cache_interface import in ci.dag)
    - docs/planning/elastic-ci-redesign-exploration-2026-05-31.md §3 Upsert state + §1.2 four-compile
  READ (alignment — no new parallel rows without §8 amendment):
    - src/v4/workflow/ci.dag L135+ — T-38 dissolution marks; `testclaim_corpus_eval_execution`,
      `testclaim_corpus_eval_signal`, `ci_upsert_testclaim_corpus_eval_*`, `ci_testclaim_corpus_selection_fn`
    - scripts/v4-testclaim-corpus-gate.sh — replacement target (grep pass condition)
    - dsl/gunbc/ci_github_actions_workflow.dag — live negative-Y steps (~L731–749)
  Host runner (downstream only):
    - src/v4/test/claim/workflow/testclaim_corpus_runner.dag — modeled corpus eval entry
    - scripts/v4-testclaim-corpus-eval.sh — T-38 scaffold host transport (evolve or replace in impl PR;
      must not re-own compile logic deleted from gate.sh)
Deepest unsound boundary:
  `ci.dag` declares the T-22/T-38 step universe (`TestClaimCorpusEvalCommand` + CiUpsertStep rows) but
  GitHub Actions still executes a shell bridge with its own compile+cache semantics — two authorities
  for the same structural corpus check; the modeled step is not positive-Y (affirmative sole authority).
Systemic fix:
  Positive-Y wiring: live CI projection + cache identity + host transport read **only** from
  `ci_upsert_testclaim_corpus_eval_execution` (+ signal gate row) and `TestClaimCorpusEvalCommand`
  with `selection_fn == ci_testclaim_corpus_selection_fn` (already `ci_select_from_affected_set`).
  Delete `scripts/v4-testclaim-corpus-gate.sh` and dissolve paired workflow steps in the **same**
  implementation PR. First concrete #4091 CiUpsertStep replacement: extend execution row `inputs` with
  `UpstreamUpsert` to M1 rust emit (and bootstrap dag artifact when modeled or via documented
  composition-edge env mapping — see §1.2).
Non-goals:
  - Full T-38 GREEN / per-row `TestClaimRun` verdict execution in CI (M1 subset blocked — TASKS §T-38)
  - Duplicating #4095 `compute_fabric.dag` / `cache_interface.dag` type catalogs in this worksheet
  - Phase 2.5 active receipt fanout; generated-only ci.yml deletion beyond this bridge slice
  - P2 bootstrap resolve-posture bridge (`scripts/v4-bootstrap-resolve-posture-gate.sh`) — separate dispatch
  - Amending `v4-ci-schema-worksheet` unless exactly one new UpsertInputRef variant is unavoidable (escalate)
Falsification probe:
  See §4 table — (F1)–(F6) mandatory for implementation PROVEN.
Metric allowed only as secondary:
  ci_v4 wall-clock; four-compile count — report after F1–F6, not acceptance.
```

---

## §1 Single-authority fact

| Field | Value |
| ----- | ----- |
| **Fact name** | Positive-Y T-22 TestClaim corpus **structural** CI check via existing `CiUpsertStep` pair |
| **Negative authority (delete)** | `scripts/v4-testclaim-corpus-gate.sh` + `t22_testclaim_corpus_cache` / bridge `RunStep` in `ci_github_actions_workflow.dag` (and emitted `ci.yml` parity) |
| **Positive authority (consume)** | `ci_upsert_testclaim_corpus_eval_execution` + `ci_upsert_testclaim_corpus_eval_signal` bound to `JobStep`/`GateStep` ids `testclaim_corpus_eval_execution`, `testclaim_corpus_eval_signal` |
| **Command carrier** | `TestClaimCorpusEvalCommand { selection_fn: ci_testclaim_corpus_selection_fn }` where `ci_testclaim_corpus_selection_fn = ci_select_from_affected_set` (IRT-1 — already enforced in `ci_command_authority_ok`) |
| **Cache authority** | `ci_upsert_step_cache_digest` / `content_hash(ci_upsert_step_projection_node(step))` — **not** `hashFiles(...)` on shell path list |
| **Canonical home** | `src/v4/workflow/ci.dag` (`v4.workflow.ci`) — **amend** existing rows only; no third parallel step vocabulary |
| **Host transport** | Thin host script or test binary invoked by projected workflow step — **must not** re-implement `gunbc compile` loops presently in gate.sh (forbidden: host-owned compile authority) |
| **Explicitly NOT** | A new `CiCommand` arm; a parallel `Symbol` cache tag as sole identity (`ci_cache_cmd_testclaim_corpus_eval_tag` must yield to full-node digest per #4091 §3.4 gap row) |

### 1.1 Positive-Y vs negative-Y (vocabulary)

| Posture | Authority | This bridge |
| ------- | --------- | ----------- |
| **Negative-Y** | Shell + YAML own compile, cache key, artifact inspection | **Today** — `v4-testclaim-corpus-gate.sh` |
| **Positive-Y** | `CiUpsertStep` projection + `TestClaimCorpusEvalCommand` drive schedule, inputs, cache, and host transport binding | **Target** — first #4091 concrete replacement |

### 1.2 #4091 four-compile alignment (composition edges only)

Per exploration §1.2, T-22 bridge duplicates M1 rust (A≈C) and bootstrap dag (B≈D). Implementation **must** express reuse through existing substrate — not new shell env blocks:

| Upstream artifact | Modeled consumption (edge) | Interim bridge env (dissolve with shell) |
| ----------------- | -------------------------- | ---------------------------------------- |
| M1 rust emit dir | `UpstreamUpsert { step_id: JobStep { job: m1_rust_emit_probe_execution, step: m1_rust_emit_probe_execution } }` on **execution** row `inputs` | `V4_TESTCLAIM_REUSE_RUST_OUT` |
| Bootstrap dag emit | `UpstreamUpsert` when bootstrap lands in `ci_pipeline` shadow universe **or** `CachedArtifactReceipt` producer ref at harness boundary (Worksheet B §composition) | `V4_TESTCLAIM_REUSE_DAG_OUT` |

If bootstrap remains workflow-local past implementation PR, worker **must** document a single carveout row in `ci_always_run_carveouts` with `dissolution_target` pointing at bootstrap `CiJob` — not extend gate.sh.

### 1.3 Existing symbols (read-only in worksheet PR)

Align implementation to **these** ids — do not fork names:

```text
ci_testclaim_corpus_selection_fn  = ci_select_from_affected_set
testclaim_corpus_eval_execution     (CiJob)
testclaim_corpus_eval_signal      (CiGate, RequiresJobAttempt { job: v2_compile_src_v4 })
ci_upsert_testclaim_corpus_eval_execution
ci_upsert_testclaim_corpus_eval_signal
ci_testclaim_corpus_eval_claim_ids + FileSet inputs (manual/**, runner.dag, scripts/v4-testclaim-*)
```

Dissolution marks at `ci.dag` L135–136 and `testclaim_corpus_runner.dag` L4–5 fire when F1–F6 pass — implementation PR removes 🟡 by deletion + projection, not comment-only edits.

---

## §2 Composition boundary (#4095 — edges only)

Compose with elastic substrate **only** through:

| Edge | Worksheet | Use in bridge replacement |
| ---- | --------- | ------------------------- |
| `UpsertInputRef` / `CiUpsertStep<T>` | v4-ci-schema §8 | Row bodies, `UpstreamUpsert`, `TestClaimRef`, `FileSet` |
| `artifact_refs_from_upsert_inputs` → `WorkUnit.inputs` | compute-fabric A §1.1 | Schedule narrowing at ingress — **do not** re-define `WorkUnit` here |
| `content_hash(step projection)` → cache lookup | cache-interface B §1.1 | Replace `hashFiles` cache step — **do not** land `CacheInterfaceFacts` rows in bridge PR unless already on main |
| `ExecutionReceipt` / `ProducerReceipt` | #4095 §composition | Optional secondary evidence for upstream hit — not acceptance |

**Forbidden in implementation PR:** `import` of `compute_fabric.dag` into `ci.dag`; duplicating Worksheet A/B §1 type catalogs; `host:` / `worker_count` on `CiUpsertStep`; new `CacheKind` enum.

---

## §2.1 Parser / substrate gates (dispatch prerequisites)

| Gate ID | Owner | Requirement before bridge worker starts | Unblock signal |
| ------- | ----- | --------------------------------------- | -------------- |
| **P5-BRIDGE-WS** | Modeling DFS | This worksheet §8 **CLOSED** | proud-pike-680 sign-off |
| **P5-BRIDGE-PARSER** | Compiler Spine | `CiUpsertStepSymbol` record literals + row `inputs` shadow compile in `ci.dag` (P1.5-PARSER baseline on main) | `gunbc compile` receipt on touched `ci.dag` module |
| **P5-BRIDGE-4095** | (met for vocabulary) | #4095 on `main` if implementation cites `ArtifactRef` / cache projection helpers | Merge commit on `origin/main` |
| **P5-BRIDGE-PROJ** | Compiler Spine | `ci_github_actions_workflow.dag` projects bridge steps from `ci_pipeline` / upsert rows — not hand `bash scripts/v4-testclaim-corpus-gate.sh` | Integration smoke: `v4_workflow_ci_runner_dag_smoke_test` updated |
| **P5-BRIDGE-HOST** | Runtime/TestClaim + Spine | Host transport ≤ compile orchestration; structural receipt matches gate.sh **obligations** (manual modules present, TestClaimRun rows in artifact, MVP rust witness until T-38 runtime) | F2 + F3 |
| **P5-BRIDGE-GREP** | Close/Receipt | §3 forbidden grep clean | F5 |

**Discipline:** gates **P5-BRIDGE-PARSER** through **P5-BRIDGE-GREP** close in the **implementation PR**, not the worksheet PR.

---

## §3 Spot-fix register (forbidden)

| Pattern | Where it shows up today | Why forbidden |
| ------- | ---------------------- | ------------- |
| `bash scripts/v4-testclaim-corpus-gate.sh` in workflow | `ci_github_actions_workflow.dag` ~L743; `ci.yml` | Negative-Y shell authority |
| `hashFiles(..., 'scripts/v4-testclaim-corpus-gate.sh', ...)` cache key | `ci.yml` ~L408–412; workflow `UsesStep` ~L735 | Parallel cache authority (Worksheet B); key ignores step projection |
| `gunbc compile --source-root src/v4` inside host transport | `v4-testclaim-corpus-gate.sh` | Compile authority must be upstream `CiUpsertStep` / shared emit steps (#4091) |
| Python/jq artifact inspection as **sole** receipt | gate.sh ~L149–507 | Structural checks must be modeled or emitted from runner projection — host may validate JSON receipt only |
| Parallel `CiJob` for “corpus gate” | temptation | Universe already has `testclaim_corpus_eval_execution` |
| `selection_fn` other than `ci_testclaim_corpus_selection_fn` | spot-fix to “run all claims” | Violates IRT-1 / `ci_command_authority_ok` |
| `ci_cache_cmd_testclaim_corpus_eval_tag` as **only** cache identity | `ci.dag` payload_type today | #4091 §3.4 — static Symbol tag must not override `content_hash(whole node)` at implementation boundary |
| New `UpsertInputRef` variant without DFS escalation | worker temptation | v4-ci-schema §8 closed — amend via manager only |
| Keep shell “for soak” after modeled step lands | operator anti-pattern | Dissolve-on-arrival: same PR deletes script (T-38 / P5 Layer 2) |
| Full M1 `cargo check` on emitted tree as P5 acceptance | T-38 / M1 lane | Out of scope §7 — not Layer 2 structural |

**Forbidden grep (implementation PR — literal strings; zero hits uncited):**

```text
v4-testclaim-corpus-gate.sh
hashFiles(.*v4-testclaim-corpus-gate
V4_TESTCLAIM_REUSE_RUST_OUT
V4_TESTCLAIM_REUSE_DAG_OUT
T-22 TestClaim corpus structural bridge
t22_testclaim_corpus_cache
ci_cache_cmd_testclaim_corpus_eval_tag
```

Allowed: references in **deleted** file history, worksheet citations, or 🟡 dissolution comments that cite this worksheet and are removed when marks fire.

Escalate to Modeling DFS: need for a second corpus gate command; bootstrap cannot be expressed as `UpstreamUpsert` without new `CiJob`; any forbidden grep hit.

---

## §4 Falsification table (acceptance = all rows PASS)

| ID | Probe | Action | Pass criterion |
| -- | ----- | ------ | -------------- |
| **F1** | Shell gone | `git ls-files 'scripts/v4-testclaim-corpus-gate.sh'` + §3 grep on `main` after merge | File absent; zero uncited grep hits |
| **F2** | Modeled sole structural authority | Change `ci_upsert_testclaim_corpus_eval_execution` `verify` projection (e.g. add `TestClaimRef` input) — re-emit workflow | Projected CI step/cache binding updates without hand-editing `ci.yml` bridge block; no parallel shell step |
| **F3** | Structural obligations preserved | Run implementation host transport / smoke on PR touching `src/v4/test/claim/manual/**` | Manual `*.dag` modules compile-closed; `TestClaimRun` data rows present in artifact registry; rust MVP witness equivalent to gate.sh receipt (until T-38 runtime replaces witness) |
| **F4** | IRT-1 selection | Attempt roster bypass in worker branch (review gate) | `ci_command_authority_ok` rejects `selection_fn != ci_testclaim_corpus_selection_fn`; affected-set narrowing unchanged |
| **F5** | Forbidden register | §3 grep on implementation diff | Zero hits except allowed citations |
| **F6** | #4091 reuse | Inspect execution row `inputs` + one CI run with v4 affected | No redundant full-tree `gunbc compile --target rust` when M1 receipt clean (upstream hit or skip); wall-clock secondary only |

**P5 Layer 2 PROVEN (predicate):** F1 + F2 + F3 + F5 **PASS**. F4 + F6 **PASS** before calling “first #4091 replacement complete.”

**Not required for P5 Layer 2:** per-row Pass/Fail/Deferred verdict emission in CI (`execution_status=blocked_m1_subset` may remain until T-38-PR2).

---

## §5 Landing order (implementation — not worksheet PR)

```text
1. Modeling DFS §8 sign-off on this worksheet (P5-BRIDGE-WS).
2. Compiler Spine — amend ci_upsert_testclaim_corpus_eval_* inputs (UpstreamUpsert to M1; bootstrap edge
   per §1.2); migrate cache_digest off static tag where projection helpers exist on main.
3. Compiler Spine — project ci_github_actions_workflow.dag + regenerate ci.yml from modeled authority;
   delete scripts/v4-testclaim-corpus-gate.sh; remove hashFiles cache step.
4. Runtime/TestClaim — host transport for TestClaimCorpusEvalCommand (evolve v4-testclaim-corpus-eval.sh
   or successor): invoke modeled path only; emit structural JSON receipt for CI.
5. Integration — extend v4_workflow_ci_runner_dag_smoke_test.rs (binding parity, grep negatives).
6. Close/Receipt — optional ci_v4 wall-clock note (secondary).
```

**Lane split:** Compiler Spine owns 2–3 + smoke 5; Runtime/TestClaim owns 4; Modeling DFS owns §8 only on worksheet PR.

---

## §6 Downstream worker brief (dispatch after §8)

```text
Implement P5 structural-bridge replacement per approved worksheet.

MUST:
  - Delete scripts/v4-testclaim-corpus-gate.sh in same PR that wires positive-Y authority.
  - Project live workflow from ci_upsert_testclaim_corpus_eval_execution + signal gate (no bridge RunStep).
  - Preserve selection_fn == ci_testclaim_corpus_selection_fn (IRT-1).
  - Pass falsification F1–F6 (§4); cite worksheet in PR body step-id → CiUpsertStep table.
  - Compose #4095 only at edges (§2) — no duplicated type catalogs.

MUST NOT:
  - Any §3 forbidden pattern (uncited).
  - Land worksheet-only scope in implementation PR (no retroactive ci.dag edits in worksheet PR).
  - Claim P5 GREEN / full T-38 CI verdict execution without Close/Receipt transcript.
  - Touch INVARIANTS.md / THESIS.md / MODELING.md unless escalated.

Escalate to Modeling DFS:
  - New UpsertInputRef variant required.
  - Bootstrap cannot be modeled as UpstreamUpsert without new CiJob authority.
  - Structural obligations cannot be met without reintroducing shell compile loops.
```

---

## §7 Non-goals

- Full T-38 modeled runner **runtime** execution in CI (M1 cargo-clean subset — `scripts/v4-testclaim-corpus-eval.sh` posture)
- Worksheet A/B **implementation** (compute_fabric / cache_interface `.dag` landings) — consume at edges only
- W2.3 bucket re-litigation (bijection already on main)
- P2 bootstrap resolve-posture bridge deletion
- SG-class emitter worksheets (SG-RC-LAYERING, SG-1b, …)
- `INVARIANTS.md` / `THESIS.md` / `MODELING.md` edits in worksheet PR

---

## §8 Manager approval checklist (`proud-pike-680`) — CLOSED 2026-05-31

- [x] Single-authority fact: positive-Y `ci_upsert_testclaim_corpus_eval_*` replaces shell bridge (§1)
- [x] Distinct from #4095 substrate worksheets (composition edges only — §2)
- [x] Distinct from full T-38 runtime verdict CI (P5 Layer 2 structural only — §7)
- [x] Spot-fix forbidden: shell retention, hashFiles cache, host-owned compile, selection_fn bypass (§3)
- [x] Parser gates §2.1 named; worksheet PR scope excludes implementation gates
- [x] Falsification table §4 (F1–F6) accepted as worker acceptance
- [x] Forbidden register §3 + grep literals accepted
- [x] Landing order §5 + lane split accepted
- [x] **READY-FOR-WORKER-DISPATCH**

---

## Related artifacts

- `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` — §1.2 four-compile redundancy; §3 Upsert state (#4091)
- `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` — §3.5 P5 two-layer gate
- `docs/planning/v4-ci-schema-worksheet-2026-05-30.md` — CiUpsertStep / UpsertInputRef §8 CLOSED
- `docs/planning/v4-w2.3-ci-upsert-step-migration-worksheet-2026-05-30.md` — existing row bijection
- `docs/planning/v4-elastic-compute-fabric-worksheet-2026-05-30.md` — Worksheet A composition
- `docs/planning/v4-elastic-cache-interface-worksheet-2026-05-30.md` — Worksheet B composition
- `src/v4/workflow/ci.dag` — T-38 marks; `testclaim_corpus_eval_*` symbols
- `scripts/v4-testclaim-corpus-gate.sh` — negative authority target
- `dsl/gunbc/ci_github_actions_workflow.dag` — live bridge projection (~L731–749)
- `src/v3/compiler/tests/integration/v4_workflow_ci_runner_dag_smoke_test.rs` — binding parity tests
- `docs/planning/v4-correctness-ladder-2026-05-30.md` — §10.0 template
