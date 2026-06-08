# Thin-Shim `ci.dag` Dissolution — Design Draft

> **Status:** DESIGN DRAFT (proud-deer-476 / node://adhoc-d2b4d72d-7c9)  
> **Scope:** `src/v4/workflow/ci.dag` (6337 lines today) → modular one-job shims aligned with BinaryShim transport  
> **Authority anchor:** ctrl#1490 dep-graph (external; not present in this worktree — see §0)  
> **Hard rail:** DO NOT cut load-bearing `ci.dag` surfaces without Mgr-C gate / escalation

---

## §0 — ctrl#1490 alignment gap

The parent brief names **ctrl#1490 dep-graph** as the single plan. That document is **not checked into gunbc** (searched worktree + sibling worktrees; `gunbc-planning/` snapshots here lack a `*1490*` artifact). This draft is derived from load-bearing in-repo authorities:

| Source | Role |
|--------|------|
| `src/v3/SELF_HOSTING.md` | Names `bootstrap.dag` + `ci.dag` as load-bearing v4 workflow authorities |
| `docs/design-pure-bootstrap-zero.md` | C4: `ci.yml` is a checked projection; bootstrap-modeled authority only |
| `dsl/gunbc/ci_emission.dag` | Ratified **BinaryShim** thin-shim: one GHA job → `gunbc-ci` runner |
| `dsl/gunbc/ci.dag` | Slim gate-topology precedent (188 lines; `CIWorkflowDag` authority) |
| `src/v4/workflow/ci.dag` | Live v4 CI model (6337 lines; sole `v4.workflow.ci` module) |

**Action for Mgr-C:** confirm or paste ctrl#1490 row map before any Phase ≥2 dispatch. Until then, treat this doc as a **scoping probe**, not a ratified cut list.

---

## §1 — Problem statement

`src/v4/workflow/ci.dag` is load-bearing (facet-4 CI-as-data, lens self-application) but has grown into a monolith:

| Metric | Value |
|--------|-------|
| Lines | 6337 |
| `fn` / `data` symbols | ~704 |
| Direct claim consumers | 6 modules under `src/v4/test/claim/workflow/` |
| `CiCommand` variants | 14 (closed coproduct at L250–271) |
| `ci_pipeline` jobs | 11 execution rows + 8 signal gates |

The file mixes **durable authority** (pipeline membership, well-formedness, affected-set selection) with **interim bridges** (live-workflow step signals, shadow receipts, per-step upsert ledgers) and **receipt witnesses** (SG-7 char projection, runner pool policy). Progress (INVARIANTS P5) requires a dissolution path that **reduces ad-hoc state** without splitting authority.

**Target shape:** mirror `dsl/gunbc/ci_emission.dag` — **one transport job** (`BinaryShim`) whose runtime dispatches the modeled gate matrix — while **factoring source** into one-job shim modules that each own a single `CiJob` execution concern.

---

## §2 — Line-budget autopsy (current monolith)

Approximate regions (line numbers drift; re-measure before cut):

| Lines | ~Count | Bucket | Dissolution posture |
|-------|--------|--------|---------------------|
| L1–320 | 320 | Core types, imports, live-workflow bridge carriers | **KEEP in core** until YamlStatic emission (T-24 / gates #98/#100) |
| L321–490 | 170 | UPSERT substrate types, carveouts | **KEEP in core** (shared `CiUpsertStep<T>` algebra) |
| L491–1086 | 596 | Wave-2 shell exception table, job `*_mk` rows, symbols | **SPLIT:** per-job shims own `*_mk` + signals; core keeps `ci_pipeline` assembly |
| L1087–1162 | 76 | `data ci_pipeline` | **LOAD-BEARING — never cut**; stays in `ci.dag` core |
| L1163–2200 | 1038 | `CiUpsertStep` row registry (Buckets A–E + Tier-0) | **MOVE** to per-job shims (largest win) |
| L2201–3470 | 1270 | Well-formedness, projections, shadow step IDs | **KEEP validators in core**; projection helpers co-locate with job shims where job-specific |
| L3471–4636 | 1166 | Shadow selection, cache digests, carveout dispatch | **MOVE** to `ci_selection.dag` submodule |
| L4637–5677 | 1041 | Wave-3 `CiSelectionReceipt` claim projection | **MOVE** to `ci_selection.dag` |
| L5678–5893 | 216 | TestClaim shadow roster / frontier filter | **MOVE** with selection submodule |
| L5894–6337 | 444 | Affected-set job selection, runner pool, SG-7 witnesses | **SPLIT:** selection vs `ci_runner.dag` |

**Interim bridge density:** 11+ `🟡 gated` marks with `feature:project-github-actions-landed` / `wave3-shadow-selection-receipt` — these are **scheduled deletions** when YamlStatic projection lands, not candidates for premature cutting.

---

## §3 — Load-bearing surface (Mgr-C gate required to cut)

Per `SELF_HOSTING.md` + `design-pure-bootstrap-zero.md`, the following **must remain reachable as a single logical authority** (`v4.workflow.ci` re-export surface):

### 3.1 Non-negotiable carriers

1. **`CiCommand` coproduct** — sole command taxonomy (LB-P4-3213). Adding variants = one file edit in the owning job shim + one arm in core.
2. **`data ci_pipeline: Outcome<CiPipeline>`** — job/gate membership authority. The `jobs` list is the roster; gates carry `CiGateRunPolicy`.
3. **`ci_pipeline_well_formed`** — eager fail-closed validator (acyclicity, needs resolution, command authority).
4. **`ci_select_ci_jobs_from_affected_set`** + **`CiComponentAffected`** — T-21/T-24 affected-set authority (replaces `detect-affected-components.sh`).
5. **`TestClaimCorpusEvalCommand` binding** — T-38 structural bridge (`ci_upsert_testclaim_corpus_eval_*`, selection_fn pin).
6. **Claim import surface** — symbols consumed by:
   - `ci_component_affected.dag`
   - `affected_set_ci_runner.dag`
   - `runner_pool_m1_probe.dag`
   - `ci_consumer_node_precise.dag`
   - `pipeline_rejections.dag`
   - `recursive_flex_inspection.dag`

### 3.2 Escalate before cutting

| Surface | Why load-bearing | Dissolution trigger |
|---------|------------------|---------------------|
| `ci_pipeline` job list | Single membership authority | Never split across competing `data` rows |
| `CiSelectionReceipt` partition | Drives GHA `if:` scheduling (Wave 4) | `feature:ci-selection-receipt-active-skip` lands |
| `CiLiveWorkflowStepSignal` / `M1CiLiveWorkflowSignal` | Hand-synced ci.yml bridge | `project_github_actions_landed` (gates #98/#100) |
| Shadow fixtures (`ci_selection_receipt_shadow_*`) | Compiler-spine CI receipts | Live PR git_diff routing (node://adhoc-331899f9-19a) |
| SG-7 char projection witnesses | T-22 cache-hash consumer pin | Substrate closed-sum equality (W-T-10) |

### 3.3 Safe to relocate (no authority split)

- Per-job `CiUpsertStep` row `data` blocks (inputs/verify/create/resolve triples).
- Per-job `ci_job_*_execution_mk` factories + `CiLiveWorkflowStepSignal` ledger rows.
- Runner pool / isolation policy tables (`ci_self_hosted_runner_pools`, `ci_runner_isolation_policy`).
- Shadow receipt **fixtures** (not live selection entry points).

---

## §4 — Target module topology (one-job shim per `CiJob`)

### 4.1 Module graph

```
v4.workflow.ci                    ← core (types, ci_pipeline, well-formedness, re-exports)
├── v4.workflow.ci.runner       ← fleet / isolation / resilience policy
├── v4.workflow.ci.selection    ← affected-set, CiSelectionReceipt, shadow receipts
└── v4.workflow.ci.jobs.*       ← one submodule per execution job id
    ├── v2_bootstrap_smoke
    ├── v2_compile_src_v4       (inline today; extract)
    ├── v3_determinism
    ├── v3_self_host_fixed_point
    ├── lens_ci_registry
    ├── v4_t15_self_host_fixed_point
    ├── m1_rust_emit_probe
    ├── leaf_model_go_verify
    ├── testclaim_corpus_eval
    ├── lens_ownership_family_eval
    ├── source_authority_receipt_eval
    └── phase1_nat_semiring_rung_gate
```

**Module naming:** `src/v4/workflow/ci/jobs/<job_id>.dag` → `module v4.workflow.ci.jobs.<job_id>`.

**Aggregation rule:** `ci.dag` core **imports** each job shim and **assembles** `ci_pipeline.jobs` from exported `*_execution_row` symbols. No job shim may declare its own `ci_pipeline`.

### 4.2 Per-job shim contract (one job = one file)

Each `jobs/<id>.dag` exports exactly:

| Export | Purpose |
|--------|---------|
| `<id>_execution_row: CiJob` | Pipeline membership row |
| `<id>_upsert_row: CiUpsertStepSymbol` (if in shadow universe) | Upsert ledger slice |
| `<id>_live_signal: CiLiveWorkflowStepSignal` (if gated bridge active) | ci.yml hand-sync bridge |
| `ci_job_<id>_projection_node` (optional) | Job-local projection helpers |

Imports: `v4.workflow.ci` types only (core), plus domain modules the command needs (e.g. `v4.lens.registry` for `LensCiCommand`).

**Forbidden in job shims:** affected-set logic, selection receipts, pipeline well-formedness, cross-job `needs` validation.

### 4.3 Transport shim (BinaryShim — already ratified)

`dsl/gunbc/ci_emission.dag` defines the GHA projection:

```dag
// jobs: single "ci" job → ./gunbc-ci --workflow ci --event "$GITHUB_EVENT_PATH"
```

`src/v3/compiler/src/bin/gunbc_ci.rs` today: dispatch **stub** (exit 2 unless `GUNBC_CI_ALLOW_DISPATCH_STUB=1`). Thin-shim dissolution **depends on** gunbc-ci reading `ci_pipeline` + `ci_select_ci_jobs_from_affected_set` at runtime — **out of scope for file splits**; tracked as T-WAD Slice 7 / BinaryShim gate-matrix wiring.

**Two-layer shim model:**

| Layer | Shim | Lines target |
|-------|------|--------------|
| **Transport** | `ci_emission.dag` BinaryShim (1 GHA job) | ~100 (done) |
| **Source** | `ci.dag` → core + N job modules | core ~800; each job ~80–200 |

---

## §5 — `ci_pipeline` job roster → shim map

| Job id | `CiCommand` variant | Est. shim lines (upsert+signal) | Mgr-C notes |
|--------|---------------------|----------------------------------|-------------|
| `v2_bootstrap_smoke_execution` | `V2BootstrapCompileCommand` | ~120 | Bankruptcy Tier-0 |
| `v2_compile_src_v4` | `BootstrapStageCompile` | ~80 | Upstream for most jobs |
| `v3_determinism_execution` | `V3DeterminismCommand` | ~100 | Schedule: PR + main |
| `v3_self_host_fixed_point_execution` | `V3SelfHostFixedPointCommand` | ~150 | Tier-0 GHA-if bridge |
| `lens_ci_registry_execution` | `LensCiCommand` | ~200 | Lens-CI gate (M2 discriminating) |
| `v4_t15_self_host_fixed_point_execution` | `V4T15SelfHostFixedPointCommand` | ~100 | |
| `m1_rust_emit_probe_execution` | `M1RustEmitProbeCommand` | ~120 | M1 probe policy |
| `leaf_model_go_verify_execution` | `LeafModelGoVerifyCommand` | ~80 | |
| `testclaim_corpus_eval_execution` | `TestClaimCorpusEvalCommand` | ~250 | **T-38 load-bearing** |
| `lens_ownership_family_eval_execution` | `LensOwnershipFamilyEvalCommand` | ~150 | Depends on corpus eval |
| `source_authority_receipt_eval_execution` | `SourceAuthorityReceiptEvalCommand` | ~100 | Branch H receipt |
| `phase1_nat_semiring_rung_gate_execution` | `Phase1NatSemiringRungGateCommand` | ~80 | Class-C demoted shell |

**ci.yml today:** multi-job (fmt, affected, ci_floor, …) — **hand-synced** transport. Dissolution does **not** delete ci.yml jobs until BinaryShim dispatch is green **and** Mgr-C signs C4 projection flip.

---

## §6 — Phased rollout (no load-bearing cuts in Phase 0–1)

| Phase | Action | Consumer gate | Risk |
|-------|--------|---------------|------|
| **0** | This design draft + ctrl#1490 alignment | Mgr-C review | — |
| **1a** | Extract `ci_runner.dag` (~150 lines) | `ci_runner_isolation_policy_ok` claims + ci.yml `infra_isolation` comment anchor | Low |
| **1b** | Extract `ci_selection.dag` (~2800 lines) | 6 workflow claims + `affected_set_ci_runner.dag` | Medium — import surface via re-export |
| **2** | Land first job shim (`leaf_model_go_verify` — smallest) | `ci_component_affected` paths unchanged | Low — pattern proof |
| **3** | Migrate remaining job shims (batch 3–4 per PR) | Per-job claim coverage | Medium |
| **4** | Delete relocated upsert rows from core | `ci_upsert_steps_slice_bijection_ok` | Medium — bijection receipts |
| **5** | Wire `gunbc-ci` dispatch (BinaryShim runtime) | Executed dispatch, not stub | **High — Mgr-C** |
| **6** | YamlStatic emission; delete live-workflow bridges | gates #98/#100 | **High — Mgr-C** |

**STOP rules (escalate, do not improvise):**

- Any PR that **reduces** `ci_pipeline` job count without a named dissolution mark.
- Any PR that **deletes** `ci_pipeline_well_formed` checks or bypasses `Outcome` fail-closed.
- Any PR that edits `.github/workflows/ci.yml` for facts owned by `ci.dag` outside T-24/T-38 named targets.
- Any PR that splits `TestClaimCorpusEvalCommand` authority across modules without core re-export.

---

## §7 — Import / re-export strategy

Claim modules import `v4.workflow.ci { … }` today. After split:

```dag
// src/v4/workflow/ci.dag (core tail)
import v4.workflow.ci.selection { … re-exported symbols … }
import v4.workflow.ci.runner { … }
import v4.workflow.ci.jobs.testclaim_corpus_eval { ci_job_testclaim_corpus_eval_execution_row }
// …
```

**Cost-of-change goal:** adding a new `CiCommand` variant = **1 core arm** + **1 new `jobs/<id>.dag`** + **1 `ci_pipeline` row** (3 files), matching the repo's "one type → one file" ratchet.

---

## §8 — Relationship to `dsl/gunbc/ci.dag`

The v3-era `dsl/gunbc/ci.dag` (188 lines) remains the **compiler-intent gate DAG** for `gunbc-ci` until v4 dispatch reads `v4.workflow.ci` directly. Do **not** conflate:

| File | Authority |
|------|-----------|
| `dsl/gunbc/ci.dag` | `CIWorkflowDag` for BinaryShim runner (lint/tests/ratchets) |
| `src/v4/workflow/ci.dag` | v4 CI pipeline (bootstrap, lenses, TestClaim, affected-set) |

**Convergence path (T-24):** `v4.workflow.ci` emits both ci.yml **and** feeds `gunbc-ci`; `dsl/gunbc/ci.dag` dissolves when v4 `ci_pipeline` subsumes its gate rows. Until then, dual models are **explicit bridges**, not drift.

---

## §9 — Verification plan

| Check | Command / claim |
|-------|-----------------|
| Compile surface | `cargo test -p v3-compiler` (v4 tree compile) |
| Workflow claims | `src/v4/test/claim/workflow/ci_component_affected.dag` + `affected_set_ci_runner.dag` |
| Pipeline well-formed | `ci_selection_receipt_shadow_well_formed_ok` data pins |
| No authority split | Grep: exactly one `data ci_pipeline` in tree |
| Line budget ratchet | `wc -l src/v4/workflow/ci.dag` ≤ 900 after Phase 4 (core only) |

---

## §10 — Open questions for Mgr-C / ctrl#1490

1. **ctrl#1490 row map** — confirm job-shim ordering vs lens-CI / get-off-v3 / affected-set-3a cluster deps.
2. **Selection submodule timing** — Phase 1b before job shims, or parallel under separate workers?
3. **ci.yml flip** — does BinaryShim replace multi-job ci.yml in one PR or staged behind `GUNBC_CI_ALLOW_DISPATCH_STUB` removal?
4. **SG-7 witnesses** — stay in core or move to `src/v4/test/claim/workflow/` as claims-only?

---

## §11 — Summary

- **6337-line monolith** decomposes into **~800-line core** + **~2800-line selection** + **~150-line runner** + **12 job shims (~80–250 lines each)**.
- **Load-bearing cuts** (`ci_pipeline`, affected-set, T-38 corpus eval) require **Mgr-C gate**; Phases 0–4 are **relocations only**.
- **One-job shim** means **one `CiJob` execution concern per `.dag` file**, not deleting CI jobs from the pipeline.
- **BinaryShim transport** (one GHA job) is already ratified; source dissolution is independent of runtime dispatch wiring.
- **ctrl#1490** must be pasted or linked before ratifying Phase ≥2 dispatch.
