# T-15 bin/main.dag lane — worker closeout receipt

**Session:** `proud-otter-724` (closeout child of `nimble-carp-710`)  
**Work item:** `node://adhoc-63221782-2d4` — *T-15 bin/main.dag lane closeout*  
**Execution PR:** [#3897](https://github.com/gunb-ai/gunbc/pull/3897) (`session/nimble-carp-710`)  
**Authority:** `src/v4/TASKS.md` §T-15, `src/v4/bin/main.dag`, `src/v4/workflow/bootstrap.dag`

## What was verified (closeout pass)

| Artifact | Status |
|----------|--------|
| `src/v4/bin/main.dag` | Trampoline `String` authority + B1 digest operand placeholders present; header updated to reference harness + claim wiring |
| `src/v4/test/claim/self_host/claim_t15_self_host_fixed_point.dag` | `EqualsClaim` over `stub_stage1_*` / `stub_stage2_*` imports from `v4.bin.main` |
| `src/v4/workflow/bootstrap.dag` | `FixptStage1Stage2` + `bootstrap_plan_well_formed` requires stage1==stage2 digest convergence (not stage0==seed) |
| `src/v4/compiler/self_host.dag` | Runner remains fail-closed (`self_host_runner_not_realized`) per scaffold contract |
| `v4_t15_self_host_fixed_point_harness_test.rs` | `t_15_self_host_fixed_point` passes locally after RC fixes (compile + clippy) |
| `v4_bin_main_dag_smoke_test.rs` | Unchanged smoke; still passes on same tree |
| CI wiring | `ci.yml` + `ci_github_actions_workflow.dag` add `t_15_self_host_fixed_point` step under v4/v3/workflow_policy gates |
| SG-0 / INVARIANTS | `EXPECTED_HAND_AUTHORED_TEST` +1 (`v4_t15_self_host_fixed_point_harness_test.rs`); `_internal/INVARIANTS_OPS.md` row; `sg0-pr-body-append.3897.txt` documents net +1 |

## RC fixes applied (closeout)

Parent draft harness had two local blockers before merge-readiness:

1. **`E0599`:** `[&status.stdout, &status.stderr].concat()` — replaced with `Vec` extend.
2. **`clippy::disallowed_macros`:** removed `eprintln!` on optional gunbc skip (silent early return; module docs note optional receipt).

Commit `2c9bbd303` on `session/nimble-carp-710`.

## What this PR does **not** close

Per `TASKS.md` §T-15 **Close-status** and §Bootstrap execution convergence:

- **T-15 full close** still requires the whole plan minus T-15, B1 merkle `content_hash` pins, T-22/T-38 claim eval execution, P5 bridge removal, etc.
- Harness is **structural / parse / bootstrap-wiring** receipt (P5 Mechanism (b)), not bit-identical self-host execution.
- Placeholder digest `Node`s and symbolic bootstrap hashes remain until B1 operands land.

## Hand-off

- **Parent (`nimble-carp-710`):** mark [#3897](https://github.com/gunb-ai/gunbc/pull/3897) ready for review; pursue two distinct dashboard approvals.
- **Operator:** merge when review policy satisfied; close dashboard work items for execution + closeout sessions.
- **Closeout session:** archive after PR merge confirmed on `main`.
