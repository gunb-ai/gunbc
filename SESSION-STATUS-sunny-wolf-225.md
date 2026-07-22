# Wind-down status — sunny-wolf-225 ("fix governor") + children

**Snapshot 2026-07-22** · captured for operator wind-down · this is a session-state record, not roadmap content (ROADMAP.md is generated/drift-gated — do not fold this into it).

---

## My PR — the governor fix

**Deliverable (DONE, in commit `5cd0155149`):** arm-time `FloorBudgetBelowMinimumFootprint` refusal in `claim_executor` for the three floor-walk plan functions (`gunbc_ci_floor_batches`, `gunbc_ci_plan_artifact_batches`, `gunbc_falsifier_batches`) when the memory-governor budget is below the measured minimum viable floor footprint (**12 GiB**, `dag/gunbc/runner_slot_allocation.dag gunbc_floor_minimum_viable_armed_budget`). Fail-fast (exit 1, requeue-able) instead of a doomed ~30-min OOM walk. Grounded on doomed (exit-137 @ 5.97–6.99 GiB) + success-class (@ 15 GiB) witness receipts; roster single-authoritied in `.dag` with a Rust seed-const drift test. Verified: closures compile clean; wall witness `true`; release bins build clean.

**PR routing:**
- **#7072** (`session/tidy-ibex-424`, another session's branch) = the **canonical merge vehicle**. Approvals: **cursor + claude + claude-opus-4-7** (≥2 distinct), **0 REQUEST_CHANGES**, mergeable. This is the one to merge.
- **#7073** (`fix/tidy-ibex-merge`, mine) = **CLOSED by me** as a byte-identical duplicate (both branches converged to `5cd0155149`; approvals don't transfer, so consolidated on the approved #7072).

**⛔ BLOCKER on #7072 — CI red, but NOT caused by the governor change:**
Whole-tree compile-clean (forced because the PR touches `claim_executor.rs`) surfaces a **pre-existing broken import on `main`**:
```
dag/gunbc/host_runner_memory_cap_verify.dag:23: error: name 'fleet_runner_unit_memory_props_read_script'
  not found in module 'gunbc.fleet_show_effective_read'
```
The symbol is actually defined in **`gunbc.fleet_show_effective_read_script`** (`fleet_show_effective_read_script.dag:22`), not `fleet_show_effective_read`. `host_runner_memory_cap_verify.dag` is **identical on my branch and `main`** — this landed on `main` via an affected-set-scoped PR (srv3 memory-cap lane, ~#7000) that never ran whole-tree compile-clean. `host_effect_realize.dag:133` imports the same symbol and should be checked for the same mis-pointed module.

**One-line fix (ready, not yet applied):** repoint the import in `host_runner_memory_cap_verify.dag:23` (and verify `host_effect_realize.dag:133`) from `gunbc.fleet_show_effective_read` → `gunbc.fleet_show_effective_read_script`. This is a `main`-lane bug independent of the governor; landing it (its own small PR, or folded onto whichever governor branch merges) unblocks #7072's CI. Not applied here to avoid pushing content onto an already-approved PR during wind-down.

---

## Children (subtree of sunny-wolf-225)

| Session | Work | PR | State | Action needed |
|---|---|---|---|---|
| stern-owl-401 | §13 refusal flip + containment-walk binding | merged | ✅ done | none — archive |
| nimble-moth-292 | Floor batch-2: dissolve once-per-entry bare-ref fixpoint | **#7062** | ✅ **MERGE-READY** (claude+cursor, 0 RC, CI passing, mergeable) | operator merge, then archive |
| fierce-crab-363 | §13 alias APPLY → `.dag` (Rust apply retired) | **#7078** | 1 approval (cursor), 0 RC, CI pending | needs 1 more approval + CI green |
| bright-heron-200 | Exclusion-lane: derive strict-walk exclusion closure | **#7059** | 0 approvals, **RC from cursor**, CI pending | address cursor REQUEST_CHANGES |
| sunny-owl-513 | Deploy fix: gunbc-tree-sync control-process failure (#7011 residual) | **#7075** | draft, 0 approvals, **RC from cursor**, **CI failing** | still in progress (was working→error) |

---

## Don't-miss checklist for wind-down
1. **Governor lands via #7072** once the broken-import CI blocker clears (fix above). #7073 is already closed — don't reopen.
2. **#7062 (nimble-moth-292) is merge-ready right now** — safe to merge.
3. **Broken import on `main`** (`host_runner_memory_cap_verify.dag` → wrong module) is a standing latent red that any whole-tree-baseline PR will trip — worth a one-line fix PR regardless of the governor.
4. Open child RCs to clear: **#7059** and **#7075** (both cursor). #7075 (deploy) also has failing CI and was mid-work.
