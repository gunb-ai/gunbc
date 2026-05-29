# CI DAG Overhaul — Design Canvas (Operator Ratification Gate)

**Status:** **DRAFT — awaiting operator audit/approval.** No Node authoring or legacy-path deletion lands until this canvas is ratified.

**Date:** 2026-05-29  
**Author lane:** clever-cat-115 (CI EFFICIENCY MANAGER, `node://adhoc-0972e492-c72`)  
**Companion read (what is wrong today):** [CI anatomy audit](https://github.com/gunb-ai/gunbc/pull/3885) (`docs/audit/ci-anatomy-and-redundancy-2026-05-29.md`, lands with #3885)  
**This document (what we build):** modeled CI authority in `src/v4/workflow/ci.dag`, migration atoms (**§6.2 = project board**), emission end-state, TASKS.md bridge (**§6**), coordination boundaries. **No separate task ledger** — dashboard work-items align 1:1 with atoms here.

**Precedent canvases:** `docs/design-ci-workflow-substrate-shape-2026-05-12.md` (substrate shape, ratified), `docs/design-ci-workflow-emitter-dispatch.md` (`WorkflowRuntime` / projection targets), `docs/design-affected-set-lens.md` (T-21 frontier).

---

## 1. Architectural target

Every CI computation is a **Node** in `src/v4/workflow/ci.dag` with declared `content_hash` inputs and typed outputs. **GHA executes CI by invoking the standard interpreter** (T-22 / `compiler/05_eval.dag`, THESIS:225 — `dag run` is the primary execution path) on `workflow/ci.dag` — not a bespoke “CI runner” binary. `.github/workflows/ci.yml` is a **minimal harness** (checkout, build/install gunbc, invoke interpreter) with optional **YamlStatic emission** (S3) as a performance/projection fallback, not a parallel authority. Legacy shell scripts and hand-Rust mirrors are **deleted in the same PR** as their modeled Node lands — including the **`scripts/check-*.sh` files themselves**, not only the `ci.yml` step that invoked them. **Discipline policy** has no enduring `scripts/*` authority: it becomes **`TestClaim` nodes** under `src/v4/test/claim/workflow/` evaluated via T-22 eval (T-38 wires eval into CI). **Exactly-once** is a structural property: unchanged input merkle → step is not scheduled (IRT-1 frontier) and prior green verdict is reused (IRT-4 `content_hash(whole TestClaim node)` per `src/v4/TASKS.md`). Speed is a consequence of correct modeling, not YAML `if:` tuning.

---

## 2. Proposed Node taxonomy

The overhaul **extends** the existing `v4.workflow.ci` carriers already on `main`. It does **not** redeclare types owned by wise-otter-34 / PR [#3853](https://github.com/gunb-ai/gunbc/pull/3853).

### 2.1 Authority layers (what is a “Node”)

| Layer | Carrier (in `ci.dag` or substrate) | Represents | Inputs (declared) | Output | Depends on |
|-------|-----------------------------------|------------|-------------------|--------|------------|
| **Witness source** | `CiGitDiffReadOutcome` *(#3853)* | Single git diff read, fail-closed | `Witness<GitDiffNameOnly>` from host transport | `Outcome<GitDiffNameOnly>` | checkout + fetch policy |
| **Component projection** | `CiComponentAffected` *(#3853)* | Coarse job-level buckets (`v2`, `v3`, `v4`, `workflow_policy`) | `GitDiffNameOnly` or `CiGitDiffReadOutcome` | `CiComponentAffected` flags | diff witness |
| **Frontier** | `AffectedSet` / `RerunNodeSet` *(T-21 lens)* | Node-precision rerun set | `Dag_before`, `Dag_after`, edit locus | `RerunNodeSet` | diff + program graph |
| **Pipeline vertex** | `CiJob` | One schedulable unit of work | `command`, `needs` | job verdict | upstream `CiJob` outputs |
| **Command** | `CiCommand` coproduct | What the job executes | per-arm fields + implicit digests | command receipt | bootstrap pool, digests |
| **Gate** | `CiGate` | Required check / policy surface | `job`, `run_policy` | gate verdict | bound `CiJob` |
| **Cache digest** | `ci_*_cache_digest` fns | IRT-4 key material for pipeline eval | command/job/gate structure | `Hash` | real input merkle *(target)* |
| **Test invocation** | `TestCommand` / `TestClaimCorpusEvalCommand` | Cargo test or T-22 eval roster | claim subgraph + binary digest | `TestClaimRun` | frontier selection |
| **Runner pool** | `CiRunnerPool` *(#3853)* | Self-hosted parallelism facts | pool id, job cap | pool binding | `workflow_policy` / M1 |

**Interim carriers to dissolve** (already marked 🟡 in `ci.dag`): `LensCiLiveWorkflowSignal`, `M1CiLiveWorkflowSignal` — forbidden permanent homes for `ci.yml` step names; projection must read `CiPipeline` only.

### 2.2 `CiCommand` arms — current → target

| `CiCommand` arm | Current CI step(s) | Target inputs | Target output | Notes |
|-----------------|-------------------|---------------|---------------|-------|
| `LintCommand` | `fmt` job: `cargo fmt --all` | `content_hash(rustfmt surface)` + toolchain pin | fmt pass/fail | Skip when surface ∅ (R01) |
| `BootstrapStageCompile` | `ci`: build `gunbc`, v4 bootstrap | `content_hash(src/v2/stage0/**)` + stage graph | stage0 binary digest | Shares artifact with downstream (R08/R09) |
| `LensCiCommand` | Lens-CI smoke + semantic compile | registry closure + entry root merkle | lens gate witnesses (T-10) | Verdict authority stays `00_compile.dag` |
| `M1RustEmitProbeCommand` | `v4-m1-rust-emit-probe.sh` | `content_hash(src/v4/**.dag)` + v2 binary digest + target=Rust | probe receipt (fail-closed) | Delete shell; fix static tag (R07, B01) |
| `TestClaimCorpusEvalCommand` | `v4-testclaim-corpus-gate.sh` | roster ∩ `ci_select_from_affected_set` | corpus eval verdicts | T-38 / T-22 |
| `TestCommand` | v3 integration filters, T-15 smoke, discipline self-tests | `content_hash(TestClaim node)` + toolchain/binary | `TestClaimRun` | IRT-1 + IRT-4 (R05, R11) |
| `IgnoredTestCommand` | `#846` zero-filter placeholders | explicit claim id | skipped receipt | Modeled “must run elsewhere” |

**New arms (to author — names indicative, exact coproduct spelling in implementation PR):**

| Proposed arm | Replaces | Declared inputs |
|--------------|----------|-----------------|
| `DisciplinePolicyCommand { policy_id, claim_refs, authority_paths }` | `check-*.sh` / `check_*.py` policy gates in `ci` | `content_hash(TestClaim node)` per claim + `authority_paths` merkle — **not** script bytes |
| `CiGitDiffPublishCommand` | `affected` job shell + `detect-affected-components.sh` | delegates to `CiGitDiffReadOutcome` → `CiComponentAffected` |
| `RustToolchainPoolCommand` | per-job Setup Rust + cache | union of downstream command digests (R02) |
| `V3IntegrationClusterCommand` | v3 job cargo test matrix | v3 subgraph merkle + single prebuilt test binary |

### 2.4 Discipline policy dissolution (P5 / single authority)

Today’s `ci` job runs ~9 discipline **shell/Python scripts** (`scripts/check-*.sh`, `scripts/check_t19_testgen_activation.py`). The ratified end state:

| Legacy path | Modeled replacement | Same-PR deletion (required) |
|-------------|---------------------|----------------------------|
| `scripts/check-pr-sg0-net-shrink-discipline.sh` | `TestClaim` + `DisciplinePolicyCommand` (SG-0) | script file + `ci.yml` steps |
| `scripts/check-r4-carve-dissolution-discipline.sh` | TestClaim (R4-carve) | script + steps |
| `scripts/check-fabrication-sentinels.sh` | TestClaim | script + step |
| `scripts/check-release-doc-authority.sh` | TestClaim | script + steps |
| `scripts/check-manager-brief-authority.sh` | TestClaim | script + steps |
| `scripts/check-rust-toolchain-single-authority.sh` | TestClaim | script + step |
| `scripts/check-workflow-path-regex-inventory.sh` | TestClaim + Gate #103 `TestCommand` | script + step |
| `scripts/check_t19_testgen_activation.py` | TestClaim (T-19) | script + step |

**Forbidden steady state:** `DisciplinePolicyCommand` that shells out to retained `scripts/check-*.sh` (content-addressed script execution still leaves a second authority). **Allowed one-time import:** an atom may mechanically port shell logic into `TestClaim` oracle text in the **same PR** that deletes the script — no follow-on PR may depend on the deleted file.

Scheduling: `DisciplinePolicyCommand` is a `CiJob` whose execution arm is **`TestCommand`** (or `TestClaimCorpusEvalCommand` for multi-claim policies) narrowed by `ci_select_from_affected_set` on each claim’s declared `authority_paths`.

### 2.5 Mapping: today’s GitHub Actions jobs → modeled graph

| GHA job (today) | Modeled as | Scheduling driver (target) |
|-----------------|------------|----------------------------|
| `affected` | `CiGitDiffPublishCommand` + outputs `CiComponentAffected` | always (cheap); **one** diff witness |
| `fmt` | `CiJob` + `LintCommand` | `LintCommand` input merkle ≠ ∅ |
| `ci` | **DAG of many `CiJob`s** inside one runner pool, not one blob job | per-command frontier + component flags |
| `v2` | `BootstrapStageCompile` / v2 verify command | `CiComponentAffected.v2` + frontier |
| `v3` | `V3IntegrationClusterCommand` + `TestCommand`s | `CiComponentAffected.v3` + frontier |
| `v4` | receipt `CiGate` only | prior `ci` job verdicts (R12) |
| `self_host_ratchet` | `TestCommand` on T-15 fixed-point | `main` push policy + v3 outcome |

---

## 3. Dependency DAG (proposed)

```mermaid
flowchart TB
  subgraph witness [single witness per workflow run]
    checkout[Checkout + fetch policy]
    diff[CiGitDiffReadOutcome]
    checkout --> diff
  end

  diff --> affected[CiComponentAffected]
  diff --> frontier[AffectedSet / RerunNodeSet]

  affected --> pool[RustToolchainPoolCommand]
  frontier --> select[ci_select_from_affected_set]

  pool --> fmt[LintCommand]
  pool --> ci_jobs[CiJob cluster]

  select --> claims[TestClaim roster narrow]

  ci_jobs --> boot[BootstrapStageCompile]
  boot --> lens[LensCiCommand]
  boot --> m1[M1RustEmitProbeCommand]
  boot --> corpus[TestClaimCorpusEvalCommand]
  boot --> disc[DisciplinePolicyCommand via TestClaim]

  claims --> tests[TestCommand / TestClaim eval]
  m1 --> v4gate[CiGate v4 receipt]
  tests --> v3job[v3 cluster]
  v3job --> ratchet[self_host_ratchet TestCommand]
```

**Edge list (authoritative scheduling):**

1. `CiGitDiffReadOutcome` → `CiComponentAffected` (component buckets for GHA job `if:` during transition).
2. `CiGitDiffReadOutcome` → `affected_set_rerun_nodes` → `ci_select_from_affected_set` (per-claim / per-command narrow).
3. `RustToolchainPoolCommand` → all rust-consuming `CiJob`s (fmt, compile, test).
4. `BootstrapStageCompile` → `LensCiCommand`, `M1RustEmitProbeCommand`, `TestClaimCorpusEvalCommand` (shared v2 binary / emit tree).
5. `M1RustEmitProbeCommand` → `CiGate` (`m1_rust_emit_probe_signal`); **blocking** verdict (B01: no `continue-on-error` at authority).
6. `DisciplinePolicyCommand` → `TestCommand`(s): no compile dependency unless claim authority touches rust surface; **no `scripts/*` invocation**.
7. `v3` job cluster: depends on pool + optional `BootstrapStageCompile` for gunbc-assisted tests only.

**Duplicate reads eliminated:** one `fetch-depth: 0` + `git fetch origin main` at witness; `ci` job must not re-fetch for discipline-only paths (R04).

---

## 4. Cache-key shape per Node type

**IRT-4 rule (non-negotiable):** verdict cache key = `content_hash(whole TestClaim node)` — input subgraph + oracle + evaluator + resources (`src/v4/TASKS.md` ~L1120). Pipeline-level `ci_command_cache_digest` interim tags are **scaffolding** until each command’s inputs are merkle-backed (audit R10).

| Node / command | Cache key function (target) | Replaces (interim) |
|----------------|----------------------------|---------------------|
| `CiGitDiffReadOutcome` | `content_hash(Witness<Diff>)` | n/a |
| `CiComponentAffected` | `content_hash(diff paths + bucket rules)` | static script version tag |
| `LintCommand` | `combine_hash(rustfmt_surface_merkle, toolchain_digest)` | `ci_cache_cmd_lint_tag` |
| `RustToolchainPoolCommand` | `content_hash(union(downstream_ci_job_digests))` | per-job duplicate setup |
| `BootstrapStageCompile` | `content_hash(src/v2/stage0/**.dag)` + plan pin | `ci_cache_cmd_bootstrap_compile_tag` + produces symbol |
| `LensCiCommand` | `combine_hash(registry_merkle, entry_root, target)` | lens tag fold |
| `M1RustEmitProbeCommand` | `combine_hash(content_hash(src/v4/**.dag), v2_stage0_binary_digest, Rust)` | **`ci_cache_cmd_m1_probe_tag` (bug R10)** |
| `TestClaimCorpusEvalCommand` | `combine_hash(corpus_merkle, selection_fn, roster_digest)` | static tag + fn symbol |
| `TestCommand` | `content_hash(TestClaim node)` | per-filter cargo invocations |
| `DisciplinePolicyCommand` | `content_hash(TestClaim node)` per scheduled claim (via `TestCommand`) | `check-*.sh` + unconditional shell steps |
| `CiGate` | `combine_hash(job_verdict_hash, run_policy_digest)` | GHA `if:` strings |
| `CiPipeline` (evaluator) | `ci_pipeline_cache_digest` → **dissolve** to `content_hash(Node projection of CiPipeline)` | hand-rolled fold (T-22) |

**Verdict reuse:** interpreter / T-38 harness checks IRT-4 receipt before host effects; GHA schedules only when eval reports `Schedule`.

---

## 5. End-state `.github/workflows/ci.yml` and execution

**Ratified direction (compose existing canvases):** substrate shape **(c-refined)** from `docs/design-ci-workflow-substrate-shape-2026-05-12.md`; emission target **`WorkflowRuntime`** from `docs/design-ci-workflow-emitter-dispatch.md`.

### 5.1 S2′ — Interpreter-direct (preferred)

**Operator framing (2026-05-29):** GHA already invokes a binary; that binary should be the **standard interpreter** on `src/v4/workflow/ci.dag`, not a special `gunbc-ci-run` pathway.

| Piece | Role |
|-------|------|
| **GHA harness** | Checkout, optional gunbc build, env (toolchain isolation), **one eval invocation** per job partition |
| **Interpreter** | T-22 `05_eval.dag` walks `ci_pipeline`, applies IRT-1 frontier skip + IRT-4 reuse, executes `CiCommand` arms |
| **Host effects** | `cargo test`, `cargo build`, `git diff` via **modeled host-effect substrate** (`extdeps/coordination.dag` + runtime carriers) — load-bearing for CI |
| **No special runner** | T-38 “modeled runner” = **wiring eval into CI** (neat-wren-762 / Lane C), not a second execution engine |

**Example harness step (illustrative):**

```yaml
- name: Run CI pipeline (interpreter)
  run: gunbc eval --entry v4.workflow.ci::ci_pipeline --witness "$DIFF_WITNESS"
```

Exact CLI spelling lands with T-38 / bootstrap entry wiring; the design choice is **interpreter-direct**, not a parallel CI binary.

**Consequences (viability analysis):**

| Factor | Assessment |
|--------|------------|
| **Less new infra** | Yes — reuses T-22; no bespoke `ci-runner` crate |
| **Host-effects model** | **Load-bearing** — interpreter must legally invoke subprocesses (cargo, git). Gap today: not all `CiCommand` arms have typed host-effect declarations; **A0** (below) tracks substrate before A4+ |
| **Interpreter on critical path** | Yes — stability/perf of eval becomes CI SLO; acceptable per THESIS:225 |
| **S3 YamlStatic** | **Fallback / optimization** — pre-expand schedule for GHA matrix fan-out or cold-start savings; not required for correctness if S2′ works |

**If S2′ were blocked:** name the missing capability in §8 (Q12) — e.g. host-effect carrier for `Subprocess(cargo)` not yet enforceable, or `CiPipeline` not yet a valid eval entry root. Current assessment: **viable** contingent on T-38 executor wiring (same stack as claim corpus eval).

### 5.2 Staged path (S0 → S1 → S2′ → S3?)

| Phase | `ci.yml` shape | Justification |
|-------|----------------|---------------|
| **S0 (today)** | Hand-authored + `ci_github_actions_workflow.dag` mirror | Dual authority; audit baseline |
| **S1 (#3853 lands)** | Hand `ci.yml` + `detect-ci-affected-components` binp; `ci.dag` owns bucket rules | Substrate merge; still dual |
| **S2′ (preferred)** | **Interpreter-direct harness** — minimal jobs invoke eval on `ci.dag` | Uniform with all `.dag` execution; T-38 provides CI wiring |
| **S3 (optional)** | **YamlStatic** emitted from `ci.dag` + `actions.dag` | Hyper-perf / branch-protection ergonomics; T-24 close allows S2′+ratchets OR S3 |

**Recommendation:** ratify **S2′** as primary; treat **S3** as optional acceleration, not a competing “real” end state. Retire the old **S2 BinaryShim + special `gunbc ci run`** framing — it duplicated the interpreter behind a new name.

**What remains in S2′ YAML (minimal):**

- Triggers, concurrency, permissions, runner labels (platform facts).
- Checkout + single diff witness (or host transport → `CiGitDiffReadOutcome`).
- Interpreter invocation(s) partitioned only where GHA parallelism requires separate runners (e.g. heavy `v3` host).
- No `bash scripts/check-*.sh` — discipline via `DisciplinePolicyCommand` → TestClaim eval (§2.4).

---

## 6. TASKS.md bridge and planning authority

This canvas **is** the CI-overhaul project board. §6.2 atoms are the tracked tasks; dashboard `work-items create` dispatches **must** reference an atom id (A0–A18). Do not maintain a parallel markdown ledger.

### 6.1 Cited task rows (do not re-author)

**T-21 — affected-set frontier** (`src/v4/TASKS.md`):

> **Why early**: load-bearing for incremental cross-run execution AND it is the structural replacement for `scripts/detect-affected-components.sh` (the interim shell bridge currently gating v2/v3/v4 CI selection).  
> …  
> **IRT-4 (result caching — this task + T-24).** A TestClaim's result is cached keyed by the `content_hash` of the **whole TestClaim node** …

**T-24 — CI pipeline AS DATA** (`src/v4/TASKS.md`):

> **Why load-bearing**: THESIS:223-226 — "adding a CI gate = editing one .dag file." …  
> **CI/YAML authority bridge:** T-24 is not closed while committed YAML and v3 string ratchets can act as parallel authorities. It dissolves when the generator emits checked YAML from `ci.dag`, the hand-authored YAML is deleted …  
> Affected-set-driven job selection consuming `lens/affected_set.dag` (T-21) — this is what dissolves `scripts/detect-affected-components.sh`

**T-38 — TestClaim execution harness** (`src/v4/TASKS.md`):

> **Why this is a T-15 gate**: T-15's close condition includes "TestClaim suite passes." …  
> T-38 closes when: (1) A CI step runs T-22 eval over `src/v4/test/claim/manual/*.dag` … (3) `scripts/v4-testclaim-corpus-gate.sh` is deleted …

**T-22 — interpreter (execution substrate)** (`src/v4/TASKS.md`):

> **Why load-bearing**: THESIS:225 — `dag run` is THE primary execution path. eval is not an afterthought to emit; it is the default.

### 6.2 Relationship to existing and proposed T-* rows

| Task | Relationship to this overhaul |
|------|------------------------------|
| **T-21** | **Consumed** — `ci_select_from_affected_set` + IRT-1/IRT-4; A1–A2 wire diff → frontier. Does not close until CI stops using coarse-only gating without frontier. |
| **T-24** | **Primary close target** — this overhaul **is** the T-24 implementation program. Close predicate: §9 + atoms A1–A18 done. S2′ satisfies “CI as data executed by interpreter”; S3 optional for YAML emission bullet. |
| **T-38** | **Executor layer partner** — neat-wren-762 (Lane C) wires T-22 eval into CI; clever-cat-115 owns `ci.dag` graph + atom deletions. **Same runner stack** — not a competing CI binary. |
| **T-22** | **Execution engine** — interpreter-direct (§5.1). No new T-* row. |
| **T-15** | **Downstream consumer** — CI schedules `TestCommand` for self-host fixed-point; T-15 implementation stays Lane C. Overhaul contributes **predicate 5** (“TestClaim suite passes” via T-38/A12) and does not own `bin/main.dag` fill. |
| **T-10 / T-23** | **Unchanged split** — lens verdict authority in `00_compile.dag`; CI schedules `LensCiCommand` only (A11). |

**Proposed TASKS.md edits (post-ratification, single PR to TASKS.md):**

| Proposal | Action |
|----------|--------|
| **T-24 body** | Add pointer: “Implementation program: `docs/design-ci-dag-overhaul.md` §6.2 (atoms A0–A18).” Clarify close allows **interpreter-direct GHA harness (S2′)** OR emitted YamlStatic (S3). |
| **T-38 body** | Explicit: CI executor = T-22 eval entry for `workflow/ci.dag` + claim corpus; owner split neat-wren-762 (harness) / clever-cat-115 (`ci.dag` + deletions). |
| **No new T-* row** | Atoms A0–A18 subsume “CI efficiency” micro-tasks unless operator adds **T-39** as umbrella — **not recommended** (avoid ledger duplication). |

### 6.3 Migration atoms — project board

Each row = **one PR** = modeled Node(s) authored + legacy path deleted + dashboard work-item (1:1).

| Atom | Node(s) / command | Legacy deleted (same PR) | Owner | Status | TASKS | Work-item |
|------|-------------------|--------------------------|-------|--------|-------|-----------|
| **A0** | Host-effect declarations for CI commands (`cargo`, `git`, …) | n/a (substrate only) | neat-wren-762 + clever-cat-115 | **blocked** (T-38 substrate) | T-38, T-22 | *(dispatch with T-38 lane)* |
| **A1** | `CiComponentAffected`, `CiGitDiffReadOutcome`, detect bin | `scripts/detect-affected-components.sh` | wise-otter-34 | **ready** (#3853) | T-21, T-24 | #3853 |
| **A2** | Sole diff witness in harness | duplicate fetch in `ci` | wise-otter-34 | **ready** | T-24 | #3853 |
| **A3** | `CiRunnerPool` + M1 pool projection | `V4_M1_CARGO_CHECK_JOBS` shell | wise-otter-34 | **ready** | T-24 | #3853 |
| **A4** | `RustToolchainPoolCommand` | 2nd/3rd rustup in `ci`/`fmt` | clever-cat-115 | **paused** (ratification) | T-24 | `node://adhoc-4221c587-e70` *(retarget)* |
| **A5** | `LintCommand` surface digest | unconditional `fmt` | clever-cat-115 | **paused** | T-24 | — |
| **A6** | `DisciplinePolicyCommand` SG-0 / R4-carve | `check-pr-sg0-*.sh`, `check-r4-carve-*.sh`, steps; + claims | clever-cat-115 | **paused** | T-24, T-38 | — |
| **A7** | `DisciplinePolicyCommand` doc/manager | `check-release-doc-*.sh`, `check-manager-brief-*.sh`, steps; + claims | clever-cat-115 | **paused** | T-24 | — |
| **A8** | `DisciplinePolicyCommand` fabrication/T-19/toolchain/G103 | `check-fabrication-*.sh`, `check-rust-toolchain-*.sh`, `check-workflow-path-regex-*.sh`, `check_t19_*.py`, steps; + claims | clever-cat-115 | **paused** | T-24, T-19 | — |
| **A9** | `M1RustEmitProbeCommand` + eval wiring | `scripts/v4-m1-rust-emit-probe.sh`, `M1CiLiveWorkflowSignal` | clever-cat-115 | **paused** | T-24, M1 | `node://adhoc-4221c587-e70` |
| **A10** | Bootstrap → M1 artifact edge | duplicate full-tree compile | clever-cat-115 | **paused** | T-24, T-20 | — |
| **A11** | `LensCiCommand` projection | `LensCiLiveWorkflowSignal`, hand semantic step | clever-cat-115 | **paused** | T-24, T-10 | — |
| **A12** | `TestClaimCorpusEvalCommand` via interpreter | `scripts/v4-testclaim-corpus-gate.sh` | clever-cat-115 + neat-wren-762 | **paused** | **T-38**, T-24 | *(neat-wren-762)* |
| **A13** | Bootstrap viability chain | `scripts/v4-bootstrap-viability.sh`, posture gate | clever-cat-115 | **paused** | T-20, T-15 P5 | — |
| **A14** | `V3IntegrationClusterCommand` | duplicate `cargo test` filters | clever-cat-115 | **paused** | T-24 | — |
| **A15** | Gate #103 from model | hand `if:` / inventory drift | clever-cat-115 | **paused** | T-24 | — |
| **A16** | YamlStatic `affected` job *(optional S3)* | hand `affected` body | clever-cat-115 | **paused** | T-24 | — |
| **A17** | Full YamlStatic `ci.yml` *(optional S3)* | hand `.github/workflows/ci.yml` | clever-cat-115 | **paused** | T-24 close | — |
| **A18** | Emission mirror removal | `ci_github_actions_workflow.dag` | clever-cat-115 | **paused** | T-24 | — |

**Dispatch rule:** `dashboard-ops work-items create "<title>"` titles **must** start with atom id, e.g. `A9: M1RustEmitProbeCommand merkle + delete v4-m1-rust-emit-probe.sh`.

**Paused until canvas ratified:** A4–A18 (clever-cat-115). **A1–A3** wise-otter-34 (#3853) may land independently.

---

## 7. Coordination contracts

| Owner | Ships | Does **not** ship |
|-------|-------|-------------------|
| **wise-otter-34** / [#3853](https://github.com/gunb-ai/gunbc/pull/3853) | `CiComponentAffected`, `CiGitDiffReadOutcome`, `detect-ci-affected-components`, runner pool facts, `ci.yml`/`ci.dag` alignment for **affected** + M1 pool, claim receipts | Per-command migration atoms A4–A18; full YAML emission |
| **clever-cat-115** | This canvas; audit #3885; post-ratification A4–A18; `ci.dag` graph + legacy deletions; discipline → TestClaim | Redefining #3853 types; T-38 harness implementation; `release.dag`; T-15 self-host |
| **neat-wren-762** (Lane C) | **T-38 executor** — T-22 eval wired into GHA; host-effect substrate for subprocesses; shares A0/A12 with clever-cat | `ci.dag` command authoring; YAML/policy atoms |
| **merry-carp-814** | `release.dag` → `release.yml` emission pattern; shared projection DSL lessons | `ci.dag` edits; `ci.yml`; CI efficiency atoms |
| **vivid-raven-55** | INVARIANTS / scaffold-ratchet **deletions** only (when trigger fires) | New CI commands; workflow policy |
| **Lane C / T-15** | Self-host fixed-point, `bin/main.dag` bootstrap | CI workflow modeling |

**No overlap rule:** one authority per fact. If a step appears in `ci.yml`, it must be projectable from `ci.dag` or be on the explicit S2 shim allowlist until A17.

**Interface handshake (clever-cat ↔ wise-otter):** clever-cat-115 consumes `CiGitDiffReadOutcome` / `CiComponentAffected` as **read-only** types; extension PRs add `CiCommand` arms only on clever-cat branches after #3853 merges.

---

## 8. Open questions — operator ratification required

| ID | Question | Options | Recommendation |
|----|----------|---------|----------------|
| Q1 | End-state emission | S2′ interpreter-only vs S3 YamlStatic vs both | **S2′ required**; **S3 optional** (perf/ergonomics); T-24 close when authority + ratchets met |
| Q2 | GHA execution model | Interpreter-direct vs special CI binary vs YAML-only | **Interpreter-direct (S2′)** per THESIS:225; retire bespoke `gunbc ci run` |
| Q12 | Executor collaborator | Separate CI runner vs T-38 harness | **neat-wren-762 (T-38)** owns eval wiring + host effects (A0); clever-cat owns `ci.dag` + deletions |
| Q13 | S2′ viability blockers | If not viable, what is missing? | **Open:** host-effect carriers for all `CiCommand` arms — track **A0** before A4; re-evaluate at T-38 milestone |
| Q3 | Branch protection check names | Rename allowed vs stable aliases | **Stable `CiGate.id` → check name map** for migration; document renames |
| Q4 | Coarse buckets during transition | Keep `v2/v3/v4/workflow_policy` outputs | **Yes until A17**; per-command schedule bitmap behind the scenes |
| Q5 | M1 probe blocking | Informational vs required | **Required fail-closed** (B01); notices OK, green-on-timeout forbidden |
| Q6 | `fmt` as separate GHA job | Separate vs merged into pool job | **Merged into pool** at S2 (one rustup) |
| Q7 | v3 on docs-only PRs | Skip entire job vs skip via frontier | **Frontier-only** (no hand `ci_rust` YAML — superseded #3879) |
| Q8 | External cache backend | None vs sccache vs shared CARGO_HOME | **Defer** (R06); per-runner temp stays until infra decision |
| Q9 | `#846` zero-filter tests | Move receipt vs fix filter | **Model `IgnoredTestCommand`** + run at correct site (audit §8) |
| Q10 | Audit §7 P0 micro-rows | Keep vs rewrite | **Rewrite** to deletion-gated atoms A1–A18 (follow-up on #3885) |
| Q11 | Discipline end-state | Retain `scripts/*` with content hash vs pure TestClaim | **Resolved:** pure TestClaim + delete scripts in A6–A8 (§2.4); no script authority at S3 |

---

## 9. Acceptance criteria (“CI overhaul done”)

Structural completion (not wall-clock promises):

1. **T-24 closed** per `src/v4/TASKS.md`: `ci.dag` is sole CI authority; GHA invokes interpreter on `ci_pipeline` (S2′); hand policy scripts deleted; **optional** S3 YamlStatic emission with ratcheted diff test.
2. **Zero** `scripts/*` invoked from CI for **policy/discipline** (including no content-addressed shell-out). Host transports only (`detect-ci-affected-components`, checkout/fetch) may remain until dissolved with explicit 🟡 marks; **no** `scripts/check-*.sh` on disk as CI policy authority.
2b. Every former discipline script in §2.4 is **deleted** and covered by a `TestClaim` + `DisciplinePolicyCommand` row in `ci_pipeline`.
3. **IRT-1 + IRT-4** wired for every `TestCommand` / `TestClaimCorpusEvalCommand` in CI roster (docs-only PR: no rust toolchain, no cargo, discipline claims skipped at frontier).
4. **M1** cache uses real `src/v4` merkle; shell probe deleted; timeout cannot yield green (B01).
5. **Single** `CiGitDiffReadOutcome` per workflow run (R04).
6. **Ratchets:** `v4_workflow_ci_runner_dag_smoke_test` + emission diff test prove `ci.yml` ≡ projection(`ci_pipeline`).
7. **Audit Table A** rows R01–R12 have a **Table B** modeled owner merged or explicitly deferred with operator sign-off.

**Optional metrics (informational):** docs-only PR total wall ≤15s (profiled run 26615772294 baseline ~77s); v4-affected PR avoids duplicate full-tree compile (R07/R08).

---

## 10. Out of scope

- **`release.dag` / `release.yml`** — merry-carp-814 lane; only pattern-sharing per §7.
- **T-15 self-host fixed-point implementation** — Lane C; CI only *schedules* the TestCommand.
- **T-21 affected-set lens math** — wise-otter-34 / lens owners; CI consumes `AffectedSet`.
- **T-10 `run_required_lens_gates` semantics** — compiler owns verdict; CI schedules `LensCiCommand`.
- **sccache / registry infra** (R06) — independent capacity project.
- **Deleting overlapping tests by hand** — §8 audit: input declaration + IRT-4 makes overlap free; only SCAFFOLD-RATCHET/DEAD deletions (vivid-raven-55).
- **v2/v3 frozen-tree feature work** — only CI scheduling around frozen components.
- **Micro-optimization PRs** (#3879, #3882, #3883-class) — superseded by this overhaul.
- **Merging this canvas or audit via agent** — operator manual merge policy unchanged.

---

## Appendix A — References

| Artifact | Role |
|----------|------|
| `src/v4/workflow/ci.dag` | Modeled pipeline (current partial fill) |
| `src/v4/TASKS.md` T-24, T-21, IRT-1/IRT-4 | Close gates |
| `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md` | Redundancy inventory |
| `docs/design-ci-workflow-substrate-shape-2026-05-12.md` | Ratified WorkflowRuntime placement |
| `docs/design-ci-workflow-emitter-dispatch.md` | YamlStatic / BinaryShim |
| PR #3853 | Substrate types (wise-otter-34) |
| PR #3885 | Audit doc |
| `src/v4/TASKS.md` T-21, T-24, T-38, T-22, T-15 | Cited close gates (§6.1–6.2) |
| neat-wren-762 / T-38 | CI interpreter harness (A0, A12) |

## Appendix B — Execution gate

**PAUSED:** `session/clever-cat-115-ci-dag-m1-node` and all atoms A4+ remain **draft-only** until operator approves this canvas. Permitted work before ratification: audit doc (#3885), this design PR, coordination messages only.
