# CI Bankruptcy & Rebuild — Integrity-First Canvas

**Status:** DRAFT — operator steering input (2026-05-29). Supersedes the *migration posture* of incremental atom landings until Tier-0 is green; does **not** repeal S2′ / `ci.dag` authority / A15 Shape-B close from [design-ci-dag-overhaul.md](./design-ci-dag-overhaul.md).

**Thesis:** Treat today’s `.github/workflows/ci.yml` + script graph as **bankrupt**. Do not “tune” or path-gate it into health. Stand up a **minimal harness** that runs only **integrity-class** work modeled in `src/v4/workflow/ci.dag`; add everything else back **one Node at a time** with measured cost and explicit operator opt-in.

**Companion (inventory, pre-bankruptcy):** [PR #3885](https://github.com/gunb-ai/gunbc/pull/3885) — `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md` (not on `main` until that PR merges; do not use a relative path here).

---

## 1. What “bankruptcy” means (and does not)

| In scope | Out of scope |
|----------|----------------|
| Delete or disable legacy jobs/steps that are not Tier-0 | Pretend the old graph can be salvaged with more `if:` |
| One thin GHA harness + interpreter on `ci_pipeline` | Parallel YAML policy during “transition” |
| Fixed-point / determinism / bootstrap integrity **modeled** | Re-running full release matrices on unchanged subgraphs |
| Opt-in backlog for discipline, docs, manager-brief, etc. | Auto-carrying forward ~9 `check-*.sh` + duplicate v3/v4 compiles |

**Operator quote (intent):** keep super-high-priority **integrity** (v2/v3/v4 fixed-point family); decide the rest deliberately.

---

## 2. Tier-0 — the only things that ship in Phase 0

These are **non-negotiable** for compiler correctness. Everything else is **Tier-1+** until explicitly promoted.

### 2.1 Integrity lattice (modeled commands)

| ID | `CiCommand` / claim family | What it proves | Legacy dissolved |
|----|---------------------------|----------------|------------------|
| **I0** | `CiGitDiffReadOutcome` witness | One fail-closed diff read per run | `detect-affected-components.sh` bucket outputs |
| **I1** | Frontier + `ci_select_*` | Schedule only affected `CiJob` / `TestClaim` | `if: v2/v3/v4`, `CiComponentAffected` → GHA |
| **I2** | `V2BootstrapCompileCommand` (minimal) | v2 compiler still builds enough to feed v4 bootstrap | redundant v2 job blob (smoke only) |
| **I3** | `V3DeterminismCommand` | Emit matrix stable (5× or configured) | `determinism_test` only inside monolithic `v3` job |
| **I4** | `V3SelfHostFixedPointCommand` | emit → compile → re-emit byte-identical | `self_host_fixed_point` bin + **`self_host_ratchet` job** |
| **I5** | `V4BootstrapStageCompile` | gunbc / v4 bootstrap artifact for eval | duplicate gunbc builds across jobs |
| **I6** | `M1RustEmitProbeCommand` | `src/v4` emit wall bounded; **fail-closed** | `v4-m1-rust-emit-probe.sh` + unconditional full-tree |
| **I7** | `TestCommand` / `TestClaim` — **T-15** | v4 self-host fixed point (content_hash of stages) | hand `cargo test` filters scattered in `ci` |
| **I8** | `TestClaimCorpusEvalCommand` (narrow) | Manual claim corpus via T-22 when frontier demands | `v4-testclaim-corpus-gate.sh` |

**Explicitly Tier-1+ (not in Phase 0):** SG-0, R4-carve, release-doc, manager-brief, fabrication, T-19 py, Gate #103 path-regex inventory, Lens-CI semantic compile, v3 integration cluster duplicate, `self_host_ratchet` as separate job, fmt as standalone job, doc-only discipline shells.

### 2.2 Scheduling rules (efficiency = modeled, not YAML hacks)

1. **One interpreter entry** per workflow run on `ci_pipeline` (S2′).
2. **IRT-1:** `AffectedSet` / `RerunNodeSet` is the only schedule authority.
3. **IRT-4:** Every Tier-0 `TestCommand` / eval result caches on `content_hash(whole TestClaim node)` (and command merkle where applicable). Unchanged subgraph → **seconds**, not recompile.
4. **Profile as data, not duplicate jobs:**

```text
data CiBuildProfile: Enum = { Smoke, Release, FixedPointMatrix }

// Example: v3 fixed point runs Release ONLY when scheduled — not "v3 job" + "self_host_ratchet job"
V3SelfHostFixedPointCommand { profile: CiBuildProfile, ... }
```

5. **Push policy as data:** `CiSchedulePolicy: Enum = { PullRequest, MainPush, Nightly }` on each `CiJob` — replaces `self_host_ratchet` “main only” vs PR stub encoded in YAML `if:`.
6. **Fail-closed:** timeout / ambiguity → superset or red; never `continue-on-error` on Tier-0 (today’s `self_host_ratchet` advisory staging ends with bankruptcy).

---

## 3. Fixed-point work — modeled correctly (v2 / v3 / v4)

### 3.1 Problem today

| Issue | Symptom |
|-------|---------|
| Duplicate authority | `v3` runs `determinism_test` (debug); `self_host_ratchet` runs it again (**release**) on main |
| Duplicate wall clock | `self_host_ratchet` `needs: v3` → job duration includes **entire v3 wait** even when ratchet stubs on PR |
| Non-modeled policy | “main only”, `continue-on-error`, 60m timeout encoded in YAML |
| v4 M1 | Full-tree emit on coarse gates; not frontier + merkle (audit R07) |
| v4 T-15 | Not yet the single self-host gate; scattered cargo tests |

### 3.2 Target shape in `ci.dag`

```text
ci_pipeline
  ├─ witness: CiGitDiffReadOutcome
  ├─ frontier: affected_set_rerun_nodes
  ├─ schedule: ci_select_ci_jobs_from_affected_set   # I1 — A2
  └─ jobs (each with declared inputs + CiSchedulePolicy + cache digest)
        ├─ v2_bootstrap_smoke          # I2 — frontier: v2/compiler/**
        ├─ v3_determinism              # I3 — frontier: src/v3/compiler/**
        ├─ v3_self_host_fixed_point    # I4 — policy: MainPush OR frontier v3
        ├─ v4_bootstrap_compile        # I5
        ├─ v4_m1_emit_probe            # I6 — frontier: src/v4/** ; fail-closed
        ├─ v4_t15_self_host_claim      # I7 — TestCommand; content_hash stages
        └─ v4_testclaim_corpus         # I8 — ci_select_from_affected_set (existing fn)
```

**Efficiency contract (each fixed-point node):**

| Node | Runs when | Must NOT run when |
|------|-----------|------------------|
| I3 v3 determinism | Frontier intersects `src/v3/compiler/**` only | Docs-only PR; **minimal** matrix (not full legacy `v3` job) |
| I4 v3 self-host FP | Frontier hit **or** `MainPush` — **minimal** release fixed-point only | Unchanged v3 subgraph (IRT-4 reuse); **transitional** until v3 lane is deleted |
| I6 M1 probe | Frontier intersects `src/v4/**` merkle | Unchanged v4 subgraph |
| I7 T-15 | Frontier intersects compiler closure **or** release pin bump | Every PR full self-host if seed unchanged |

**Deletion in same PR as authorship:** `.github/workflows/ci.yml` jobs `v2`, `v3`, `v4`, `self_host_ratchet`, monolithic `ci` policy steps — replaced by harness + interpreter. Keep **one** `ci` harness job until A15 Shape-B projection.

### 3.3 `self_host_ratchet` specifically

**Bankrupt:** separate GHA job, `needs: v3`, duplicate release compile, advisory `continue-on-error`.

**Rebuild:** `V3DeterminismCommand` + `V3SelfHostFixedPointCommand` as **`CiJob`s** inside `ci_pipeline`, with:

- **Operator (2026-05-29):** run **minimal** I3/I4 when v3 frontier is hit; `MainPush` may run the same arms — **not** both redundantly on the same commit. v3 is **transitional** (v3 deletion expected); do not expand v3 CI surface beyond minimal fixed-point + determinism.
- `build_profile: Release` for I4 only; lighter profile acceptable for I3 on PR where sufficient.
- **No second job** — interpreter executes selected arms once

---

## 3.5 Reconciliation with `src/v4/TASKS.md` T-24 (same PR)

`TASKS.md` **Phase 1** is **not** “delete all `scripts/check-*` in one step.” It splits to match this doc:

| Name | Bankruptcy | TASKS / canvas atoms | Discipline `check-*` |
|------|------------|----------------------|----------------------|
| **Phase 1a / B1** | Tier-0 green (I0–I8) | **A0–A2** + integrity arms (I3–I8 as landed) | **Off CI critical path**; files may remain until 1b |
| **Phase 1b / B2** | Opt-in promotions | **A3–A14** one PR each | **Deleted** in same atom as `TestClaim` port (A6–A8) |
| **Phase 2 / B3** | Shape-B emission | **A15** | N/A (T-24 **[DONE]**) |

**§6.1.1** in [design-ci-dag-overhaul.md](./design-ci-dag-overhaul.md) and **§9.1** “lane done” refer to **Phase 1a + 1b** (B1 then B2), not B1 alone.

---

## 4. Phased rebuild (replaces incremental A2–A14 “migrate everything” first)

| Phase | Deliverable | CI surface |
|-------|-------------|------------|
| **B0 — Bankruptcy cut** | New minimal `ci.yml` harness; **delete** legacy jobs/steps (D3); `ci.dag` Tier-0 skeleton + smoke test | PRs run **only** Tier-0; may be red until B1 |
| **B1 — Tier-0 green** | I0–I8 modeled + legacy deleted; interpreter S2′; fail-closed M1 + T-15 path | Integrity green on main + representative PRs |
| **B2 — Opt-in backlog** | Promote audit Table B rows one at a time → `CiCommand` + delete script | Each promotion = one PR + cost note |
| **B3 — Shape-B** | A15: emit checked `ci.yml` from `CiPipeline` | T-24 / C4 close |

### 4.1 P5 receipt — v3 binding smoke (B0/B1)

Per **INVARIANTS** P5 Mechanism **(b)** (`_internal/INVARIANTS_OPS.md`): bankruptcy Tier-0 parity is a **same-path expansion** of `src/v3/compiler/tests/integration/v4_workflow_ci_runner_dag_smoke_test.rs` — **+0** new `EXPECTED_HAND_AUTHORED_TEST` census paths (file already registered for T-21/T-24 Wave-0).

| Receipt field | Authority |
|---------------|-----------|
| **Lane** | `_internal/ROADMAP_OPS.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero` |
| **INVARIANTS row** | `_internal/INVARIANTS_OPS.md` → `v4_workflow_ci_runner_dag_smoke_test.rs` (**PR #4101** expansion bullet) |
| **Dissolve-on** | **B3 / A15 Shape-B:** `ci_pipeline` emits checked `ci.yml`; retire `v4_workflow_ci_bankruptcy_tier0_*` substring ratchets when `.dag` `TestClaim` harness exercises CI facts without hand-Rust `include_str!` probes |
| **Interim ratchet** | `v4_workflow_ci_bankruptcy_tier0_*` tests; ci.yml modeled binding step runs `cargo test … v4_workflow_ci_bankruptcy_tier0_ -- --quiet` (D3; mirrors existing M1 binding-smoke pattern in [design-ci-dag-overhaul.md](./design-ci-dag-overhaul.md)) |

**Mapping from old atoms (design canvas §6.3):**

| Old | Bankruptcy phase |
|-----|------------------|
| A0 host effects | **B0** (smart-otter-98 — in flight) |
| A1 witness | **Done** (#3853) |
| A2 selector + harness | **B0/B1** (neat-carp-699 — align to Tier-0 cut, not full discipline migration) |
| A3–A14 discipline/lens/v3 cluster | **B2 only** (opt-in) |
| A15 Shape-B | **B3** |

---

## 5. Tier-1+ backlog (nothing ships unless promoted)

Promote from audit with **measured wall** (warm-cache methodology per #3881):

| Candidate | Default | Promotion requires |
|-----------|---------|-------------------|
| `fmt` / rustfmt | Tier-1 | Frontier on rustfmt surface + cost < N s on cold path |
| Discipline `check-*.sh` | Tier-1 | Ported to `TestClaim` + same-PR delete; not shell |
| Manager-brief / release-doc | Tier-2 | Operator explicit (doc authority) |
| Lens-CI semantic compile | Tier-1 | Frontier on lens registry |
| v3 full `cargo test` matrix | Tier-1 | Collapsed to `V3IntegrationClusterCommand` once, not job + ratchet |
| Gate #103 path-regex | Tier-2 | After I1 stable |
| **L-7** substrate accessor reconstruction (`scripts/check-l7-*.sh` or successor) | Tier-1 | B2 promotion: `TestClaim` port + delete shell; was legacy `v3` job only |
| **L-8** lens surface gate | Tier-1 | B2 promotion: same; was legacy `v3` job only |
| Compiler-std consolidation ratchet | Tier-1 | B2 promotion: `check-compiler-std-ratchet.sh` → TestClaim |
| Banked-dissolutions ratchet | Tier-1 | B2 promotion: `check-banked-dissolutions.sh` → TestClaim |
| v3 `cargo clippy` / `cargo test --no-run` (`bootstrap-regen-fresh`) | Tier-1 | B2: fold into `V3IntegrationClusterCommand` or discipline; **not** Tier-0 I3/I4 |

---

## 6. Acceptance — bankruptcy lane done

**B1 complete when:**

1. Tier-0 commands exist in `ci.dag` with declared `content_hash` inputs and IRT-4 reuse demonstrated (hermetic test: unchanged PR → cache hit).
2. No standalone `self_host_ratchet`, `v3`, `v4` policy jobs; **no** `scripts/check-*` on the **Tier-0 critical path** (deletion of script files = Phase 1b / A6–A8, not B1).
3. v3 fixed-point + v4 T-15 + M1 probe **fail-closed**; no `continue-on-error` on Tier-0.
4. Warm-cache wall (operator 2026-05-29): **docs-only PR → ~instant** (witness + affected + IRT-4 cache hits only); **code-touch PR → &lt; 1 min** total interpreter path on warm cache. Measure after B1; fail the lane if not met.

**Not required for B1:** full discipline port, Shape-B emission, all audit Table B rows.

---

## 7. Operator decisions (ratified 2026-05-29)

| # | Decision |
|---|----------|
| **D1 — v3 fixed-point** | **Minimal** I3/I4 when v3 frontier is hit; **`MainPush`** may run the same minimal arms. v3 CI is **transitional** — lane will delete v3 eventually; do not preserve full legacy `v3` job + `self_host_ratchet` duplication. |
| **D2 — v3 smoke** | Minimal determinism + fixed-point only; no full `cargo test -p v3-compiler` matrix unless explicitly promoted in B2. |
| **D3 — bankruptcy cut** | **Delete** legacy jobs/steps outright — no feature-flag bridge. |
| **D4 — A2 scope** | **B0/B1 Tier-0 harness** (neat-carp-699), not A3–A8 discipline in the same PR. |
| **D5 — CI wall clock** | Docs-only PR: **~instant** on warm cache. Code-touch PR: **&lt; 1 min** warm-cache interpreter path (not 2 min). |

---

## 8. References

- [design-ci-dag-overhaul.md](./design-ci-dag-overhaul.md) — S2′, A15, P5 dissolution (still authoritative for end-state)
- [design-fixed-point-ratchet.md](./design-fixed-point-ratchet.md) — v3 cycle semantics (absorbed into I3/I4)
- `src/v4/TASKS.md` T-15, T-24, T-38
- `src/v4/workflow/ci.dag` — sole policy authority post-bankruptcy
