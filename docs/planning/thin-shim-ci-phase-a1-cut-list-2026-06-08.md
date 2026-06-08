# Thin-Shim CI — Phase A1 Mgr-C Cut List

**Status:** Mgr-C gate artifact (analysis only — no `ci.dag` / `ci.yml` / ratchet / `ci_affected_components` edits)  
**Parent design:** [#4527](https://github.com/gunb-ai/gunbc/pull/4527) merged @ `5617c790` → `docs/planning/thin-shim-ci-dag-dissolution-design-2026-06-08.md`  
**Plan authority:** ctrl#1490 (single plan; no forked view)  
**Producer:** warm-ibex-571 (Phase A1)

---

## 1. Executive summary

| Surface | Today | After Phase A2 (thin YAML) | After Phase A4 (model slim) |
|---------|-------|---------------------------|----------------------------|
| `.github/workflows/ci.yml` | 556 lines, 8 jobs, 51 named steps | ~60–90 lines: envelope + `infra_isolation` + one `gunbc-ci` job (+ optional aggregator) | Emitted from model (Phase B) |
| `src/v4/workflow/ci.dag` | 6,337 lines | Unchanged in A2 | Delete ~150–250 lines of YAML-mirror bridge rows (A4) |
| `gunbc-ci` | Stub dispatch (exit 2) | Invoked from YAML | Run-all (A3) then selection (A5) |

**Classification rule (from parent design §2):**

- **GitHub-only** — only GitHub Actions can do it, or must remain in YAML until Shape-B emission (checkout, toolchain install, coarse `actions/cache`, secrets, `needs`/`outputs` graph, `needs.*.result` aggregator, draft/`if:` expressions).
- **Runner-executable** — `gunbc-ci` can own after A3: discover diff, select jobs, per-operation cache, run commands/scripts/cargo, exit code.

---

## 2. Live `ci.yml` inventory (51 steps across 8 jobs)

### 2.1 Workflow envelope (GitHub-only — keep in YAML)

| Item | Lines (approx) | Notes |
|------|----------------|-------|
| `name`, `on` (push + pull_request types) | 1–13 | Trigger surface |
| `permissions` | 14–16 | GHA token policy |
| `concurrency` | 17–19 | Cancel-in-progress |
| Workflow `env` (`CARGO_TERM_COLOR`, `RUSTFLAGS`, `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`) | 20–29 | Uniform action runtime |
| Per-job `if: github.event.pull_request.draft != true` | multiple | Draft gate (6 jobs) |
| Per-job `runs-on: [self-hosted, linux, arm64]` | all jobs | Runner label (until fleet modeled in emission) |
| Per-job `timeout-minutes` | all jobs | GHA job timeout |
| Job `outputs` on `affected` | 151–158 | `needs.affected.outputs.*` wiring for future selection |
| Job `needs` graph on `ci` aggregator | 521 | Branch-protection aggregation |

### 2.2 Per-step cut list

Legend: **G** = GitHub-only (stays in thin YAML) · **R** = runner-executable (moves to `gunbc-ci`) · **G→R** = interim GHA action step, runner owns cache key semantics later

| Job | Step | Class | Rationale |
|-----|------|-------|-----------|
| **infra_isolation** | Assert runner is de-privileged (adversarial) | **G** (job) + **R** (probe body) | Design A2: **keep separate job**; no checkout before probe. Shell probe can move to runner *inside* this job, but job must remain a distinct required check (`ci_runner_isolation_policy.guard_job_name`). |
| **fmt** | Isolate toolchain dirs | **R** | Shared-runner `CARGO_HOME`/`RUSTUP_HOME` + sccache probe — runner prep |
| | Clear inherited GitHub auth header | **R** | Host hygiene |
| | Checkout | **G** | `actions/checkout@v5` |
| | Setup Rust | **G** | `actions-rust-lang/setup-rust-toolchain@v1.16.0` |
| | Pin global rustup default | **R** | Isolated `RUSTUP_HOME` workaround |
| | cargo fmt --all --check | **R** | Gate command (`FmtCheck` floor) |
| **doc_refs** | Clear inherited GitHub auth header | **R** | Host hygiene |
| | Checkout (`fetch-depth: 0`) | **G** | `actions/checkout@v5` |
| | Doc references resolve (touched docs) | **R** | `python3 scripts/check_doc_refs.py --changed` |
| | Doc-chain diagram is current | **R** | `python3 scripts/check_doc_refs.py --check-graph` |
| **affected** | Isolate toolchain dirs | **R** | Duplicate boilerplate → runner runs once |
| | Clear inherited GitHub auth header | **R** | Duplicate |
| | Checkout (`fetch-depth: 0`) | **G** | |
| | Setup Rust | **G** | |
| | Pin global rustup default | **R** | |
| | Fetch main for diff base | **R** | `git fetch` when `pull_request` |
| | Detect affected components | **R** | `cargo run -p ci_affected_components --bin detect-ci-affected-components` |
| | Emit Wave 3 shadow selection receipt | **R** | Class C instrumentation; `continue-on-error: true` is **G** policy |
| | Emit affected-set CI kill-criterion receipt | **R** | Instrumentation only |
| | Upload affected-set CI receipt | **G** | `actions/upload-artifact@v6` |
| **ci_floor** | Isolate toolchain dirs (+ `CARGO_BUILD_JOBS=2`) | **R** | |
| | Clear inherited GitHub auth header | **R** | |
| | Checkout | **G** | |
| | Setup Rust (+ rustfmt component) | **G** | |
| | Pin global rustup default | **R** | |
| | Cache Cargo (ci floor) | **G→R** | Interim: `actions/cache@v5` in YAML per design §2; keys/projections belong in model (`cache_interface.dag` consumer) |
| | Cache gunbc binary (v4 floor) | **G→R** | Same |
| | Build v2 compiler (v4 floor) | **R** | `cargo build -p v2-compiler --release` |
| | v2 stage0 freshness receipt | **R** | `regen_stage0 --verify` |
| | v2 DAG emit parity receipt (shared closure) | **R** | `cargo test -p v2-compiler-tests --release pipeline::dag_emit…` |
| **ci_floor_emit** | Isolate toolchain dirs | **R** | Duplicate ×3 jobs |
| | Clear inherited GitHub auth header | **R** | |
| | Checkout | **G** | |
| | Setup Rust | **G** | |
| | Pin global rustup default | **R** | |
| | Cache Cargo / Cache gunbc binary | **G→R** | |
| | Build v2 compiler | **R** | |
| | M1 v4 full-tree rust emit probe (v2 emit) | **R** | `.github/ci-floor/v4-m1-rust-emit-probe.sh` — modeled `M1RustEmitProbeCommand` |
| **v4_lens_ci** | Isolate toolchain dirs | **R** | |
| | Clear inherited GitHub auth header | **R** | |
| | Checkout | **G** | |
| | Setup Rust | **G** | |
| | Pin global rustup default | **R** | |
| | Cache Cargo / Cache gunbc binary | **G→R** | |
| | Build v2 compiler | **R** | |
| | v4 lens analysis must-pass discriminating witnesses | **R** | `scripts/v4-lens-ci-gate.sh --perturb-check` — modeled `LensCiCommand` + `lens_ci_claim_run_rows` |
| **ci** | Validate prerequisites (fail-closed under skipped/failed deps) | **G** | Uses `needs.*.result` — not expressible in runner |
| | ci receipt (section 11.7.1 floor) | **G** | Notice-only; optional to drop when single-job runner reports |

### 2.3 Duplicate boilerplate to delete in A2 (runner runs once)

| Step name | Occurrences | Jobs |
|-----------|-------------|------|
| Isolate toolchain dirs (…) | 5 | fmt, affected, ci_floor, ci_floor_emit, v4_lens_ci |
| Clear inherited GitHub auth header | 6 | fmt, doc_refs, affected, ci_floor, ci_floor_emit, v4_lens_ci |
| Checkout | 6 | all except infra_isolation, ci |
| Setup Rust | 5 | fmt, affected, ci_floor, ci_floor_emit, v4_lens_ci |
| Pin global rustup default | 5 | same |
| Cache Cargo + Cache gunbc binary + Build v2 | 3 | ci_floor, ci_floor_emit, v4_lens_ci |

**A2 thin-YAML target shape** (matches `dsl/gunbc/ci_emission.dag` BinaryShim + design §3):

```yaml
jobs:
  infra_isolation: { ... unchanged ... }
  ci:
    steps:
      - Checkout          # G
      - Setup Rust        # G (coarse toolchain)
      - Invoke gunbc-ci   # R: ./gunbc-ci --workflow ci --event "$GITHUB_EVENT_PATH"
```

Optional: retain `ci` aggregator job as **G** shell until branch-protection job IDs are reconciled (see §5).

---

## 3. `ci.dag` mirror rows vs authority to keep

### 3.1 DELETE in Phase A4 (duplicate YAML mirror — manager-approved cuts)

These rows exist only to bind hand-authored `ci.yml` step names / shell paths / raw `if:` strings. Authority already lives in `ci_pipeline` + `CiCommand` + `CiUpsertStep` + `lens_ci_claim_run_rows`.

| Row / block | Lines (approx) | Mirror of | Drift? |
|-------------|----------------|-----------|--------|
| `lens_ci_live_workflow_signal` | 937–941 | Step names in YAML | **YES** — cites `"Lens-CI registry activation smoke"` / `"Lens-CI registry semantic v4 compile (rust target)"`; live YAML step is `"v4 lens analysis must-pass discriminating witnesses"` |
| `m1_ci_live_workflow_signal` | 943–955 | `ci_floor_emit` M1 step | Aligned today |
| `testclaim_corpus_eval_ci_live_workflow_signal` | 957–963 | Bankruptcy Tier-0 step (not in Wave-1 YAML) | N/A — pre-projection placeholder |
| `ci_v3_self_host_fixed_point_ci_live_workflow_binding` | 978–981 | Tier-0 `if:` for v3 self-host | Not in live Wave-1 `ci.yml` |
| `ci_v4_bootstrap_gate_result_skip_guard_if` | 983 | Raw GHA `if:` string | Not in live Wave-1 `ci.yml` (consumed by `ci_affected_components` receipt logic) |
| `m1_rust_dag_emit_parity_receipt_test` | 967 | Inline cargo test command string | Duplicates `ci_floor` step body |
| Bridge types `LensCiLiveWorkflowSignal`, `CiLiveWorkflowStepSignal`, `M1CiLiveWorkflowSignal`, `CiGhaWorkflowStepBinding`, `StrictEnvBinding`, `CiWorkflowHostScript` | 696–732, 973–976 | YAML projection interim | Delete with rows above when Shape-B emits |

**Estimated A4 deletion:** ~150–250 lines (types + data + gated comments), not the 6,337-line file.

### 3.2 KEEP (structural authority — do not cut in Phase A)

| Block | Lines (approx) | Why |
|-------|----------------|-----|
| `CiCommand` coproduct + `ci_command_*` projections | 250–271, 2275+ | Command semantics |
| `ci_pipeline` jobs + gates | 1087–1160 | Pipeline topology |
| `ci_upsert_*` rows (Buckets A–E) | 1163–2100+ | Per-operation cache keys / inputs |
| `ci_select_*` / `CiComponentAffected` / selection receipts | scattered | Affected-set + T-24 selection |
| `ci_class_a_shell_exceptions` | 540–553 | Shell→Upsert retirement registry (ratcheted) |
| `ci_runner_isolation_policy` | 838–860 | Policy facts (job id + sentinel paths); slim string mirrors only at emission |
| `lens_ci_claim_run_rows` | 904–933 | M2 discriminating witness roster (authority for `v4_lens_ci`) |
| `ci_runner_pool_*` / resilience playbook | 758–816, 6262+ | Fleet facts (not YAML duplicates) |
| Well-formedness / shadow bijection / TestClaim bindings | 2000–5600+ | Compiler-spine CI claims |

### 3.3 DEFER (not mirror deletion — separate lanes)

| Item | Reason |
|------|--------|
| Whole `ci.dag` deletion | Explicitly out of scope per parent brief |
| `dsl/gunbc/ci.dag` (188 lines) | Legacy gate topology; T-24 defers to `src/v4/workflow/ci.dag` |
| `tools/ci_workflow_ratchet/*` string ratchets | Dissolve with A15 Shape-B / TestClaim execution — not A1 |

---

## 4. Mapping: live YAML jobs ↔ `ci_pipeline` jobs

Wave-1 `ci.yml` and bankruptcy `ci_pipeline` are **intentionally divergent** today: YAML runs the §11.7.1 safety floor; `ci_pipeline` models Tier-0 bankruptcy jobs not yet wired to live YAML.

| Live `ci.yml` job | `ci_pipeline` / model anchor | Wired? |
|-------------------|------------------------------|--------|
| `infra_isolation` | `ci_runner_isolation_policy` | Policy only (not a `CiJob` in pipeline) |
| `fmt` | `FmtCheck` via `CiSafetyFloorItem` | Shell only |
| `doc_refs` | — | YAML-only floor |
| `affected` | `CiComponentAffected` + `ci_select_*` | `ci_affected_components` binary |
| `ci_floor` | `V2BootstrapCompileCommand`, stage0, DAG parity upserts | Partial |
| `ci_floor_emit` | `M1RustEmitProbeCommand` | Via shell exception row |
| `v4_lens_ci` | `LensCiCommand` + `lens_ci_claim_run_rows` | Live gate (M2) |
| `ci` | Aggregator (not in pipeline) | GHA-only |
| *(not in YAML)* | `v2_bootstrap_smoke`, `v3_determinism`, `v3_self_host_fixed_point`, `v4_t15`, `testclaim_corpus_eval`, `lens_ownership_family_eval`, `source_authority_receipt_eval`, `phase1_nat_semiring_rung_gate`, `leaf_model_go_verify` | Bankruptcy Tier-0 — runner run-all (A3) before YAML wiring |

---

## 5. Required before checks (branch protection)

**Live aggregator** (`ci` job `needs`, line 521):

`affected`, `ci_floor`, `ci_floor_emit`, `v4_lens_ci`, `infra_isolation`, `doc_refs`

**Also required per workflow comments but NOT in aggregator `needs`:**

- `fmt` (parallel job; branch protection likely lists it separately)

**Ratchet drift note:** `tools/ci_workflow_ratchet/tests/v4_workflow_ci_runner_dag_smoke_test.rs` still asserts `needs: […, ci_corpus_eval]` in places — live YAML uses `v4_lens_ci` instead. **Do not fix in A1** (ratchet edit out of scope); flag for Mgr-C before A2.

### Before checks to run before any A2/A3 implementation PR

```bash
cargo test -p ci_workflow_ratchet
cargo test -p ci_affected_components
cargo test --workspace   # full gate if operator requests
```

Plus: confirm GitHub required-check list matches live job IDs above (operator / branch-protection settings — not in repo).

---

## 6. Stop / escalation points

| # | Condition | Action |
|---|-----------|--------|
| E1 | Branch protection requires **8 distinct job names** but A2 collapses to 2 jobs | **Escalate to operator/Mgr-C:** either (a) keep thin wrapper jobs that delegate to `gunbc-ci` with distinct `needs`, or (b) update branch-protection required checks before merge. |
| E2 | Implement A2 before A3 run-all | **STOP** — stub `gunbc-ci` exits 2; thinning YAML without runner breaks CI. Order: approve cut list → A3 dispatch → A2 thin YAML (or land together). |
| E3 | Delete `ci_pipeline` jobs or selection functions | **STOP** — violates design §5; escalate. |
| E4 | Edit `ci.dag` load-bearing pipeline stages / gates under this brief | **STOP** — escalate per `SELF_HOSTING.md` / INVARIANTS. |
| E5 | `lens_ci_live_workflow_signal` stale names | Safe A4 delete only after `v4_lens_ci` transport reads `LensCiCommand` / `lens_ci_claim_run_rows` directly (post-A3). |
| E6 | `actions/cache` key strings | Interim YAML transport per `cache_interface.dag` note (ci.dag ~862–885); do not hand-cut cache keys in A2 without cache projection consumer. |
| E7 | Smoke ratchet expects `ci_corpus_eval` | Reconcile ratchet vs live YAML in implementation PR (not A1). |

---

## 7. Suggested Mgr-C approval checklist

- [ ] Approve §2 step classification (G vs R vs G→R)
- [ ] Approve A2 target: `infra_isolation` + single `gunbc-ci` job (+ optional aggregator strategy per E1)
- [ ] Approve A4 mirror row deletion list (§3.1) — no `ci_pipeline` topology cuts
- [ ] Confirm implementation order: **A3 run-all before or with A2**; A5 after affected-set cluster
- [ ] Confirm branch-protection job-ID strategy (E1)
- [ ] Acknowledge ratchet/`ci_corpus_eval` drift (§5)

---

## 8. References

- `docs/planning/thin-shim-ci-dag-dissolution-design-2026-06-08.md` (merged #4527)
- `dsl/gunbc/ci_emission.dag` — BinaryShim target workflow
- `src/v3/compiler/src/bin/gunbc_ci.rs` — dispatch stub
- `src/v4/workflow/ci.dag` — structural authority
- `.github/workflows/ci.yml` — live transport (556 lines @ HEAD)
