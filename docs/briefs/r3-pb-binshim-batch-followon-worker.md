---
status: PROPOSAL
owning_manager: Pure Bootstrap Manager (R3)
lane: T-LensProducer-Retirement Item-5 follow-on (8 remaining bin-shim retirements)
authored: 2026-05-09 (warm-dove-618 — PB Mgr; Director re-task gunbc#828 c#4413892216 Task A)
depends_on: smart-tern-649 Stage A landing (BinShimFilesSubsetPredicate carriers + runtime predicate body); warm-crab-600 gate #7 (regen_lens.rs canonical first-cut precedent)
parent_brief: docs/briefs/r3-pb-binshim-retirement-worker.md
---

# R3 PB — bin-shim batch follow-on retirement (8 remaining shims)

**Status:** PROPOSAL — dispatch-gated. Authored under Director re-task standing authority (gunbc#828 [c#4413892216](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4413892216) Task A) per the canonical template at [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md). This brief governs the **8 bin-shim retirements** not covered by the named §1.8 gates (#5/#6/#7).

## Scope

Per the canonical brief §"Substrate landings (locked shape)" + §"First slice", `regen_lens.rs` is the canonical first cut (gate #7, warm-crab-600). The remaining **8 bin-shim files** in `src/v3/compiler/src/bin/` are NOT covered by the named #5/#6/#7 gates and follow the same retirement template:

```
src/v3/compiler/src/bin/emit_method_template_projection.rs
src/v3/compiler/src/bin/r1c_e_emit_gates.rs
src/v3/compiler/src/bin/regen_bootstrap.rs
src/v3/compiler/src/bin/regen_parse.rs
src/v3/compiler/src/bin/regen_parse_tables.rs
src/v3/compiler/src/bin/regen_tokenize.rs
src/v3/compiler/src/bin/regen_v3.rs
src/v3/compiler/src/bin/self_host_fixed_point.rs
```

Each retirement contributes one decrement to the `BinShimFilesSubsetPredicate` count (initial 9; closure at 0 closes §1.8 sub-gate #7 / `no_new_bin_shim_hand_rust`).

## Dispatch staging

**Hard prerequisite (do not author against missing carriers)**:
- **Stage A landing on main**: smart-tern-649's substrate landings (the 4 strict-mirror artifacts at `r3-pb-binshim-retirement-worker.md` §"Substrate landings"). Per `feedback_substrate_grep_before_authoring`, worker MUST grep main at dispatch time to confirm `BinShimFilesSubsetPredicate` type + `bin_shim_files_subset_predicate` data declaration + `is_bin_shim_census_path` runtime helper + `eval_census_subset_count_shape` dispatch branch are present. If missing, STOP-AND-PING PB Mgr.
- **Recommended precedent**: warm-crab-600's gate #7 first-cut PR for `regen_lens.rs` retired. Inspect the merged PR at dispatch time for the exact emit-via-`.dag` pattern + `// AUTO-GENERATED` header + `REGEN_OUTPUTS` build.rs update + SG-0 census decrement shape.

## Per-shim retirement template

For each of the 8 shim files, the worker:

1. **Author per-shim instance declaration** at `dsl/std/runtime/bin_shims/<shim_name>.dag`:
   ```dag
   data <shim_name>_shim: BinShim = {
     name: "<shim_name>",
     entry: <existing_lib_fn_for_this_shim>,
     ...  // shape per BinShim carrier (Substrate-locked per design-pb-runtime-interpreter.md §4.2)
   }
   ```
2. **Wire into bin-shim emitter** so the emitter generates `src/v3/compiler/src/bin/<shim_name>.rs` from the `.dag` declaration with `// AUTO-GENERATED from <path> — DO NOT EDIT.` header.
3. **Verify behavioral equivalence** vs the current hand-Rust shim per parent brief §7.2 fixture (NOT byte-identity; emitted form may differ in formatting / comment shape).
4. **Atomic update**:
   - Delete hand-Rust `src/v3/compiler/src/bin/<shim_name>.rs`.
   - Add `<shim_name>` to `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`.
   - SG-0 census auto-shrinks: `EXPECTED_HAND_AUTHORED_NON_TEST` count drops by 1; `GENERATED_FILES` grows by 1.
5. **No new TestClaim per shim** — the bounded retirement gate is `no_new_bin_shim_hand_rust` (single TestClaim, count==0; per parent brief §7.3 zero-only semantics). Per-shim retirement decrements the observed count; no per-shim claim authoring needed.

## Dispatch sequencing

**Three acceptable shapes** (worker-choice):

(a) **Single mega-PR** — all 8 shims retired in one PR. Pros: minimum churn; atomic substrate-update. Cons: large diff; long review cycle.

(b) **Serial per-shim PRs** — 8 separate PRs, one per shim. Pros: small diffs; easy review; clear SG-0 census decrement per PR. Cons: 8 review cycles; head-iteration risk per `feedback_head_iteration_invalidates_approves`.

(c) **Batched 2-3 PRs** — group by emit-pattern similarity (e.g., `regen_*` family in one PR; `r1c_e_emit_gates` + `emit_method_template_projection` in another; `self_host_fixed_point` separate). Pros: balance of diff size + review cycles. Cons: emit-pattern grouping requires worker judgment.

PB Mgr leans **(c)** — `regen_*` family (`regen_bootstrap.rs` / `regen_parse.rs` / `regen_parse_tables.rs` / `regen_tokenize.rs` / `regen_v3.rs`) all share the regen-driver shape and likely share emit-pattern infrastructure with `regen_lens.rs` from gate #7 precedent. Worker may override at dispatch.

## Acceptance criteria (lane-level)

Lane closes when:
- All 8 shims retired (hand-Rust files deleted; emit-via-`.dag` generates the bin-shim files; `// AUTO-GENERATED` header present)
- `BinShimFilesSubsetPredicate` count observed at 0 (down from 9 at start of cycle, after gate #7 lands at 8)
- `no_new_bin_shim_hand_rust` `TestClaim` flips to Pass (single TestClaim from parent brief)
- §1.8 sub-gate #7 closes (cascade-side-effect of count==0)
- All 9 retired bin-shim paths in `REGEN_OUTPUTS` and absent from `EXPECTED_HAND_AUTHORED_NON_TEST`

## STOP-AND-PING conditions

Worker MUST STOP and ping PB Mgr (#2074) when:
- **Substrate carriers absent** at dispatch time (Stage A grep miss — author authority not yet on main).
- **BinShim carrier shape pressure** — a shim's actual entry-function signature can't fit the locked carrier per `design-pb-runtime-interpreter.md` §4.2 (escalates to Substrate Mgr per parent brief §"STOP conditions").
- **Per-shim equivalence fixture missing** — §7.2 BinShim equivalence fixture not authored per Stage B; can't verify behavioral equivalence (cascade-blocked on smart-tern-649 Stage B).
- **Emit-pattern divergence from `regen_lens.rs` precedent** — if the per-shim worker finds itself authoring parallel emit logic that doesn't match warm-crab-600's gate #7 PR, STOP — that's a sign the carrier or pattern is wrong.
- **Substrate-grep mismatch** — locked-shape carriers found on main with different shape than parent brief enumerates (already covered by parent §"STOP conditions" drift-detection).

## Cross-refs

- **Parent brief**: [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) — canonical template, substrate landings (locked shape), §7.2/§7.3 acceptance.
- **Design lock**: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) — §4.1 bin-shim class enumeration; §4.2 BinShim carrier-shape; §4.3 dissolution path.
- **Substrate disposition**: gunbc#2068 [c#4411574142](https://github.com/gunb-ai/gunbc/issues/2068#issuecomment-4411574142) (Substrate Mgr ratification of shape (b) `CensusSubsetCount` filter).
- **Director re-task authority**: gunbc#828 [c#4413892216](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4413892216) Task A (per-shim retirement worker dispatches under standing authority).
- **Sibling brief — gate #7 first cut**: warm-crab-600 PR (when opened against `regen_lens.rs` retirement; reference for emit-pattern + `REGEN_OUTPUTS` update + SG-0 census decrement shape).
- **Tracker**: eager-koi-853 (#2137) — coordinates 4 in-flight per-gate workers + this batch follow-on under tracker-watch B.

## Worker dispatch posture

**PROPOSAL** — dispatch held until Stage A landing + gate #7 first-cut PR observable on main. Worker spawns under PM #846 (queue position TBD per Director cross-Mgr cadence). At dispatch time, worker:

1. Greps main for `BinShimFilesSubsetPredicate` type/data + `is_bin_shim_census_path` predicate.
2. Inspects gate #7 first-cut PR (warm-crab-600's `regen_lens.rs` retirement) for emit-pattern precedent.
3. Picks staging shape (a/b/c per §"Dispatch sequencing") and proceeds.
4. Pings PB Mgr at first PR-open with chosen staging + scope manifest.

— Authored by warm-dove-618 (PB Mgr, inbox #2074); reply at #2074
