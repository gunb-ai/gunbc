# T-WAD Slice 7 — Affected-set selection via `BinaryShim` (gate `ci_uses_affected_set_selection`, program row 103)

**Status:** Verification-lane design canvas (implementation **not** in this PR).  
**Authority:** `docs/r3-structure.md` (gate `ci_uses_affected_set_selection`), `docs/r3-program-plan.md` row 103, `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §0–§1, PR #2744 WI / FULL R3-close scope context, `docs/design-ci-workflow-emitter-dispatch.md`, `docs/design-affected-set-lens.md`.  
**Parent emitter canvas:** `docs/design-ci-workflow-emitter-dispatch.md` §5.2, §6 (this document **specializes** Slice 7; it does not reopen (c-refined) placement or `WorkflowRuntime` shape).  
**Upstream lens:** PR #2713 (affected-set lens substrate; merged per scope docs).  
**Blocking emitter:** Slice 5 `BinaryShim` projection arm + runtime — **Substrate Mgr** session **warm-wolf-698** (see §6).

**PM sequencing (warm-wolf-698 lane):** **Slice 4** (`YamlStatic` parity) lands **first**. **Slices 5 and 8** (`BinaryShim` emitter + `ci.yml` hand-authority dissolution per FULL R3-close plan) proceed **in parallel** after Slice 4. Slice 7 **implementation** still **waits on the Slice 5 hand-off** (compiled runner + thin shim + projection hook); Slice 8 can advance repository hygiene in parallel but does **not** replace binary-side affected-set consumption.

---

## §1. How `BinaryShim` consumes affected-set output (PR #2713)

### §1.1 Data flow (conceptual)

1. **Invocation boundary:** `project_github_actions(ci_workflow_dag, BinaryShim)` yields a provider-faithful `Workflow` plus emitted **thin shim** YAML and a **compiled CI runner binary** (exact artifact layout is Slice 5; see §6).
2. **At PR time** the runner binary obtains `Dag_before` (merge-base / base ref compilation) and `Dag_after` (PR head compilation), using the same compiler revision and feature flags as the repository policy encodes in `CIWorkflowDag` / pinned toolchain facts.
3. The binary **does not** re-derive affectedness from path globs. It calls the **affected-set lens pipeline** landed with PR #2713: per-dimension `affected_set(Dag_before, Dag_after, dim)` then aggregates per `docs/design-affected-set-lens.md` §2.
4. **Interchange shape (structural):** consumption is **typed lens output**, not stderr text or ad-hoc file lists:
   - Aggregate: `Set<NodeRef>` (or the NodeRef-keyed record carrying dimension + provenance if the landed lens emits the richer form — the consumer must treat the **authoritative serialized form** from PR #2713 as the single source of truth).
   - Per-dimension: proof receipts for exclusions / unknown deltas as required by `docs/design-affected-set-lens.md` §2 fail-closed discipline (“without that receipt, the consumer falls back to the default-include branch”).
5. The binary **maps** `NodeRef` sets to **executable CI actions** (cargo filters, job labels, gate commands) using metadata anchored in `CIWorkflowDag`, `TestClaim` declarations, and gate records — not by parsing GitHub `paths`/`paths-ignore` in YAML.

### §1.2 Interface stub (pending Slice 5)

The **exact** Rust (or generated) API surface between “emitted binary entrypoint” and “lens evaluation” is owned by the Slice 5 implementation PR. This canvas requires only:

- A **stable, versioned** internal contract: “runner requests affected-set bundle for `(before_ref, after_ref)`; receives `Ok(structured_receipt)` or `Err/Unknown`.”
- **No** dependency on YAML `github` context expressions for that bundle.

Verification Mgr may request a written **Slice 5 public surface** from warm-wolf-698 / PM before first integration PR; until then, integration work assumes the contract above.

---

## §2. Matrix-spec emission vs job-skip emission — tradeoff and **chosen initial shape**

| Surface | Pros | Cons |
|--------|------|------|
| **Dynamic matrix JSON** (`strategy.matrix` from a prior step output) | Native GitHub fan-out, visible per-matrix-cell checks | GH matrix limits, expression fragility, harder local reproduction |
| **In-runner selection** (single or few jobs; **selected test list** / **selected job list** executed inside the runner) | Matches `design-ci-workflow-emitter-dispatch.md` §6.1 preference; simpler shim; easier fail-closed logging | Less visible fan-out in Actions UI |

**Chosen initial shape (Slice 7 first landing):** **In-runner selection** as primary:

- The thin YAML runs a **small fixed job graph** (bootstrap checkout, toolchain, invoke `gunbc-ci` or emitted binary name TBD by Slice 5).
- The binary computes affected-set, derives **selected test names / gate commands / internal phase list**, and invokes `cargo test`, scripts, or dag-run equivalents **directly** with those filters.
- **Dynamic matrix JSON** remains a **documented escalation path** when a workflow **requires** GitHub-native parallelism (e.g., shard budget > single runner) — not required for gate `ci_uses_affected_set_selection` to pass initially.

**Rationale:** Aligns with emitter canvas §6.1 (“first implementation should prefer selected test/job lists”); minimizes duplicate policy in YAML; keeps fail-closed selection in one typed process.

---

## §3. Fail-closed behavior (unavailable / diagnostic-bearing / unknown)

When **any** of the following holds, the runner MUST select the **conservative superset** (run all checks that are “required” for the workflow policy — identical to pre-narrowing behavior, modulo unrelated skips such as draft PR workflow policy if retained at YAML level per repository policy):

| Condition | Action |
|-----------|--------|
| `Dag_before` or `Dag_after` cannot be built (missing refs, compile error, toolchain mismatch) | Superset + **actionable diagnostic** (non-zero exit if the failure is infra; otherwise treat as unknown-affectedness per policy table below) |
| Lens returns **unknown** dimension delta for any consumer edge needed for a narrowing decision | Superset for that dimension’s consumers |
| Missing **per-dimension proof receipt** for an exclusion (`design-affected-set-lens.md` §2) | Treat as **not proven narrow** → include consumer |
| Lens output carries **diagnostic-bearing** incomplete state (partial evaluation, staged substrate gap) | Superset until diagnostics cleared at source |
| `TestClaim` / job metadata does not declare asserted dimensions (§4) | That test participates in **superset** path (cannot prove skip-safe) |
| Workflow node referenced by YAML job is **unmapped** to `CIWorkflowDag` / test metadata | Superset for that job’s obligations |

**No silent skips:** skipping is only allowed when the chain `(proven receipt ∪ structural metadata)` proves the check is not in the aggregate affected set for its asserted dimensions.

**Observability:** emit `::notice::` / structured log lines summarizing: mode (`superset` vs `narrow`), reason code, counts of nodes in affected set, and which dimensions forced superset.

---

## §4. Deriving selected checks — union over dimensions + `TestClaim` / job metadata

Per `docs/design-affected-set-lens.md` §2 and §5:

1. **Compute** `A = ⋃_dim affected_set(Dag_before, Dag_after, dim)` (aggregate affected nodes).
2. **Baseline obligation set** `B`: all tests / gates / jobs the workflow treats as **merge-blocking** when no narrowing is active (from `CIWorkflowDag` + gunbc `ci.dag` gate intent, projected through the same source as Slice 4–5).
3. For each **TestClaim** or test-shaped obligation `t ∈ B`:
   - Read **asserted dimensions** `D(t)` from substrate metadata (must be modeled; absent `D(t)` → §3 superset for `t`).
   - Let `Δ(t)` be the set of dimensions with **non-empty proven delta** OR **unknown delta** on any node referenced by `t`’s structural closure (per lens rules).
   - **Select** `t` if `D(t) ∩ Δ(t) ≠ ∅` OR `t` references a node in `A` (aggregate) — implementation must use one **coherent** rule consistent with lens math; default rule: **if any asserted dimension may have changed** (`unknown` counts as may-have-changed), run `t`.
4. **Union semantics:** dimension-aware selection is **not** value-equivalence-only; cost / complexity / effect / refinement assertions keep tests live when those dimensions move (`design-affected-set-lens.md` §4.1, §5).
5. **Non-test jobs** (fmt, bootstrap verify, shell policy): map job steps to **NodeRef** / gate anchors in `CIWorkflowDag`. If mapping is absent, job stays in **superset**.

---

## §5. Invariant: YAML path-regex shortcuts — removed or **non-authoritative**

**Gate text (`docs/r3-structure.md`):** Layer 2 path-regex `if:` gates are removed from workflow files once `ci_uses_affected_set_selection` is claimed.

**Canvas rule (two-tier):**

1. **Selection authority:** After Slice 7 lands, **no** workflow `if:` condition may **encode** “run vs skip heavy compute” based on `git diff` path patterns, `paths-filter` equivalents, or hand-maintained regex allowlists. That logic lives **only** in the `BinaryShim` runner using PR #2713 output (+ §3 fail-closed).
2. **Non-selection `if:`:** Workflow conditions that are **orthogonal** to affected-set (e.g., `github.event.pull_request.draft != true`, concurrency keys) may remain in YAML as **platform/event** mechanics, not as a second selection engine. They must be **non-authoritative** for “which tests/gates prove merge safety.”
3. **Dissolution evidence:** Removal of path-regex selection is **proven** by (a) grep/ratchet `workflow_no_path_regex_policy` from `design-ci-workflow-emitter-dispatch.md` §11 and (b) code review showing selection callsite is binary-only.

---

## §6. Explicit dependency on warm-wolf-698 — Slice 5 `BinaryShim` emitter

Slice 7 **does not** implement:

- `emit_binary_shim` / `project_github_actions(..., BinaryShim)` body,
- Rust crate layout for the CI binary,
- Shim YAML bytes beyond policy constraints in `design-ci-workflow-emitter-dispatch.md` §5.2.

**Hard dependency:** warm-wolf-698 Slice 5 delivers:

- Emitted **thin** `.github/workflows/ci.yml` shim and **compiled runner** artifact path(s),
- A **hook point** (library or CLI) where Slice 7 wires lens evaluation and selection,
- **Local reproduction** command documented in-repo.

Until that lands, Slice 7 integration PRs are **blocked** except for documentation and offline fixtures.

---

## §7. Implementation dispatch checklist (post–Slice 5)

Use this as the ordered work queue for the **implementation** PR(s) after warm-wolf-698 Slice 5 merges:

1. [ ] **Pin** `Dag_before` / `Dag_after` acquisition to the same compiler + std revision policy as CI (no drift between lens and compile).
2. [ ] **Wire** PR #2713 affected-set evaluation into the Slice 5 runner entrypoint; serialize receipts for debugging.
3. [ ] **Implement** §4 mapping from `Set<NodeRef>` + metadata → `cargo test` filters / script invocations / dag-run list.
4. [ ] **Implement** §3 superset fallback table + structured reason codes + logs.
5. [ ] **Implement** §2 initial shape (in-runner selection); document matrix escalation if sharding is needed later.
6. [ ] **Delete** Layer 1/Layer 2 path-regex selection jobs and `needs:` edges that exist only for those bridges; rewire `needs:` / branch-protection assumptions (including skipped-parent semantics if any).
7. [ ] **Add** ratchet tests: `workflow_no_path_regex_policy`, `workflow_affected_set_fail_closed` (per emitter canvas §11).
8. [ ] **Add** fixture-driven tests from `design-ci-workflow-emitter-dispatch.md` §7 table (draft PR, docs-only, unknown receipt, workflow edit → superset).
9. [ ] **Prove** gate `ci_uses_affected_set_selection`: BinaryShim path consumes PR #2713 output; YAML contains no authoritative path-regex selection.
10. [ ] **Coordinate** Substrate Mgr for any `CIWorkflowDag` metadata gaps discovered when mapping jobs → nodes.

---

## §8. Review / test plan (canvas PR — no runtime code here)

**Reviewers should verify:**

- All seven parent bullets (§1–§7 here) are addressed in prose with **no contradiction** to `design-ci-workflow-emitter-dispatch.md` or `design-affected-set-lens.md`.
- Fail-closed and “no silent skips” match `design-affected-set-lens.md` §2.
- Initial shape choice (§2) is explicit and justified.
- Path-regex invariant (§5) distinguishes selection vs orthogonal `if:`.

**After implementation (future PR test obligations — not this canvas):**

- Unit tests for §3 reason codes (mock lens returning unknown / missing receipt).
- Integration test: small repo fixture where only one dimension changes and only the expected `cargo test` filter runs.
- Ratchet: no `paths:`/`paths-ignore:`-style selection in `.github/workflows/*.yml` (exact pattern set to be defined in implementation PR).

---

**End of canvas.**
